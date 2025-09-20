#!/bin/bash

# MEV Shield Uniswap Deployment Script
# Version 1.4.0 with Uniswap V3 Integration

set -e

echo "🚀 MEV Shield Uniswap Deployment Script v1.4.0"
echo "=============================================="

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
REMOTE_HOST="dev.mevshield.ai"
REMOTE_PORT="2237"
REMOTE_USER="subbu"
REMOTE_PASSWORD="subbuFuture@2025"
PROJECT_DIR="/opt/mev-shield"
BACKUP_DIR="/opt/mev-shield-backups"
PACKAGE="/tmp/mev-shield-uniswap-v1.4.0.tar.gz"

# Create deployment commands
DEPLOY_COMMANDS=$(cat << 'EOF'
# Create backup
echo "📦 Creating backup..."
sudo mkdir -p /opt/mev-shield-backups
sudo tar -czf /opt/mev-shield-backups/backup-$(date +%Y%m%d_%H%M%S).tar.gz -C /opt/mev-shield . 2>/dev/null || true

# Extract new deployment
echo "📂 Extracting deployment package..."
cd /tmp
tar -xzf mev-shield-uniswap-v1.4.0.tar.gz

# Stop current services
echo "🛑 Stopping current services..."
sudo systemctl stop mev-shield.service 2>/dev/null || true
sudo killall node 2>/dev/null || true

# Update backend
echo "🔄 Updating backend API..."
sudo cp -r backend-mock/* /opt/mev-shield/backend-mock/ 2>/dev/null || true
sudo mkdir -p /opt/mev-shield/backend-mock
sudo cp -r backend-mock/* /opt/mev-shield/backend-mock/

# Update dashboard
echo "🎨 Updating dashboard..."
sudo rm -rf /opt/mev-shield/dashboard/build
sudo cp -r dashboard/build /opt/mev-shield/dashboard/

# Update smart contracts
echo "📝 Updating smart contracts..."
sudo mkdir -p /opt/mev-shield/contracts
sudo cp -r contracts/* /opt/mev-shield/contracts/ 2>/dev/null || true

# Update source files
echo "📁 Updating source files..."
sudo mkdir -p /opt/mev-shield/src
sudo cp -r src/* /opt/mev-shield/src/ 2>/dev/null || true

# Update test files
echo "🧪 Updating test files..."
sudo mkdir -p /opt/mev-shield/test
sudo cp -r test/* /opt/mev-shield/test/ 2>/dev/null || true

# Update scripts
echo "📜 Updating scripts..."
sudo mkdir -p /opt/mev-shield/scripts
sudo cp -r scripts/* /opt/mev-shield/scripts/ 2>/dev/null || true

# Update package files
echo "📦 Updating package files..."
sudo cp package.json /opt/mev-shield/
sudo cp package-lock.json /opt/mev-shield/

# Install dependencies
echo "📚 Installing dependencies..."
cd /opt/mev-shield
sudo npm install --production

cd /opt/mev-shield/backend-mock
sudo npm install --production

# Create systemd service for backend
echo "⚙️ Creating systemd service..."
sudo tee /etc/systemd/system/mev-shield.service > /dev/null <<EOL
[Unit]
Description=MEV Shield Backend Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/mev-shield/backend-mock
ExecStart=/usr/bin/node server.js
Restart=always
Environment=NODE_ENV=production
Environment=PORT=8080

[Install]
WantedBy=multi-user.target
EOL

# Update nginx configuration
echo "🌐 Updating nginx configuration..."
sudo tee /etc/nginx/sites-available/mevshield > /dev/null <<EOL
server {
    listen 80;
    listen [::]:80;
    server_name dev.mevshield.ai;
    return 301 https://\$server_name\$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name dev.mevshield.ai;

    ssl_certificate /etc/letsencrypt/live/dev.mevshield.ai/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/dev.mevshield.ai/privkey.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    
    # Frontend
    root /opt/mev-shield/dashboard/build;
    index index.html;
    
    location / {
        try_files \$uri \$uri/ /index.html;
    }
    
    # Backend API
    location /api/ {
        proxy_pass http://localhost:8080/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
    
    location /auth/ {
        proxy_pass http://localhost:8080/auth/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
    
    location /health {
        proxy_pass http://localhost:8080/health;
        proxy_http_version 1.1;
    }
    
    # WebSocket support for real-time updates
    location /ws {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOL

# Reload services
echo "🔄 Reloading services..."
sudo systemctl daemon-reload
sudo systemctl enable mev-shield.service
sudo systemctl start mev-shield.service
sudo nginx -t && sudo systemctl reload nginx

# Test deployment
echo "🧪 Testing deployment..."
sleep 5
curl -s http://localhost:8080/health > /dev/null && echo "✅ Backend is healthy" || echo "❌ Backend health check failed"
curl -s https://dev.mevshield.ai/health > /dev/null && echo "✅ HTTPS endpoint is healthy" || echo "❌ HTTPS health check failed"

# Test Uniswap endpoints
echo "🦄 Testing Uniswap integration..."
curl -s -X POST http://localhost:8080/api/uniswap/quote \
  -H "Content-Type: application/json" \
  -d '{"tokenIn":"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2","tokenOut":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","amountIn":"1000000000000000000"}' \
  > /dev/null && echo "✅ Uniswap quote endpoint working" || echo "❌ Uniswap quote endpoint failed"

echo ""
echo "✨ Deployment complete!"
echo "Version: 1.4.0 (with Uniswap V3 Integration)"
echo "URL: https://dev.mevshield.ai"
echo ""
echo "📊 Service Status:"
sudo systemctl status mev-shield.service --no-pager | head -10
EOF
)

# Execute remote deployment
echo -e "${BLUE}Connecting to remote server...${NC}"
sshpass -p "$REMOTE_PASSWORD" ssh -p $REMOTE_PORT $REMOTE_USER@$REMOTE_HOST "$DEPLOY_COMMANDS"

echo ""
echo -e "${GREEN}✅ Deployment completed successfully!${NC}"
echo "🌐 Access the application at: https://dev.mevshield.ai"
echo ""
echo "Test accounts:"
echo "  - Admin: admin@mevshield.ai / admin123"
echo "  - User: user@mevshield.ai / user123"
echo "  - Builder: builder@mevshield.ai / builder123"
echo "  - Trader: trader@mevshield.ai / trader123"
echo ""
echo "🦄 New Uniswap Features:"
echo "  - Real-time price quotes"
echo "  - MEV risk analysis"
echo "  - Protected swap execution"
echo "  - Liquidity pool information"