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
