# JIRA-GitHub Synchronization Guide

## Overview

This document describes the comprehensive JIRA-GitHub synchronization system for the MEV Shield project. The system provides bidirectional sync between JIRA tickets and GitHub issues/PRs, ensuring both platforms stay updated with the latest information.

## Features

### 🔄 Bidirectional Synchronization
- **GitHub → JIRA**: Updates JIRA tickets when GitHub issues/PRs change
- **JIRA → GitHub**: Creates/updates GitHub issues from JIRA tickets
- **Real-time sync**: Via webhooks for immediate updates
- **Scheduled sync**: Regular synchronization every 15 minutes

### 📊 Status Mapping
Automatic status mapping between platforms:

| GitHub Status | JIRA Status |
|--------------|-------------|
| open | To Do |
| in_progress | In Progress |
| draft (PR) | In Progress |
| review_required | In Review |
| merged | Done |
| closed | Done |

### 📝 Description Synchronization
- Full description sync from GitHub PR/Issue body to JIRA
- Metadata preservation (author, timestamps, URLs)
- Markdown to JIRA Doc format conversion
- Comment synchronization

### 🏷️ Label Management
- GitHub labels sync to JIRA labels
- Automatic labeling with sync metadata
- Priority mapping from JIRA to GitHub

## Setup Instructions

### 1. Prerequisites

- JIRA project with API access
- GitHub repository with Actions enabled
- API tokens for both platforms

### 2. Configure GitHub Secrets

Set the following secrets in your GitHub repository:

```bash
JIRA_BASE_URL=https://aurigraphdlt.atlassian.net
JIRA_USER_EMAIL=your-email@domain.com
JIRA_API_TOKEN=your-jira-api-token
```

To set these via GitHub CLI:
```bash
gh secret set JIRA_BASE_URL --body "https://aurigraphdlt.atlassian.net"
gh secret set JIRA_USER_EMAIL --body "your-email@domain.com"
gh secret set JIRA_API_TOKEN --body "your-api-token"
```

### 3. Configure JIRA Webhooks

1. Go to JIRA Settings → System → Webhooks
2. Create a new webhook with:
   - **URL**: Your webhook endpoint (e.g., `https://your-domain.com/webhook/jira`)
   - **Events**: 
     - Issue created
     - Issue updated
     - Issue deleted
   - **JQL**: `project = MEV`

### 4. Deploy Webhook Handler

Option A: Deploy as serverless function (Vercel, Netlify, AWS Lambda)
```javascript
// api/jira-webhook.js
const { handleWebhook } = require('./scripts/jira-webhook-handler');

module.exports = async (req, res) => {
    await handleWebhook(req.body, req.headers['x-hub-signature-256']);
    res.status(200).json({ success: true });
};
```

Option B: Deploy as Express server
```bash
cd scripts
npm install express
node jira-webhook-handler.js
```

## Usage

### Manual Synchronization

#### Sync Specific JIRA Ticket
```bash
# Using the sync script
./scripts/jira-github-sync.sh

# Using GitHub Actions (manual trigger)
gh workflow run jira-sync.yml -f jira_key=MEV-123
```

#### Bulk Sync All Tickets
```bash
# Trigger bidirectional sync
gh workflow run jira-bidirectional-sync.yml -f direction=both

# GitHub to JIRA only
gh workflow run jira-bidirectional-sync.yml -f direction=github-to-jira

# JIRA to GitHub only
gh workflow run jira-bidirectional-sync.yml -f direction=jira-to-github
```

### Automatic Synchronization

The system automatically syncs when:

1. **GitHub Events** (via `jira-sync.yml` workflow):
   - Issue opened, edited, closed
   - PR opened, edited, merged, closed
   - Comments added
   - Labels changed

2. **JIRA Events** (via webhooks):
   - Issue created
   - Issue updated (status, description, summary)
   - Issue deleted

3. **Scheduled** (via `jira-bidirectional-sync.yml`):
   - Every 15 minutes (configurable)

### Smart Commit Messages

Use JIRA smart commits in your Git messages:

```bash
# Link to JIRA ticket
git commit -m "MEV-123: Implement user authentication"

# Log time
git commit -m "MEV-123: Fix login bug #time 2h"

# Add comment
git commit -m "MEV-123: Update API #comment Fixed the timeout issue"

# Resolve ticket
git commit -m "MEV-123: Complete feature #resolve"

# Multiple actions
git commit -m "MEV-123: Final implementation #time 3h #comment All tests passing #resolve"
```

## Sync Behavior

### Creating New Items

#### GitHub Issue/PR → JIRA Ticket
When a GitHub issue/PR contains a JIRA key (e.g., MEV-123):
1. Updates the existing JIRA ticket
2. Adds GitHub link to JIRA
3. Syncs status and description

When no JIRA key is found:
1. Optionally creates new JIRA ticket (if configured)
2. Adds JIRA key to GitHub labels

#### JIRA Ticket → GitHub Issue
When a JIRA ticket is created:
1. Creates corresponding GitHub issue
2. Adds JIRA key as label
3. Links back to JIRA

### Updating Existing Items

#### Status Changes
- GitHub merged PR → JIRA "Done"
- GitHub closed issue → JIRA "Done" 
- JIRA "In Progress" → GitHub remains open
- JIRA "Done" → GitHub closed

#### Description Updates
- Preserves existing content
- Appends sync metadata
- Maintains formatting where possible

## Testing

### Run Test Suite
```bash
# Make script executable
chmod +x scripts/test-jira-sync.sh

# Run tests
./scripts/test-jira-sync.sh
```

### Test Individual Components

#### Test JIRA Connection
```bash
./test-jira-integration.sh
```

#### Test GitHub Workflow
```bash
# Create test issue with JIRA key
gh issue create --title "MEV-123: Test sync" --body "Testing sync functionality"

# Check JIRA ticket was updated
open https://aurigraphdlt.atlassian.net/browse/MEV-123
```

#### Test Webhook
```bash
curl -X POST https://your-webhook-url/webhook/jira \
  -H "Content-Type: application/json" \
  -d '{
    "webhookEvent": "jira:issue_created",
    "issue": {
      "key": "MEV-999",
      "fields": {
        "summary": "Test webhook",
        "status": {"name": "To Do"}
      }
    }
  }'
```

## Monitoring

### Check Sync Status

#### GitHub Actions
```bash
# View recent workflow runs
gh run list --workflow=jira-sync.yml

# View specific run details
gh run view <run-id>

# View workflow logs
gh run view <run-id> --log
```

#### JIRA Audit Log
1. Go to JIRA → Settings → System → Audit log
2. Filter by "Issue updated" events
3. Look for API user activity

### Sync Reports

Check GitHub Actions summary for sync reports:
1. Go to Actions tab in GitHub
2. Click on a workflow run
3. Check "Summary" section for sync details

## Troubleshooting

### Common Issues

#### Sync Not Working
- Check GitHub secrets are set correctly
- Verify JIRA API token has correct permissions
- Check webhook URL is accessible
- Review workflow logs for errors

#### Status Not Updating
- Verify status mapping configuration
- Check if transition is available in JIRA workflow
- Ensure user has permission to transition

#### Duplicate Issues
- Check for existing JIRA key in GitHub labels
- Verify webhook deduplication logic
- Review sync timestamp tracking

#### Missing Updates
- Check webhook events are configured
- Verify GitHub Actions triggers
- Review scheduled sync frequency

### Debug Mode

Enable debug logging:
```bash
# For sync script
DEBUG=true ./scripts/jira-github-sync.sh

# For GitHub Actions
gh workflow run jira-sync.yml -f debug=true
```

## Configuration Files

### Workflows
- `.github/workflows/jira-sync.yml` - Main sync workflow
- `.github/workflows/jira-bidirectional-sync.yml` - Scheduled bidirectional sync
- `.github/workflows/jira-integration.yml` - Smart commit processing
- `.github/workflows/jira-task-management.yml` - 1:1 task mapping

### Scripts
- `scripts/jira-github-sync.sh` - Manual sync script
- `scripts/jira-webhook-handler.js` - Webhook handler
- `scripts/test-jira-sync.sh` - Test suite

### Configuration
- `.env.jira` - Local JIRA configuration (git-ignored)

## Security Considerations

1. **API Tokens**: Store securely as GitHub secrets
2. **Webhook Signatures**: Verify webhook authenticity
3. **Rate Limiting**: Implement rate limits for API calls
4. **Permissions**: Use minimal required permissions
5. **Audit Logging**: Track all sync operations

## Best Practices

1. **Use JIRA Keys**: Always include JIRA keys in PR/issue titles
2. **Smart Commits**: Use smart commit syntax for automation
3. **Regular Sync**: Keep scheduled sync enabled
4. **Monitor Logs**: Regularly check sync reports
5. **Test Changes**: Test sync configuration changes in dev environment

## Support

For issues or questions:
1. Check workflow logs for detailed error messages
2. Review this documentation
3. Create an issue in the GitHub repository
4. Contact the development team

---

**Last Updated**: 2024
**Version**: 1.0.0
**Status**: Active