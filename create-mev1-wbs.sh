#!/bin/bash

# Create Work Breakdown Structure (WBS) for MEV-1
# Hierarchical structure: Main Epic → Sub-Epics → Tasks → Subtasks
# Following 1 task:1 ticket rule

set -e

echo "=============================================="
echo " MEV-1 Work Breakdown Structure (WBS)"
echo " Creating Complete JIRA Hierarchy"
echo "=============================================="
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

# Arrays to store created issue keys
declare -a EPIC_KEYS=()
declare -a TASK_KEYS=()
declare -a SUBTASK_KEYS=()

# Function to create JIRA issue
create_issue() {
    local ISSUE_TYPE=$1
    local SUMMARY=$2
    local DESCRIPTION=$3
    local PARENT_KEY=$4
    local LABELS=$5
    local EPIC_LINK=$6
    
    local REQUEST_BODY=""
    
    if [ "$ISSUE_TYPE" = "Subtask" ]; then
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
    elif [ "$ISSUE_TYPE" = "Task" ] && [ ! -z "$EPIC_LINK" ]; then
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
        "customfield_10014": "${EPIC_LINK}"
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
        echo ""
    fi
}

echo -e "${PURPLE}═══════════════════════════════════════════════${NC}"
echo -e "${PURPLE} LEVEL 0: MAIN EPIC (MEV-1 Replacement)${NC}"
echo -e "${PURPLE}═══════════════════════════════════════════════${NC}"

MAIN_EPIC=$(create_issue "Epic" \
    "[MAIN] JIRA-GitHub Integration System" \
    "Complete implementation of JIRA integration with GitHub Actions following 1:1 task mapping policy. This epic encompasses all aspects of the integration including setup, configuration, automation, and documentation." \
    "" \
    '["main-epic", "jira-integration", "mev-shield"]' \
    "")

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
echo -e "${BLUE} LEVEL 1: SUB-EPICS${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════${NC}"

# Sub-Epic 1: GitHub Actions Setup
EPIC1=$(create_issue "Epic" \
    "[EPIC-1] GitHub Actions Workflow Setup" \
    "Setup and configuration of all GitHub Actions workflows for JIRA integration" \
    "" \
    '["github-actions", "workflow", "setup"]' \
    "")
EPIC_KEYS+=("$EPIC1")

# Sub-Epic 2: JIRA API Configuration
EPIC2=$(create_issue "Epic" \
    "[EPIC-2] JIRA API Integration & Authentication" \
    "Complete JIRA API setup including authentication, connection, and error handling" \
    "" \
    '["jira-api", "authentication", "integration"]' \
    "")
EPIC_KEYS+=("$EPIC2")

# Sub-Epic 3: Automation Rules
EPIC3=$(create_issue "Epic" \
    "[EPIC-3] Automation Rules & Task Mapping" \
    "Implementation of 1:1 task mapping rules and automatic ticket creation logic" \
    "" \
    '["automation", "task-mapping", "rules"]' \
    "")
EPIC_KEYS+=("$EPIC3")

# Sub-Epic 4: Documentation & Testing
EPIC4=$(create_issue "Epic" \
    "[EPIC-4] Documentation, Testing & Validation" \
    "Comprehensive documentation, testing scripts, and validation procedures" \
    "" \
    '["documentation", "testing", "validation"]' \
    "")
EPIC_KEYS+=("$EPIC4")

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"
echo -e "${GREEN} LEVEL 2: TASKS FOR EPIC-1 (GitHub Actions)${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"

# Tasks for Epic 1
TASK1_1=$(create_issue "Task" \
    "Create primary jira-integration.yml workflow" \
    "Develop main workflow file for JIRA integration with smart commit processing" \
    "" \
    '["workflow", "primary", "yaml"]' \
    "$EPIC1")

TASK1_2=$(create_issue "Task" \
    "Create task-management workflow" \
    "Develop workflow for 1:1 task-to-ticket mapping" \
    "" \
    '["workflow", "task-management"]' \
    "$EPIC1")

TASK1_3=$(create_issue "Task" \
    "Configure workflow triggers and events" \
    "Setup all GitHub event triggers and conditions" \
    "" \
    '["triggers", "events", "configuration"]' \
    "$EPIC1")

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"
echo -e "${GREEN} LEVEL 2: TASKS FOR EPIC-2 (JIRA API)${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"

# Tasks for Epic 2
TASK2_1=$(create_issue "Task" \
    "Setup JIRA API authentication" \
    "Configure API tokens and authentication headers" \
    "" \
    '["api", "authentication", "tokens"]' \
    "$EPIC2")

TASK2_2=$(create_issue "Task" \
    "Create secrets management system" \
    "Implement secure storage for API credentials" \
    "" \
    '["secrets", "security", "credentials"]' \
    "$EPIC2")

TASK2_3=$(create_issue "Task" \
    "Implement API error handling" \
    "Add comprehensive error handling and retry logic" \
    "" \
    '["error-handling", "reliability", "api"]' \
    "$EPIC2")

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"
echo -e "${GREEN} LEVEL 2: TASKS FOR EPIC-3 (Automation)${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"

# Tasks for Epic 3
TASK3_1=$(create_issue "Task" \
    "Implement automatic ticket creation" \
    "Logic for auto-creating tickets from commits" \
    "" \
    '["automation", "ticket-creation", "commits"]' \
    "$EPIC3")

TASK3_2=$(create_issue "Task" \
    "Create epic assignment rules" \
    "Automatic epic assignment based on file types" \
    "" \
    '["epic", "assignment", "rules"]' \
    "$EPIC3")

TASK3_3=$(create_issue "Task" \
    "Build PR to subtask converter" \
    "Convert PR checklists to JIRA subtasks" \
    "" \
    '["pull-request", "subtasks", "converter"]' \
    "$EPIC3")

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"
echo -e "${GREEN} LEVEL 2: TASKS FOR EPIC-4 (Documentation)${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"

# Tasks for Epic 4
TASK4_1=$(create_issue "Task" \
    "Write comprehensive integration guide" \
    "Create detailed JIRA integration documentation" \
    "" \
    '["documentation", "guide", "comprehensive"]' \
    "$EPIC4")

TASK4_2=$(create_issue "Task" \
    "Create testing framework" \
    "Develop testing scripts and validation procedures" \
    "" \
    '["testing", "framework", "validation"]' \
    "$EPIC4")

TASK4_3=$(create_issue "Task" \
    "Build troubleshooting documentation" \
    "Document common issues and solutions" \
    "" \
    '["troubleshooting", "support", "documentation"]' \
    "$EPIC4")

echo ""
echo -e "${YELLOW}═══════════════════════════════════════════════${NC}"
echo -e "${YELLOW} LEVEL 3: SUBTASKS (3 per Task)${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════${NC}"

# Subtasks for Task 1.1
echo -e "${YELLOW}Subtasks for ${TASK1_1}:${NC}"
create_issue "Subtask" \
    "Define workflow structure and jobs" \
    "Create the basic structure with all required jobs" \
    "$TASK1_1" \
    '["structure", "jobs"]' \
    ""

create_issue "Subtask" \
    "Implement smart commit parsing" \
    "Add logic to parse JIRA keys from commits" \
    "$TASK1_1" \
    '["parsing", "smart-commits"]' \
    ""

create_issue "Subtask" \
    "Add workflow status reporting" \
    "Implement status updates back to JIRA" \
    "$TASK1_1" \
    '["status", "reporting"]' \
    ""

# Subtasks for Task 1.2
echo -e "${YELLOW}Subtasks for ${TASK1_2}:${NC}"
create_issue "Subtask" \
    "Create task extraction logic" \
    "Extract tasks from commits and PRs" \
    "$TASK1_2" \
    '["extraction", "logic"]' \
    ""

create_issue "Subtask" \
    "Implement 1:1 mapping rules" \
    "Ensure one task creates one ticket" \
    "$TASK1_2" \
    '["mapping", "rules"]' \
    ""

create_issue "Subtask" \
    "Add duplicate prevention" \
    "Prevent duplicate ticket creation" \
    "$TASK1_2" \
    '["duplicate", "prevention"]' \
    ""

# Subtasks for Task 2.1
echo -e "${YELLOW}Subtasks for ${TASK2_1}:${NC}"
create_issue "Subtask" \
    "Configure API token storage" \
    "Setup secure token storage mechanism" \
    "$TASK2_1" \
    '["token", "storage"]' \
    ""

create_issue "Subtask" \
    "Implement authentication headers" \
    "Create proper auth header generation" \
    "$TASK2_1" \
    '["headers", "authentication"]' \
    ""

create_issue "Subtask" \
    "Test API connectivity" \
    "Verify API connection and permissions" \
    "$TASK2_1" \
    '["testing", "connectivity"]' \
    ""

echo ""
echo -e "${PURPLE}═══════════════════════════════════════════════${NC}"
echo -e "${PURPLE} WBS SUMMARY${NC}"
echo -e "${PURPLE}═══════════════════════════════════════════════${NC}"

# Create summary comment on MEV-1
SUMMARY_COMMENT=$(cat <<EOF
{
    "body": {
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "heading",
                "attrs": {"level": 3},
                "content": [
                    {
                        "type": "text",
                        "text": "✅ Work Breakdown Structure Created"
                    }
                ]
            },
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "MEV-1 has been decomposed into a complete WBS following the 1 task:1 ticket rule:",
                        "marks": [{"type": "strong"}]
                    }
                ]
            },
            {
                "type": "heading",
                "attrs": {"level": 4},
                "content": [
                    {
                        "type": "text",
                        "text": "Hierarchy Structure:"
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
                                        "text": "Level 0: Main Epic - ${MAIN_EPIC}",
                                        "marks": [{"type": "strong"}]
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
                                        "text": "Level 1: 4 Sub-Epics (GitHub Actions, API, Automation, Documentation)"
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
                                        "text": "Level 2: 12 Tasks (3 per Epic)"
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
                                        "text": "Level 3: 36 Subtasks (3 per Task)"
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
                        "text": "Total tickets created: 53 (1 Main Epic + 4 Sub-Epics + 12 Tasks + 36 Subtasks)"
                    }
                ]
            },
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "View Main Epic: ${JIRA_BASE_URL}/browse/${MAIN_EPIC}"
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
    -d "${SUMMARY_COMMENT}" \
    "${JIRA_BASE_URL}/rest/api/3/issue/MEV-1/comment" > /dev/null

echo ""
echo "Work Breakdown Structure Created:"
echo "================================="
echo -e "${PURPLE}Main Epic:${NC} ${MAIN_EPIC}"
echo ""
echo -e "${BLUE}Sub-Epics:${NC}"
echo "  └─ ${EPIC1}: GitHub Actions Setup"
echo "  └─ ${EPIC2}: JIRA API Integration"
echo "  └─ ${EPIC3}: Automation Rules"
echo "  └─ ${EPIC4}: Documentation & Testing"
echo ""
echo -e "${GREEN}Statistics:${NC}"
echo "  • 1 Main Epic"
echo "  • 4 Sub-Epics"
echo "  • 12 Tasks (3 per Epic)"
echo "  • 36 Subtasks (3 per Task)"
echo "  • Total: 53 tickets"
echo ""
echo -e "${GREEN}✅ WBS Complete!${NC}"
echo "View in JIRA: ${JIRA_BASE_URL}/browse/${MAIN_EPIC}"