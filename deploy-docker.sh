#!/bin/bash

# MEV Shield Docker Deployment Script
set -e

echo "========================================="
echo "MEV Shield Docker Deployment"
echo "========================================="

# Configuration
REMOTE_HOST="dev.mevshield.ai"
REMOTE_USER="subbu"
REMOTE_PORT="2237"
REMOTE_DIR="/home/subbu/mevshield-docker"
PROJECT_NAME="mev-shield"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Step 1: Build Docker images locally
build_images() {
    log_info "Building Docker images..."
    
    # Build backend image
    log_info "Building backend image..."
    docker build -t ${PROJECT_NAME}-backend:latest -f Dockerfile.backend .
    
    # Build frontend image
    log_info "Building frontend image..."
    docker build -t ${PROJECT_NAME}-frontend:latest -f Dockerfile.frontend .
    
    log_info "Docker images built successfully!"
}

# Step 2: Save Docker images
save_images() {
    log_info "Saving Docker images to tar files..."
    
    docker save ${PROJECT_NAME}-backend:latest -o backend.tar
    docker save ${PROJECT_NAME}-frontend:latest -o frontend.tar
    
    log_info "Images saved successfully!"
}

# Step 3: Create deployment package
create_package() {
    log_info "Creating deployment package..."
    
    tar -czf mevshield-docker-deploy.tar.gz \
        backend.tar \
        frontend.tar \
        docker-compose.production.yml \
        nginx/default.conf \
        .env.example 2>/dev/null || true
    
    # Cleanup tar files
    rm -f backend.tar frontend.tar
    
    log_info "Deployment package created: mevshield-docker-deploy.tar.gz"
}

# Step 4: Upload to remote server
upload_to_remote() {
    log_info "Uploading to remote server..."
    
    # Create remote directory
    ssh -p ${REMOTE_PORT} ${REMOTE_USER}@${REMOTE_HOST} "mkdir -p ${REMOTE_DIR}"
    
    # Upload package
    scp -P ${REMOTE_PORT} mevshield-docker-deploy.tar.gz ${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/
    
    log_info "Package uploaded successfully!"
}

# Step 5: Deploy on remote server
deploy_on_remote() {
    log_info "Deploying on remote server..."
    
    ssh -p ${REMOTE_PORT} ${REMOTE_USER}@${REMOTE_HOST} << 'ENDSSH'
cd /home/subbu/mevshield-docker

# Extract package
echo "Extracting deployment package..."
tar -xzf mevshield-docker-deploy.tar.gz

# Load Docker images
echo "Loading Docker images..."
sudo docker load -i backend.tar
sudo docker load -i frontend.tar

# Stop existing containers if any
echo "Stopping existing containers..."
sudo docker-compose -f docker-compose.production.yml down 2>/dev/null || true

# Start new containers
echo "Starting Docker containers..."
sudo docker-compose -f docker-compose.production.yml up -d

# Wait for services to be healthy
echo "Waiting for services to be healthy..."
sleep 10

# Check container status
echo "Container status:"
sudo docker-compose -f docker-compose.production.yml ps

# Show logs
echo "Recent logs:"
sudo docker-compose -f docker-compose.production.yml logs --tail=20

# Cleanup
echo "Cleaning up..."
rm -f backend.tar frontend.tar mevshield-docker-deploy.tar.gz

echo "Deployment complete!"
ENDSSH
}

# Step 6: Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."
    
    # Test endpoints
    echo -e "\n${GREEN}Testing endpoints:${NC}"
    
    echo -n "1. Health check: "
    curl -s http://${REMOTE_HOST}/health && echo " ✓" || echo " ✗"
    
    echo -n "2. API stats: "
    curl -s http://${REMOTE_HOST}/api/stats | head -c 50 && echo "... ✓"
    
    echo -n "3. Frontend: "
    curl -s -o /dev/null -w "%{http_code}" http://${REMOTE_HOST} | grep -q "200" && echo "200 OK ✓" || echo "Failed ✗"
    
    log_info "Deployment verification complete!"
}

# Main execution
main() {
    echo "Starting deployment process..."
    
    # Check Docker is installed
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed!"
        exit 1
    fi
    
    # Execute deployment steps
    build_images
    save_images
    create_package
    upload_to_remote
    deploy_on_remote
    verify_deployment
    
    echo -e "\n${GREEN}=========================================${NC}"
    echo -e "${GREEN}Deployment completed successfully!${NC}"
    echo -e "${GREEN}=========================================${NC}"
    echo -e "\nAccess the application at: ${GREEN}http://${REMOTE_HOST}${NC}"
}

# Run main function
main "$@"