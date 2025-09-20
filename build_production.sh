#!/bin/bash

# MEV Shield Production Build Script
# Creates production-ready deployment package

set -e

echo "🛡️  MEV Shield - Production Build"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Create build directory
BUILD_DIR="mev-shield-production"
print_status "Creating production build directory..."
rm -rf $BUILD_DIR
mkdir -p $BUILD_DIR

# Build frontend
print_status "Building frontend production bundle..."
cd dashboard
npm run build
print_success "Frontend build completed"
cd ..

# Copy frontend build
cp -r dashboard/build $BUILD_DIR/frontend
print_success "Frontend files copied"

# Prepare backend files
print_status "Preparing backend files..."
mkdir -p $BUILD_DIR/backend-mock
mkdir -p $BUILD_DIR/backend-live

# Copy backend files (excluding node_modules)
rsync -av --exclude 'node_modules' --exclude '*.log' backend-mock/ $BUILD_DIR/backend-mock/
rsync -av --exclude 'node_modules' --exclude '*.log' backend-live/ $BUILD_DIR/backend-live/

print_success "Backend files copied"

# Create Docker Compose configuration
print_status "Creating production Docker Compose..."
cat > $BUILD_DIR/docker-compose.yml << 'EOF'
version: '3.8'

services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./frontend:/usr/share/nginx/html:ro
      - ./ssl:/etc/ssl/certs:ro
      - ./logs:/var/log/nginx
    depends_on:
      - backend-mock
      - backend-live
    restart: unless-stopped

  backend-mock:
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

  backend-live:
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

networks:
  default:
    driver: bridge

volumes:
  logs:
EOF

# Create nginx configuration
print_status "Creating nginx configuration..."
cat > $BUILD_DIR/nginx.conf << 'EOF'
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
    upstream backend_mock {
        server backend-mock:8080;
    }

    upstream backend_live {
        server backend-live:8096;
    }

    # HTTP to HTTPS redirect
    server {
        listen 80;
        server_name dev.mevshield.ai localhost;
        return 301 https://$server_name$request_uri;
    }

    # HTTPS server
    server {
        listen 443 ssl http2;
        server_name dev.mevshield.ai localhost;

        # SSL configuration
        ssl_certificate /etc/ssl/certs/fullchain.pem;
        ssl_certificate_key /etc/ssl/certs/privkey.pem;
        ssl_session_timeout 1d;
        ssl_session_cache shared:SSL:50m;
        ssl_session_tickets off;

        # Modern configuration
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
        ssl_prefer_server_ciphers off;

        # Security headers
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
        add_header Referrer-Policy "strict-origin-when-cross-origin";

        # API routes - Live MEV Protection
        location /api/live/ {
            limit_req zone=api burst=20 nodelay;
            proxy_pass http://backend_live/;
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
            proxy_pass http://backend_mock/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_read_timeout 60s;
            proxy_connect_timeout 10s;
        }

        # WebSocket support for live updates
        location /ws/ {
            proxy_pass http://backend_live/;
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
            root /usr/share/nginx/html;
            index index.html;
            try_files $uri $uri/ /index.html;
        }

        # Health check
        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }

        # Static assets caching
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            root /usr/share/nginx/html;
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }
}
EOF

# Create Dockerfiles
print_status "Creating Dockerfiles..."

# Backend Mock Dockerfile
cat > $BUILD_DIR/backend-mock/Dockerfile << 'EOF'
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production --silent

COPY . .
RUN addgroup -g 1001 -S nodejs && adduser -S nodejs -u 1001

USER nodejs
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

CMD ["node", "server.js"]
EOF

# Backend Live Dockerfile
cat > $BUILD_DIR/backend-live/Dockerfile << 'EOF'
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production --silent

COPY . .
RUN addgroup -g 1001 -S nodejs && adduser -S nodejs -u 1001

USER nodejs
EXPOSE 8096

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8096/health || exit 1

CMD ["node", "server.js"]
EOF

# Create deployment script
print_status "Creating deployment script..."
cat > $BUILD_DIR/deploy.sh << 'EOF'
#!/bin/bash

set -e

echo "🛡️  MEV Shield Production Deployment"
echo "=================================="

# Create directories
mkdir -p logs ssl

# Generate self-signed SSL certificates if not present
if [ ! -f ssl/privkey.pem ] || [ ! -f ssl/fullchain.pem ]; then
    echo "🔒 Generating SSL certificates..."
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout ssl/privkey.pem \
        -out ssl/fullchain.pem \
        -subj '/C=US/ST=CA/L=SF/O=MEV Shield/CN=dev.mevshield.ai' \
        2>/dev/null
    echo "✅ SSL certificates generated"
fi

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose not found. Please install Docker Compose first."
    exit 1
fi

# Stop existing containers
echo "🛑 Stopping existing containers..."
docker-compose down 2>/dev/null || true

# Build and start services
echo "🚀 Building and starting MEV Shield..."
docker-compose up --build -d

# Wait for services to start
echo "⏳ Waiting for services to start..."
sleep 30

# Health checks
echo "🔍 Performing health checks..."
for i in {1..10}; do
    if curl -f -s http://localhost/health > /dev/null 2>&1; then
        echo "✅ Frontend is healthy"
        break
    fi
    if [ $i -eq 10 ]; then
        echo "❌ Frontend health check failed"
    fi
    sleep 3
done

for i in {1..10}; do
    if curl -f -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ Backend Mock is healthy"
        break
    fi
    if [ $i -eq 10 ]; then
        echo "❌ Backend Mock health check failed"
    fi
    sleep 3
done

for i in {1..10}; do
    if curl -f -s http://localhost:8096/health > /dev/null 2>&1; then
        echo "✅ Backend Live is healthy"
        break
    fi
    if [ $i -eq 10 ]; then
        echo "❌ Backend Live health check failed"
    fi
    sleep 3
done

echo ""
echo "🎉 MEV Shield deployment completed!"
echo "================================"
echo "🌐 Access the platform:"
echo "  HTTPS: https://localhost (or your domain)"
echo "  HTTP:  http://localhost (redirects to HTTPS)"
echo ""
echo "📊 API Endpoints:"
echo "  Mock API: http://localhost:8080"
echo "  Live API: http://localhost:8096"
echo ""
echo "📋 Management:"
echo "  View logs: docker-compose logs -f"
echo "  Stop: docker-compose down"
echo "  Restart: docker-compose restart"
echo ""
echo "🔑 Test Accounts:"
echo "  admin@mevshield.ai / admin123"
echo "  user@mevshield.ai / user123"
echo "  builder@mevshield.ai / builder123"
echo "  trader@mevshield.ai / trader123"
EOF

chmod +x $BUILD_DIR/deploy.sh

# Create environment file
print_status "Creating environment configuration..."
cat > $BUILD_DIR/.env << 'EOF'
# MEV Shield Production Environment
NODE_ENV=production
SIMULATE_ACTIVITY=true

# Backend Mock Configuration
MOCK_PORT=8080

# Backend Live Configuration
LIVE_PORT=8096

# SSL Configuration
SSL_CERT_PATH=/etc/ssl/certs/fullchain.pem
SSL_KEY_PATH=/etc/ssl/certs/privkey.pem

# Logging
LOG_LEVEL=info
EOF

# Create README for deployment
print_status "Creating deployment README..."
cat > $BUILD_DIR/README.md << 'EOF'
# MEV Shield Production Deployment

This package contains everything needed to deploy MEV Shield in production.

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ RAM
- 20GB+ disk space

## Quick Start

1. Extract this package to your server
2. Run the deployment script:
   ```bash
   chmod +x deploy.sh
   ./deploy.sh
   ```

## Services

- **Frontend**: React application served by Nginx
- **Backend Mock**: Node.js API server (port 8080)
- **Backend Live**: Real-time MEV protection (port 8096)
- **Nginx**: Reverse proxy with SSL termination

## URLs

- Main Application: https://localhost
- Admin Dashboard: https://localhost/admin
- User Dashboard: https://localhost/dashboard
- Builder Dashboard: https://localhost/builder
- Trader Dashboard: https://localhost/trader

## API Endpoints

- Mock API: https://localhost/api/
- Live Protection: https://localhost/api/live/
- WebSocket: wss://localhost/ws/

## Management Commands

```bash
# View logs
docker-compose logs -f

# Stop all services
docker-compose down

# Restart specific service
docker-compose restart [service-name]

# Update and restart
git pull  # if using git
docker-compose up --build -d
```

## SSL Certificates

The deployment script generates self-signed certificates. For production:

1. Replace `ssl/privkey.pem` and `ssl/fullchain.pem` with real certificates
2. Restart nginx: `docker-compose restart nginx`

## Test Accounts

- admin@mevshield.ai / admin123
- user@mevshield.ai / user123
- builder@mevshield.ai / builder123
- trader@mevshield.ai / trader123

## Troubleshooting

1. Check container status: `docker-compose ps`
2. View specific logs: `docker-compose logs [service-name]`
3. Check ports: `netstat -tlnp | grep -E '(80|443|8080|8096)'`
4. Restart all: `docker-compose down && docker-compose up -d`
EOF

# Create package archive
print_status "Creating deployment package..."
tar -czf mev-shield-production.tar.gz $BUILD_DIR
print_success "Production package created: mev-shield-production.tar.gz"

# Show package contents
print_status "Package contents:"
echo "├── docker-compose.yml"
echo "├── nginx.conf"
echo "├── deploy.sh"
echo "├── .env"
echo "├── README.md"
echo "├── frontend/ (React build)"
echo "├── backend-mock/ (Node.js API)"
echo "├── backend-live/ (MEV Protection)"
echo "└── ssl/ (will be created)"

echo ""
print_success "🎉 Production build completed!"
echo "=============================="
echo "📦 Package: mev-shield-production.tar.gz"
echo "📁 Directory: $BUILD_DIR/"
echo ""
echo "🚀 To deploy:"
echo "1. Copy mev-shield-production.tar.gz to your server"
echo "2. Extract: tar -xzf mev-shield-production.tar.gz"
echo "3. Deploy: cd $BUILD_DIR && ./deploy.sh"
echo ""
echo "🌐 Access at: https://your-domain (or https://localhost)"
print_success "Build successful! 🎉"