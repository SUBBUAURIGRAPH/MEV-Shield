#!/bin/bash

# Test JIRA Integration
# This script tests the JIRA API connection and creates a test ticket

set -e

echo "========================================="
echo " Testing JIRA Integration for MEV Shield"
echo "========================================="
echo ""

# Load environment variables
if [ -f ".env.jira" ]; then
    source .env.jira
else
    echo "Error: .env.jira file not found"
    exit 1
fi

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "Testing JIRA API Connection..."
echo "================================"

# Test 1: Verify Authentication
echo -n "1. Testing authentication... "
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "Authorization: Basic $(echo -n "${JIRA_USER_EMAIL}:${JIRA_API_TOKEN}" | base64)" \
    "${JIRA_BASE_URL}/rest/api/3/myself")

if [ "$RESPONSE" = "200" ]; then
    echo -e "${GREEN}✓ Success${NC}"
else
    echo -e "${RED}✗ Failed (HTTP $RESPONSE)${NC}"
    exit 1
fi

# Test 2: Check Project Access
echo -n "2. Checking MEV project access... "
PROJECT_DATA=$(curl -s \
    -H "Authorization: Basic $(echo -n "${JIRA_USER_EMAIL}:${JIRA_API_TOKEN}" | base64)" \
    "${JIRA_BASE_URL}/rest/api/3/project/${JIRA_PROJECT_KEY}")

if echo "$PROJECT_DATA" | grep -q '"key":"MEV"'; then
    echo -e "${GREEN}✓ Project found${NC}"
else
    echo -e "${RED}✗ Project not accessible${NC}"
    exit 1
fi

# Test 3: Create Test Ticket
echo -n "3. Creating test ticket... "
TICKET_DATA=$(cat <<EOF
{
    "fields": {
        "project": {
            "key": "${JIRA_PROJECT_KEY}"
        },
        "summary": "[TEST] GitHub Actions Integration Test - $(date +%Y%m%d_%H%M%S)",
        "description": {
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "This is a test ticket created to verify JIRA integration with GitHub Actions."
                        }
                    ]
                },
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "Test Details:"
                        }
                    ]
                },
                {
                    "type": "bulletList",
                    "content": [
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": "Created by: test-jira-integration.sh"
                                        }
                                    ]
                                }
                            ]
                        },
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": "Timestamp: $(date)"
                                        }
                                    ]
                                }
                            ]
                        },
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": "Purpose: Verify API connectivity"
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        },
        "issuetype": {
            "name": "Task"
        },
        "labels": ["test", "github-integration", "mev-shield"]
    }
}
EOF
)

TICKET_RESPONSE=$(curl -s -X POST \
    -H "Authorization: Basic $(echo -n "${JIRA_USER_EMAIL}:${JIRA_API_TOKEN}" | base64)" \
    -H "Content-Type: application/json" \
    -d "$TICKET_DATA" \
    "${JIRA_BASE_URL}/rest/api/3/issue")

TICKET_KEY=$(echo "$TICKET_RESPONSE" | grep -o '"key":"[^"]*"' | cut -d'"' -f4)

if [ ! -z "$TICKET_KEY" ]; then
    echo -e "${GREEN}✓ Created: $TICKET_KEY${NC}"
    TICKET_CREATED=true
else
    echo -e "${RED}✗ Failed to create ticket${NC}"
    echo "Response: $TICKET_RESPONSE"
    TICKET_CREATED=false
fi

# Test 4: Add Comment to Ticket (if created)
if [ "$TICKET_CREATED" = true ]; then
    echo -n "4. Adding comment to ticket... "
    
    COMMENT_DATA=$(cat <<EOF
{
    "body": {
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "✅ Test comment added successfully via API"
                    }
                ]
            }
        ]
    }
}
EOF
)

    COMMENT_RESPONSE=$(curl -s -X POST \
        -H "Authorization: Basic $(echo -n "${JIRA_USER_EMAIL}:${JIRA_API_TOKEN}" | base64)" \
        -H "Content-Type: application/json" \
        -d "$COMMENT_DATA" \
        "${JIRA_BASE_URL}/rest/api/3/issue/${TICKET_KEY}/comment")
    
    if echo "$COMMENT_RESPONSE" | grep -q '"id"'; then
        echo -e "${GREEN}✓ Comment added${NC}"
    else
        echo -e "${YELLOW}⚠ Could not add comment${NC}"
    fi
fi

# Test 5: List Issue Types
echo -n "5. Checking available issue types... "
ISSUE_TYPES=$(curl -s \
    -H "Authorization: Basic $(echo -n "${JIRA_USER_EMAIL}:${JIRA_API_TOKEN}" | base64)" \
    "${JIRA_BASE_URL}/rest/api/3/project/${JIRA_PROJECT_KEY}" | \
    python3 -c "import sys, json; data = json.load(sys.stdin); print(', '.join([it['name'] for it in data.get('issueTypes', [])]))" 2>/dev/null)

if [ ! -z "$ISSUE_TYPES" ]; then
    echo -e "${GREEN}✓ Found: $ISSUE_TYPES${NC}"
else
    echo -e "${YELLOW}⚠ Could not retrieve issue types${NC}"
fi

echo ""
echo "========================================="
echo -e "${GREEN} JIRA Integration Test Complete!${NC}"
echo "========================================="
echo ""

if [ "$TICKET_CREATED" = true ]; then
    echo "Test ticket created: ${TICKET_KEY}"
    echo "View in JIRA: ${JIRA_BASE_URL}/browse/${TICKET_KEY}"
    echo ""
    echo -e "${YELLOW}Note: Remember to delete the test ticket after verification${NC}"
fi

echo ""
echo "Configuration verified:"
echo "- JIRA URL: ${JIRA_BASE_URL}"
echo "- Project: ${JIRA_PROJECT_KEY}"
echo "- User: ${JIRA_USER_EMAIL}"
echo ""
echo "GitHub Actions workflows are ready to use!"