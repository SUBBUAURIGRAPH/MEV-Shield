# JIRA Integration Guide - MEV Shield

## 📋 Overview

This repository implements a **1 Task : 1 Ticket** policy with automatic JIRA synchronization through GitHub Actions. Every meaningful task gets its own JIRA ticket, and all tickets are organized under relevant Epics.

## 🎯 Core Principle: 1 Task : 1 Ticket

- **Every task** = **One JIRA ticket**
- **Related tasks** = **Grouped in Epics**
- **Automatic tracking** = **No manual ticket creation needed**

## 🏗️ Epic Structure

MEV Shield tasks are organized into 5 main epics:

| Epic Key | Epic Name | Description |
|----------|-----------|-------------|
| MEV-101 | Frontend Development | UI/UX, dashboards, React components |
| MEV-102 | Backend Development | APIs, smart contracts, core logic |
| MEV-103 | Security Implementation | Audits, testing, hardening |
| MEV-104 | Deployment & Infrastructure | CI/CD, Docker, Kubernetes |
| MEV-105 | Testing & QA | Unit, integration, E2E tests |

## 🚀 Setup

### 1. Configure GitHub Secrets

Run the setup script:
```bash
./setup-jira-github-secrets.sh
```

Or manually add these secrets to your repository:
- `JIRA_BASE_URL`: https://aurigraphdlt.atlassian.net
- `JIRA_USER_EMAIL`: Your JIRA email
- `JIRA_API_TOKEN`: Your JIRA API token ([Generate here](https://id.atlassian.com/manage-profile/security/api-tokens))

### 2. Enable GitHub Actions

The workflows are automatically triggered on:
- Push to any branch
- Pull request events
- Issue creation
- Manual workflow dispatch

## 💬 Smart Commit Messages

### Basic Syntax

```bash
# Link to existing ticket
git commit -m "MEV-123: Implement user authentication"

# Auto-create ticket (if no key provided)
git commit -m "Add dashboard component for MEV protection metrics"

# With time tracking
git commit -m "MEV-124: Fix CORS issue #time 2h"

# With comments
git commit -m "MEV-125: Update API endpoint #comment Resolved performance issue"

# Resolve ticket
git commit -m "MEV-126: Complete security audit #resolve"

# Multiple actions
git commit -m "MEV-127: Refactor authentication #time 3h #comment Used JWT tokens #resolve"
```

### Commit Message Best Practices

1. **First Line**: Task summary (max 50 characters)
2. **Include JIRA Key**: If working on existing ticket
3. **Be Specific**: Clear, actionable descriptions
4. **Use Hashtags**: For JIRA smart commits

### Examples by Category

#### Frontend Tasks
```bash
git commit -m "MEV-201: Add real-time MEV attack visualization dashboard"
git commit -m "MEV-202: Implement responsive design for mobile devices #time 4h"
git commit -m "MEV-203: Fix chart rendering issue in Firefox #resolve"
```

#### Backend Tasks
```bash
git commit -m "MEV-301: Implement WebSocket connection for live updates"
git commit -m "MEV-302: Add rate limiting to API endpoints #time 2h"
git commit -m "MEV-303: Optimize database queries for better performance"
```

#### Security Tasks
```bash
git commit -m "MEV-401: Add input validation for all API endpoints"
git commit -m "MEV-402: Implement JWT token refresh mechanism #time 3h"
git commit -m "MEV-403: Fix XSS vulnerability in comment section #resolve"
```

#### Deployment Tasks
```bash
git commit -m "MEV-501: Configure nginx for HTTPS support"
git commit -m "MEV-502: Add Docker health checks #time 1h"
git commit -m "MEV-503: Set up GitHub Actions for automated deployment"
```

#### Testing Tasks
```bash
git commit -m "MEV-601: Add unit tests for authentication module"
git commit -m "MEV-602: Create E2E tests for user flow #time 5h"
git commit -m "MEV-603: Fix flaky integration tests #resolve"
```

## 📝 Pull Request Integration

### PR Title Formats

```markdown
# With JIRA key
MEV-123: Add user authentication

# Multiple tickets
MEV-123, MEV-124: Implement auth and session management

# Auto-create parent story
Feature: Add complete authentication system
```

### PR Description Template

```markdown
## Summary
Brief description of changes

## JIRA Tickets
- MEV-123: Main authentication implementation
- MEV-124: Session management
- MEV-125: Password reset functionality

## Tasks
- [ ] Implement login endpoint
- [ ] Add password hashing
- [ ] Create session middleware
- [ ] Add password reset flow
- [ ] Write unit tests
- [ ] Update documentation

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Related Epic
Epic: MEV-101 (Frontend Development)
```

## 🔄 Workflow Automation

### Automatic Ticket Creation

When you push commits without JIRA keys, the system:
1. **Creates a ticket** automatically
2. **Links to appropriate Epic** based on changed files
3. **Adds commit details** to ticket description
4. **Tags with category** (Frontend, Backend, etc.)

### Automatic Status Transitions

| Event | JIRA Transition |
|-------|----------------|
| Push to `develop` | → In Progress |
| PR opened | → In Review |
| PR merged to `main` | → Done |
| Issue closed | → Resolved |

### File-Based Epic Assignment

| File Pattern | Assigned Epic | Category |
|-------------|---------------|----------|
| `*.tsx`, `*.jsx`, `*.css` | MEV-101 | Frontend |
| `*.rs`, `*.go`, `/api/` | MEV-102 | Backend |
| `*.sol`, `/security/` | MEV-103 | Security |
| `*.yml`, `Dockerfile` | MEV-104 | Deployment |
| `*.test.*`, `*.spec.*` | MEV-105 | Testing |

## 🎮 Manual Workflow Triggers

### Create Task with Epic

```bash
gh workflow run jira-task-management.yml \
  -f task_name="Implement new feature" \
  -f epic_key="MEV-101" \
  -f task_type="Story"
```

### View Workflow Runs

```bash
# List all workflow runs
gh run list --workflow=jira-integration.yml

# View specific run details
gh run view <run-id>

# Watch workflow in real-time
gh run watch
```

## 📊 JIRA Board Management

### Board Structure

```
📋 MEV Shield Board
├── 📁 Backlog
├── 📁 To Do
├── 📁 In Progress
├── 📁 In Review
├── 📁 Testing
└── 📁 Done
```

### Labels Applied Automatically

- `github-auto`: Created by GitHub Actions
- `github-pr`: Related to Pull Request
- `github-issue`: Created from GitHub Issue
- `Frontend`, `Backend`, `Security`, `Deployment`, `Testing`: Category labels

## 🔍 Tracking & Reporting

### Find Related Tickets

```bash
# Find all tickets from recent commits
git log --oneline | grep -oE 'MEV-[0-9]+' | sort -u

# Find tickets in current branch
git log main..HEAD --oneline | grep -oE 'MEV-[0-9]+'

# Find tickets by author
git log --author="your-name" --oneline | grep -oE 'MEV-[0-9]+'
```

### JIRA JQL Queries

```sql
-- All GitHub-created tickets
labels = "github-auto"

-- Frontend tasks in current sprint
labels = "Frontend" AND sprint in openSprints()

-- Unresolved security issues
labels = "Security" AND resolution = Unresolved

-- My tasks from GitHub
labels = "github-auto" AND assignee = currentUser()

-- Tasks in specific epic
"Epic Link" = MEV-101
```

## 🚨 Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Workflow not triggering | Check GitHub Actions is enabled in repo settings |
| JIRA ticket not created | Verify secrets are set correctly |
| Wrong epic assignment | Check file patterns in workflow |
| Duplicate tickets | Use JIRA keys in commit messages |

### Debug Workflow

```bash
# Enable debug logging
gh workflow run jira-integration.yml --ref main

# Check secret configuration
gh secret list

# Validate JIRA connection
curl -u email@example.com:api_token \
  https://aurigraphdlt.atlassian.net/rest/api/3/myself
```

## 📚 Best Practices

### Do's ✅

1. **Use JIRA keys** when working on existing tickets
2. **Write clear commit messages** - they become ticket descriptions
3. **Group related commits** in one PR
4. **Link PRs to epics** using labels or description
5. **Update ticket status** using smart commits

### Don'ts ❌

1. **Don't skip commit messages** - they're used for tickets
2. **Don't combine unrelated tasks** in one commit
3. **Don't manually create tickets** for code changes
4. **Don't forget to pull** before starting new work

## 🔗 Quick Links

- [JIRA Board](https://aurigraphdlt.atlassian.net/jira/software/projects/MEV/boards/855)
- [GitHub Actions](https://github.com/your-org/mev-shield/actions)
- [API Token Management](https://id.atlassian.com/manage-profile/security/api-tokens)
- [JIRA Smart Commits Docs](https://support.atlassian.com/jira-software-cloud/docs/process-issues-with-smart-commits/)

## 📞 Support

For issues with JIRA integration:
1. Check workflow logs in GitHub Actions
2. Verify JIRA API token is valid
3. Ensure JIRA project permissions are correct
4. Contact: subbu@aurigraph.io

---

**Remember**: Every task gets a ticket, every ticket tells a story! 🎯