#!/bin/bash
# MEV Shield Environment Validation Script
# Validates that all required environment variables are set and secure

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
ERRORS=0
WARNINGS=0
PASSED=0

# Helper functions
error() {
    echo -e "${RED}❌ ERROR: $1${NC}"
    ((ERRORS++))
}

warning() {
    echo -e "${YELLOW}⚠️  WARNING: $1${NC}"
    ((WARNINGS++))
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
    ((PASSED++))
}

info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if variable is set and not empty
check_required() {
    local var_name="$1"
    local description="$2"
    
    if [[ -z "${!var_name:-}" ]]; then
        error "$description ($var_name) is required but not set"
        return 1
    else
        success "$description is set"
        return 0
    fi
}

# Check if variable is set and meets minimum length requirement
check_min_length() {
    local var_name="$1"
    local min_length="$2"
    local description="$3"
    
    local value="${!var_name:-}"
    
    if [[ -z "$value" ]]; then
        error "$description ($var_name) is required but not set"
        return 1
    elif [[ ${#value} -lt $min_length ]]; then
        error "$description ($var_name) is too short (minimum $min_length characters)"
        return 1
    else
        success "$description meets minimum length requirement"
        return 0
    fi
}

# Check if variable contains insecure default values
check_not_default() {
    local var_name="$1"
    local default_value="$2"
    local description="$3"
    
    local value="${!var_name:-}"
    
    if [[ "$value" == "$default_value" ]]; then
        error "$description ($var_name) is using default value - SECURITY RISK"
        return 1
    elif [[ -n "$value" ]]; then
        success "$description is not using default value"
        return 0
    else
        return 0  # Already handled by check_required
    fi
}

# Check if URL is valid format
check_url_format() {
    local var_name="$1"
    local description="$2"
    local required_schemes="$3"
    
    local value="${!var_name:-}"
    
    if [[ -z "$value" ]]; then
        return 0  # Already handled by check_required
    fi
    
    local scheme_found=false
    IFS=',' read -ra SCHEMES <<< "$required_schemes"
    for scheme in "${SCHEMES[@]}"; do
        if [[ "$value" =~ ^$scheme:// ]]; then
            scheme_found=true
            break
        fi
    done
    
    if [[ "$scheme_found" == "true" ]]; then
        success "$description has valid URL format"
        return 0
    else
        error "$description ($var_name) has invalid URL format. Expected schemes: $required_schemes"
        return 1
    fi
}

# Check if numeric value is within valid range
check_numeric_range() {
    local var_name="$1"
    local min_value="$2"
    local max_value="$3"
    local description="$4"
    
    local value="${!var_name:-}"
    
    if [[ -z "$value" ]]; then
        return 0  # Use defaults
    fi
    
    if [[ ! "$value" =~ ^[0-9]+$ ]]; then
        error "$description ($var_name) must be a number"
        return 1
    fi
    
    if [[ "$value" -lt "$min_value" ]] || [[ "$value" -gt "$max_value" ]]; then
        error "$description ($var_name) must be between $min_value and $max_value"
        return 1
    else
        success "$description is within valid range"
        return 0
    fi
}

echo -e "${BLUE}🔐 MEV Shield Environment Validation${NC}"
echo "========================================"

# Load .env file if it exists
if [[ -f ".env" ]]; then
    info "Loading environment variables from .env file"
    set -a
    source .env
    set +a
else
    warning ".env file not found - using system environment variables"
fi

echo
echo "Validating Required Configuration..."
echo "-----------------------------------"

# Critical Security Variables
check_required JWT_SECRET "JWT Secret Key"
check_min_length JWT_SECRET 64 "JWT Secret Key"
check_not_default JWT_SECRET "your-super-secret-jwt-key-change-this-in-production-please-use-openssl-rand-base64-64" "JWT Secret Key"

# Database Configuration
check_required DATABASE_URL "Database URL"
check_url_format DATABASE_URL "Database URL" "postgresql,postgres"

check_required DATABASE_PASSWORD "Database Password"
check_min_length DATABASE_PASSWORD 12 "Database Password"
check_not_default DATABASE_PASSWORD "secure_password" "Database Password"

# Encryption Configuration
check_numeric_range THRESHOLD_N 3 100 "Total Validators (N)"
check_numeric_range THRESHOLD_K 2 67 "Threshold Validators (K)"

if [[ -n "${THRESHOLD_N:-}" ]] && [[ -n "${THRESHOLD_K:-}" ]]; then
    if [[ "$THRESHOLD_K" -gt "$THRESHOLD_N" ]]; then
        error "Threshold K ($THRESHOLD_K) cannot be greater than total N ($THRESHOLD_N)"
    else
        success "Threshold configuration is valid (K=$THRESHOLD_K, N=$THRESHOLD_N)"
    fi
fi

# API Configuration
check_numeric_range API_PORT 1024 65535 "API Port"

# Optional but Recommended Variables
echo
echo "Validating Optional Configuration..."
echo "-----------------------------------"

# Redis Configuration
if [[ -n "${REDIS_URL:-}" ]]; then
    check_url_format REDIS_URL "Redis URL" "redis"
fi

# Blockchain RPC URLs
if [[ -n "${ETHEREUM_RPC_URL:-}" ]]; then
    check_url_format ETHEREUM_RPC_URL "Ethereum RPC URL" "https,wss"
fi

if [[ -n "${POLYGON_RPC_URL:-}" ]]; then
    check_url_format POLYGON_RPC_URL "Polygon RPC URL" "https,wss"
fi

# Private Keys (warn if using defaults)
if [[ "${VALIDATOR_PRIVATE_KEY:-}" == "0x0000000000000000000000000000000000000000000000000000000000000001" ]]; then
    error "Validator private key is using default value - CRITICAL SECURITY RISK"
fi

if [[ "${ADMIN_PRIVATE_KEY:-}" == "0x0000000000000000000000000000000000000000000000000000000000000002" ]]; then
    error "Admin private key is using default value - CRITICAL SECURITY RISK"
fi

# Environment-specific checks
echo
echo "Environment-Specific Validation..."
echo "--------------------------------"

ENVIRONMENT=${ENVIRONMENT:-development}
info "Environment: $ENVIRONMENT"

case "$ENVIRONMENT" in
    "production")
        # Production-specific security checks
        if [[ "${DEBUG:-false}" == "true" ]]; then
            warning "Debug mode is enabled in production environment"
        fi
        
        if [[ "${API_CORS_ENABLED:-true}" == "true" ]]; then
            if [[ "${CORS_ALLOWED_ORIGINS:-}" == "*" ]]; then
                error "CORS is set to allow all origins in production - SECURITY RISK"
            fi
        fi
        
        if [[ "${DATABASE_SSL_MODE:-}" != "require" ]]; then
            warning "Database SSL mode should be 'require' in production"
        fi
        
        check_required TLS_CERT_PATH "TLS Certificate Path (production)"
        check_required TLS_KEY_PATH "TLS Private Key Path (production)"
        ;;
        
    "development"|"test")
        info "Development/test environment detected - relaxed security checks"
        ;;
        
    *)
        warning "Unknown environment '$ENVIRONMENT' - using default checks"
        ;;
esac

# File Permission Checks (if files exist)
echo
echo "File Permission Validation..."
echo "----------------------------"

check_file_permissions() {
    local file_path="$1"
    local description="$2"
    local max_permissions="$3"
    
    if [[ -f "$file_path" ]]; then
        local actual_permissions
        actual_permissions=$(stat -c "%a" "$file_path" 2>/dev/null || stat -f "%A" "$file_path" 2>/dev/null || echo "000")
        
        if [[ "$actual_permissions" -le "$max_permissions" ]]; then
            success "$description has secure permissions ($actual_permissions)"
        else
            error "$description has overly permissive permissions ($actual_permissions). Should be $max_permissions or less."
        fi
    fi
}

check_file_permissions ".env" ".env file" 600
check_file_permissions "${TLS_CERT_PATH:-/nonexistent}" "TLS Certificate" 644
check_file_permissions "${TLS_KEY_PATH:-/nonexistent}" "TLS Private Key" 600

# Summary
echo
echo "Validation Summary"
echo "=================="

if [[ $ERRORS -eq 0 ]] && [[ $WARNINGS -eq 0 ]]; then
    echo -e "${GREEN}🎉 All validations passed! Your environment is secure.${NC}"
    exit 0
elif [[ $ERRORS -eq 0 ]]; then
    echo -e "${YELLOW}⚠️  Validation completed with $WARNINGS warning(s). Consider addressing them.${NC}"
    echo -e "   - Passed: $PASSED"
    echo -e "   - Warnings: $WARNINGS"
    echo -e "   - Errors: $ERRORS"
    exit 0
else
    echo -e "${RED}❌ Validation failed with $ERRORS error(s) and $WARNINGS warning(s).${NC}"
    echo -e "   - Passed: $PASSED"
    echo -e "   - Warnings: $WARNINGS" 
    echo -e "   - Errors: $ERRORS"
    echo
    echo -e "${RED}CRITICAL: Fix all errors before deploying to production!${NC}"
    exit 1
fi