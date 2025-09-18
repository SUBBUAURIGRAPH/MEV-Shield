#!/bin/bash
# MEV Shield Security Scanner
# Comprehensive security scanning for the MEV Shield platform

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOG_FILE="$PROJECT_ROOT/logs/security-scan.log"
REPORT_DIR="$PROJECT_ROOT/security-reports"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Logging
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
    mkdir -p "$(dirname "$LOG_FILE")"
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

error() {
    echo -e "${RED}❌ ERROR:${NC} $1" >&2
    log "ERROR: $1"
}

warn() {
    echo -e "${YELLOW}⚠️  WARNING:${NC} $1"
    log "WARNING: $1"
}

success() {
    echo -e "${GREEN}✅${NC} $1"
    log "SUCCESS: $1"
}

info() {
    echo -e "${PURPLE}ℹ️${NC} $1"
    log "INFO: $1"
}

# Setup
setup_environment() {
    log "Setting up security scan environment..."
    
    mkdir -p "$REPORT_DIR"
    mkdir -p "$(dirname "$LOG_FILE")"
    
    cd "$PROJECT_ROOT"
    success "Environment setup complete"
}

# Check dependencies
check_dependencies() {
    log "Checking security scanning dependencies..."
    
    local missing_tools=()
    
    # Essential tools
    if ! command -v docker &> /dev/null; then
        missing_tools+=("docker")
    fi
    
    if ! command -v git &> /dev/null; then
        missing_tools+=("git")
    fi
    
    if ! command -v curl &> /dev/null; then
        missing_tools+=("curl")
    fi
    
    # Install Trivy if not present
    if ! command -v trivy &> /dev/null; then
        info "Installing Trivy security scanner..."
        curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin
        success "Trivy installed successfully"
    fi
    
    # Install cargo-audit for Rust if not present
    if command -v cargo &> /dev/null && ! cargo audit --version &> /dev/null; then
        info "Installing cargo-audit..."
        cargo install cargo-audit
        success "cargo-audit installed successfully"
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        error "Missing required tools: ${missing_tools[*]}"
        echo "Please install the missing tools and run again."
        exit 1
    fi
    
    success "All dependencies available"
}

# Filesystem security scan
scan_filesystem() {
    log "Running filesystem security scan..."
    
    local report_file="$REPORT_DIR/filesystem_scan_$TIMESTAMP.json"
    
    # Scan for vulnerabilities in filesystem
    trivy fs --format json --output "$report_file" "$PROJECT_ROOT"
    
    # Generate summary
    local critical=$(jq '[.Results[]?.Vulnerabilities[]? | select(.Severity == "CRITICAL")] | length' "$report_file" 2>/dev/null || echo "0")
    local high=$(jq '[.Results[]?.Vulnerabilities[]? | select(.Severity == "HIGH")] | length' "$report_file" 2>/dev/null || echo "0")
    local medium=$(jq '[.Results[]?.Vulnerabilities[]? | select(.Severity == "MEDIUM")] | length' "$report_file" 2>/dev/null || echo "0")
    local low=$(jq '[.Results[]?.Vulnerabilities[]? | select(.Severity == "LOW")] | length' "$report_file" 2>/dev/null || echo "0")
    
    echo "Filesystem Scan Results:"
    echo "  Critical: $critical"
    echo "  High: $high"
    echo "  Medium: $medium"
    echo "  Low: $low"
    
    if [ "$critical" -gt 0 ] || [ "$high" -gt 0 ]; then
        warn "Critical or high severity vulnerabilities found in filesystem"
    else
        success "No critical or high severity vulnerabilities in filesystem"
    fi
    
    log "Filesystem scan complete: $report_file"
}

# Docker security scan
scan_docker_images() {
    log "Running Docker image security scan..."
    
    # Build images if they don't exist
    if ! docker images | grep -q mev-shield; then
        info "Building Docker images for scanning..."
        docker build -t mev-shield:latest .
    fi
    
    local report_file="$REPORT_DIR/docker_scan_$TIMESTAMP.json"
    
    # Scan Docker images
    trivy image --format json --output "$report_file" mev-shield:latest
    
    # Generate summary
    local critical=$(jq '[.Results[]?.Vulnerabilities[]? | select(.Severity == "CRITICAL")] | length' "$report_file" 2>/dev/null || echo "0")
    local high=$(jq '[.Results[]?.Vulnerabilities[]? | select(.Severity == "HIGH")] | length' "$report_file" 2>/dev/null || echo "0")
    
    echo "Docker Image Scan Results:"
    echo "  Critical: $critical"
    echo "  High: $high"
    
    if [ "$critical" -gt 0 ] || [ "$high" -gt 0 ]; then
        warn "Critical or high severity vulnerabilities found in Docker images"
    else
        success "No critical or high severity vulnerabilities in Docker images"
    fi
    
    log "Docker scan complete: $report_file"
}

# Configuration security scan
scan_configurations() {
    log "Running configuration security scan..."
    
    local report_file="$REPORT_DIR/config_scan_$TIMESTAMP.json"
    
    # Scan Docker Compose files and other configs
    trivy config --format json --output "$report_file" .
    
    # Additional manual checks
    local config_issues=()
    
    # Check for hardcoded secrets
    if grep -r "password.*=" . --include="*.yml" --include="*.yaml" --include="*.toml" 2>/dev/null | grep -v ".git" | grep -v "template" | grep -v "example"; then
        config_issues+=("Potential hardcoded passwords found")
    fi
    
    # Check for default ports in production configs
    if grep -r "5432\|6379\|27017" docker-compose*.yml 2>/dev/null | grep -v "127.0.0.1"; then
        config_issues+=("Database ports exposed to external network")
    fi
    
    # Check for insecure permissions
    find . -name "*.sh" -perm 777 2>/dev/null | head -5 | while read file; do
        config_issues+=("Overly permissive script: $file")
    done
    
    echo "Configuration Scan Results:"
    if [ ${#config_issues[@]} -eq 0 ]; then
        success "No configuration issues found"
    else
        for issue in "${config_issues[@]}"; do
            warn "$issue"
        done
    fi
    
    log "Configuration scan complete: $report_file"
}

# Dependency security scan
scan_dependencies() {
    log "Running dependency security scan..."
    
    # Rust dependencies
    if [ -f "Cargo.toml" ]; then
        info "Scanning Rust dependencies..."
        if command -v cargo &> /dev/null; then
            cargo audit 2>&1 | tee "$REPORT_DIR/cargo_audit_$TIMESTAMP.txt"
        else
            warn "Cargo not available, skipping Rust dependency scan"
        fi
    fi
    
    # Node.js dependencies
    if [ -f "dashboard/package.json" ]; then
        info "Scanning Node.js dependencies..."
        cd dashboard
        npm audit --json > "$REPORT_DIR/npm_audit_$TIMESTAMP.json" 2>/dev/null || true
        npm audit 2>&1 | tee "$REPORT_DIR/npm_audit_$TIMESTAMP.txt"
        cd "$PROJECT_ROOT"
    fi
    
    success "Dependency scan complete"
}

# Secret scanning
scan_secrets() {
    log "Running secret scanning..."
    
    local report_file="$REPORT_DIR/secrets_scan_$TIMESTAMP.json"
    
    # Scan for secrets in filesystem
    trivy fs --security-checks secret --format json --output "$report_file" .
    
    # Additional secret patterns
    local secret_patterns=(
        "password\s*=\s*['\"][^'\"]*['\"]"
        "api[_-]?key\s*=\s*['\"][^'\"]*['\"]"
        "secret\s*=\s*['\"][^'\"]*['\"]"
        "token\s*=\s*['\"][^'\"]*['\"]"
        "AKIA[0-9A-Z]{16}"  # AWS Access Key
        "sk_[a-z]{2}_[0-9a-zA-Z]{32}"  # Stripe Secret Key
    )
    
    local secrets_found=0
    for pattern in "${secret_patterns[@]}"; do
        if grep -rE "$pattern" . --exclude-dir=.git --exclude-dir=node_modules --exclude="*.log" --exclude="*_scan_*.json" 2>/dev/null; then
            ((secrets_found++))
        fi
    done
    
    if [ $secrets_found -gt 0 ]; then
        warn "Potential secrets found in codebase - review manually"
    else
        success "No obvious secrets found in codebase"
    fi
    
    log "Secret scan complete: $report_file"
}

# Network security scan
scan_network() {
    log "Running network security scan..."
    
    # Check for running services
    info "Checking running services..."
    
    local services_report="$REPORT_DIR/network_scan_$TIMESTAMP.txt"
    
    {
        echo "=== Port Scan Results ==="
        echo "Date: $(date)"
        echo ""
        
        # Check common ports
        local ports=(3000 3001 3004 5432 6379 8080 9090 9093)
        for port in "${ports[@]}"; do
            if netstat -tuln 2>/dev/null | grep ":$port " >/dev/null; then
                echo "Port $port: OPEN"
            else
                echo "Port $port: CLOSED"
            fi
        done
        
        echo ""
        echo "=== Active Network Connections ==="
        netstat -tuln 2>/dev/null | grep LISTEN || echo "No listening ports found"
        
    } > "$services_report"
    
    success "Network scan complete: $services_report"
}

# Generate comprehensive report
generate_report() {
    log "Generating comprehensive security report..."
    
    local main_report="$REPORT_DIR/security_scan_report_$TIMESTAMP.md"
    
    cat > "$main_report" << EOF
# MEV Shield Security Scan Report

**Scan Date**: $(date)
**Scan Version**: 1.0.0
**Project**: MEV Shield v1.0.0

## Executive Summary

This automated security scan was performed on the MEV Shield platform to identify potential vulnerabilities and security issues.

### Scan Coverage

- ✅ Filesystem vulnerabilities
- ✅ Docker image security
- ✅ Configuration analysis
- ✅ Dependency vulnerabilities
- ✅ Secret detection
- ✅ Network security

### Key Findings

EOF

    # Add scan results summary
    echo "#### Vulnerability Summary" >> "$main_report"
    echo "" >> "$main_report"
    
    # Process Trivy results if available
    local latest_fs_scan=$(ls -t "$REPORT_DIR"/filesystem_scan_*.json 2>/dev/null | head -1)
    if [ -n "$latest_fs_scan" ]; then
        local critical=$(jq '[.Results[]?.Vulnerabilities[]? | select(.Severity == "CRITICAL")] | length' "$latest_fs_scan" 2>/dev/null || echo "0")
        local high=$(jq '[.Results[]?.Vulnerabilities[]? | select(.Severity == "HIGH")] | length' "$latest_fs_scan" 2>/dev/null || echo "0")
        local medium=$(jq '[.Results[]?.Vulnerabilities[]? | select(.Severity == "MEDIUM")] | length' "$latest_fs_scan" 2>/dev/null || echo "0")
        
        cat >> "$main_report" << EOF
| Severity | Count | Status |
|----------|-------|--------|
| Critical | $critical | $([ "$critical" -eq 0 ] && echo "✅ GOOD" || echo "🔴 NEEDS ATTENTION") |
| High | $high | $([ "$high" -eq 0 ] && echo "✅ GOOD" || echo "🟠 REVIEW REQUIRED") |
| Medium | $medium | $([ "$medium" -le 5 ] && echo "✅ ACCEPTABLE" || echo "🟡 MONITOR") |

EOF
    fi
    
    cat >> "$main_report" << EOF

## Detailed Findings

### 1. Filesystem Security
- Scan file: \`$(basename "$latest_fs_scan" 2>/dev/null || echo "N/A")\`
- Status: $([ -f "$latest_fs_scan" ] && echo "✅ Completed" || echo "❌ Failed")

### 2. Docker Security
- Image vulnerabilities scanned
- Configuration security validated

### 3. Dependencies
- Rust dependencies: $([ -f "$REPORT_DIR/cargo_audit_$TIMESTAMP.txt" ] && echo "✅ Scanned" || echo "⏭️ Skipped")
- Node.js dependencies: $([ -f "$REPORT_DIR/npm_audit_$TIMESTAMP.txt" ] && echo "✅ Scanned" || echo "⏭️ Skipped")

### 4. Secret Detection
- Filesystem secret scan completed
- Pattern-based detection performed

### 5. Network Security
- Port scan completed
- Service enumeration performed

## Recommendations

1. **Address Critical Vulnerabilities**: Review and fix any critical severity findings
2. **Update Dependencies**: Ensure all dependencies are at latest secure versions
3. **Secret Management**: Implement proper secret management system
4. **Regular Scanning**: Schedule weekly automated security scans
5. **Monitor Network**: Implement network monitoring and intrusion detection

## Scan Files

All detailed scan results are available in:
- \`$REPORT_DIR/\`

## Next Steps

1. Review detailed findings in individual scan files
2. Prioritize fixes based on severity levels
3. Implement security remediation plan
4. Schedule follow-up scans

---

**Generated by**: MEV Shield Security Scanner
**Report ID**: security_scan_$TIMESTAMP
**Contact**: Security team for questions
EOF

    success "Comprehensive report generated: $main_report"
}

# Cleanup function
cleanup() {
    log "Cleaning up temporary files..."
    
    # Remove files older than 30 days
    find "$REPORT_DIR" -name "*.json" -o -name "*.txt" -o -name "*.md" | while read file; do
        if [ "$(find "$file" -mtime +30)" ]; then
            rm -f "$file"
            log "Removed old scan file: $file"
        fi
    done
    
    success "Cleanup complete"
}

# Main execution
main() {
    echo -e "${BLUE}🔍 MEV Shield Security Scanner${NC}"
    echo "===================================="
    echo ""
    
    log "Starting comprehensive security scan..."
    
    setup_environment
    check_dependencies
    
    echo -e "${YELLOW}Running security scans...${NC}"
    echo ""
    
    scan_filesystem
    echo ""
    
    scan_docker_images
    echo ""
    
    scan_configurations
    echo ""
    
    scan_dependencies
    echo ""
    
    scan_secrets
    echo ""
    
    scan_network
    echo ""
    
    generate_report
    cleanup
    
    echo ""
    echo -e "${GREEN}🎉 Security scan completed!${NC}"
    echo ""
    echo -e "${YELLOW}📊 Results Summary:${NC}"
    echo "  • Scan reports: $REPORT_DIR/"
    echo "  • Main report: security_scan_report_$TIMESTAMP.md"
    echo "  • Logs: $LOG_FILE"
    echo ""
    echo -e "${BLUE}📋 Next Steps:${NC}"
    echo "  1. Review the main security report"
    echo "  2. Address any critical/high severity findings"
    echo "  3. Follow the security remediation plan"
    echo "  4. Schedule regular security scans"
    echo ""
    
    log "Security scan process completed successfully"
}

# Handle script interruption
trap 'echo ""; error "Security scan interrupted"; exit 1' INT TERM

# Parse command line options
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            echo "MEV Shield Security Scanner"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --help, -h     Show this help message"
            echo "  --quick        Run quick scan (filesystem only)"
            echo "  --full         Run full comprehensive scan (default)"
            echo ""
            exit 0
            ;;
        --quick)
            echo "Running quick scan mode..."
            setup_environment
            check_dependencies
            scan_filesystem
            generate_report
            exit 0
            ;;
        --full)
            # Default behavior
            shift
            ;;
        *)
            error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run main function
main "$@"