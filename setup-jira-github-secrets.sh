#!/bin/bash

# Setup GitHub Secrets for JIRA Integration
# This script helps configure the required secrets for JIRA-GitHub integration

set -e

echo "========================================="
echo " GitHub Secrets Setup for JIRA Integration"
echo " MEV Shield Project - 1 Task : 1 Ticket"
echo "========================================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI (gh) is not installed${NC}"
    echo "Please install it from: https://cli.github.com/"
    exit 1
fi

# Check if logged into GitHub
if ! gh auth status &> /dev/null; then
    echo -e "${YELLOW}Please login to GitHub CLI first${NC}"
    gh auth login
fi

echo -e "${BLUE}Setting up GitHub Secrets for JIRA Integration${NC}"
echo ""

# Get repository name
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "")
if [ -z "$REPO" ]; then
    read -p "Enter GitHub repository (owner/repo): " REPO
fi

echo -e "${GREEN}Using repository: $REPO${NC}"
echo ""

# JIRA Configuration
echo -e "${YELLOW}=== JIRA Configuration ===${NC}"
echo "Please provide your JIRA details:"
echo ""

# JIRA Base URL
read -p "JIRA Base URL (default: https://aurigraphdlt.atlassian.net): " JIRA_BASE_URL
JIRA_BASE_URL=${JIRA_BASE_URL:-"https://aurigraphdlt.atlassian.net"}

# JIRA User Email
read -p "JIRA User Email (e.g., subbu@aurigraph.io): " JIRA_USER_EMAIL
if [ -z "$JIRA_USER_EMAIL" ]; then
    echo -e "${RED}Error: JIRA User Email is required${NC}"
    exit 1
fi

# JIRA API Token
echo ""
echo "To generate a JIRA API Token:"
echo "1. Go to: https://id.atlassian.com/manage-profile/security/api-tokens"
echo "2. Click 'Create API token'"
echo "3. Give it a name (e.g., 'GitHub Actions')"
echo "4. Copy the token"
echo ""
read -s -p "JIRA API Token: " JIRA_API_TOKEN
echo ""

if [ -z "$JIRA_API_TOKEN" ]; then
    echo -e "${RED}Error: JIRA API Token is required${NC}"
    exit 1
fi

# Set GitHub Secrets
echo ""
echo -e "${BLUE}Setting GitHub Secrets...${NC}"

# Set JIRA_BASE_URL
gh secret set JIRA_BASE_URL --body "$JIRA_BASE_URL" --repo "$REPO" 2>/dev/null && \
    echo -e "${GREEN}✓ JIRA_BASE_URL set${NC}" || \
    echo -e "${RED}✗ Failed to set JIRA_BASE_URL${NC}"

# Set JIRA_USER_EMAIL
gh secret set JIRA_USER_EMAIL --body "$JIRA_USER_EMAIL" --repo "$REPO" 2>/dev/null && \
    echo -e "${GREEN}✓ JIRA_USER_EMAIL set${NC}" || \
    echo -e "${RED}✗ Failed to set JIRA_USER_EMAIL${NC}"

# Set JIRA_API_TOKEN
gh secret set JIRA_API_TOKEN --body "$JIRA_API_TOKEN" --repo "$REPO" 2>/dev/null && \
    echo -e "${GREEN}✓ JIRA_API_TOKEN set${NC}" || \
    echo -e "${RED}✗ Failed to set JIRA_API_TOKEN${NC}"

echo ""
echo -e "${YELLOW}=== Epic Configuration ===${NC}"
echo "Would you like to create default epics for MEV Shield components?"
read -p "Create epics? (y/n): " CREATE_EPICS

if [ "$CREATE_EPICS" = "y" ]; then
    echo ""
    echo "Creating epics via JIRA API..."
    
    # Base64 encode credentials
    AUTH_HEADER=$(echo -n "$JIRA_USER_EMAIL:$JIRA_API_TOKEN" | base64)
    
    # Epic names
    EPICS=(
        "Frontend Development:MEV Shield UI/UX and Dashboard Components"
        "Backend Development:API, Smart Contracts, and Core Logic"
        "Security Implementation:Audits, Penetration Testing, and Hardening"
        "Deployment & Infrastructure:CI/CD, Docker, Kubernetes"
        "Testing & QA:Unit, Integration, and E2E Testing"
    )
    
    for EPIC in "${EPICS[@]}"; do
        IFS=':' read -r EPIC_NAME EPIC_DESC <<< "$EPIC"
        
        RESPONSE=$(curl -s -X POST \
            -H "Authorization: Basic $AUTH_HEADER" \
            -H "Content-Type: application/json" \
            -d "{
                \"fields\": {
                    \"project\": {\"key\": \"MEV\"},
                    \"summary\": \"$EPIC_NAME\",
                    \"description\": \"$EPIC_DESC\",
                    \"issuetype\": {\"name\": \"Epic\"},
                    \"labels\": [\"mev-shield\", \"github-integration\"]
                }
            }" \
            "$JIRA_BASE_URL/rest/api/3/issue")
        
        EPIC_KEY=$(echo "$RESPONSE" | grep -o '"key":"[^"]*"' | cut -d'"' -f4)
        
        if [ ! -z "$EPIC_KEY" ]; then
            echo -e "${GREEN}✓ Created Epic: $EPIC_KEY - $EPIC_NAME${NC}"
        else
            echo -e "${YELLOW}⚠ Epic might already exist or failed: $EPIC_NAME${NC}"
        fi
    done
fi

echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN} Setup Complete!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "Next steps:"
echo "1. Commit and push the workflow files to your repository"
echo "2. Use smart commits with JIRA issue keys (e.g., 'MEV-123: Fix login bug')"
echo "3. Create PRs with JIRA keys in title or description"
echo "4. Tasks will be automatically created and linked to epics"
echo ""
echo -e "${BLUE}Smart Commit Examples:${NC}"
echo "  git commit -m 'MEV-123: Implement user authentication'"
echo "  git commit -m 'Add dashboard component #time 2h #comment Initial implementation'"
echo "  git commit -m 'Fix security vulnerability #resolve'"
echo ""
echo -e "${YELLOW}Workflow Commands:${NC}"
echo "  # Manually create a task"
echo "  gh workflow run jira-task-management.yml -f task_name='New Feature' -f epic_key='MEV-101'"
echo ""
echo "  # View workflow runs"
echo "  gh run list --workflow=jira-integration.yml"
echo ""
echo -e "${GREEN}Happy coding with automated JIRA tracking!${NC}"