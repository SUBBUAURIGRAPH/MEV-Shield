# GitHub Repository Secrets Setup

## Required Secrets for MEV Shield CI/CD

### 1. Production Server Secrets
```bash
# Add these secrets in GitHub Repository Settings → Secrets and Variables → Actions

PRODUCTION_HOST=dev.mevshield.ai
PRODUCTION_USER=mevshield
PRODUCTION_SSH_KEY=-----BEGIN OPENSSH PRIVATE KEY-----
# Your private SSH key content here
-----END OPENSSH PRIVATE KEY-----
PRODUCTION_PORT=22
```

### 2. Staging Server Secrets (Optional)
```bash
STAGING_HOST=staging.mevshield.ai
STAGING_USER=mevshield
STAGING_SSH_KEY=-----BEGIN OPENSSH PRIVATE KEY-----
# Your staging private SSH key content here
-----END OPENSSH PRIVATE KEY-----
STAGING_PORT=22
```

### 3. Notification Secrets
```bash
# Slack webhook URL for deployment notifications
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK
```

## CLI Commands to Set Secrets

### Using GitHub CLI (gh)
```bash
# Install GitHub CLI first: https://cli.github.com/

# Login to GitHub
gh auth login

# Set production secrets
gh secret set PRODUCTION_HOST --body "dev.mevshield.ai"
gh secret set PRODUCTION_USER --body "mevshield"
gh secret set PRODUCTION_PORT --body "22"

# Set SSH key from file
gh secret set PRODUCTION_SSH_KEY < ~/.ssh/mev_shield_production_key

# Set Slack webhook
gh secret set SLACK_WEBHOOK_URL --body "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"
```

### Using Environment Variables
```bash
# Export environment variables and use them
export PRODUCTION_HOST="dev.mevshield.ai"
export PRODUCTION_USER="mevshield"
export PRODUCTION_SSH_KEY="$(cat ~/.ssh/mev_shield_production_key)"
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"

# Set all at once
gh secret set PRODUCTION_HOST --body "$PRODUCTION_HOST"
gh secret set PRODUCTION_USER --body "$PRODUCTION_USER"
gh secret set PRODUCTION_SSH_KEY --body "$PRODUCTION_SSH_KEY"
gh secret set SLACK_WEBHOOK_URL --body "$SLACK_WEBHOOK_URL"
```

## Server SSH Key Setup

### 1. Generate SSH Key Pair
```bash
# Generate new SSH key for GitHub Actions
ssh-keygen -t rsa -b 4096 -C "github-actions@mevshield.ai" -f ~/.ssh/mev_shield_production_key

# This creates:
# ~/.ssh/mev_shield_production_key (private key - add to GitHub secrets)
# ~/.ssh/mev_shield_production_key.pub (public key - add to server)
```

### 2. Add Public Key to Production Server
```bash
# Copy public key to server
ssh-copy-id -i ~/.ssh/mev_shield_production_key.pub mevshield@dev.mevshield.ai

# Or manually add to authorized_keys
cat ~/.ssh/mev_shield_production_key.pub | ssh mevshield@dev.mevshield.ai "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"
```

### 3. Test SSH Connection
```bash
# Test the connection
ssh -i ~/.ssh/mev_shield_production_key mevshield@dev.mevshield.ai "echo 'SSH connection successful'"
```

## Quick Setup Script

```bash
#!/bin/bash
# quick-setup-secrets.sh

echo "🔧 Setting up MEV Shield GitHub Actions secrets..."

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI not found. Install from: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "🔑 Please login to GitHub first:"
    gh auth login
fi

# Set basic secrets
echo "📝 Setting production server secrets..."
gh secret set PRODUCTION_HOST --body "dev.mevshield.ai"
gh secret set PRODUCTION_USER --body "mevshield"
gh secret set PRODUCTION_PORT --body "22"

# SSH key setup
if [ -f ~/.ssh/mev_shield_production_key ]; then
    echo "🔑 Setting SSH key from ~/.ssh/mev_shield_production_key"
    gh secret set PRODUCTION_SSH_KEY < ~/.ssh/mev_shield_production_key
else
    echo "⚠️  SSH key not found at ~/.ssh/mev_shield_production_key"
    echo "   Generate with: ssh-keygen -t rsa -b 4096 -C 'github-actions@mevshield.ai' -f ~/.ssh/mev_shield_production_key"
fi

# Slack webhook (optional)
read -p "🔔 Enter Slack webhook URL (or press Enter to skip): " webhook_url
if [ ! -z "$webhook_url" ]; then
    gh secret set SLACK_WEBHOOK_URL --body "$webhook_url"
    echo "✅ Slack webhook configured"
fi

echo "🎉 GitHub Actions secrets setup complete!"
echo "📋 Next steps:"
echo "   1. Verify secrets in GitHub Repository Settings → Secrets"
echo "   2. Push a commit to trigger the CI/CD pipeline"
echo "   3. Check Actions tab for pipeline status"
```

## Verification

### Check Current Secrets
```bash
# List all repository secrets
gh secret list

# Should show:
# PRODUCTION_HOST
# PRODUCTION_SSH_KEY  
# PRODUCTION_USER
# SLACK_WEBHOOK_URL
```

### Test Deployment
```bash
# Trigger manual deployment test
gh workflow run "MEV Shield CI/CD Pipeline"

# Check workflow status
gh run list --workflow="MEV Shield CI/CD Pipeline"
```

## Security Notes

1. **Never commit private keys** to the repository
2. **Use separate SSH keys** for GitHub Actions (not your personal key)
3. **Regularly rotate** SSH keys and secrets
4. **Limit SSH key permissions** on the server
5. **Use GitHub Environments** for production deployments with approvals
6. **Monitor secret usage** in GitHub Actions logs

---

✅ **Status**: Ready for production deployment
🔗 **Repository**: https://github.com/SUBBUAURIGRAPH/MEV-Shield
🚀 **Actions**: https://github.com/SUBBUAURIGRAPH/MEV-Shield/actions