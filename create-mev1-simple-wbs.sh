#!/bin/bash

# Simple WBS for MEV-1 using available issue types
# Structure: Epic → Task → Subtask

set -e

echo "========================================="
echo " Creating MEV-1 WBS Structure"
echo " Using available issue types: Epic, Task, Subtask"
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
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Base64 encode credentials
AUTH_HEADER=$(echo -n "${JIRA_USER_EMAIL}:${JIRA_API_TOKEN}" | base64)

# Function to create issue
create_issue() {
    local TYPE=$1
    local SUMMARY=$2
    local DESC=$3
    local PARENT=$4
    
    local BODY=""
    
    if [ "$TYPE" = "Epic" ]; then
        BODY=$(cat <<EOF
{
    "fields": {
        "project": {"key": "MEV"},
        "summary": "$SUMMARY",
        "description": "$DESC",
        "issuetype": {"name": "Epic"},
        "customfield_10011": "$SUMMARY"
    }
}
EOF
)
    elif [ "$TYPE" = "Task" ] && [ ! -z "$PARENT" ]; then
        BODY=$(cat <<EOF
{
    "fields": {
        "project": {"key": "MEV"},
        "summary": "$SUMMARY",
        "description": "$DESC",
        "issuetype": {"name": "Task"},
        "customfield_10014": "$PARENT"
    }
}
EOF
)
    elif [ "$TYPE" = "Subtask" ] && [ ! -z "$PARENT" ]; then
        BODY=$(cat <<EOF
{
    "fields": {
        "project": {"key": "MEV"},
        "parent": {"key": "$PARENT"},
        "summary": "$SUMMARY",
        "description": "$DESC",
        "issuetype": {"name": "Subtask"}
    }
}
EOF
)
    fi
    
    RESPONSE=$(curl -s -X POST \
        -H "Authorization: Basic ${AUTH_HEADER}" \
        -H "Content-Type: application/json" \
        -d "${BODY}" \
        "https://aurigraphdlt.atlassian.net/rest/api/3/issue")
    
    KEY=$(echo "$RESPONSE" | grep -o '"key":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$KEY" ]; then
        echo -e "${GREEN}✓ Created $TYPE: $KEY - $SUMMARY${NC}"
        echo "$KEY"
    else
        echo -e "${YELLOW}⚠ Failed: $SUMMARY${NC}"
        echo "$RESPONSE" | head -1
        echo ""
    fi
}

echo -e "${PURPLE}Creating Main Epic for JIRA Integration${NC}"
echo "========================================="

MAIN_EPIC=$(create_issue "Epic" \
    "JIRA-GitHub Integration Implementation" \
    "Complete implementation of JIRA integration with GitHub Actions following 1:1 task mapping" \
    "")

echo ""
echo -e "${BLUE}Creating Tasks under Epic${NC}"
echo "========================="

# Task 1: Workflow Setup
TASK1=$(create_issue "Task" \
    "Setup GitHub Actions Workflows" \
    "Configure all GitHub Actions workflows for JIRA integration" \
    "$MAIN_EPIC")

# Task 2: API Configuration
TASK2=$(create_issue "Task" \
    "Configure JIRA API Integration" \
    "Setup API authentication and connection" \
    "$MAIN_EPIC")

# Task 3: Automation Rules
TASK3=$(create_issue "Task" \
    "Implement Automation Rules" \
    "Create 1:1 mapping and auto-ticket creation" \
    "$MAIN_EPIC")

# Task 4: Documentation
TASK4=$(create_issue "Task" \
    "Create Documentation and Testing" \
    "Write guides and test scripts" \
    "$MAIN_EPIC")

echo ""
echo -e "${YELLOW}Creating Subtasks${NC}"
echo "================="

# Subtasks for Task 1
if [ ! -z "$TASK1" ]; then
    echo "Subtasks for $TASK1:"
    create_issue "Subtask" \
        "Create jira-integration.yml" \
        "Main workflow file" \
        "$TASK1"
    
    create_issue "Subtask" \
        "Create task-management.yml" \
        "Task mapping workflow" \
        "$TASK1"
    
    create_issue "Subtask" \
        "Configure triggers" \
        "Setup event triggers" \
        "$TASK1"
fi

# Subtasks for Task 2
if [ ! -z "$TASK2" ]; then
    echo "Subtasks for $TASK2:"
    create_issue "Subtask" \
        "Setup API credentials" \
        "Configure tokens" \
        "$TASK2"
    
    create_issue "Subtask" \
        "Create secrets script" \
        "GitHub secrets setup" \
        "$TASK2"
    
    create_issue "Subtask" \
        "Test API connection" \
        "Verify connectivity" \
        "$TASK2"
fi

# Subtasks for Task 3
if [ ! -z "$TASK3" ]; then
    echo "Subtasks for $TASK3:"
    create_issue "Subtask" \
        "Auto-ticket creation" \
        "Automatic ticket logic" \
        "$TASK3"
    
    create_issue "Subtask" \
        "Epic assignment rules" \
        "File-based epic mapping" \
        "$TASK3"
    
    create_issue "Subtask" \
        "PR integration" \
        "PR to subtask conversion" \
        "$TASK3"
fi

# Subtasks for Task 4
if [ ! -z "$TASK4" ]; then
    echo "Subtasks for $TASK4:"
    create_issue "Subtask" \
        "Write integration guide" \
        "Main documentation" \
        "$TASK4"
    
    create_issue "Subtask" \
        "Create test scripts" \
        "Testing framework" \
        "$TASK4"
    
    create_issue "Subtask" \
        "Troubleshooting docs" \
        "Common issues guide" \
        "$TASK4"
fi

echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN} WBS Creation Complete!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "Structure created:"
echo "└── Epic: $MAIN_EPIC"
echo "    ├── Task: Workflow Setup (3 subtasks)"
echo "    ├── Task: API Configuration (3 subtasks)"
echo "    ├── Task: Automation Rules (3 subtasks)"
echo "    └── Task: Documentation (3 subtasks)"
echo ""
echo "Total: 1 Epic + 4 Tasks + 12 Subtasks = 17 tickets"
echo ""
echo "View in JIRA: https://aurigraphdlt.atlassian.net/browse/$MAIN_EPIC"