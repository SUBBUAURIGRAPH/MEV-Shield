#!/bin/bash

# Quick setup script for GitHub secrets
# Run this to configure JIRA integration

echo "Setting up GitHub Secrets for JIRA Integration..."
echo ""

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "Installing GitHub CLI..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install gh
    else
        echo "Please install GitHub CLI: https://cli.github.com/"
        exit 1
    fi
fi

# Authenticate if needed
if ! gh auth status &> /dev/null; then
    echo "Authenticating with GitHub..."
    gh auth login
fi

# Set the secrets
echo "Configuring secrets for SUBBUAURIGRAPH/MEV-Shield..."
gh secret set JIRA_USER_EMAIL --body="subbu@aurigraph.io" --repo="SUBBUAURIGRAPH/MEV-Shield"
echo "✅ JIRA_USER_EMAIL configured"

# For security, we'll prompt for the token
echo ""
echo "Enter JIRA API Token (will be hidden):"
read -s JIRA_TOKEN
echo "$JIRA_TOKEN" | gh secret set JIRA_API_TOKEN --repo="SUBBUAURIGRAPH/MEV-Shield"
echo "✅ JIRA_API_TOKEN configured"

echo ""
echo "Setup complete! Testing the integration..."
gh workflow run test-jira-integration.yml --repo=SUBBUAURIGRAPH/MEV-Shield -f test_type=all -f issue_key=MEV-1

echo ""
echo "Check the test results at:"
echo "https://github.com/SUBBUAURIGRAPH/MEV-Shield/actions/workflows/test-jira-integration.yml"