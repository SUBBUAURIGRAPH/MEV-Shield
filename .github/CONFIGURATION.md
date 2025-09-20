# GitHub Actions Configuration Guide

This document outlines the required configuration for the MEV Shield CI/CD pipelines.

## Required Repository Secrets

### Production Deployment
```
PRODUCTION_HOST=dev.mevshield.ai
PRODUCTION_USER=mevshield
PRODUCTION_SSH_KEY=<private SSH key for production server>
PRODUCTION_PORT=22
```

### Staging Deployment (Optional)
```
STAGING_HOST=staging.mevshield.ai
STAGING_USER=mevshield
STAGING_SSH_KEY=<private SSH key for staging server>
STAGING_PORT=22
```

### Notifications
```
SLACK_WEBHOOK_URL=<your Slack webhook URL for notifications>
```

## GitHub Environments Setup

### 1. Production Environment
- Go to Repository Settings → Environments
- Create "production" environment
- Add protection rules:
  - Required reviewers (recommended)
  - Wait timer (optional)
- Environment secrets inherit from repository secrets

### 2. Staging Environment (Optional)
- Create "staging" environment
- Less restrictive protection rules

## Repository Settings

### 1. Enable GitHub Container Registry
- Repository Settings → Actions → General
- Enable "Read and write permissions" for GITHUB_TOKEN

### 2. Security Features
- Repository Settings → Security & analysis
- Enable:
  - Dependency graph
  - Dependabot alerts
  - Dependabot security updates
  - Code scanning alerts

### 3. Branch Protection Rules
- Repository Settings → Branches
- Protect "main" branch:
  - Require pull request reviews
  - Require status checks (CI/CD tests)
  - Require branches to be up to date

## Workflow Configuration Files

### 1. Lighthouse CI Configuration
Create `.lighthouserc.json`:
```json
{
  "ci": {
    "collect": {
      "numberOfRuns": 3,
      "url": [
        "https://dev.mevshield.ai",
        "https://dev.mevshield.ai/admin",
        "https://dev.mevshield.ai/dashboard"
      ]
    },
    "assert": {
      "assertions": {
        "categories:performance": ["warn", {"minScore": 0.8}],
        "categories:accessibility": ["error", {"minScore": 0.9}],
        "categories:best-practices": ["warn", {"minScore": 0.8}],
        "categories:seo": ["warn", {"minScore": 0.8}]
      }
    },
    "upload": {
      "target": "temporary-public-storage"
    }
  }
}
```

### 2. CodeQL Configuration (Optional)
Create `.github/codeql/codeql-config.yml`:
```yaml
name: "MEV Shield CodeQL Config"
queries:
  - name: security-and-quality
    uses: security-and-quality
paths-ignore:
  - node_modules
  - "**/*.test.js"
  - "**/*.spec.js"
```

## Server Preparation

### 1. Production Server Setup
```bash
# Create deployment user
sudo useradd -m -s /bin/bash mevshield
sudo usermod -aG docker mevshield

# Create deployment directory
sudo mkdir -p /opt/mev-shield
sudo chown mevshield:mevshield /opt/mev-shield

# Setup backup directory
sudo mkdir -p /opt/mev-shield-backups
sudo chown mevshield:mevshield /opt/mev-shield-backups

# Install Docker and Docker Compose (if not already installed)
sudo apt update
sudo apt install -y docker.io docker-compose
sudo systemctl enable docker
sudo systemctl start docker
```

### 2. SSH Key Setup
```bash
# On production server, add GitHub Actions SSH public key
sudo -u mevshield mkdir -p /home/mevshield/.ssh
sudo -u mevshield echo "ssh-rsa YOUR_PUBLIC_KEY_HERE" >> /home/mevshield/.ssh/authorized_keys
sudo -u mevshield chmod 600 /home/mevshield/.ssh/authorized_keys
sudo -u mevshield chmod 700 /home/mevshield/.ssh
```

### 3. Docker Compose Files
Create production-specific Docker Compose files on the server:

**docker-compose.production.yml**:
```yaml
version: '3.8'
services:
  frontend:
    image: ghcr.io/your-username/mev-shield-frontend:latest
    ports:
      - "80:80"
      - "443:443"
    environment:
      - NODE_ENV=production
    volumes:
      - ./ssl:/etc/ssl/certs:ro

  backend-mock:
    image: ghcr.io/your-username/mev-shield-backend-mock:latest
    ports:
      - "8080:8080"
    environment:
      - NODE_ENV=production
    env_file:
      - .env.production

  backend-live:
    image: ghcr.io/your-username/mev-shield-backend-live:latest
    ports:
      - "8081:8081"
    environment:
      - NODE_ENV=production
    env_file:
      - .env.production

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/ssl/certs:ro
    depends_on:
      - frontend
      - backend-mock
      - backend-live
```

## Testing the Setup

### 1. Manual Workflow Trigger
- Go to Actions tab in GitHub
- Select any workflow
- Click "Run workflow" → "Run workflow"

### 2. Push Test
```bash
# Create a test commit
git add .
git commit -m "test: trigger CI/CD pipeline"
git push origin main
```

### 3. Release Test
```bash
# Create and push a tag
git tag v1.0.0
git push origin v1.0.0
```

## Monitoring and Troubleshooting

### 1. Workflow Logs
- Check GitHub Actions tab for detailed logs
- Each job shows individual step results

### 2. Common Issues
- **SSH connection failed**: Check SSH key configuration
- **Docker login failed**: Verify GITHUB_TOKEN permissions
- **Build failures**: Check Node.js versions and dependencies
- **Deployment failures**: Verify server connectivity and Docker setup

### 3. Slack Notifications
Notifications will be sent for:
- Successful/failed deployments
- Security vulnerabilities detected
- Service downtime alerts
- Release announcements

## Additional Features

### 1. Dependabot Configuration
Create `.github/dependabot.yml`:
```yaml
version: 2
updates:
  - package-ecosystem: "npm"
    directory: "/dashboard"
    schedule:
      interval: "weekly"
  - package-ecosystem: "npm"
    directory: "/backend-mock"
    schedule:
      interval: "weekly"
  - package-ecosystem: "npm"
    directory: "/backend-live"
    schedule:
      interval: "weekly"
  - package-ecosystem: "docker"
    directory: "/"
    schedule:
      interval: "weekly"
```

### 2. Issue Templates
Create `.github/ISSUE_TEMPLATE/` with templates for bugs, features, and security issues.

### 3. Pull Request Template
Create `.github/pull_request_template.md` for consistent PR descriptions.

---

**Next Steps:**
1. Configure repository secrets
2. Set up GitHub environments
3. Prepare production server
4. Test workflows with a small change
5. Monitor first deployment

All workflows are now ready for production use with proper configuration!