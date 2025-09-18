#!/bin/bash

# MEV Shield Remote Development Environment Setup (Dev4)
# This script sets up the development environment on a remote server

set -e

echo "🛡️ MEV Shield - Remote Dev4 Environment Setup"
echo "=============================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Configuration
PROJECT_DIR="/opt/mev-shield"
SERVICE_USER="mevshield"
DOCKER_COMPOSE_FILE="docker-compose.yml"
ENV_FILE=".env.dev4"

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root for initial setup"
        echo "Usage: sudo $0"
        exit 1
    fi
}

# Install system dependencies
install_dependencies() {
    print_info "Installing system dependencies..."
    
    # Update package list
    apt-get update -y
    
    # Install required packages
    apt-get install -y \
        curl \
        wget \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        postgresql-client \
        redis-tools \
        htop \
        vim \
        unzip
    
    print_status "System dependencies installed"
}

# Install Docker and Docker Compose
install_docker() {
    print_info "Installing Docker and Docker Compose..."
    
    # Install Docker
    if ! command -v docker &> /dev/null; then
        curl -fsSL https://get.docker.com -o get-docker.sh
        sh get-docker.sh
        rm get-docker.sh
        print_status "Docker installed"
    else
        print_status "Docker already installed"
    fi
    
    # Install Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        chmod +x /usr/local/bin/docker-compose
        print_status "Docker Compose installed"
    else
        print_status "Docker Compose already installed"
    fi
    
    # Add service user to docker group
    usermod -aG docker $SERVICE_USER 2>/dev/null || true
}

# Install Rust
install_rust() {
    print_info "Installing Rust for $SERVICE_USER..."
    
    # Switch to service user and install Rust
    sudo -u $SERVICE_USER bash -c '
        if ! command -v rustc &> /dev/null; then
            curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source ~/.cargo/env
            rustup default stable
            echo "✓ Rust installed"
        else
            echo "✓ Rust already installed"
        fi
    '
}

# Create service user and directories
setup_user_and_dirs() {
    print_info "Setting up service user and directories..."
    
    # Create service user
    if ! id "$SERVICE_USER" &>/dev/null; then
        useradd -r -m -s /bin/bash $SERVICE_USER
        print_status "Created service user: $SERVICE_USER"
    else
        print_status "Service user already exists: $SERVICE_USER"
    fi
    
    # Create project directory
    mkdir -p $PROJECT_DIR
    chown -R $SERVICE_USER:$SERVICE_USER $PROJECT_DIR
    print_status "Created project directory: $PROJECT_DIR"
    
    # Create log directory
    mkdir -p /var/log/mev-shield
    chown -R $SERVICE_USER:$SERVICE_USER /var/log/mev-shield
    print_status "Created log directory: /var/log/mev-shield"
}

# Setup firewall
setup_firewall() {
    print_info "Configuring firewall..."
    
    # Install ufw if not present
    if ! command -v ufw &> /dev/null; then
        apt-get install -y ufw
    fi
    
    # Configure firewall rules
    ufw --force reset
    ufw default deny incoming
    ufw default allow outgoing
    
    # Allow SSH
    ufw allow ssh
    
    # Allow MEV Shield ports
    ufw allow 8080/tcp  # API port
    ufw allow 9090/tcp  # Metrics port
    ufw allow 3000/tcp  # Grafana
    ufw allow 9091/tcp  # Prometheus
    
    # Enable firewall
    ufw --force enable
    print_status "Firewall configured"
}

# Create environment file
create_env_file() {
    print_info "Creating environment configuration..."
    
    cat > $PROJECT_DIR/$ENV_FILE << EOF
# MEV Shield Dev4 Environment Configuration
ENVIRONMENT=dev4
RUST_LOG=info
POSTGRES_PASSWORD=mevshield_dev4_$(openssl rand -hex 16)
GRAFANA_PASSWORD=admin_$(openssl rand -hex 8)
DATABASE_URL=postgresql://mevshield:\${POSTGRES_PASSWORD}@postgres:5432/mevshield
REDIS_URL=redis://redis:6379
API_HOST=0.0.0.0
API_PORT=8080
METRICS_PORT=9090
EOF
    
    chown $SERVICE_USER:$SERVICE_USER $PROJECT_DIR/$ENV_FILE
    chmod 600 $PROJECT_DIR/$ENV_FILE
    print_status "Environment file created: $ENV_FILE"
}

# Setup systemd service
setup_systemd_service() {
    print_info "Setting up systemd service..."
    
    cat > /etc/systemd/system/mev-shield.service << EOF
[Unit]
Description=MEV Shield Protection Service
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=$PROJECT_DIR
User=$SERVICE_USER
Group=$SERVICE_USER
ExecStart=/usr/local/bin/docker-compose -f $DOCKER_COMPOSE_FILE up -d
ExecStop=/usr/local/bin/docker-compose -f $DOCKER_COMPOSE_FILE down
TimeoutStartSec=300
TimeoutStopSec=120

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    systemctl enable mev-shield
    print_status "Systemd service configured"
}

# Main setup function
main() {
    echo ""
    print_info "Starting MEV Shield Dev4 remote setup..."
    echo ""
    
    check_root
    install_dependencies
    setup_user_and_dirs
    install_docker
    install_rust
    setup_firewall
    create_env_file
    setup_systemd_service
    
    echo ""
    print_status "Remote Dev4 environment setup completed!"
    echo ""
    print_info "Next steps:"
    echo "1. Clone MEV Shield repository to $PROJECT_DIR"
    echo "2. Run deploy_dev4_complete.sh to build and deploy"
    echo "3. Monitor with: systemctl status mev-shield"
    echo ""
    print_warning "Remember to:"
    echo "- Configure SSL certificates for production"
    echo "- Set up backup procedures"
    echo "- Configure monitoring alerts"
    echo "- Review security settings"
    echo ""
}

# Run main function
main "$@"
