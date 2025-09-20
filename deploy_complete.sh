#!/bin/bash

# MEV Shield Complete Deployment Script
# Deploys the full MEV Shield platform to remote server

set -e

SERVER="dev.mevshield.ai"
USER="root"
REMOTE_DIR="/opt/mev-shield"
PROJECT_NAME="MEV-Shield"

echo "🛡️  MEV Shield - Complete Deployment"
echo "=================================="
echo "Target: $SERVER"
echo "User: $USER"
echo "Directory: $REMOTE_DIR"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we can connect to the server
print_status "Checking server connectivity..."
if ! ssh -o ConnectTimeout=10 $USER@$SERVER "echo 'Connected successfully'" 2>/dev/null; then
    print_error "Cannot connect to $SERVER. Please check:"
    echo "1. Server is accessible"
    echo "2. SSH key is configured"
    echo "3. User $USER has access"
    exit 1
fi
print_success "Server connectivity verified"

# Create backup of existing deployment
print_status "Creating backup of existing deployment..."
BACKUP_NAME="mev-shield-backup-$(date +%Y%m%d_%H%M%S)"
ssh $USER@$SERVER "
    if [ -d '$REMOTE_DIR' ]; then
        sudo cp -r $REMOTE_DIR /opt/$BACKUP_NAME
        echo 'Backup created: /opt/$BACKUP_NAME'
    else
        echo 'No existing deployment found'
    fi
"

# Stop existing services
print_status "Stopping existing services..."
ssh $USER@$SERVER "
    sudo systemctl stop mev-shield-frontend || true
    sudo systemctl stop mev-shield-backend || true
    sudo systemctl stop mev-shield-live || true
    sudo pkill -f 'node.*mev' || true
    sudo pkill -f 'npm.*start' || true
    echo 'Services stopped'
"

# Create project directory
print_status "Setting up remote directory..."
ssh $USER@$SERVER "
    sudo mkdir -p $REMOTE_DIR
    sudo chown -R $USER:$USER $REMOTE_DIR
    cd $REMOTE_DIR
    
    # Create subdirectories
    mkdir -p logs
    mkdir -p ssl
    mkdir -p backups
    mkdir -p config
"

# Build production bundles locally
print_status "Building production bundles..."

# Build frontend
cd dashboard
npm run build
print_success "Frontend built successfully"

# Build backend packages
cd ../backend-mock
npm install --production
cd ../backend-live
npm install --production
print_success "Backend packages prepared"

cd ..

# Create Docker configuration
print_status "Creating Docker configuration..."
cat > docker-compose.production.yml << 'EOF'
version: '3.8'

services:
  mev-shield-frontend:
    build:
      context: ./dashboard
      dockerfile: Dockerfile.production
    ports:
      - "3000:80"
    environment:
      - NODE_ENV=production
      - REACT_APP_API_URL=https://dev.mevshield.ai/api
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:80/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  mev-shield-backend:
    build:
      context: ./backend-mock
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - NODE_ENV=production
      - PORT=8080
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  mev-shield-live:
    build:
      context: ./backend-live
      dockerfile: Dockerfile
    ports:
      - "8096:8096"
    environment:
      - NODE_ENV=production
      - PORT=8096
      - SIMULATE_ACTIVITY=true
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8096/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
      - ./logs:/var/log/nginx
    depends_on:
      - mev-shield-frontend
      - mev-shield-backend
      - mev-shield-live
    restart: unless-stopped

networks:
  default:
    driver: bridge
EOF

# Create Dockerfile for frontend
print_status "Creating frontend Dockerfile..."
cat > dashboard/Dockerfile.production << 'EOF'
FROM node:18-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production --silent

COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/build /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
EOF

# Create Dockerfile for backend-mock
print_status "Creating backend mock Dockerfile..."
cat > backend-mock/Dockerfile << 'EOF'
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production --silent

COPY . .

USER node
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["node", "server.js"]
EOF

# Create Dockerfile for backend-live
print_status "Creating backend live Dockerfile..."
cat > backend-live/Dockerfile << 'EOF'
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production --silent

COPY . .

USER node
EXPOSE 8096

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8096/health || exit 1

CMD ["node", "server.js"]
EOF

# Create nginx configuration
print_status "Creating nginx configuration..."
cat > nginx.conf << 'EOF'
events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Logging
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;
    error_log /var/log/nginx/error.log warn;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1000;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=app:10m rate=5r/s;

    # Upstream backends
    upstream mev_backend {
        server mev-shield-backend:8080;
    }

    upstream mev_live {
        server mev-shield-live:8096;
    }

    upstream mev_frontend {
        server mev-shield-frontend:80;
    }

    # HTTP to HTTPS redirect
    server {
        listen 80;
        server_name dev.mevshield.ai;
        return 301 https://$server_name$request_uri;
    }

    # Main HTTPS server
    server {
        listen 443 ssl http2;
        server_name dev.mevshield.ai;

        # SSL configuration
        ssl_certificate /etc/nginx/ssl/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/privkey.pem;
        ssl_session_timeout 1d;
        ssl_session_cache shared:SSL:50m;
        ssl_session_tickets off;

        # Modern configuration
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
        ssl_prefer_server_ciphers off;

        # HSTS
        add_header Strict-Transport-Security "max-age=63072000" always;

        # Security headers
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
        add_header Referrer-Policy "strict-origin-when-cross-origin";

        # API routes - Live MEV Protection
        location /api/live/ {
            limit_req zone=api burst=20 nodelay;
            proxy_pass http://mev_live/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_read_timeout 60s;
            proxy_connect_timeout 10s;
        }

        # API routes - Mock Backend
        location /api/ {
            limit_req zone=api burst=20 nodelay;
            proxy_pass http://mev_backend/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_read_timeout 60s;
            proxy_connect_timeout 10s;
        }

        # WebSocket support for live MEV updates
        location /ws/ {
            proxy_pass http://mev_live/;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_read_timeout 86400;
        }

        # Frontend application
        location / {
            limit_req zone=app burst=10 nodelay;
            proxy_pass http://mev_frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # Handle React Router
            try_files $uri $uri/ @fallback;
        }

        location @fallback {
            proxy_pass http://mev_frontend/index.html;
        }

        # Health checks
        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }

        # Static assets
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
            proxy_pass http://mev_frontend;
        }
    }
}
EOF

# Create deployment script for server
print_status "Creating server deployment script..."
cat > deploy_server.sh << 'EOF'
#!/bin/bash

set -e

REMOTE_DIR="/opt/mev-shield"
LOG_FILE="$REMOTE_DIR/logs/deployment.log"

echo "🛡️  MEV Shield Server Deployment" | tee -a $LOG_FILE
echo "===============================" | tee -a $LOG_FILE
echo "$(date): Starting deployment" | tee -a $LOG_FILE

# Install Docker if not present
if ! command -v docker &> /dev/null; then
    echo "Installing Docker..." | tee -a $LOG_FILE
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
fi

# Install Docker Compose if not present
if ! command -v docker-compose &> /dev/null; then
    echo "Installing Docker Compose..." | tee -a $LOG_FILE
    sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    sudo chmod +x /usr/local/bin/docker-compose
fi

cd $REMOTE_DIR

# Stop existing containers
echo "Stopping existing containers..." | tee -a $LOG_FILE
sudo docker-compose -f docker-compose.production.yml down || true

# Clean up old images
echo "Cleaning up old images..." | tee -a $LOG_FILE
sudo docker system prune -f

# Build and start services
echo "Building and starting services..." | tee -a $LOG_FILE
sudo docker-compose -f docker-compose.production.yml up --build -d

# Wait for services to be ready
echo "Waiting for services to start..." | tee -a $LOG_FILE
sleep 30

# Health checks
echo "Performing health checks..." | tee -a $LOG_FILE
for service in mev-shield-frontend mev-shield-backend mev-shield-live; do
    if sudo docker-compose -f docker-compose.production.yml ps | grep -q "$service.*Up"; then
        echo "✅ $service is running" | tee -a $LOG_FILE
    else
        echo "❌ $service failed to start" | tee -a $LOG_FILE
    fi
done

# Check nginx
if sudo docker-compose -f docker-compose.production.yml ps | grep -q "nginx.*Up"; then
    echo "✅ Nginx is running" | tee -a $LOG_FILE
else
    echo "❌ Nginx failed to start" | tee -a $LOG_FILE
fi

echo "$(date): Deployment completed" | tee -a $LOG_FILE
echo "🚀 MEV Shield is now live at https://dev.mevshield.ai" | tee -a $LOG_FILE
EOF

chmod +x deploy_server.sh

# Copy all files to remote server
print_status "Copying files to remote server..."
rsync -avz --progress \
    --exclude 'node_modules' \
    --exclude '.git' \
    --exclude 'logs' \
    --exclude '*.log' \
    ./ $USER@$SERVER:$REMOTE_DIR/

print_success "Files copied successfully"

# Copy SSL certificates if they exist locally
if [ -d "ssl" ]; then
    print_status "Copying SSL certificates..."
    scp -r ssl/* $USER@$SERVER:$REMOTE_DIR/ssl/
    print_success "SSL certificates copied"
else
    print_warning "No local SSL certificates found. Generating self-signed certificates..."
    ssh $USER@$SERVER "
        cd $REMOTE_DIR/ssl
        sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
            -keyout privkey.pem \
            -out fullchain.pem \
            -subj '/C=US/ST=CA/L=SF/O=MEV Shield/CN=dev.mevshield.ai'
        sudo chown -R $USER:$USER /opt/mev-shield/ssl
        echo 'Self-signed certificates generated'
    "
fi

# Execute deployment on remote server
print_status "Executing deployment on remote server..."
ssh $USER@$SERVER "cd $REMOTE_DIR && chmod +x deploy_server.sh && ./deploy_server.sh"

# Verify deployment
print_status "Verifying deployment..."
sleep 10

# Test endpoints
print_status "Testing endpoints..."
if curl -f -s https://dev.mevshield.ai/health > /dev/null; then
    print_success "✅ Frontend is accessible"
else
    print_warning "⚠️  Frontend accessibility check failed"
fi

if curl -f -s https://dev.mevshield.ai/api/health > /dev/null; then
    print_success "✅ Backend API is accessible"
else
    print_warning "⚠️  Backend API accessibility check failed"
fi

if curl -f -s https://dev.mevshield.ai/api/live/health > /dev/null; then
    print_success "✅ Live MEV Protection API is accessible"
else
    print_warning "⚠️  Live MEV Protection API accessibility check failed"
fi

# Display deployment summary
echo ""
print_success "🚀 MEV Shield Deployment Complete!"
echo "=================================="
echo "🌐 Website: https://dev.mevshield.ai"
echo "📊 API: https://dev.mevshield.ai/api/"
echo "🛡️  Live Protection: https://dev.mevshield.ai/api/live/"
echo "📈 WebSocket: wss://dev.mevshield.ai/ws/"
echo ""
echo "📱 Dashboards:"
echo "  👨‍💼 Admin: https://dev.mevshield.ai/admin"
echo "  👤 User: https://dev.mevshield.ai/dashboard"
echo "  🏗️  Builder: https://dev.mevshield.ai/builder"
echo "  📈 Trader: https://dev.mevshield.ai/trader"
echo ""
echo "🔒 Test Accounts:"
echo "  admin@mevshield.ai / admin123"
echo "  user@mevshield.ai / user123"
echo "  builder@mevshield.ai / builder123"
echo "  trader@mevshield.ai / trader123"
echo ""
echo "📋 Logs location: $REMOTE_DIR/logs/"
echo "🔄 Restart: ssh $USER@$SERVER 'cd $REMOTE_DIR && docker-compose -f docker-compose.production.yml restart'"
echo ""
print_success "Deployment successful! 🎉"

