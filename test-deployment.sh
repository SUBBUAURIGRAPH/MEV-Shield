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
