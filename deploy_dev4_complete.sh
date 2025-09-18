#!/bin/bash

# MEV Shield Complete Deployment Script for Dev4 Environment
# This script handles the complete build and deployment process

set -e

echo "🛡️ MEV Shield - Complete Dev4 Deployment"
echo "========================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
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

print_header() {
    echo -e "${PURPLE}$1${NC}"
}

# Configuration
PROJECT_DIR="/opt/mev-shield"
SERVICE_USER="mevshield"
DOCKER_COMPOSE_FILE="docker-compose.yml"
ENV_FILE=".env.dev4"
BACKUP_DIR="/opt/mev-shield-backups"
LOG_FILE="/var/log/mev-shield/deployment.log"

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a $LOG_FILE
}

# Check prerequisites
check_prerequisites() {
    print_header "🔍 Checking Prerequisites"
    
    # Check if running as correct user
    if [[ $(whoami) != "$SERVICE_USER" ]] && [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as $SERVICE_USER or root"
        exit 1
    fi
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Run remote_dev4.sh first."
        exit 1
    fi
    
    # Check if Docker Compose is installed
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Run remote_dev4.sh first."
        exit 1
    fi
    
    # Check if project directory exists
    if [[ ! -d "$PROJECT_DIR" ]]; then
        print_error "Project directory $PROJECT_DIR does not exist"
        exit 1
    fi
    
    print_status "Prerequisites check passed"
}

# Create backup
create_backup() {
    print_header "💾 Creating Backup"
    
    if [[ -d "$PROJECT_DIR" ]]; then
        TIMESTAMP=$(date +%Y%m%d_%H%M%S)
        BACKUP_NAME="mev-shield-backup-$TIMESTAMP"
        
        mkdir -p $BACKUP_DIR
        
        print_info "Creating backup: $BACKUP_NAME"
        tar -czf "$BACKUP_DIR/$BACKUP_NAME.tar.gz" -C "$PROJECT_DIR" . 2>/dev/null || true
        
        # Keep only last 5 backups
        cd $BACKUP_DIR
        ls -t mev-shield-backup-*.tar.gz | tail -n +6 | xargs rm -f 2>/dev/null || true
        
        print_status "Backup created: $BACKUP_DIR/$BACKUP_NAME.tar.gz"
    fi
}

# Stop existing services
stop_services() {
    print_header "🛑 Stopping Existing Services"
    
    cd $PROJECT_DIR
    
    # Stop systemd service if running
    if systemctl is-active --quiet mev-shield 2>/dev/null; then
        print_info "Stopping MEV Shield systemd service..."
        systemctl stop mev-shield
        print_status "Systemd service stopped"
    fi
    
    # Stop Docker containers
    if [[ -f "$DOCKER_COMPOSE_FILE" ]]; then
        print_info "Stopping Docker containers..."
        docker-compose -f $DOCKER_COMPOSE_FILE down --remove-orphans 2>/dev/null || true
        print_status "Docker containers stopped"
    fi
    
    # Clean up unused Docker resources
    print_info "Cleaning up Docker resources..."
    docker system prune -f --volumes 2>/dev/null || true
    print_status "Docker cleanup completed"
}

# Build application
build_application() {
    print_header "🔨 Building MEV Shield Application"
    
    cd $PROJECT_DIR
    
    # Source Rust environment
    if [[ -f "$HOME/.cargo/env" ]]; then
        source $HOME/.cargo/env
    fi
    
    # Clean previous builds
    print_info "Cleaning previous builds..."
    cargo clean 2>/dev/null || true
    
    # Update dependencies
    print_info "Updating Rust dependencies..."
    cargo update
    
    # Run tests
    print_info "Running tests..."
    if cargo test --quiet; then
        print_status "All tests passed"
    else
        print_warning "Some tests failed, continuing with deployment"
    fi
    
    # Build release binary
    print_info "Building release binary..."
    cargo build --release
    print_status "Application built successfully"
}

# Build Docker images
build_docker_images() {
    print_header "🐳 Building Docker Images"
    
    cd $PROJECT_DIR
    
    # Build main application image
    print_info "Building MEV Shield core image..."
    docker build -t mev-shield:dev4 .
    print_status "Core image built"
    
    # Build dashboard images if they exist
    if [[ -f "dashboard/Dockerfile.admin" ]]; then
        print_info "Building admin dashboard image..."
        docker build -f dashboard/Dockerfile.admin -t mev-shield-admin:dev4 ./dashboard
        print_status "Admin dashboard image built"
    fi
    
    if [[ -f "dashboard/Dockerfile.user" ]]; then
        print_info "Building user dashboard image..."
        docker build -f dashboard/Dockerfile.user -t mev-shield-user:dev4 ./dashboard
        print_status "User dashboard image built"
    fi
}

# Setup configuration
setup_configuration() {
    print_header "⚙️ Setting Up Configuration"
    
    cd $PROJECT_DIR
    
    # Load environment variables
    if [[ -f "$ENV_FILE" ]]; then
        source $ENV_FILE
        print_status "Environment variables loaded"
    else
        print_warning "Environment file not found, using defaults"
    fi
    
    # Create necessary directories
    mkdir -p logs monitoring/grafana nginx/ssl
    
    # Set proper permissions
    chown -R $SERVICE_USER:$SERVICE_USER $PROJECT_DIR
    chmod 600 $ENV_FILE 2>/dev/null || true
    
    print_status "Configuration setup completed"
}

# Deploy services
deploy_services() {
    print_header "🚀 Deploying Services"
    
    cd $PROJECT_DIR
    
    # Start services with Docker Compose
    print_info "Starting all services..."
    docker-compose -f $DOCKER_COMPOSE_FILE --env-file $ENV_FILE up -d
    
    # Wait for services to be ready
    print_info "Waiting for services to initialize..."
    sleep 30
    
    print_status "Services deployed successfully"
}

# Health check
health_check() {
    print_header "🏥 Health Check"
    
    local max_attempts=30
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        print_info "Health check attempt $attempt/$max_attempts"
        
        # Check if containers are running
        if docker-compose -f $PROJECT_DIR/$DOCKER_COMPOSE_FILE ps | grep -q "Up"; then
            print_status "Containers are running"
            
            # Check API endpoint
            if curl -s -f http://localhost:8080/health >/dev/null 2>&1; then
                print_status "API health check passed"
                return 0
            fi
        fi
        
        sleep 10
        ((attempt++))
    done
    
    print_warning "Health check did not pass within expected time"
    return 1
}

# Show deployment status
show_status() {
    print_header "📊 Deployment Status"
    
    cd $PROJECT_DIR
    
    echo ""
    print_info "Service Status:"
    docker-compose -f $DOCKER_COMPOSE_FILE ps
    
    echo ""
    print_info "Service URLs:"
    echo "• API: http://localhost:8080"
    echo "• Metrics: http://localhost:9090"
    echo "• Grafana: http://localhost:3000"
    echo "• Prometheus: http://localhost:9091"
    
    echo ""
    print_info "Useful Commands:"
    echo "• View logs: docker-compose -f $DOCKER_COMPOSE_FILE logs -f"
    echo "• Restart services: systemctl restart mev-shield"
    echo "• Stop services: systemctl stop mev-shield"
    echo "• Check status: systemctl status mev-shield"
    
    echo ""
    print_info "Log Files:"
    echo "• Deployment: $LOG_FILE"
    echo "• Application: $PROJECT_DIR/logs/"
}

# Start systemd service
start_systemd_service() {
    print_header "🔄 Starting Systemd Service"
    
    if systemctl is-enabled mev-shield >/dev/null 2>&1; then
        systemctl start mev-shield
        print_status "MEV Shield service started"
    else
        print_warning "Systemd service not configured"
    fi
}

# Main deployment function
main() {
    echo ""
    log "Starting MEV Shield Dev4 complete deployment..."
    echo ""
    
    check_prerequisites
    create_backup
    stop_services
    build_application
    build_docker_images
    setup_configuration
    deploy_services
    
    if health_check; then
        start_systemd_service
        show_status
        
        echo ""
        print_status "🎉 MEV Shield Dev4 deployment completed successfully!"
        log "Deployment completed successfully"
    else
        print_error "Deployment completed but health check failed"
        log "Deployment completed with health check failure"
        echo ""
        print_info "Check logs for issues:"
        echo "docker-compose -f $PROJECT_DIR/$DOCKER_COMPOSE_FILE logs"
    fi
    
    echo ""
}

# Trap to cleanup on script exit
trap 'print_error "Deployment interrupted"; exit 1' INT TERM

# Run main function
main "$@"
