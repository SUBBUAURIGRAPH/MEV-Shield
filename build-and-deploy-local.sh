#!/bin/bash

# MEV Shield Local Build and Deploy Script
# This script builds and deploys MEV Shield locally using Docker

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/logs/deploy.log"

# Create logs directory
mkdir -p "$SCRIPT_DIR/logs"

# Logging function
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" | tee -a "$LOG_FILE"
}

warning() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1" | tee -a "$LOG_FILE"
}

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $1" | tee -a "$LOG_FILE"
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if Docker is running
    if ! docker info &> /dev/null; then
        error "Docker is not running. Please start Docker first."
        exit 1
    fi
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        warning "Rust is not installed. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        warning "Node.js is not installed. Please install Node.js 18+ first."
        exit 1
    fi
    
    # Check npm
    if ! command -v npm &> /dev/null; then
        error "npm is not installed. Please install npm first."
        exit 1
    fi
    
    log "✅ All prerequisites met!"
}

# Clean up function
cleanup() {
    log "Cleaning up previous deployment..."
    
    # Stop existing containers
    docker-compose -f docker-compose.local.yml down --remove-orphans || true
    
    # Remove old images (optional)
    if [ "$1" = "--clean" ]; then
        info "Removing old Docker images..."
        docker image prune -f
        docker system prune -f
    fi
    
    log "✅ Cleanup completed!"
}

# Build Rust backend
build_backend() {
    log "Building Rust backend..."
    
    # Create target directory if it doesn't exist
    mkdir -p target/release
    
    # Check if Cargo.toml exists
    if [ ! -f "Cargo.toml" ]; then
        error "Cargo.toml not found. Are you in the right directory?"
        exit 1
    fi
    
    # Build the backend
    info "Running cargo build --release..."
    if cargo build --release; then
        log "✅ Backend build completed!"
    else
        error "Backend build failed!"
        exit 1
    fi
}

# Build dashboard
build_dashboard() {
    log "Building React dashboard..."
    
    cd dashboard
    
    # Check if package.json exists
    if [ ! -f "package.json" ]; then
        error "package.json not found in dashboard directory!"
        exit 1
    fi
    
    # Install dependencies
    info "Installing npm dependencies..."
    if npm ci; then
        log "✅ Dependencies installed!"
    else
        error "Failed to install dependencies!"
        exit 1
    fi
    
    # Build the dashboard
    info "Building React app..."
    if npm run build; then
        log "✅ Dashboard build completed!"
    else
        error "Dashboard build failed!"
        exit 1
    fi
    
    cd ..
}

# Build Docker images
build_docker_images() {
    log "Building Docker images..."
    
    # Build main application image
    info "Building MEV Shield core image..."
    if docker build -t mev-shield:local .; then
        log "✅ Core image built successfully!"
    else
        error "Failed to build core image!"
        exit 1
    fi
    
    # Build dashboard images
    info "Building admin dashboard image..."
    if docker build -f dashboard/Dockerfile.admin -t mev-shield-admin:local ./dashboard; then
        log "✅ Admin dashboard image built!"
    else
        error "Failed to build admin dashboard image!"
        exit 1
    fi
    
    info "Building user dashboard image..."
    if docker build -f dashboard/Dockerfile.user -t mev-shield-user:local ./dashboard; then
        log "✅ User dashboard image built!"
    else
        error "Failed to build user dashboard image!"
        exit 1
    fi
}

# Deploy services
deploy_services() {
    log "Deploying services with Docker Compose..."
    
    # Create necessary directories
    mkdir -p logs nginx/ssl monitoring/grafana
    
    # Start services
    info "Starting all services..."
    if docker-compose -f docker-compose.local.yml up -d; then
        log "✅ Services deployed successfully!"
    else
        error "Failed to deploy services!"
        exit 1
    fi
    
    # Wait for services to be ready
    log "Waiting for services to be ready..."
    sleep 10
    
    # Check health
    check_health
}

# Health check
check_health() {
    log "Performing health checks..."
    
    local services=(
        "http://localhost:8080/health:MEV Shield Core"
        "http://localhost:3001:Admin Dashboard"
        "http://localhost:3002:User Dashboard"
        "http://localhost:5432:PostgreSQL"
        "http://localhost:6379:Redis"
        "http://localhost:9091:Prometheus"
        "http://localhost:3000:Grafana"
    )
    
    for service in "${services[@]}"; do
        IFS=':' read -r url name <<< "$service"
        
        info "Checking $name..."
        
        # Try to connect for up to 60 seconds
        for i in {1..12}; do
            if curl -f -s "$url" > /dev/null 2>&1; then
                log "✅ $name is healthy!"
                break
            else
                if [ $i -eq 12 ]; then
                    warning "⚠️ $name might not be ready yet"
                fi
                sleep 5
            fi
        done
    done
}

# Show status
show_status() {
    log "Deployment Status:"
    echo ""
    info "🌐 Services are running at:"
    echo "  • MEV Shield Core API: http://localhost:8080"
    echo "  • Admin Dashboard:     http://localhost:3001"
    echo "  • User Dashboard:      http://localhost:3002"
    echo "  • Nginx Proxy:         http://localhost"
    echo "  • Prometheus:          http://localhost:9091"
    echo "  • Grafana:             http://localhost:3000 (admin/admin)"
    echo ""
    info "📊 Database connections:"
    echo "  • PostgreSQL:          localhost:5432 (mev_user/mev_password)"
    echo "  • Redis:               localhost:6379"
    echo ""
    info "📝 Logs:"
    echo "  • Deployment log:      $LOG_FILE"
    echo "  • Container logs:      docker-compose -f docker-compose.local.yml logs"
    echo ""
    info "🔧 Useful commands:"
    echo "  • Stop services:       docker-compose -f docker-compose.local.yml down"
    echo "  • View logs:           docker-compose -f docker-compose.local.yml logs -f"
    echo "  • Rebuild:             $0 --rebuild"
    echo "  • Clean rebuild:       $0 --clean"
}

# Main execution
main() {
    log "🚀 Starting MEV Shield local deployment..."
    echo ""
    
    # Parse arguments
    case "${1:-}" in
        --clean)
            cleanup --clean
            ;;
        --rebuild)
            cleanup
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --clean     Clean rebuild (removes Docker images)"
            echo "  --rebuild   Rebuild and redeploy"
            echo "  --help      Show this help message"
            exit 0
            ;;
        "")
            # Default behavior
            ;;
        *)
            error "Unknown option: $1"
            exit 1
            ;;
    esac
    
    # Run deployment steps
    check_prerequisites
    
    if [ "${1:-}" = "--clean" ] || [ "${1:-}" = "--rebuild" ]; then
        cleanup "${1:-}"
    fi
    
    build_backend
    build_dashboard
    build_docker_images
    deploy_services
    show_status
    
    log "🎉 MEV Shield deployment completed successfully!"
    echo ""
    warning "First-time setup may take a few minutes for all services to fully initialize."
    info "Check service logs if anything isn't working: docker-compose -f docker-compose.local.yml logs"
}

# Trap to cleanup on script exit
trap 'error "Script interrupted"; exit 1' INT TERM

# Run main function
main "$@"