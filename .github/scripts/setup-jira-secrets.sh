#!/bin/bash

# JIRA Integration Secret Setup Script
# This script helps configure GitHub secrets for JIRA integration

set -e

echo "================================================"
echo "   MEV Shield - JIRA Integration Setup"
echo "================================================"
echo ""

# Check if GitHub CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed."
    echo "Please install it first: https://cli.github.com/"
    exit 1
fi

# Check if user is authenticated
if ! gh auth status &> /dev/null; then
    echo "❌ Not authenticated with GitHub CLI."
    echo "Please run: gh auth login"
    exit 1
fi

echo "✅ GitHub CLI is installed and authenticated"
echo ""

# Repository details
REPO="SUBBUAURIGRAPH/MEV-Shield"
echo "📦 Repository: $REPO"
echo ""

# Set the secrets
echo "🔐 Setting up GitHub Secrets..."
echo ""

# JIRA User Email
JIRA_EMAIL="subbu@aurigraph.io"
echo "Setting JIRA_USER_EMAIL..."
gh secret set JIRA_USER_EMAIL --body="$JIRA_EMAIL" --repo="$REPO"
echo "✅ JIRA_USER_EMAIL set to: $JIRA_EMAIL"

# JIRA API Token (stored securely)
echo ""
echo "Setting JIRA_API_TOKEN..."
echo "⚠️  For security, please enter your JIRA API token when prompted."
echo "The token will not be displayed on screen."
echo ""

# Prompt for token
read -s -p "Enter JIRA API Token: " JIRA_TOKEN
echo ""

if [ -z "$JIRA_TOKEN" ]; then
    echo "❌ No token provided. Exiting."
    exit 1
fi

# Set the token as a secret
echo "$JIRA_TOKEN" | gh secret set JIRA_API_TOKEN --repo="$REPO"
echo "✅ JIRA_API_TOKEN has been set securely"

echo ""
echo "================================================"
echo "   Testing JIRA Connection"
echo "================================================"
echo ""

# Test the connection
echo "Testing API connection to JIRA..."
RESPONSE=$(curl -s -X GET \
    -H "Authorization: Basic $(echo -n "$JIRA_EMAIL:$JIRA_TOKEN" | base64)" \
    -w "\n%{http_code}" \
    "https://aurigraphdlt.atlassian.net/rest/api/3/myself" 2>/dev/null)

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Successfully connected to JIRA!"
    USER_INFO=$(echo "$RESPONSE" | head -n-1)
    echo "Connected as: $(echo "$USER_INFO" | jq -r '.displayName')"
else
    echo "❌ Failed to connect to JIRA (HTTP $HTTP_CODE)"
    echo "Please check your API token and try again."
    exit 1
fi

echo ""
echo "================================================"
echo "   Setup Complete!"
echo "================================================"
echo ""
echo "✅ GitHub Secrets have been configured successfully!"
echo ""
echo "Next steps:"
echo "1. Trigger the test workflow to verify integration:"
echo "   gh workflow run test-jira-integration.yml --repo=$REPO"
echo ""
echo "2. Or visit GitHub Actions:"
echo "   https://github.com/$REPO/actions/workflows/test-jira-integration.yml"
echo ""
echo "3. Use MEV-XXX in your commit messages to link to JIRA issues"
echo ""
echo "🔒 Security Reminder:"
echo "   - Never share your API token publicly"
echo "   - Rotate tokens every 90 days"
echo "   - Delete tokens when no longer needed"
echo ""
echo "JIRA Board: https://aurigraphdlt.atlassian.net/jira/software/projects/MEV/boards/855"