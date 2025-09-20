#!/bin/bash

# GitHub Personal Access Token Setup Script
# This script helps you create and configure a GitHub PAT

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo "========================================="
echo " GitHub Personal Access Token Setup"
echo " MEV Shield Project"
echo "========================================="
echo ""

echo -e "${BLUE}Step 1: Create a Personal Access Token${NC}"
echo "----------------------------------------"
echo "1. Open your browser and go to:"
echo -e "${GREEN}https://github.com/settings/tokens/new${NC}"
echo ""
echo "2. Configure the token:"
echo "   • Note: 'MEV Shield CLI Access'"
echo "   • Expiration: 90 days (or your preference)"
echo "   • Select scopes:"
echo "     ✓ repo (Full control of private repositories)"
echo "     ✓ workflow (Update GitHub Action workflows)"
echo "     ✓ write:packages (optional, for package management)"
echo "     ✓ read:org (optional, for org access)"
echo ""
echo "3. Click 'Generate token' at the bottom"
echo "4. COPY THE TOKEN NOW (it won't be shown again!)"
echo ""
echo -e "${YELLOW}Press Enter when you have copied your token...${NC}"
read -r

echo ""
echo -e "${BLUE}Step 2: Enter Your Token${NC}"
echo "------------------------"
echo -e "${YELLOW}Paste your token here (it will be hidden):${NC}"
read -s GITHUB_TOKEN

if [ -z "$GITHUB_TOKEN" ]; then
    echo -e "${RED}Error: No token provided${NC}"
    exit 1
fi

echo ""
echo ""
echo -e "${BLUE}Step 3: Choose Storage Method${NC}"
echo "-----------------------------"
echo "How would you like to store the token?"
echo "1. GitHub CLI (gh) - Recommended"
echo "2. Environment variable (.zshrc/.bashrc)"
echo "3. Both"
echo ""
read -p "Enter choice (1-3): " CHOICE

case $CHOICE in
    1|3)
        echo ""
        echo -e "${GREEN}Configuring GitHub CLI...${NC}"
        echo "$GITHUB_TOKEN" | gh auth login --with-token
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ GitHub CLI authenticated successfully!${NC}"
            
            # Test the authentication
            echo ""
            echo -e "${BLUE}Testing authentication...${NC}"
            if gh auth status 2>/dev/null; then
                echo -e "${GREEN}✓ Authentication verified!${NC}"
            fi
        else
            echo -e "${YELLOW}⚠ GitHub CLI authentication failed. Continuing...${NC}"
        fi
        ;;&
        
    2|3)
        echo ""
        echo -e "${GREEN}Configuring environment variable...${NC}"
        
        # Detect shell
        if [ -n "$ZSH_VERSION" ]; then
            SHELL_RC="$HOME/.zshrc"
        elif [ -n "$BASH_VERSION" ]; then
            SHELL_RC="$HOME/.bashrc"
        else
            SHELL_RC="$HOME/.profile"
        fi
        
        # Check if already exists
        if grep -q "export GH_TOKEN=" "$SHELL_RC" 2>/dev/null; then
            echo -e "${YELLOW}Updating existing GH_TOKEN in $SHELL_RC${NC}"
            # Use a temporary file for sed on macOS
            sed -i '' "s/export GH_TOKEN=.*/export GH_TOKEN=$GITHUB_TOKEN/" "$SHELL_RC"
        else
            echo -e "${GREEN}Adding GH_TOKEN to $SHELL_RC${NC}"
            echo "" >> "$SHELL_RC"
            echo "# GitHub Personal Access Token for MEV Shield" >> "$SHELL_RC"
            echo "export GH_TOKEN=$GITHUB_TOKEN" >> "$SHELL_RC"
            echo "export GITHUB_TOKEN=$GITHUB_TOKEN" >> "$SHELL_RC"
        fi
        
        echo -e "${GREEN}✓ Environment variable configured!${NC}"
        echo -e "${YELLOW}Run 'source $SHELL_RC' or restart terminal to apply${NC}"
        
        # Also set for current session
        export GH_TOKEN=$GITHUB_TOKEN
        export GITHUB_TOKEN=$GITHUB_TOKEN
        ;;
        
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${BLUE}Step 4: Test Workflow Trigger${NC}"
echo "-----------------------------"
read -p "Would you like to test by triggering the JIRA sync workflow? (y/n): " TEST

if [ "$TEST" = "y" ] || [ "$TEST" = "Y" ]; then
    echo ""
    echo -e "${GREEN}Triggering jira-bidirectional-sync workflow...${NC}"
    
    if command -v gh &> /dev/null && gh auth status &> /dev/null; then
        # Use GitHub CLI
        gh workflow run jira-bidirectional-sync.yml \
            -f direction=both \
            -f dry_run=false
        echo -e "${GREEN}✓ Workflow triggered via GitHub CLI!${NC}"
    else
        # Use curl
        curl -s -X POST \
            -H "Accept: application/vnd.github.v3+json" \
            -H "Authorization: token $GITHUB_TOKEN" \
            https://api.github.com/repos/SUBBUAURIGRAPH/MEV-Shield/actions/workflows/jira-bidirectional-sync.yml/dispatches \
            -d '{"ref":"main","inputs":{"direction":"both","dry_run":"false"}}' \
            && echo -e "${GREEN}✓ Workflow triggered via API!${NC}" \
            || echo -e "${RED}✗ Failed to trigger workflow${NC}"
    fi
    
    echo ""
    echo "View workflow at:"
    echo -e "${BLUE}https://github.com/SUBBUAURIGRAPH/MEV-Shield/actions${NC}"
fi

echo ""
echo "========================================="
echo -e "${GREEN} Setup Complete!${NC}"
echo "========================================="
echo ""
echo "You can now use:"
echo "• gh workflow run <workflow-name>"
echo "• gh api <endpoint>"
echo "• gh pr create"
echo "• gh issue create"
echo "• ./trigger-sync.sh"
echo ""
echo "Token scopes configured:"
echo "• repo - Full repository access"
echo "• workflow - GitHub Actions access"
echo ""

# Create a reference file
cat > ~/.github-mev-shield <<EOF
# GitHub Token Configuration for MEV Shield
# Created: $(date)
# Token Prefix: ghp_${GITHUB_TOKEN:4:8}...
# Scopes: repo, workflow
# Repository: SUBBUAURIGRAPH/MEV-Shield

# Common commands:
# gh workflow run jira-bidirectional-sync.yml
# gh workflow list
# gh run list
# gh api repos/SUBBUAURIGRAPH/MEV-Shield
EOF

echo "Reference saved to: ~/.github-mev-shield"
echo ""
echo -e "${GREEN}Happy coding! 🚀${NC}"