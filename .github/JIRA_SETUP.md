# JIRA Integration Setup for MEV Shield

## Overview

This repository is configured to automatically sync with JIRA board: https://aurigraphdlt.atlassian.net/jira/software/projects/MEV/boards/855

## Features

### 1. Automatic Issue Syncing
- Commits mentioning `MEV-XXX` automatically update corresponding JIRA tickets
- Pull requests linked to JIRA issues update ticket status
- GitHub issues automatically create JIRA tickets

### 2. Smart Transitions
- **To Do → In Progress**: When feature branch is created
- **In Progress → Code Review**: When PR is opened
- **Code Review → In Progress**: When PR converted to draft
- **Code Review → Testing**: When PR merged to develop
- **Testing → Done**: When PR merged to main

### 3. Sprint Management
- Daily sync between GitHub and JIRA
- Automatic sprint report generation
- Release version tracking

## Setup Instructions

### Step 1: Create JIRA API Token

1. Go to https://id.atlassian.com/manage-profile/security/api-tokens
2. Click "Create API token"
3. Name it "GitHub Actions - MEV Shield"
4. Copy the token (you'll need this for GitHub secrets)

### Step 2: Add GitHub Secrets

Go to your repository settings → Secrets and variables → Actions, then add:

```bash
JIRA_USER_EMAIL=subbu@aurigraph.io
JIRA_API_TOKEN=<your-api-token-from-step-1>
```

### Step 3: Configure JIRA Project Settings

1. Go to https://aurigraphdlt.atlassian.net/jira/software/projects/MEV/boards/855
2. Ensure these workflow states exist:
   - To Do
   - In Progress
   - Code Review
   - Testing
   - Done

### Step 4: Enable GitHub Webhooks (Optional)

For real-time updates from JIRA to GitHub:

1. In JIRA, go to Settings → System → Webhooks
2. Create webhook:
   - Name: `GitHub MEV Shield Sync`
   - URL: `https://api.github.com/repos/SUBBUAURIGRAPH/MEV-Shield/dispatches`
   - Events: Issue created, updated, deleted
   - Headers:
     ```
     Authorization: token <GitHub Personal Access Token>
     Accept: application/vnd.github.v3+json
     ```

## Usage

### Linking Commits to JIRA

Include the JIRA issue key in your commit message:

```bash
git commit -m "MEV-123: Implement threshold encryption"
```

### Linking PRs to JIRA

Include the JIRA issue key in your PR title or description:

```markdown
MEV-456: Add VDF ordering system

This PR implements the VDF-based ordering system for MEV protection.
```

### Branch Naming Convention

Use JIRA issue keys in branch names for automatic tracking:

```bash
git checkout -b feature/MEV-789-neural-predictor
git checkout -b bugfix/MEV-321-fix-encryption
git checkout -b hotfix/MEV-654-critical-fix
```

### Manual Workflow Triggers

Trigger workflows manually from GitHub Actions tab:

1. **Sync**: Full sync between GitHub and JIRA
2. **Create Sprint**: Generate sprint with default stories
3. **Update Sprint**: Update current sprint status
4. **Generate Report**: Create sprint report

### Monitoring

Check workflow runs at:
https://github.com/SUBBUAURIGRAPH/MEV-Shield/actions

## JIRA Issue Types

The integration supports these JIRA issue types:

- **Epic**: Major features or modules
- **Story**: User-facing features
- **Task**: Technical tasks
- **Bug**: Defects and issues
- **Sub-task**: Breakdown of larger items

## Automation Rules

### Automatic Labels

The following labels are automatically added:

- `github-issue`: Issues created from GitHub
- `has-pr`: Issues with associated pull requests
- `feature`: Features from feature/* branches
- `bugfix`: Fixes from bugfix/* branches
- `hotfix`: Critical fixes from hotfix/* branches
- `deployed-to-main`: Issues deployed to production
- `mev-shield`: All MEV Shield related issues
- `auto-created`: Issues created by automation

### Status Transitions

| Event | Transition | Condition |
|-------|------------|-----------|
| Branch created | To Do → In Progress | feature/* branch |
| PR opened | In Progress → Code Review | Any PR |
| PR to draft | Code Review → In Progress | Draft conversion |
| PR ready | In Progress → Code Review | Ready for review |
| PR merged to develop | Code Review → Testing | Merge to develop |
| PR merged to main | Any → Done | Merge to main |

## Troubleshooting

### Common Issues

1. **Workflow not triggering**
   - Check if secrets are properly set
   - Verify JIRA API token is valid
   - Ensure issue keys follow pattern `MEV-XXX`

2. **Transitions failing**
   - Verify workflow states exist in JIRA
   - Check user permissions in JIRA project
   - Review workflow logs in GitHub Actions

3. **Comments not appearing**
   - Check JIRA API token permissions
   - Verify issue key is correct
   - Check network connectivity

### Debug Mode

Enable debug logging by adding this secret:
```
ACTIONS_RUNNER_DEBUG=true
```

### Support

For issues with the integration:
1. Check GitHub Actions logs
2. Review JIRA audit log
3. Contact: subbu@aurigraph.io

## Advanced Features

### Custom Fields

To update custom fields in JIRA, modify the workflows to include:

```json
{
  "fields": {
    "customfield_10001": "value",
    "customfield_10002": 123
  }
}
```

### Sprint Automation

The system can automatically:
- Create sprints on schedule
- Move incomplete items to next sprint
- Generate burndown reports
- Calculate team velocity

### Metrics Tracking

Track these metrics automatically:
- Cycle time (commit to deploy)
- Lead time (issue created to done)
- PR review time
- Deploy frequency
- Failed deployments

## Security Notes

- API tokens are stored as GitHub secrets
- Never commit tokens to the repository
- Rotate tokens every 90 days
- Use least privilege principle for JIRA user
- Enable audit logging in both systems

## Next Steps

1. ✅ Configure GitHub Secrets
2. ✅ Test with a sample commit including MEV-XXX
3. ✅ Create your first PR with JIRA issue reference
4. ✅ Monitor automation in GitHub Actions tab
5. ✅ Review JIRA board for updates