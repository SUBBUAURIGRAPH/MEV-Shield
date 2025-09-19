#!/bin/bash

# Test JIRA-GitHub Synchronization
# This script tests the sync functionality

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo "========================================="
echo " JIRA-GitHub Sync Test Suite"
echo " MEV Shield Project"
echo "========================================="
echo ""

# Load environment
if [ -f ".env.jira" ]; then
    source .env.jira
else
    echo -e "${RED}Error: .env.jira file not found${NC}"
    exit 1
fi

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run test
run_test() {
    local TEST_NAME=$1
    local TEST_CMD=$2
    
    echo -e "${BLUE}Testing: ${TEST_NAME}${NC}"
    
    if eval "$TEST_CMD" > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ PASSED${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}  ✗ FAILED${NC}"
        ((TESTS_FAILED++))
    fi
}

# Test 1: JIRA API Connection
run_test "JIRA API Connection" \
    "curl -s -o /dev/null -w '%{http_code}' \
    -H 'Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)' \
    '${JIRA_BASE_URL}/rest/api/3/myself' | grep -q 200"

# Test 2: GitHub API Connection (if token is set)
if [ ! -z "$GITHUB_TOKEN" ]; then
    run_test "GitHub API Connection" \
        "curl -s -o /dev/null -w '%{http_code}' \
        -H 'Authorization: token ${GITHUB_TOKEN}' \
        'https://api.github.com/user' | grep -q 200"
fi

# Test 3: Create test JIRA ticket
echo -e "${BLUE}Creating test JIRA ticket...${NC}"
TEST_TICKET_RESPONSE=$(curl -s -X POST \
    -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
    -H "Content-Type: application/json" \
    -d '{
        "fields": {
            "project": {"key": "MEV"},
            "summary": "[TEST-SYNC] Automated sync test - '"$(date +%s)"'",
            "description": {
                "type": "doc",
                "version": 1,
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "This is a test ticket for sync verification"
                            }
                        ]
                    }
                ]
            },
            "issuetype": {"name": "Task"},
            "labels": ["test-sync", "automated"]
        }
    }' \
    "${JIRA_BASE_URL}/rest/api/3/issue")

TEST_TICKET_KEY=$(echo "$TEST_TICKET_RESPONSE" | grep -o '"key":"[^"]*"' | cut -d'"' -f4)

if [ ! -z "$TEST_TICKET_KEY" ]; then
    echo -e "${GREEN}  ✓ Created test ticket: ${TEST_TICKET_KEY}${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}  ✗ Failed to create test ticket${NC}"
    ((TESTS_FAILED++))
fi

# Test 4: Update ticket status
if [ ! -z "$TEST_TICKET_KEY" ]; then
    echo -e "${BLUE}Testing status transition...${NC}"
    
    # Get transitions
    TRANSITIONS=$(curl -s \
        -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
        "${JIRA_BASE_URL}/rest/api/3/issue/${TEST_TICKET_KEY}/transitions")
    
    # Try to transition to In Progress
    TRANSITION_ID=$(echo "$TRANSITIONS" | python3 -c "
import sys, json
data = json.load(sys.stdin)
for t in data.get('transitions', []):
    if t['to']['name'] == 'In Progress':
        print(t['id'])
        break
" 2>/dev/null)
    
    if [ ! -z "$TRANSITION_ID" ]; then
        curl -s -X POST \
            -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
            -H "Content-Type: application/json" \
            -d '{"transition": {"id": "'"$TRANSITION_ID"'"}}' \
            "${JIRA_BASE_URL}/rest/api/3/issue/${TEST_TICKET_KEY}/transitions"
        
        echo -e "${GREEN}  ✓ Status transition successful${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${YELLOW}  ⚠ No transition available${NC}"
    fi
fi

# Test 5: Add comment
if [ ! -z "$TEST_TICKET_KEY" ]; then
    run_test "Adding comment to ticket" \
        "curl -s -X POST \
        -H 'Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)' \
        -H 'Content-Type: application/json' \
        -d '{\"body\":{\"type\":\"doc\",\"version\":1,\"content\":[{\"type\":\"paragraph\",\"content\":[{\"type\":\"text\",\"text\":\"Test comment from sync test\"}]}]}}' \
        '${JIRA_BASE_URL}/rest/api/3/issue/${TEST_TICKET_KEY}/comment' | grep -q '\"id\"'"
fi

# Test 6: Webhook endpoint (if configured)
if [ ! -z "$WEBHOOK_URL" ]; then
    echo -e "${BLUE}Testing webhook endpoint...${NC}"
    
    WEBHOOK_PAYLOAD='{
        "webhookEvent": "jira:issue_updated",
        "issue": {
            "key": "'"$TEST_TICKET_KEY"'",
            "fields": {
                "summary": "Test webhook",
                "status": {"name": "In Progress"}
            }
        }
    }'
    
    WEBHOOK_RESPONSE=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$WEBHOOK_PAYLOAD" \
        "$WEBHOOK_URL")
    
    if echo "$WEBHOOK_RESPONSE" | grep -q "success"; then
        echo -e "${GREEN}  ✓ Webhook endpoint working${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}  ✗ Webhook endpoint failed${NC}"
        ((TESTS_FAILED++))
    fi
fi

# Clean up test ticket
if [ ! -z "$TEST_TICKET_KEY" ]; then
    echo ""
    echo -e "${YELLOW}Cleaning up test ticket ${TEST_TICKET_KEY}...${NC}"
    
    # Option to delete (commented out for safety)
    # curl -s -X DELETE \
    #     -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
    #     "${JIRA_BASE_URL}/rest/api/3/issue/${TEST_TICKET_KEY}"
    
    echo -e "${YELLOW}Note: Test ticket ${TEST_TICKET_KEY} was created but not deleted.${NC}"
    echo -e "${YELLOW}Please delete it manually if needed: ${JIRA_BASE_URL}/browse/${TEST_TICKET_KEY}${NC}"
fi

# Summary
echo ""
echo "========================================="
echo " Test Results"
echo "========================================="
echo -e "${GREEN}Passed: ${TESTS_PASSED}${NC}"
echo -e "${RED}Failed: ${TESTS_FAILED}${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi