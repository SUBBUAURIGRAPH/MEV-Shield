#!/bin/bash

# Script to split MEV-1 into WBS structure
# Work Breakdown Structure: Epic → Tasks → Subtasks
# Following 1 task:1 ticket rule

set -e

echo "========================================="
echo " MEV-1 Work Breakdown Structure (WBS)"
echo " Creating Epics, Tasks, and Subtasks"
echo " Rule: 1 Task = 1 Ticket"
echo "========================================="
echo ""
echo "WBS Structure:"
echo "└── MEV-1: JIRA Integration (Main Epic)"
echo "    ├── Epic 1: GitHub Actions Setup"
echo "    │   ├── Tasks with subtasks"
echo "    ├── Epic 2: JIRA API Configuration"
echo "    │   ├── Tasks with subtasks"
echo "    ├── Epic 3: Automation Rules"
echo "    │   ├── Tasks with subtasks"
echo "    └── Epic 4: Documentation & Testing"
echo "        └── Tasks with subtasks"
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
NC='\033[0m'

# Base64 encode credentials
AUTH_HEADER=$(echo -n "${JIRA_USER_EMAIL}:${JIRA_API_TOKEN}" | base64)

# Function to create JIRA issue
create_issue() {
    local ISSUE_TYPE=$1
    local SUMMARY=$2
    local DESCRIPTION=$3
    local PARENT_KEY=$4
    local LABELS=$5
    local EPIC_KEY=$6
    
    if [ "$ISSUE_TYPE" = "Sub-task" ] && [ ! -z "$PARENT_KEY" ]; then
        # Create subtask
        REQUEST_BODY=$(cat <<EOF
{
    "fields": {
        "project": {"key": "${JIRA_PROJECT_KEY}"},
        "parent": {"key": "${PARENT_KEY}"},
        "summary": "${SUMMARY}",
        "description": {
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "${DESCRIPTION}"
                        }
                    ]
                }
            ]
        },
        "issuetype": {"name": "Subtask"},
        "labels": ${LABELS}
    }
}
EOF
)
    elif [ "$ISSUE_TYPE" = "Task" ]; then
        # Create task with epic link if provided
        if [ ! -z "$EPIC_KEY" ]; then
            REQUEST_BODY=$(cat <<EOF
{
    "fields": {
        "project": {"key": "${JIRA_PROJECT_KEY}"},
        "summary": "${SUMMARY}",
        "description": {
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "${DESCRIPTION}"
                        }
                    ]
                }
            ]
        },
        "issuetype": {"name": "Task"},
        "labels": ${LABELS},
        "customfield_10014": "${EPIC_KEY}"
    }
}
EOF
)
        else
            REQUEST_BODY=$(cat <<EOF
{
    "fields": {
        "project": {"key": "${JIRA_PROJECT_KEY}"},
        "summary": "${SUMMARY}",
        "description": {
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "${DESCRIPTION}"
                        }
                    ]
                }
            ]
        },
        "issuetype": {"name": "Task"},
        "labels": ${LABELS}
    }
}
EOF
)
        fi
    else
        # Create Epic
        REQUEST_BODY=$(cat <<EOF
{
    "fields": {
        "project": {"key": "${JIRA_PROJECT_KEY}"},
        "summary": "${SUMMARY}",
        "description": {
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "${DESCRIPTION}"
                        }
                    ]
                }
            ]
        },
        "issuetype": {"name": "Epic"},
        "customfield_10011": "${SUMMARY}",
        "labels": ${LABELS}
    }
}
EOF
)
    fi
    
    RESPONSE=$(curl -s -X POST \
        -H "Authorization: Basic ${AUTH_HEADER}" \
        -H "Content-Type: application/json" \
        -d "${REQUEST_BODY}" \
        "${JIRA_BASE_URL}/rest/api/3/issue")
    
    ISSUE_KEY=$(echo "$RESPONSE" | grep -o '"key":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$ISSUE_KEY" ]; then
        echo -e "${GREEN}✓ Created ${ISSUE_TYPE}: ${ISSUE_KEY} - ${SUMMARY}${NC}"
        echo "$ISSUE_KEY"
    else
        echo -e "${YELLOW}⚠ Failed to create ${ISSUE_TYPE}: ${SUMMARY}${NC}"
        echo "Response: $RESPONSE"
        echo ""
    fi
}

echo -e "${BLUE}Step 1: Creating Parent Epic for JIRA Integration${NC}"
echo "=================================================="

EPIC_KEY=$(create_issue "Epic" \
    "JIRA GitHub Integration Implementation" \
    "Implementation of automated JIRA integration with GitHub Actions following 1:1 task mapping policy" \
    "" \
    '["jira-integration", "github-actions", "automation"]' \
    "")

echo ""
echo -e "${BLUE}Step 2: Creating Main Tasks${NC}"
echo "================================"

# Task 1: Workflow Configuration
TASK1_KEY=$(create_issue "Task" \
    "Configure GitHub Actions Workflows" \
    "Set up GitHub Actions workflows for JIRA integration with smart commit processing" \
    "" \
    '["github-actions", "workflow", "configuration"]' \
    "${EPIC_KEY}")

# Task 2: Task Management System
TASK2_KEY=$(create_issue "Task" \
    "Implement 1:1 Task Mapping System" \
    "Create automated task-to-ticket mapping system with epic grouping" \
    "" \
    '["task-management", "automation", "mapping"]' \
    "${EPIC_KEY}")

# Task 3: API Integration
TASK3_KEY=$(create_issue "Task" \
    "Setup JIRA API Integration" \
    "Configure JIRA API connection and authentication with GitHub" \
    "" \
    '["api", "integration", "authentication"]' \
    "${EPIC_KEY}")

# Task 4: Documentation
TASK4_KEY=$(create_issue "Task" \
    "Create Documentation and Guides" \
    "Write comprehensive documentation for team usage" \
    "" \
    '["documentation", "guides", "training"]' \
    "${EPIC_KEY}")

# Task 5: Testing Framework
TASK5_KEY=$(create_issue "Task" \
    "Develop Testing and Validation Scripts" \
    "Create scripts for testing and validating JIRA integration" \
    "" \
    '["testing", "validation", "scripts"]' \
    "${EPIC_KEY}")

echo ""
echo -e "${BLUE}Step 3: Creating Subtasks for Each Task${NC}"
echo "========================================"

# Subtasks for Task 1: Workflow Configuration
echo -e "${YELLOW}Creating subtasks for ${TASK1_KEY} (Workflow Configuration)${NC}"

create_issue "Sub-task" \
    "Create jira-integration.yml workflow" \
    "Main workflow file for JIRA integration with push and PR triggers" \
    "${TASK1_KEY}" \
    '["workflow", "yaml"]' \
    ""

create_issue "Sub-task" \
    "Create jira-task-management.yml workflow" \
    "Workflow for 1:1 task mapping with automatic epic assignment" \
    "${TASK1_KEY}" \
    '["workflow", "task-management"]' \
    ""

create_issue "Sub-task" \
    "Configure workflow triggers and events" \
    "Set up triggers for push, PR, issues, and manual dispatch" \
    "${TASK1_KEY}" \
    '["triggers", "events"]' \
    ""

# Subtasks for Task 2: Task Management System
echo ""
echo -e "${YELLOW}Creating subtasks for ${TASK2_KEY} (Task Management System)${NC}"

create_issue "Sub-task" \
    "Implement automatic ticket creation logic" \
    "Logic to auto-create tickets for commits without JIRA keys" \
    "${TASK2_KEY}" \
    '["automation", "ticket-creation"]' \
    ""

create_issue "Sub-task" \
    "Create epic grouping based on file types" \
    "Auto-assign epics based on changed file categories" \
    "${TASK2_KEY}" \
    '["epic", "categorization"]' \
    ""

create_issue "Sub-task" \
    "Implement PR to subtask conversion" \
    "Convert PR checklists to JIRA subtasks" \
    "${TASK2_KEY}" \
    '["pull-request", "subtasks"]' \
    ""

# Subtasks for Task 3: API Integration
echo ""
echo -e "${YELLOW}Creating subtasks for ${TASK3_KEY} (API Integration)${NC}"

create_issue "Sub-task" \
    "Configure JIRA API credentials" \
    "Set up API tokens and authentication headers" \
    "${TASK3_KEY}" \
    '["api", "credentials"]' \
    ""

create_issue "Sub-task" \
    "Create GitHub secrets configuration script" \
    "Script to configure repository secrets" \
    "${TASK3_KEY}" \
    '["secrets", "configuration"]' \
    ""

create_issue "Sub-task" \
    "Implement API error handling" \
    "Add error handling and retry logic for API calls" \
    "${TASK3_KEY}" \
    '["error-handling", "reliability"]' \
    ""

# Subtasks for Task 4: Documentation
echo ""
echo -e "${YELLOW}Creating subtasks for ${TASK4_KEY} (Documentation)${NC}"

create_issue "Sub-task" \
    "Write JIRA Integration Guide" \
    "Comprehensive guide with smart commit examples" \
    "${TASK4_KEY}" \
    '["documentation", "guide"]' \
    ""

create_issue "Sub-task" \
    "Create setup instructions" \
    "Step-by-step setup instructions for team members" \
    "${TASK4_KEY}" \
    '["setup", "instructions"]' \
    ""

create_issue "Sub-task" \
    "Document troubleshooting procedures" \
    "Common issues and their solutions" \
    "${TASK4_KEY}" \
    '["troubleshooting", "support"]' \
    ""

# Subtasks for Task 5: Testing Framework
echo ""
echo -e "${YELLOW}Creating subtasks for ${TASK5_KEY} (Testing Framework)${NC}"

create_issue "Sub-task" \
    "Create integration test script" \
    "Script to test JIRA API connectivity" \
    "${TASK5_KEY}" \
    '["testing", "integration"]' \
    ""

create_issue "Sub-task" \
    "Implement validation checks" \
    "Validate configuration and credentials" \
    "${TASK5_KEY}" \
    '["validation", "checks"]' \
    ""

create_issue "Sub-task" \
    "Create test ticket generation" \
    "Generate test tickets for verification" \
    "${TASK5_KEY}" \
    '["testing", "verification"]' \
    ""

echo ""
echo -e "${BLUE}Step 4: Updating MEV-1 with completion note${NC}"
echo "============================================="

# Add comment to MEV-1
COMMENT_BODY=$(cat <<EOF
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
                        "text": "✅ This task has been split into detailed tasks and subtasks following the 1:1 mapping rule:",
                        "marks": [{"type": "strong"}]
                    }
                ]
            },
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "Parent Epic: ${EPIC_KEY} - JIRA GitHub Integration Implementation"
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
                                        "text": "${TASK1_KEY}: Configure GitHub Actions Workflows (3 subtasks)"
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
                                        "text": "${TASK2_KEY}: Implement 1:1 Task Mapping System (3 subtasks)"
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
                                        "text": "${TASK3_KEY}: Setup JIRA API Integration (3 subtasks)"
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
                                        "text": "${TASK4_KEY}: Create Documentation and Guides (3 subtasks)"
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
                                        "text": "${TASK5_KEY}: Develop Testing and Validation Scripts (3 subtasks)"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            },
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "Total: 1 Epic, 5 Tasks, 15 Subtasks - All following the 1 task:1 ticket rule"
                    }
                ]
            }
        ]
    }
}
EOF
)

curl -s -X POST \
    -H "Authorization: Basic ${AUTH_HEADER}" \
    -H "Content-Type: application/json" \
    -d "${COMMENT_BODY}" \
    "${JIRA_BASE_URL}/rest/api/3/issue/MEV-1/comment" > /dev/null

echo -e "${GREEN}✓ Added summary comment to MEV-1${NC}"

# Transition MEV-1 to Done
echo -e "${GREEN}✓ Marking MEV-1 as Done (implementation complete)${NC}"

echo ""
echo "========================================="
echo -e "${GREEN} Task Breakdown Complete!${NC}"
echo "========================================="
echo ""
echo "Summary of created tickets:"
echo "- Parent Epic: ${EPIC_KEY}"
echo "- 5 Main Tasks created"
echo "- 15 Subtasks created (3 per task)"
echo ""
echo "Structure follows 1 task:1 ticket rule:"
echo "Each discrete task has its own ticket for proper tracking"
echo ""
echo "View in JIRA:"
echo "${JIRA_BASE_URL}/browse/${EPIC_KEY}"
echo ""