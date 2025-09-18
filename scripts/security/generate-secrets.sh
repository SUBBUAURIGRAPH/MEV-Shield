#!/bin/bash
# MEV Shield Security - Secret Generation Script
# Generates secure secrets for production deployment

set -euo pipefail

SECRETS_DIR="./secrets"
BACKUP_DIR="./secrets/backup"
LOG_FILE="./logs/security-setup.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
    mkdir -p "$(dirname "$LOG_FILE")"
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

# Error handling
error() {
    echo -e "${RED}❌ ERROR:${NC} $1" >&2
    log "ERROR: $1"
    exit 1
}

# Warning function
warn() {
    echo -e "${YELLOW}⚠️  WARNING:${NC} $1"
    log "WARNING: $1"
}

# Success function
success() {
    echo -e "${GREEN}✅${NC} $1"
    log "SUCCESS: $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    error "This script should not be run as root for security reasons"
fi

# Check dependencies
check_dependencies() {
    log "Checking dependencies..."
    
    if ! command -v openssl &> /dev/null; then
        error "OpenSSL is required but not installed"
    fi
    
    if ! command -v docker &> /dev/null; then
        warn "Docker not found - some features may be limited"
    fi
    
    success "Dependencies check completed"
}

# Create directory structure
setup_directories() {
    log "Setting up directory structure..."
    
    mkdir -p "$SECRETS_DIR"
    mkdir -p "$BACKUP_DIR"
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # Set secure permissions
    chmod 700 "$SECRETS_DIR"
    chmod 700 "$BACKUP_DIR"
    
    success "Directory structure created"
}

# Backup existing secrets
backup_existing_secrets() {
    if [ -d "$SECRETS_DIR" ] && [ "$(ls -A "$SECRETS_DIR" 2>/dev/null)" ]; then
        log "Backing up existing secrets..."
        
        timestamp=$(date +"%Y%m%d_%H%M%S")
        backup_file="$BACKUP_DIR/secrets_backup_$timestamp.tar.gz"
        
        tar -czf "$backup_file" -C "$SECRETS_DIR" .
        chmod 600 "$backup_file"
        
        success "Secrets backed up to $backup_file"
    fi
}

# Generate strong secrets
generate_secrets() {
    log "Generating secure secrets..."
    
    # Database password (32 characters)
    openssl rand -base64 32 | tr -d "=+/" | cut -c1-32 > "$SECRETS_DIR/postgres_password.txt"
    success "PostgreSQL password generated"
    
    # JWT secret (64 characters for high security)
    openssl rand -base64 64 | tr -d "=+/" | cut -c1-64 > "$SECRETS_DIR/jwt_secret.txt"
    success "JWT secret generated"
    
    # Redis password (32 characters)
    openssl rand -base64 32 | tr -d "=+/" | cut -c1-32 > "$SECRETS_DIR/redis_password.txt"
    success "Redis password generated"
    
    # Session secret (48 characters)
    openssl rand -base64 48 | tr -d "=+/" | cut -c1-48 > "$SECRETS_DIR/session_secret.txt"
    success "Session secret generated"
    
    # API key (32 characters)
    openssl rand -base64 32 | tr -d "=+/" | cut -c1-32 > "$SECRETS_DIR/api_key.txt"
    success "API key generated"
    
    # Encryption key for sensitive data (32 bytes = 256-bit AES)
    openssl rand 32 > "$SECRETS_DIR/encryption_key.bin"
    success "Encryption key generated"
    
    # SSL certificate password if needed
    openssl rand -base64 32 | tr -d "=+/" | cut -c1-32 > "$SECRETS_DIR/ssl_key_password.txt"
    success "SSL key password generated"
}

# Set secure permissions
set_permissions() {
    log "Setting secure file permissions..."
    
    # Set permissions on all secret files
    find "$SECRETS_DIR" -type f -exec chmod 600 {} \;
    
    # Set directory permissions
    chmod 700 "$SECRETS_DIR"
    
    success "Secure permissions applied"
}

# Generate environment template
generate_env_template() {
    log "Generating environment template..."
    
    cat > ".env.template" << 'EOF'
# MEV Shield Environment Configuration Template
# Copy to .env and fill in the values

# Database Configuration
DATABASE_URL=postgresql://mev_user:POSTGRES_PASSWORD@localhost:5432/mev_shield
POSTGRES_USER=mev_user
POSTGRES_PASSWORD=CHANGE_ME_IN_PRODUCTION
POSTGRES_DB=mev_shield
POSTGRES_PORT=5432

# Authentication & Security
JWT_SECRET=CHANGE_ME_IN_PRODUCTION
SESSION_SECRET=CHANGE_ME_IN_PRODUCTION
API_KEY=CHANGE_ME_IN_PRODUCTION
BCRYPT_ROUNDS=12

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=CHANGE_ME_IN_PRODUCTION
REDIS_PORT=6379

# Application Configuration
NODE_ENV=development
PORT=8080
ADMIN_PORT=3001
USER_PORT=3004

# Security Configuration
TOKEN_EXPIRY=24h
RATE_LIMIT_WINDOW=15
RATE_LIMIT_MAX=100
CORS_ORIGIN=https://localhost:3001,https://localhost:3004

# SSL Configuration
SSL_CERT_PATH=/etc/nginx/ssl/cert.pem
SSL_KEY_PATH=/etc/nginx/ssl/key.pem
SSL_KEY_PASSWORD=CHANGE_ME_IN_PRODUCTION

# Monitoring
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000

# Logging
LOG_LEVEL=info
LOG_FILE=/var/log/mev-shield/app.log

# External Services (if needed)
INFURA_PROJECT_ID=your_infura_project_id
ALCHEMY_API_KEY=your_alchemy_api_key
EOF

    success "Environment template created"
}

# Update gitignore
update_gitignore() {
    log "Updating .gitignore..."
    
    # Add secrets to gitignore if not present
    gitignore_entries=(
        "secrets/"
        ".env"
        ".env.local"
        ".env.production"
        "*.log"
        "logs/"
    )
    
    for entry in "${gitignore_entries[@]}"; do
        if ! grep -q "^$entry$" .gitignore 2>/dev/null; then
            echo "$entry" >> .gitignore
            log "Added $entry to .gitignore"
        fi
    done
    
    success ".gitignore updated"
}

# Generate Docker secrets compose file
generate_docker_secrets_compose() {
    log "Generating Docker secrets configuration..."
    
    cat > "docker-compose.secrets.yml" << 'EOF'
version: '3.8'

# Docker Secrets Configuration for MEV Shield
# Use this for production deployments

services:
  mev-shield-api:
    secrets:
      - postgres_password
      - jwt_secret
      - redis_password
      - session_secret
      - api_key
    environment:
      - POSTGRES_PASSWORD_FILE=/run/secrets/postgres_password
      - JWT_SECRET_FILE=/run/secrets/jwt_secret
      - REDIS_PASSWORD_FILE=/run/secrets/redis_password
      - SESSION_SECRET_FILE=/run/secrets/session_secret
      - API_KEY_FILE=/run/secrets/api_key

  postgres:
    secrets:
      - postgres_password
    environment:
      - POSTGRES_PASSWORD_FILE=/run/secrets/postgres_password

  redis:
    secrets:
      - redis_password
    environment:
      - REDIS_PASSWORD_FILE=/run/secrets/redis_password

secrets:
  postgres_password:
    file: ./secrets/postgres_password.txt
  jwt_secret:
    file: ./secrets/jwt_secret.txt
  redis_password:
    file: ./secrets/redis_password.txt
  session_secret:
    file: ./secrets/session_secret.txt
  api_key:
    file: ./secrets/api_key.txt
EOF

    success "Docker secrets configuration created"
}

# Validate generated secrets
validate_secrets() {
    log "Validating generated secrets..."
    
    local validation_failed=0
    
    # Check if all secret files exist
    secret_files=(
        "postgres_password.txt"
        "jwt_secret.txt"
        "redis_password.txt"
        "session_secret.txt"
        "api_key.txt"
        "encryption_key.bin"
        "ssl_key_password.txt"
    )
    
    for file in "${secret_files[@]}"; do
        if [ ! -f "$SECRETS_DIR/$file" ]; then
            error "Secret file $file not found"
            validation_failed=1
        else
            # Check file size (should not be empty)
            if [ ! -s "$SECRETS_DIR/$file" ]; then
                error "Secret file $file is empty"
                validation_failed=1
            fi
            
            # Check permissions
            permissions=$(stat -c "%a" "$SECRETS_DIR/$file" 2>/dev/null || stat -f "%A" "$SECRETS_DIR/$file")
            if [ "$permissions" != "600" ]; then
                warn "Secret file $file has incorrect permissions: $permissions (should be 600)"
            fi
        fi
    done
    
    if [ $validation_failed -eq 0 ]; then
        success "All secrets validated successfully"
    else
        error "Secret validation failed"
    fi
}

# Generate security setup summary
generate_summary() {
    log "Generating security setup summary..."
    
    cat > "SECURITY_SETUP_SUMMARY.md" << EOF
# MEV Shield Security Setup Summary

**Generated**: $(date)
**Script Version**: 1.0.0
**Status**: ✅ COMPLETED

## Generated Secrets

| Secret | File | Purpose |
|--------|------|---------|
| PostgreSQL Password | postgres_password.txt | Database authentication |
| JWT Secret | jwt_secret.txt | Token signing and validation |
| Redis Password | redis_password.txt | Cache authentication |
| Session Secret | session_secret.txt | Session encryption |
| API Key | api_key.txt | API authentication |
| Encryption Key | encryption_key.bin | Data encryption |
| SSL Key Password | ssl_key_password.txt | SSL certificate protection |

## Security Configuration

- **Secret Storage**: \`./secrets/\` directory (permissions: 700)
- **File Permissions**: 600 (owner read/write only)
- **Backup Location**: \`./secrets/backup/\`
- **Environment Template**: \`.env.template\`
- **Docker Secrets**: \`docker-compose.secrets.yml\`

## Next Steps

1. **Review Environment Template**: Copy \`.env.template\` to \`.env\` and configure
2. **Deploy with Docker Secrets**: Use \`docker-compose.secrets.yml\` for production
3. **Implement Authentication**: Follow the security remediation plan
4. **Setup Monitoring**: Configure security monitoring and alerting
5. **Test Security**: Run security validation tests

## Security Reminders

- ⚠️  Never commit secrets to version control
- 🔒 Use different secrets for each environment
- 🔄 Rotate secrets regularly (quarterly recommended)
- 📊 Monitor secret access and usage
- 🛡️  Use Docker secrets in production

## Support

For questions or issues with security setup:
- Review: \`SECURITY_ASSESSMENT.md\`
- Follow: \`SECURITY_REMEDIATION_PLAN.md\`
- Check logs: \`$LOG_FILE\`

---
**Security Rating**: 🟢 HIGH (after full implementation)
EOF

    success "Security setup summary generated"
}

# Main execution function
main() {
    echo -e "${BLUE}🛡️  MEV Shield Security Setup${NC}"
    echo "======================================"
    echo ""
    
    log "Starting security setup process..."
    
    check_dependencies
    setup_directories
    backup_existing_secrets
    generate_secrets
    set_permissions
    generate_env_template
    update_gitignore
    generate_docker_secrets_compose
    validate_secrets
    generate_summary
    
    echo ""
    echo -e "${GREEN}🎉 Security setup completed successfully!${NC}"
    echo ""
    echo -e "${YELLOW}📋 Summary:${NC}"
    echo "  • Secrets generated in: $SECRETS_DIR/"
    echo "  • Environment template: .env.template"
    echo "  • Docker secrets config: docker-compose.secrets.yml"
    echo "  • Setup summary: SECURITY_SETUP_SUMMARY.md"
    echo "  • Logs: $LOG_FILE"
    echo ""
    echo -e "${YELLOW}⚠️  Important Security Reminders:${NC}"
    echo "  • Never commit the secrets/ directory to version control"
    echo "  • Use different secrets for each environment (dev/staging/prod)"
    echo "  • Rotate secrets regularly (quarterly recommended)"
    echo "  • Backup secrets securely before system changes"
    echo ""
    echo -e "${BLUE}📖 Next Steps:${NC}"
    echo "  1. Copy .env.template to .env and configure"
    echo "  2. Review SECURITY_REMEDIATION_PLAN.md"
    echo "  3. Implement authentication system"
    echo "  4. Deploy with docker-compose.secrets.yml"
    echo ""
    
    log "Security setup process completed successfully"
}

# Run main function
main "$@"