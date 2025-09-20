#!/bin/bash

# Alternative method to trigger workflow using curl
# Replace YOUR_GITHUB_TOKEN with your actual token

GITHUB_TOKEN="${GH_TOKEN:-YOUR_GITHUB_TOKEN}"
OWNER="SUBBUAURIGRAPH"
REPO="MEV-Shield"

if [ "$GITHUB_TOKEN" = "YOUR_GITHUB_TOKEN" ]; then
    echo "Please set your GitHub token:"
    echo "export GH_TOKEN=ghp_YOUR_TOKEN_HERE"
    echo ""
    echo "Or edit this script and replace YOUR_GITHUB_TOKEN"
    exit 1
fi

echo "Triggering JIRA Bidirectional Sync workflow..."

curl -X POST \
  -H "Accept: application/vnd.github.v3+json" \
  -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/repos/$OWNER/$REPO/actions/workflows/jira-bidirectional-sync.yml/dispatches \
  -d '{
    "ref": "main",
    "inputs": {
      "direction": "both",
      "dry_run": "false"
    }
  }'

echo ""
echo "✅ Workflow triggered!"
echo "View at: https://github.com/$OWNER/$REPO/actions"