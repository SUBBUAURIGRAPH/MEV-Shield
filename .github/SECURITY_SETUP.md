# 🔐 Secure JIRA API Token Configuration

## ⚠️ IMPORTANT SECURITY NOTICE

**NEVER share your API tokens publicly!** If you've accidentally exposed a token:
1. Immediately revoke it at: https://id.atlassian.com/manage-profile/security/api-tokens
2. Create a new token
3. Update all systems using the old token

## Setting Up GitHub Secrets

### Method 1: GitHub Web Interface (Recommended)

1. Go to your repository: https://github.com/SUBBUAURIGRAPH/MEV-Shield
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Add these secrets:

   **Secret 1:**
   - Name: `JIRA_USER_EMAIL`
   - Value: `subbu@aurigraph.io`

   **Secret 2:**
   - Name: `JIRA_API_TOKEN`
   - Value: `[Your JIRA API Token]`

### Method 2: GitHub CLI

```bash
# Install GitHub CLI if not already installed
# macOS: brew install gh
# Linux: See https://github.com/cli/cli#installation

# Authenticate
gh auth login

# Set secrets
gh secret set JIRA_USER_EMAIL --body="subbu@aurigraph.io" --repo="SUBBUAURIGRAPH/MEV-Shield"
gh secret set JIRA_API_TOKEN --body="YOUR_TOKEN_HERE" --repo="SUBBUAURIGRAPH/MEV-Shield"
```

### Method 3: Using Setup Script

```bash
# Run the automated setup script
cd .github/scripts
./setup-jira-secrets.sh
```

## Verify Configuration

### Test the Integration

1. **Via GitHub Actions UI:**
   - Go to: https://github.com/SUBBUAURIGRAPH/MEV-Shield/actions
   - Click on "Test JIRA Integration"
   - Click "Run workflow"
   - Select test type: "all"
   - Click "Run workflow"

2. **Via GitHub CLI:**
   ```bash
   gh workflow run test-jira-integration.yml \
     --repo=SUBBUAURIGRAPH/MEV-Shield \
     -f test_type=all \
     -f issue_key=MEV-1
   ```

3. **Check Results:**
   - Green checkmarks = Success ✅
   - Red X = Configuration issue ❌

## Security Best Practices

### Token Management

1. **Create Dedicated Tokens**
   - Use separate tokens for different integrations
   - Name tokens descriptively (e.g., "GitHub Actions - MEV Shield")

2. **Limit Permissions**
   - Only grant necessary permissions
   - Use read-only where possible

3. **Rotate Regularly**
   - Set calendar reminders every 90 days
   - Update tokens before they expire

4. **Monitor Usage**
   - Check JIRA audit logs regularly
   - Review GitHub Actions logs for anomalies

### If Token is Compromised

**Immediate Actions:**
1. Revoke the token immediately
2. Create new token
3. Update GitHub secrets
4. Check JIRA audit log for unauthorized access
5. Review recent changes in both systems

**Revoke Token:**
https://id.atlassian.com/manage-profile/security/api-tokens

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| 401 Unauthorized | Token is invalid or expired |
| 403 Forbidden | User lacks permissions in JIRA |
| 404 Not Found | Issue key doesn't exist |
| Connection timeout | Check network/firewall settings |

### Debug Commands

```bash
# Test token locally (be careful not to expose token)
curl -X GET \
  -H "Authorization: Basic $(echo -n 'subbu@aurigraph.io:YOUR_TOKEN' | base64)" \
  https://aurigraphdlt.atlassian.net/rest/api/3/myself

# Check GitHub secrets
gh secret list --repo=SUBBUAURIGRAPH/MEV-Shield

# View workflow runs
gh run list --repo=SUBBUAURIGRAPH/MEV-Shield
```

## Contact for Issues

- **JIRA Admin**: subbu@aurigraph.io
- **Repository**: https://github.com/SUBBUAURIGRAPH/MEV-Shield
- **JIRA Board**: https://aurigraphdlt.atlassian.net/jira/software/projects/MEV/boards/855

## Next Steps After Setup

1. ✅ Verify secrets are configured
2. ✅ Run test workflow
3. ✅ Create first issue with MEV- prefix
4. ✅ Make commit referencing issue
5. ✅ Watch automation in action!

---

**Remember:** Keep your API tokens secret and secure! 🔐