#!/bin/bash
# MEV Shield Secure Environment Setup Script
# Creates necessary directories, generates secrets, and sets up secure defaults

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Helper functions
error() {
    echo -e "${RED}❌ ERROR: $1${NC}" >&2
    exit 1
}

warning() {
    echo -e "${YELLOW}⚠️  WARNING: $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Generate secure random string
generate_secret() {
    local length="${1:-64}"
    if command_exists openssl; then
        openssl rand -base64 "$length" | tr -d '\n'
    elif command_exists head && [ -r /dev/urandom ]; then
        head -c "$length" /dev/urandom | base64 | tr -d '\n'
    else
        error "Cannot generate secure random strings. Install openssl or ensure /dev/urandom is available."
    fi
}

# Generate secure password
generate_password() {
    local length="${1:-32}"
    if command_exists openssl; then
        openssl rand -base64 "$length" | tr -d '\n' | tr '/' '_' | tr '+' '-'
    else
        generate_secret "$length"
    fi
}

# Create directory with specific permissions
create_secure_dir() {
    local dir_path="$1"
    local permissions="${2:-755}"
    local owner="${3:-$(whoami)}"
    
    if [[ ! -d "$dir_path" ]]; then
        mkdir -p "$dir_path"
        chmod "$permissions" "$dir_path"
        if [[ "$owner" != "$(whoami)" ]] && command_exists chown; then
            chown "$owner" "$dir_path" 2>/dev/null || warning "Could not change owner of $dir_path to $owner"
        fi
        success "Created directory: $dir_path (permissions: $permissions)"
    else
        info "Directory already exists: $dir_path"
    fi
}

# Create file with content and specific permissions
create_secure_file() {
    local file_path="$1"
    local content="$2"
    local permissions="${3:-600}"
    local owner="${4:-$(whoami)}"
    
    echo -n "$content" > "$file_path"
    chmod "$permissions" "$file_path"
    if [[ "$owner" != "$(whoami)" ]] && command_exists chown; then
        chown "$owner" "$file_path" 2>/dev/null || warning "Could not change owner of $file_path to $owner"
    fi
    success "Created secure file: $file_path (permissions: $permissions)"
}

# Validate required tools
check_prerequisites() {
    info "Checking prerequisites..."
    
    local missing_tools=()
    
    if ! command_exists openssl; then
        missing_tools+=("openssl")
    fi
    
    if ! command_exists docker; then
        missing_tools+=("docker")
    fi
    
    if ! command_exists docker-compose; then
        missing_tools+=("docker-compose")
    fi
    
    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        error "Missing required tools: ${missing_tools[*]}. Please install them first."
    fi
    
    success "All prerequisites are available"
}

# Create directory structure
setup_directories() {
    info "Setting up directory structure..."
    
    cd "$PROJECT_ROOT"
    
    # Data directories (with restrictive permissions)
    create_secure_dir "data" "755"
    create_secure_dir "data/mev-shield" "700"
    create_secure_dir "data/postgres" "700"
    create_secure_dir "data/redis" "700"
    
    # Log directories
    create_secure_dir "logs" "755"
    create_secure_dir "logs/mev-shield" "755"
    create_secure_dir "logs/nginx" "755"
    
    # Configuration directories
    create_secure_dir "config" "755"
    create_secure_dir "nginx" "755"
    
    # Secrets directory (highly restrictive)
    create_secure_dir "secrets" "700"
    
    # TLS certificate directory
    create_secure_dir "ssl" "700"
    
    # Backup directory
    create_secure_dir "backups" "700"
}

# Generate secrets
generate_secrets() {
    info "Generating secure secrets..."
    
    cd "$PROJECT_ROOT"
    
    # JWT secret (64-byte base64 encoded)
    if [[ ! -f "secrets/jwt_secret.txt" ]]; then
        JWT_SECRET=$(generate_secret 64)
        create_secure_file "secrets/jwt_secret.txt" "$JWT_SECRET" "600"
    else
        info "JWT secret already exists"
    fi
    
    # Database password
    if [[ ! -f "secrets/db_password.txt" ]]; then
        DB_PASSWORD=$(generate_password 32)
        create_secure_file "secrets/db_password.txt" "$DB_PASSWORD" "600"
    else
        info "Database password already exists"
    fi
    
    # Redis password
    if [[ ! -f "secrets/redis_password.txt" ]]; then
        REDIS_PASSWORD=$(generate_password 32)
        create_secure_file "secrets/redis_password.txt" "$REDIS_PASSWORD" "600"
    else
        info "Redis password already exists"
    fi
    
    # Encryption key for application
    if [[ ! -f "secrets/encryption_key.txt" ]]; then
        ENCRYPTION_KEY=$(generate_secret 32)
        create_secure_file "secrets/encryption_key.txt" "$ENCRYPTION_KEY" "600"
    else
        info "Encryption key already exists"
    fi
    
    # Admin API key
    if [[ ! -f "secrets/admin_api_key.txt" ]]; then
        ADMIN_API_KEY=$(generate_secret 32)
        create_secure_file "secrets/admin_api_key.txt" "$ADMIN_API_KEY" "600"
    else
        info "Admin API key already exists"
    fi
}

# Create .env file with secure defaults
create_env_file() {
    info "Creating environment configuration..."
    
    cd "$PROJECT_ROOT"
    
    if [[ -f ".env" ]]; then
        warning ".env file already exists. Creating .env.new instead."
        ENV_FILE=".env.new"
    else
        ENV_FILE=".env"
    fi
    
    # Read secrets
    JWT_SECRET=$(cat secrets/jwt_secret.txt)
    DB_PASSWORD=$(cat secrets/db_password.txt)
    REDIS_PASSWORD=$(cat secrets/redis_password.txt)
    ENCRYPTION_KEY=$(cat secrets/encryption_key.txt)
    ADMIN_API_KEY=$(cat secrets/admin_api_key.txt)
    
    # Create .env file
    cat > "$ENV_FILE" << EOF
# MEV Shield Environment Configuration
# Generated on $(date)

# =============================================================================
# JWT AUTHENTICATION CONFIGURATION
# =============================================================================
JWT_SECRET=$JWT_SECRET
JWT_ISSUER=mev-shield
JWT_AUDIENCE=mev-shield-api
JWT_ACCESS_TOKEN_EXPIRY_HOURS=1
JWT_REFRESH_TOKEN_EXPIRY_DAYS=30

# =============================================================================
# DATABASE CONFIGURATION
# =============================================================================
DATABASE_URL=postgresql://mev_shield_user:$DB_PASSWORD@localhost:5432/mev_shield_db
DATABASE_HOST=postgres
DATABASE_PORT=5432
DATABASE_NAME=mev_shield_db
DATABASE_USER=mev_shield_user
DATABASE_PASSWORD=$DB_PASSWORD
DATABASE_MAX_CONNECTIONS=20
DATABASE_SSL_MODE=require

# =============================================================================
# REDIS CONFIGURATION
# =============================================================================
REDIS_URL=redis://:$REDIS_PASSWORD@localhost:6379
REDIS_HOST=redis
REDIS_PORT=6379
REDIS_PASSWORD=$REDIS_PASSWORD
REDIS_DATABASE=0

# =============================================================================
# API CONFIGURATION
# =============================================================================
API_HOST=0.0.0.0
API_PORT=8080
API_CORS_ENABLED=false
API_RATE_LIMITING=true
API_ADMIN_ENDPOINTS=true
ADMIN_API_KEY=$ADMIN_API_KEY

# =============================================================================
# ENCRYPTION CONFIGURATION
# =============================================================================
ENCRYPTION_KEY=$ENCRYPTION_KEY
THRESHOLD_N=5
THRESHOLD_K=3

# =============================================================================
# SECURITY CONFIGURATION
# =============================================================================
ENVIRONMENT=production
DEBUG=false
LOG_LEVEL=info
PASSWORD_MIN_LENGTH=12
RATE_LIMIT_REQUESTS_PER_MINUTE=60
SESSION_TIMEOUT_MINUTES=60

# =============================================================================
# CORS CONFIGURATION
# =============================================================================
CORS_ALLOWED_ORIGINS=https://yourdomain.com
CORS_ALLOWED_METHODS=GET,POST,PUT,DELETE,OPTIONS
CORS_ALLOWED_HEADERS=Content-Type,Authorization

# =============================================================================
# MONITORING CONFIGURATION
# =============================================================================
METRICS_ENABLED=true
METRICS_PORT=9090
HEALTH_CHECK_ENABLED=true

# =============================================================================
# TLS CONFIGURATION
# =============================================================================
TLS_CERT_PATH=./ssl/server.crt
TLS_KEY_PATH=./ssl/server.key

# =============================================================================
# FRONTEND CONFIGURATION
# =============================================================================
REACT_APP_API_URL=https://localhost:8080
REACT_APP_ENVIRONMENT=production
REACT_APP_ENABLE_DEBUG=false
EOF

    create_secure_file "$ENV_FILE" "$(cat "$ENV_FILE")" "600"
    
    if [[ "$ENV_FILE" == ".env.new" ]]; then
        warning "Review .env.new and replace .env when ready"
    fi
}

# Generate self-signed certificate for development
generate_dev_certificate() {
    info "Generating development TLS certificate..."
    
    cd "$PROJECT_ROOT"
    
    if [[ ! -f "ssl/server.crt" ]] || [[ ! -f "ssl/server.key" ]]; then
        openssl req -x509 -newkey rsa:4096 -nodes \
            -keyout ssl/server.key \
            -out ssl/server.crt \
            -days 365 \
            -subj "/C=US/ST=Development/L=Local/O=MEV Shield/OU=Development/CN=localhost" \
            -extensions v3_req \
            -config <(
                echo '[req]'
                echo 'distinguished_name = req'
                echo '[v3_req]'
                echo 'keyUsage = keyEncipherment, dataEncipherment'
                echo 'extendedKeyUsage = serverAuth'
                echo 'subjectAltName = @alt_names'
                echo '[alt_names]'
                echo 'DNS.1 = localhost'
                echo 'DNS.2 = mev-shield.local'
                echo 'IP.1 = 127.0.0.1'
                echo 'IP.2 = ::1'
            ) 2>/dev/null
        
        chmod 600 ssl/server.key
        chmod 644 ssl/server.crt
        
        success "Generated development TLS certificate"
    else
        info "TLS certificate already exists"
    fi
}

# Create nginx configuration
create_nginx_config() {
    info "Creating nginx configuration..."
    
    cd "$PROJECT_ROOT"
    
    if [[ ! -f "nginx/nginx.conf" ]]; then
        cat > nginx/nginx.conf << 'EOF'
# Nginx configuration for MEV Shield
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;
    
    # Security headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';" always;
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/s;
    
    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    
    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 10240;
    gzip_proxied expired no-cache no-store private must-revalidate;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;
    
    # Logging
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';
    access_log /var/log/nginx/access.log main;
    
    # Upstream backend
    upstream mev_shield_backend {
        server mev-shield:8080 max_fails=3 fail_timeout=30s;
    }
    
    # HTTP to HTTPS redirect
    server {
        listen 80;
        server_name _;
        return 301 https://$host$request_uri;
    }
    
    # HTTPS server
    server {
        listen 443 ssl http2;
        server_name localhost mev-shield.local;
        
        ssl_certificate /etc/nginx/ssl/server.crt;
        ssl_certificate_key /etc/nginx/ssl/server.key;
        
        # Security
        client_max_body_size 10m;
        client_body_timeout 60s;
        client_header_timeout 60s;
        keepalive_timeout 65s;
        
        # Health check endpoint
        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }
        
        # API endpoints with rate limiting
        location /api/ {
            limit_req zone=api burst=20 nodelay;
            proxy_pass http://mev_shield_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_connect_timeout 30s;
            proxy_send_timeout 30s;
            proxy_read_timeout 30s;
        }
        
        # Authentication endpoints with stricter rate limiting
        location /auth/ {
            limit_req zone=auth burst=10 nodelay;
            proxy_pass http://mev_shield_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
        
        # Admin endpoints (additional restrictions)
        location /api/v1/admin/ {
            allow 127.0.0.1;
            allow 10.0.0.0/8;
            allow 172.16.0.0/12;
            allow 192.168.0.0/16;
            deny all;
            
            limit_req zone=api burst=5 nodelay;
            proxy_pass http://mev_shield_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
EOF
        success "Created nginx configuration"
    else
        info "Nginx configuration already exists"
    fi
}

# Set up Docker environment
setup_docker_env() {
    info "Setting up Docker environment..."
    
    cd "$PROJECT_ROOT"
    
    # Create .dockerignore if it doesn't exist
    if [[ ! -f ".dockerignore" ]]; then
        cat > .dockerignore << 'EOF'
# Development files
.env*
.git/
.gitignore
*.md
docs/

# Build artifacts
target/
node_modules/
dist/
build/

# Secrets and sensitive data
secrets/
ssl/
*.key
*.crt
*.pem

# Logs
logs/
*.log

# Data directories
data/
backups/

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db
EOF
        success "Created .dockerignore file"
    fi
    
    # Validate Docker Compose file
    if command_exists docker-compose; then
        if docker-compose -f docker-compose.secure.yml config > /dev/null 2>&1; then
            success "Docker Compose configuration is valid"
        else
            warning "Docker Compose configuration may have issues"
        fi
    fi
}

# Create startup script
create_startup_script() {
    info "Creating startup script..."
    
    cd "$PROJECT_ROOT"
    
    cat > start-secure.sh << 'EOF'
#!/bin/bash
# MEV Shield Secure Startup Script

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

info "Starting MEV Shield in secure mode..."

# Validate environment
./scripts/security/validate-env.sh

# Start services
info "Starting Docker services..."
docker-compose -f docker-compose.secure.yml up -d

# Wait for services to be healthy
info "Waiting for services to become healthy..."
timeout=300
elapsed=0
while [ $elapsed -lt $timeout ]; do
    if docker-compose -f docker-compose.secure.yml ps | grep -q "healthy"; then
        success "All services are healthy"
        break
    fi
    sleep 5
    elapsed=$((elapsed + 5))
done

if [ $elapsed -ge $timeout ]; then
    echo "❌ Services did not become healthy within $timeout seconds"
    docker-compose -f docker-compose.secure.yml logs
    exit 1
fi

success "MEV Shield is running securely"
echo "🔗 API available at: https://localhost:8080"
echo "📊 Metrics available at: http://localhost:9090"
echo "🏥 Health check: https://localhost:8080/api/v1/health"
EOF

    chmod +x start-secure.sh
    success "Created startup script: start-secure.sh"
}

# Main execution
main() {
    echo -e "${BLUE}🔐 MEV Shield Secure Environment Setup${NC}"
    echo "====================================="
    echo
    
    check_prerequisites
    setup_directories
    generate_secrets
    create_env_file
    generate_dev_certificate
    create_nginx_config
    setup_docker_env
    create_startup_script
    
    echo
    success "🎉 Secure environment setup completed!"
    echo
    echo "Next steps:"
    echo "1. Review and customize .env file"
    echo "2. Update nginx/nginx.conf for your domain"
    echo "3. Replace development certificates with production ones"
    echo "4. Run: ./start-secure.sh"
    echo "5. Run: ./scripts/security/validate-env.sh"
    echo
    warning "⚠️  Remember to:"
    echo "   - Keep secrets/ directory secure (never commit to git)"
    echo "   - Use proper TLS certificates in production"
    echo "   - Configure firewall rules"
    echo "   - Set up monitoring and alerting"
    echo "   - Regularly rotate secrets"
}

# Run main function
main "$@"