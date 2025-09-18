#!/bin/bash
# MEV Shield Local Deployment Script (Without Docker)
# Deploys all components locally for testing

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 MEV Shield Local Deployment${NC}"
echo "=================================="
echo ""

# Function to check if a port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to wait for service
wait_for_service() {
    local url=$1
    local service=$2
    local max_attempts=30
    local attempt=1
    
    echo -e "${YELLOW}⏳ Waiting for $service to be ready...${NC}"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ $service is ready${NC}"
            return 0
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    echo -e "${RED}❌ $service failed to start${NC}"
    return 1
}

# Check if services are already running
echo -e "${BLUE}📊 Checking existing services...${NC}"

if check_port 3001; then
    echo -e "${GREEN}✅ Admin Dashboard already running on port 3001${NC}"
else
    echo -e "${YELLOW}⚠️  Admin Dashboard not running on port 3001${NC}"
fi

if check_port 3004; then
    echo -e "${GREEN}✅ User Dashboard already running on port 3004${NC}"
else
    echo -e "${YELLOW}⚠️  User Dashboard not running on port 3004${NC}"
fi

if check_port 8080; then
    echo -e "${GREEN}✅ Backend API already running on port 8080${NC}"
else
    echo -e "${YELLOW}⚠️  Backend API not running on port 8080${NC}"
    
    # Check if Rust binary exists
    if [ -f "target/release/mev-shield" ]; then
        echo -e "${BLUE}🔧 Starting Backend API...${NC}"
        
        # Create .env if it doesn't exist
        if [ ! -f ".env" ]; then
            echo -e "${YELLOW}📝 Creating .env from template...${NC}"
            cp .env.template .env
            # Generate a secure JWT secret
            JWT_SECRET=$(openssl rand -base64 64 | tr -d '\n')
            sed -i '' "s|your-super-secret-jwt-key-change-this-in-production-please-use-openssl-rand-base64-64|$JWT_SECRET|" .env 2>/dev/null || \
            sed -i "s|your-super-secret-jwt-key-change-this-in-production-please-use-openssl-rand-base64-64|$JWT_SECRET|" .env
        fi
        
        # Start the backend
        echo -e "${BLUE}🚀 Starting MEV Shield Backend...${NC}"
        nohup ./target/release/mev-shield > logs/backend.log 2>&1 &
        echo $! > .backend.pid
        
        # Wait for backend to be ready
        wait_for_service "http://localhost:8080/health" "Backend API"
    else
        echo -e "${RED}❌ Backend binary not found. Please run: cargo build --release${NC}"
    fi
fi

echo ""
echo -e "${GREEN}🎉 Deployment Summary${NC}"
echo "======================="
echo ""

# Check all services
if check_port 8080; then
    echo -e "${GREEN}✅ Backend API:${NC} http://localhost:8080"
    echo "   - Health Check: http://localhost:8080/health"
    echo "   - API Docs: http://localhost:8080/api/docs"
else
    echo -e "${RED}❌ Backend API: Not running${NC}"
fi

if check_port 3001; then
    echo -e "${GREEN}✅ Admin Dashboard:${NC} http://localhost:3001"
    echo "   - Default Login: admin@mevshield.com / AdminPassword123!"
else
    echo -e "${RED}❌ Admin Dashboard: Not running${NC}"
fi

if check_port 3004; then
    echo -e "${GREEN}✅ User Dashboard:${NC} http://localhost:3004"
    echo "   - Public access available"
else
    echo -e "${RED}❌ User Dashboard: Not running${NC}"
fi

echo ""
echo -e "${BLUE}📚 Security Features Enabled:${NC}"
echo "  • JWT Authentication with role-based access"
echo "  • Argon2id password hashing"
echo "  • Rate limiting (60 req/min per IP)"
echo "  • Input validation and XSS prevention"
echo "  • CORS protection"
echo "  • SQL injection prevention"
echo ""

echo -e "${YELLOW}📋 Quick Commands:${NC}"
echo "  Stop backend:    kill \$(cat .backend.pid)"
echo "  View logs:       tail -f logs/backend.log"
echo "  Test auth:       curl http://localhost:8080/health"
echo "  Run tests:       cargo test"
echo ""

echo -e "${BLUE}🔒 Security Notes:${NC}"
echo "  • Change default admin password immediately"
echo "  • Configure SSL/TLS for production"
echo "  • Review .env settings before production"
echo "  • Enable monitoring and alerting"
echo ""

# Create a simple test script
cat > test-deployment.sh << 'EOF'
#!/bin/bash
# Test MEV Shield Deployment

echo "Testing MEV Shield deployment..."

# Test backend health
echo -n "Testing backend health... "
if curl -s http://localhost:8080/health | grep -q "ok"; then
    echo "✅ OK"
else
    echo "❌ FAILED"
fi

# Test admin dashboard
echo -n "Testing admin dashboard... "
if curl -s http://localhost:3001 | grep -q "MEV Shield"; then
    echo "✅ OK"
else
    echo "❌ FAILED"
fi

# Test user dashboard
echo -n "Testing user dashboard... "
if curl -s http://localhost:3004 | grep -q "MEV Shield"; then
    echo "✅ OK"
else
    echo "❌ FAILED"
fi

# Test authentication endpoint
echo -n "Testing authentication... "
response=$(curl -s -X POST http://localhost:8080/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"admin@mevshield.com","password":"AdminPassword123!"}' \
    2>/dev/null)

if echo "$response" | grep -q "token"; then
    echo "✅ OK"
else
    echo "❌ FAILED"
fi

echo ""
echo "Deployment test complete!"
EOF

chmod +x test-deployment.sh

echo -e "${GREEN}✨ Local deployment complete!${NC}"
echo ""
echo "Run ./test-deployment.sh to verify all services"