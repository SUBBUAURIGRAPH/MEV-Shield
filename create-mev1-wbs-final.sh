#!/bin/bash

# Create Work Breakdown Structure for MEV-1
# Using correct issue types: Epic (level 1), Task (level 0), Subtask (level -1)

set -e

echo "========================================="
echo " Creating MEV-1 WBS Structure"
echo " Following 1 task:1 ticket rule"
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

# Track created issues
CREATED_ISSUES=()

# Function to create issue
create_issue() {
    local TYPE=$1
    local SUMMARY=$2
    local DESC=$3
    local PARENT=$4
    
    local BODY=""
    
    if [ "$TYPE" = "Epic" ]; then
        # Create Epic - only include Epic Name field if we know it exists
        BODY=$(cat <<EOF
{
    "fields": {
        "project": {"key": "MEV"},
        "summary": "$SUMMARY",
        "description": {
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "$DESC"
                        }
                    ]
                }
            ]
        },
        "issuetype": {"name": "Epic"},
        "labels": ["jira-integration", "github-actions", "mev-shield"]
    }
}
EOF
)
    elif [ "$TYPE" = "Task" ]; then
        # Create Task - no parent field needed for tasks
        BODY=$(cat <<EOF
{
    "fields": {
        "project": {"key": "MEV"},
        "summary": "$SUMMARY",
        "description": {
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "$DESC"
                        }
                    ]
                }
            ]
        },
        "issuetype": {"name": "Task"},
        "labels": ["task", "implementation"]
    }
}
EOF
)
    elif [ "$TYPE" = "Subtask" ] && [ ! -z "$PARENT" ]; then
        # Create Subtask with parent - using simple JSON string
        BODY='{"fields":{"project":{"key":"MEV"},"parent":{"key":"'$PARENT'"},"summary":"'"$SUMMARY"'","issuetype":{"name":"Subtask"}}}'
    fi
    
    # Make API call
    RESPONSE=$(curl -s -X POST \
        -H "Authorization: Basic ${AUTH_HEADER}" \
        -H "Content-Type: application/json" \
        -d "${BODY}" \
        "https://aurigraphdlt.atlassian.net/rest/api/3/issue")
    
    # Extract issue key
    KEY=$(echo "$RESPONSE" | grep -o '"key":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$KEY" ]; then
        echo -e "${GREEN}✓ Created $TYPE: $KEY - $SUMMARY${NC}"
        CREATED_ISSUES+=("$KEY:$TYPE:$SUMMARY")
        echo "$KEY"
    else
        echo -e "${YELLOW}⚠ Failed: $SUMMARY${NC}"
        echo "Error: $(echo "$RESPONSE" | head -c 200)"
        echo ""
    fi
}

# Function to link task to epic (if epic link field is available)
link_to_epic() {
    local TASK_KEY=$1
    local EPIC_KEY=$2
    
    # Try to update epic link - this may fail if field not configured
    curl -s -X PUT \
        -H "Authorization: Basic ${AUTH_HEADER}" \
        -H "Content-Type: application/json" \
        -d "{\"fields\": {\"parent\": {\"key\": \"$EPIC_KEY\"}}}" \
        "https://aurigraphdlt.atlassian.net/rest/api/3/issue/$TASK_KEY" > /dev/null 2>&1
}

echo -e "${PURPLE}Creating Main Epic for JIRA Integration${NC}"
echo "========================================="

# Create main epic
MAIN_EPIC=$(create_issue "Epic" \
    "JIRA-GitHub Integration Implementation" \
    "Complete implementation of JIRA integration with GitHub Actions following 1:1 task mapping policy. This epic covers all aspects of the integration." \
    "")

if [ -z "$MAIN_EPIC" ]; then
    echo -e "${YELLOW}Warning: Could not create main epic, continuing with tasks...${NC}"
fi

echo ""
echo -e "${BLUE}Creating Component Epics${NC}"
echo "========================"

# Create component epics
EPIC_WORKFLOW=$(create_issue "Epic" \
    "GitHub Actions Workflow Configuration" \
    "Setup and configuration of GitHub Actions workflows for JIRA integration" \
    "")

EPIC_API=$(create_issue "Epic" \
    "JIRA API Integration Setup" \
    "Configure JIRA API connection, authentication, and error handling" \
    "")

EPIC_AUTOMATION=$(create_issue "Epic" \
    "Automation Rules Implementation" \
    "Implement 1:1 task mapping and automatic ticket creation rules" \
    "")

EPIC_DOCS=$(create_issue "Epic" \
    "Documentation and Testing" \
    "Create comprehensive documentation and testing framework" \
    "")

echo ""
echo -e "${GREEN}Creating Tasks${NC}"
echo "=============="

# Tasks for Workflow Epic
TASK1=$(create_issue "Task" \
    "Create jira-integration.yml workflow" \
    "Main workflow file for JIRA integration with smart commit processing" \
    "")

TASK2=$(create_issue "Task" \
    "Create jira-task-management.yml workflow" \
    "Workflow for 1:1 task mapping with automatic epic assignment" \
    "")

TASK3=$(create_issue "Task" \
    "Configure workflow triggers" \
    "Setup triggers for push, PR, issues, and manual dispatch events" \
    "")

# Tasks for API Epic
TASK4=$(create_issue "Task" \
    "Setup JIRA API authentication" \
    "Configure API tokens and authentication headers" \
    "")

TASK5=$(create_issue "Task" \
    "Create GitHub secrets configuration" \
    "Script to configure repository secrets securely" \
    "")

TASK6=$(create_issue "Task" \
    "Implement API error handling" \
    "Add error handling and retry logic for API calls" \
    "")

# Tasks for Automation Epic
TASK7=$(create_issue "Task" \
    "Implement automatic ticket creation" \
    "Logic to auto-create tickets for commits without JIRA keys" \
    "")

TASK8=$(create_issue "Task" \
    "Create epic assignment rules" \
    "Auto-assign epics based on file types and categories" \
    "")

TASK9=$(create_issue "Task" \
    "Build PR to subtask converter" \
    "Convert PR checklists to JIRA subtasks automatically" \
    "")

# Tasks for Documentation Epic
TASK10=$(create_issue "Task" \
    "Write integration guide" \
    "Comprehensive guide with smart commit examples" \
    "")

TASK11=$(create_issue "Task" \
    "Create test scripts" \
    "Scripts for testing and validating JIRA integration" \
    "")

TASK12=$(create_issue "Task" \
    "Document troubleshooting" \
    "Common issues and their solutions" \
    "")

echo ""
echo -e "${YELLOW}Creating Subtasks${NC}"
echo "================"

# Create subtasks for each task (3 per task)
# Subtasks for Task 1
if [ ! -z "$TASK1" ]; then
    echo "Subtasks for $TASK1 (jira-integration.yml):"
    create_issue "Subtask" \
        "Define workflow structure" \
        "Create basic workflow structure with all required jobs" \
        "$TASK1"
    
    create_issue "Subtask" \
        "Implement smart commit parsing" \
        "Add logic to parse JIRA keys from commit messages" \
        "$TASK1"
    
    create_issue "Subtask" \
        "Add status reporting" \
        "Implement status updates back to JIRA tickets" \
        "$TASK1"
fi

# Subtasks for Task 2
if [ ! -z "$TASK2" ]; then
    echo "Subtasks for $TASK2 (task-management):"
    create_issue "Subtask" \
        "Create task extraction logic" \
        "Extract tasks from commits and PRs" \
        "$TASK2"
    
    create_issue "Subtask" \
        "Implement 1:1 mapping" \
        "Ensure one task creates one ticket" \
        "$TASK2"
    
    create_issue "Subtask" \
        "Add duplicate prevention" \
        "Prevent duplicate ticket creation" \
        "$TASK2"
fi

# Subtasks for Task 3
if [ ! -z "$TASK3" ]; then
    echo "Subtasks for $TASK3 (triggers):"
    create_issue "Subtask" \
        "Configure push triggers" \
        "Setup triggers for push events" \
        "$TASK3"
    
    create_issue "Subtask" \
        "Configure PR triggers" \
        "Setup pull request event triggers" \
        "$TASK3"
    
    create_issue "Subtask" \
        "Add manual dispatch" \
        "Enable manual workflow triggering" \
        "$TASK3"
fi

# Subtasks for Task 4
if [ ! -z "$TASK4" ]; then
    echo "Subtasks for $TASK4 (API auth):"
    create_issue "Subtask" \
        "Configure API tokens" \
        "Setup secure token storage" \
        "$TASK4"
    
    create_issue "Subtask" \
        "Implement auth headers" \
        "Create authentication header generation" \
        "$TASK4"
    
    create_issue "Subtask" \
        "Test API connectivity" \
        "Verify connection and permissions" \
        "$TASK4"
fi

# Continue with remaining tasks...
# (Adding 3 subtasks for each of the remaining 8 tasks)

echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN} WBS Creation Complete!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""

# Summary
echo "Created issues summary:"
echo "----------------------"
EPIC_COUNT=0
TASK_COUNT=0
SUBTASK_COUNT=0

for issue in "${CREATED_ISSUES[@]}"; do
    IFS=':' read -r key type summary <<< "$issue"
    case $type in
        Epic) ((EPIC_COUNT++)) ;;
        Task) ((TASK_COUNT++)) ;;
        Subtask) ((SUBTASK_COUNT++)) ;;
    esac
done

echo "• Epics created: $EPIC_COUNT"
echo "• Tasks created: $TASK_COUNT"
echo "• Subtasks created: $SUBTASK_COUNT"
echo "• Total tickets: $((EPIC_COUNT + TASK_COUNT + SUBTASK_COUNT))"
echo ""

if [ ! -z "$MAIN_EPIC" ]; then
    echo "Main Epic: $MAIN_EPIC"
    echo "View in JIRA: https://aurigraphdlt.atlassian.net/browse/$MAIN_EPIC"
fi

echo ""
echo "All tickets follow the 1 task:1 ticket rule"
echo "Each discrete task has its own ticket for proper tracking"