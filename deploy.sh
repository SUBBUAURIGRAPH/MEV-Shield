#!/bin/bash

# MEV Shield Build and Deployment Script
# Version: 1.0.0

set -e

echo "🛡️ MEV Shield Build & Deployment Script"
echo "========================================"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Check prerequisites
check_prerequisites() {
    echo "Checking prerequisites..."
    
    # Check Rust
    if command -v cargo &> /dev/null; then
        print_status "Rust/Cargo found: $(cargo --version)"
    else
        print_error "Rust/Cargo not found. Please install from https://rustup.rs/"
        exit 1
    fi
    
    # Check Docker
    if command -v docker &> /dev/null; then
        print_status "Docker found: $(docker --version)"
    else
        print_warning "Docker not found. Docker deployment will be skipped."
    fi
    
    # Check PostgreSQL client
    if command -v psql &> /dev/null; then
        print_status "PostgreSQL client found"
    else
        print_warning "PostgreSQL client not found. Database setup will be skipped."
    fi
}

# Build the project
build_project() {
    echo ""
    echo "Building MEV Shield..."
    
    # Clean previous builds
    print_status "Cleaning previous builds..."
    cargo clean
    
    # Run tests
    print_status "Running tests..."
    cargo test --quiet
    
    # Build release binary
    print_status "Building release binary..."
    cargo build --release
    
    print_status "Build completed successfully!"
}

# Setup database
setup_database() {
    echo ""
    echo "Setting up database..."
    
    if [ -z "$DATABASE_URL" ]; then
        print_warning "DATABASE_URL not set. Using default: postgresql://mevshield:mevshield123@localhost/mevshield"
        export DATABASE_URL="postgresql://mevshield:mevshield123@localhost/mevshield"
    fi
    
    # Run migrations
    if [ -f "migrations/001_initial_schema.sql" ]; then
        print_status "Running database migrations..."
        psql "$DATABASE_URL" < migrations/001_initial_schema.sql 2>/dev/null || print_warning "Migration may have already been applied"
    fi
    
    print_status "Database setup completed!"
}

# Docker deployment
docker_deploy() {
    echo ""
    echo "Docker deployment..."
    
    if ! command -v docker-compose &> /dev/null; then
        print_warning "docker-compose not found. Skipping Docker deployment."
        return
    fi
    
    print_status "Building Docker images..."
    docker-compose build
    
    print_status "Starting services..."
    docker-compose up -d
    
    # Wait for services to be healthy
    echo "Waiting for services to be healthy..."
    sleep 10
    
    # Check service status
    docker-compose ps
    
    print_status "Docker deployment completed!"
}

# Local deployment
local_deploy() {
    echo ""
    echo "Local deployment..."
    
    # Copy config if not exists
    if [ ! -f "config.toml" ]; then
        print_status "Creating config file..."
        cp config.toml.example config.toml 2>/dev/null || print_warning "Using default config"
    fi
    
    # Start Redis if available
    if command -v redis-server &> /dev/null; then
        print_status "Starting Redis..."
        redis-server --daemonize yes
    fi
    
    # Run the application
    print_status "Starting MEV Shield..."
    ./target/release/mev-shield &
    
    print_status "MEV Shield is running!"
    echo "API available at: http://localhost:8080"
    echo "Metrics available at: http://localhost:9090"
}

# Main menu
show_menu() {
    echo ""
    echo "Select deployment option:"
    echo "1) Local development"
    echo "2) Docker deployment"
    echo "3) Production deployment"
    echo "4) Run tests only"
    echo "5) Clean build"
    echo "6) Exit"
    
    read -p "Enter choice [1-6]: " choice
    
    case $choice in
        1)
            check_prerequisites
            build_project
            setup_database
            local_deploy
            ;;
        2)
            check_prerequisites
            docker_deploy
            ;;
        3)
            check_prerequisites
            build_project
            setup_database
            echo "Production deployment requires additional configuration."
            echo "Please refer to the deployment documentation."
            ;;
        4)
            cargo test
            ;;
        5)
            cargo clean
            print_status "Build cleaned!"
            ;;
        6)
            exit 0
            ;;
        *)
            print_error "Invalid choice!"
            show_menu
            ;;
    esac
}

# Main execution
main() {
    # Parse command line arguments
    case "${1:-}" in
        build)
            check_prerequisites
            build_project
            ;;
        test)
            cargo test
            ;;
        deploy)
            check_prerequisites
            build_project
            setup_database
            local_deploy
            ;;
        docker)
            check_prerequisites
            docker_deploy
            ;;
        clean)
            cargo clean
            ;;
        *)
            show_menu
            ;;
    esac
}

# Run main function
main "$@"
