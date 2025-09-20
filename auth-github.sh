#!/bin/bash

echo "========================================="
echo " GitHub CLI Authentication Setup"
echo "========================================="
echo ""
echo "To authenticate and run the workflow:"
echo ""
echo "1. Run: gh auth login"
echo "2. Select: GitHub.com"
echo "3. Select: HTTPS"
echo "4. Authenticate with: Web browser"
echo "5. Follow browser instructions"
echo ""
echo "After authentication, the workflow will run automatically."
echo ""

# Check if already authenticated
if gh auth status >/dev/null 2>&1; then
    echo "✅ Already authenticated!"
    echo ""
    echo "Running jira-bidirectional-sync workflow..."
    gh workflow run jira-bidirectional-sync.yml \
        -f direction=both \
        -f dry_run=false
    echo ""
    echo "✅ Workflow triggered successfully!"
    echo ""
    echo "View runs at: https://github.com/SUBBUAURIGRAPH/MEV-Shield/actions"
else
    echo "Please run: gh auth login"
    echo ""
    echo "Or set GitHub token:"
    echo "export GH_TOKEN=your_github_personal_access_token"
    echo ""
    echo "To create a token:"
    echo "1. Go to: https://github.com/settings/tokens/new"
    echo "2. Select scopes: repo, workflow"
    echo "3. Generate token"
    echo "4. Run: export GH_TOKEN=ghp_YOUR_TOKEN_HERE"
    echo "5. Run this script again"
fi