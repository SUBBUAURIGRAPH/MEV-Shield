#!/bin/bash

# JIRA-GitHub Sync Script
# Syncs GitHub Issues/PRs with JIRA tickets
# Updates descriptions, status, and metadata

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo "========================================="
echo " JIRA-GitHub Synchronization Tool"
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

# GitHub configuration (can be passed as arguments or env vars)
GITHUB_OWNER=${1:-"your-github-owner"}
GITHUB_REPO=${2:-"mev-shield"}
GITHUB_TOKEN=${3:-$GITHUB_TOKEN}

# Status mapping
declare -A STATUS_MAP=(
    ["open"]="To Do"
    ["in_progress"]="In Progress"
    ["review"]="In Review"
    ["closed"]="Done"
    ["merged"]="Done"
    ["draft"]="In Progress"
)

# Function to get GitHub issues/PRs
get_github_items() {
    local TYPE=$1  # issues or pulls
    
    echo -e "${BLUE}Fetching GitHub ${TYPE}...${NC}"
    
    ITEMS=$(curl -s \
        -H "Authorization: token ${GITHUB_TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/${TYPE}?state=all&per_page=100")
    
    echo "$ITEMS"
}

# Function to extract JIRA keys from text
extract_jira_keys() {
    local TEXT=$1
    echo "$TEXT" | grep -oE 'MEV-[0-9]+' | sort -u
}

# Function to get JIRA issue
get_jira_issue() {
    local KEY=$1
    
    curl -s \
        -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
        "${JIRA_BASE_URL}/rest/api/3/issue/${KEY}"
}

# Function to update JIRA description
update_jira_description() {
    local KEY=$1
    local DESCRIPTION=$2
    
    echo -e "${YELLOW}  Updating description for ${KEY}${NC}"
    
    # Prepare JSON payload
    PAYLOAD=$(jq -n \
        --arg desc "$DESCRIPTION" \
        '{
            fields: {
                description: {
                    type: "doc",
                    version: 1,
                    content: [
                        {
                            type: "paragraph",
                            content: [
                                {
                                    type: "text",
                                    text: $desc
                                }
                            ]
                        }
                    ]
                }
            }
        }')
    
    RESPONSE=$(curl -s -X PUT \
        -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
        -H "Content-Type: application/json" \
        -d "$PAYLOAD" \
        "${JIRA_BASE_URL}/rest/api/3/issue/${KEY}")
    
    if [[ $? -eq 0 ]]; then
        echo -e "${GREEN}    ✓ Description updated${NC}"
    else
        echo -e "${RED}    ✗ Failed to update description${NC}"
    fi
}

# Function to transition JIRA status
transition_jira_status() {
    local KEY=$1
    local TARGET_STATUS=$2
    
    echo -e "${YELLOW}  Transitioning ${KEY} to ${TARGET_STATUS}${NC}"
    
    # Get available transitions
    TRANSITIONS=$(curl -s \
        -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
        "${JIRA_BASE_URL}/rest/api/3/issue/${KEY}/transitions")
    
    # Find transition ID
    TRANSITION_ID=$(echo "$TRANSITIONS" | jq -r ".transitions[] | select(.to.name==\"${TARGET_STATUS}\") | .id" | head -1)
    
    if [[ -n "$TRANSITION_ID" ]]; then
        # Perform transition
        curl -s -X POST \
            -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
            -H "Content-Type: application/json" \
            -d "{\"transition\": {\"id\": \"$TRANSITION_ID\"}}" \
            "${JIRA_BASE_URL}/rest/api/3/issue/${KEY}/transitions"
        
        echo -e "${GREEN}    ✓ Status updated to ${TARGET_STATUS}${NC}"
    else
        echo -e "${YELLOW}    ⚠ No transition available to ${TARGET_STATUS}${NC}"
    fi
}

# Function to add GitHub link to JIRA
add_github_link() {
    local JIRA_KEY=$1
    local GITHUB_URL=$2
    local TITLE=$3
    
    echo -e "${YELLOW}  Adding GitHub link to ${JIRA_KEY}${NC}"
    
    LINK_PAYLOAD=$(jq -n \
        --arg url "$GITHUB_URL" \
        --arg title "$TITLE" \
        '{
            object: {
                url: $url,
                title: $title,
                icon: {
                    url16x16: "https://github.githubassets.com/favicon.ico"
                }
            }
        }')
    
    curl -s -X POST \
        -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
        -H "Content-Type: application/json" \
        -d "$LINK_PAYLOAD" \
        "${JIRA_BASE_URL}/rest/api/3/issue/${JIRA_KEY}/remotelink"
    
    echo -e "${GREEN}    ✓ GitHub link added${NC}"
}

# Function to sync single GitHub item to JIRA
sync_github_to_jira() {
    local ITEM=$1
    local TYPE=$2  # issue or pr
    
    # Extract data
    NUMBER=$(echo "$ITEM" | jq -r '.number')
    TITLE=$(echo "$ITEM" | jq -r '.title')
    BODY=$(echo "$ITEM" | jq -r '.body // ""')
    STATE=$(echo "$ITEM" | jq -r '.state')
    HTML_URL=$(echo "$ITEM" | jq -r '.html_url')
    USER=$(echo "$ITEM" | jq -r '.user.login')
    CREATED=$(echo "$ITEM" | jq -r '.created_at')
    UPDATED=$(echo "$ITEM" | jq -r '.updated_at')
    
    # For PRs, check if merged
    if [[ "$TYPE" == "pr" ]]; then
        MERGED=$(echo "$ITEM" | jq -r '.merged')
        DRAFT=$(echo "$ITEM" | jq -r '.draft')
        
        if [[ "$MERGED" == "true" ]]; then
            GITHUB_STATUS="merged"
        elif [[ "$DRAFT" == "true" ]]; then
            GITHUB_STATUS="draft"
        else
            GITHUB_STATUS="$STATE"
        fi
    else
        GITHUB_STATUS="$STATE"
    fi
    
    # Extract JIRA keys
    JIRA_KEYS=$(extract_jira_keys "$TITLE $BODY")
    
    if [[ -z "$JIRA_KEYS" ]]; then
        return
    fi
    
    echo -e "${BLUE}Syncing GitHub ${TYPE} #${NUMBER} to JIRA${NC}"
    echo "  Title: $TITLE"
    echo "  State: $GITHUB_STATUS"
    echo "  JIRA Keys: $JIRA_KEYS"
    
    # Process each JIRA key
    for KEY in $JIRA_KEYS; do
        echo -e "${YELLOW}Processing ${KEY}...${NC}"
        
        # Build description
        DESCRIPTION="📋 GitHub ${TYPE^} #${NUMBER}

Title: ${TITLE}
Author: @${USER}
Status: ${STATE}
Created: ${CREATED}
Updated: ${UPDATED}
URL: ${HTML_URL}

--- Description ---
${BODY}

--- Metadata ---
Synced at: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Source: GitHub API"
        
        # Update JIRA
        update_jira_description "$KEY" "$DESCRIPTION"
        
        # Map and update status
        TARGET_STATUS=${STATUS_MAP[$GITHUB_STATUS]:-"To Do"}
        transition_jira_status "$KEY" "$TARGET_STATUS"
        
        # Add GitHub link
        add_github_link "$KEY" "$HTML_URL" "GitHub ${TYPE^} #${NUMBER}"
        
        # Add sync comment
        add_sync_comment "$KEY" "$TYPE" "$NUMBER" "$GITHUB_STATUS" "$TARGET_STATUS"
    done
}

# Function to add sync comment to JIRA
add_sync_comment() {
    local KEY=$1
    local TYPE=$2
    local NUMBER=$3
    local GITHUB_STATUS=$4
    local JIRA_STATUS=$5
    
    COMMENT="🔄 Synchronized from GitHub ${TYPE} #${NUMBER}
    
GitHub Status: ${GITHUB_STATUS}
JIRA Status: ${JIRA_STATUS}
Sync Time: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    
    COMMENT_PAYLOAD=$(jq -n \
        --arg comment "$COMMENT" \
        '{
            body: {
                type: "doc",
                version: 1,
                content: [
                    {
                        type: "paragraph",
                        content: [
                            {
                                type: "text",
                                text: $comment
                            }
                        ]
                    }
                ]
            }
        }')
    
    curl -s -X POST \
        -H "Authorization: Basic $(echo -n ${JIRA_USER_EMAIL}:${JIRA_API_TOKEN} | base64)" \
        -H "Content-Type: application/json" \
        -d "$COMMENT_PAYLOAD" \
        "${JIRA_BASE_URL}/rest/api/3/issue/${KEY}/comment" > /dev/null
}

# Main sync process
main() {
    echo -e "${GREEN}Starting GitHub to JIRA synchronization...${NC}"
    echo ""
    
    # Check GitHub token
    if [[ -z "$GITHUB_TOKEN" ]]; then
        echo -e "${RED}Error: GitHub token not provided${NC}"
        echo "Usage: $0 [github-owner] [github-repo] [github-token]"
        exit 1
    fi
    
    # Sync Issues
    echo -e "${BLUE}=== Syncing Issues ===${NC}"
    ISSUES=$(get_github_items "issues")
    
    echo "$ISSUES" | jq -c '.[]' | while read -r ISSUE; do
        sync_github_to_jira "$ISSUE" "issue"
    done
    
    # Sync Pull Requests
    echo ""
    echo -e "${BLUE}=== Syncing Pull Requests ===${NC}"
    PULLS=$(get_github_items "pulls")
    
    echo "$PULLS" | jq -c '.[]' | while read -r PR; do
        sync_github_to_jira "$PR" "pr"
    done
    
    echo ""
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN} Synchronization Complete!${NC}"
    echo -e "${GREEN}=========================================${NC}"
}

# Run main function
main