#!/bin/bash
# Export Security Tickets to GitHub Issues
# MEV Shield Security Remediation Project

set -euo pipefail

# Configuration
REPO_OWNER="your-org"
REPO_NAME="mev-shield"
GITHUB_TOKEN="${GITHUB_TOKEN:-}"
OUTPUT_DIR="github_issues"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# Check if GitHub CLI is installed
check_gh_cli() {
    if ! command -v gh &> /dev/null; then
        echo -e "${RED}❌ GitHub CLI (gh) is required but not installed${NC}"
        echo "Install from: https://cli.github.com/"
        exit 1
    fi
}

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to create issue JSON
create_issue() {
    local id=$1
    local title=$2
    local body=$3
    local priority=$4
    local labels=$5
    local milestone=$6
    local assignee=$7
    
    cat > "$OUTPUT_DIR/issue_${id}.json" << EOF
{
  "title": "$title",
  "body": "$body",
  "labels": [$labels],
  "milestone": "$milestone",
  "assignees": ["$assignee"]
}
EOF
}

# Create GitHub CLI commands file
create_gh_commands() {
    cat > "$OUTPUT_DIR/create_issues.sh" << 'EOF'
#!/bin/bash
# GitHub Issues Creation Script
# Run this script to create all security tickets in GitHub

# Phase 1 Critical Issues
echo "Creating Phase 1 Critical Security Issues..."

gh issue create \
  --title "MEVS-SEC-001: JWT Authentication Backend Implementation" \
  --body "## Description
Implement JWT-based authentication system for API endpoints

## Technical Requirements
- JWT token generation with 24-hour expiry
- Token validation middleware
- Refresh token mechanism
- Token blacklist for logout
- Rate limiting on auth endpoints

## Acceptance Criteria
- [ ] JWT token generation with 24-hour expiry
- [ ] Token validation middleware implemented
- [ ] Refresh token mechanism in place
- [ ] Token blacklist for logout functionality
- [ ] Rate limiting on authentication endpoints

## Dependencies
- jsonwebtoken = \"8.3\"
- uuid = { version = \"1.0\", features = [\"v4\"] }
- chrono = { version = \"0.4\", features = [\"serde\"] }" \
  --label "security" \
  --label "authentication" \
  --label "phase1" \
  --label "critical" \
  --label "backend" \
  --milestone "v1.1.0"

gh issue create \
  --title "MEVS-SEC-002: Password Hashing Implementation" \
  --body "## Description
Implement Argon2 password hashing for user credentials

## Requirements
- Argon2id algorithm
- Password strength validation (min 12 chars)
- Password history tracking
- Secure password reset flow
- Migration script for existing passwords

## Acceptance Criteria
- [ ] Argon2id algorithm implemented
- [ ] Password strength validation (min 12 chars)
- [ ] Password history tracking (prevent reuse)
- [ ] Secure password reset flow
- [ ] Migration script for existing passwords

## Technical Specs
Memory cost: 65536 KB
Time cost: 2 iterations
Parallelism: 1" \
  --label "security" \
  --label "authentication" \
  --label "phase1" \
  --label "critical" \
  --label "backend" \
  --milestone "v1.1.0"

gh issue create \
  --title "MEVS-SEC-003: Frontend Authentication Context" \
  --body "## Description
Implement React authentication context and protected routes

## Components
- AuthContext with login/logout functionality
- Protected route wrapper component
- Token persistence in secure storage
- Automatic token refresh
- Session timeout handling

## Acceptance Criteria
- [ ] AuthContext with login/logout functionality
- [ ] Protected route wrapper component
- [ ] Token persistence in secure storage
- [ ] Automatic token refresh
- [ ] Session timeout handling

## Files to Create
- src/auth/AuthContext.tsx
- src/auth/ProtectedRoute.tsx
- src/auth/useAuth.hook.ts" \
  --label "security" \
  --label "frontend" \
  --label "phase1" \
  --label "critical" \
  --milestone "v1.1.0"

gh issue create \
  --title "MEVS-SEC-004: Login/Registration UI" \
  --body "## Description
Create secure login and registration interfaces

## Requirements
- Login form with validation
- Registration form with password requirements
- Two-factor authentication support
- Remember me functionality (secure)
- Password strength indicator

## Acceptance Criteria
- [ ] Login form with validation
- [ ] Registration form with password requirements
- [ ] Two-factor authentication support
- [ ] Remember me functionality (secure)
- [ ] Password strength indicator" \
  --label "security" \
  --label "frontend" \
  --label "phase1" \
  --label "critical" \
  --label "ui" \
  --milestone "v1.1.0"

gh issue create \
  --title "MEVS-SEC-005: Environment Variable Security" \
  --body "## Description
Restructure environment variables for security

## Requirements
- .env.template created with all variables
- Separate configs for dev/staging/prod
- No hardcoded secrets in codebase
- Documentation for secret rotation
- Validation script for env variables

## Acceptance Criteria
- [ ] .env.template created with all variables
- [ ] Separate configs for dev/staging/prod
- [ ] No hardcoded secrets in codebase
- [ ] Documentation for secret rotation
- [ ] Validation script for env variables

## Files to Create
- .env.template
- .env.development
- .env.production.example" \
  --label "security" \
  --label "configuration" \
  --label "phase1" \
  --label "critical" \
  --label "devops" \
  --milestone "v1.1.0"

# Phase 2 High Priority Issues
echo "Creating Phase 2 High Priority Issues..."

gh issue create \
  --title "MEVS-SEC-011: Frontend Input Sanitization" \
  --body "## Description
Implement DOMPurify for XSS prevention

## Requirements
- DOMPurify integrated in all user inputs
- Custom sanitization rules defined
- HTML content properly escaped
- Rich text editor security configured
- Unit tests for sanitization

## Acceptance Criteria
- [ ] DOMPurify integrated in all user inputs
- [ ] Custom sanitization rules defined
- [ ] HTML content properly escaped
- [ ] Rich text editor security configured
- [ ] Unit tests for sanitization

## Implementation
npm install dompurify @types/dompurify" \
  --label "security" \
  --label "frontend" \
  --label "phase2" \
  --label "high" \
  --label "xss-prevention" \
  --milestone "v1.2.0"

gh issue create \
  --title "MEVS-SEC-012: Backend Input Validation" \
  --body "## Description
Implement comprehensive input validation on API

## Requirements
- Validation schemas for all endpoints
- Ethereum address validation
- Amount/number validation
- String length limits
- SQL injection prevention

## Acceptance Criteria
- [ ] Validation schemas for all endpoints
- [ ] Ethereum address validation
- [ ] Amount/number validation
- [ ] String length limits
- [ ] SQL injection prevention" \
  --label "security" \
  --label "backend" \
  --label "phase2" \
  --label "high" \
  --label "validation" \
  --milestone "v1.2.0"

echo "✅ GitHub issues created successfully!"
EOF

    chmod +x "$OUTPUT_DIR/create_issues.sh"
}

# Create issue templates
create_issue_templates() {
    mkdir -p "$OUTPUT_DIR/templates"
    
    # Security issue template
    cat > "$OUTPUT_DIR/templates/security_issue_template.md" << 'EOF'
---
name: Security Task
about: Template for security remediation tasks
title: 'MEVS-SEC-XXX: '
labels: security
assignees: ''

---

## Description
[Brief description of the security task]

## Priority
- [ ] 🔴 Critical (Production Blocker)
- [ ] 🟠 High
- [ ] 🟡 Medium  
- [ ] 🟢 Low

## Requirements
- [ ] Requirement 1
- [ ] Requirement 2
- [ ] Requirement 3

## Acceptance Criteria
- [ ] Criteria 1
- [ ] Criteria 2
- [ ] Criteria 3

## Technical Details
```
[Any code snippets or technical specifications]
```

## Testing Requirements
- [ ] Unit tests
- [ ] Integration tests
- [ ] Security tests

## Dependencies
- Depends on: #
- Blocks: #

## Definition of Done
- [ ] Code implemented and reviewed
- [ ] Tests written and passing
- [ ] Documentation updated
- [ ] Security scan passed
- [ ] Deployed to staging
EOF
}

# Create GitHub project board configuration
create_project_board() {
    cat > "$OUTPUT_DIR/project_board.json" << 'EOF'
{
  "name": "MEV Shield Security Remediation",
  "body": "Security hardening project for MEV Shield v1.0.0",
  "columns": [
    {
      "name": "📋 Backlog",
      "cards": []
    },
    {
      "name": "🔴 Phase 1 (Critical)",
      "cards": ["MEVS-SEC-001", "MEVS-SEC-002", "MEVS-SEC-003", "MEVS-SEC-004", "MEVS-SEC-005"]
    },
    {
      "name": "🟠 Phase 2 (High)",
      "cards": ["MEVS-SEC-011", "MEVS-SEC-012", "MEVS-SEC-013", "MEVS-SEC-014", "MEVS-SEC-015"]
    },
    {
      "name": "🏃 In Progress",
      "cards": []
    },
    {
      "name": "👀 Review",
      "cards": []
    },
    {
      "name": "✅ Done",
      "cards": []
    }
  ]
}
EOF
}

# Create labels configuration
create_labels() {
    cat > "$OUTPUT_DIR/create_labels.sh" << 'EOF'
#!/bin/bash
# Create GitHub labels for security project

echo "Creating security labels..."

# Priority labels
gh label create "critical" --description "Production blocking issue" --color "FF0000"
gh label create "high" --description "High priority issue" --color "FF6600"
gh label create "medium" --description "Medium priority issue" --color "FFCC00"
gh label create "low" --description "Low priority issue" --color "00FF00"

# Phase labels
gh label create "phase1" --description "Phase 1: Critical Remediation" --color "FF0000"
gh label create "phase2" --description "Phase 2: High Priority" --color "FF6600"
gh label create "phase3" --description "Phase 3: Medium Priority" --color "FFCC00"
gh label create "phase4" --description "Phase 4: Ongoing Maintenance" --color "00FF00"

# Component labels
gh label create "backend" --description "Backend component" --color "0052CC"
gh label create "frontend" --description "Frontend component" --color "0052CC"
gh label create "infrastructure" --description "Infrastructure/DevOps" --color "0052CC"
gh label create "devops" --description "DevOps related" --color "0052CC"

# Security type labels
gh label create "authentication" --description "Authentication related" --color "8B0000"
gh label create "authorization" --description "Authorization related" --color "8B0000"
gh label create "encryption" --description "Encryption related" --color "8B0000"
gh label create "validation" --description "Input validation" --color "8B0000"
gh label create "xss-prevention" --description "XSS prevention" --color "8B0000"
gh label create "configuration" --description "Configuration security" --color "8B0000"
gh label create "docker" --description "Docker security" --color "8B0000"
gh label create "monitoring" --description "Security monitoring" --color "8B0000"

echo "✅ Labels created successfully!"
EOF

    chmod +x "$OUTPUT_DIR/create_labels.sh"
}

# Create milestones
create_milestones() {
    cat > "$OUTPUT_DIR/create_milestones.sh" << 'EOF'
#!/bin/bash
# Create GitHub milestones for security project

echo "Creating milestones..."

# Calculate due dates
WEEK2_DUE=$(date -d "+2 weeks" +%Y-%m-%d)
WEEK4_DUE=$(date -d "+4 weeks" +%Y-%m-%d)
WEEK8_DUE=$(date -d "+8 weeks" +%Y-%m-%d)

gh api repos/:owner/:repo/milestones \
  --method POST \
  --field title="v1.1.0 - Critical Security Fixes" \
  --field description="Phase 1: Authentication, credential management, Docker hardening" \
  --field due_on="${WEEK2_DUE}T23:59:59Z"

gh api repos/:owner/:repo/milestones \
  --method POST \
  --field title="v1.2.0 - High Priority Security" \
  --field description="Phase 2: Input validation, network security, CORS" \
  --field due_on="${WEEK4_DUE}T23:59:59Z"

gh api repos/:owner/:repo/milestones \
  --method POST \
  --field title="v1.3.0 - Security Monitoring" \
  --field description="Phase 3: Monitoring, compliance, secrets management" \
  --field due_on="${WEEK8_DUE}T23:59:59Z"

gh api repos/:owner/:repo/milestones \
  --method POST \
  --field title="v2.0.0 - Security Maintenance" \
  --field description="Phase 4: Ongoing security maintenance and improvements"

echo "✅ Milestones created successfully!"
EOF

    chmod +x "$OUTPUT_DIR/create_milestones.sh"
}

# Main execution
main() {
    echo -e "${BLUE}🔒 MEV Shield Security Tickets - GitHub Export${NC}"
    echo "=============================================="
    echo ""
    
    check_gh_cli
    
    echo -e "${YELLOW}📝 Creating GitHub issues export...${NC}"
    
    create_gh_commands
    create_issue_templates
    create_project_board
    create_labels
    create_milestones
    
    echo ""
    echo -e "${GREEN}✅ Export completed successfully!${NC}"
    echo ""
    echo -e "${BLUE}📁 Output Directory: $OUTPUT_DIR/${NC}"
    echo ""
    echo "Files created:"
    echo "  • create_issues.sh - Script to create all issues"
    echo "  • create_labels.sh - Script to create labels"
    echo "  • create_milestones.sh - Script to create milestones"
    echo "  • project_board.json - Project board configuration"
    echo "  • templates/ - Issue templates"
    echo ""
    echo -e "${YELLOW}📋 Usage Instructions:${NC}"
    echo "1. Authenticate with GitHub:"
    echo "   gh auth login"
    echo ""
    echo "2. Set your repository:"
    echo "   gh repo set-default OWNER/REPO"
    echo ""
    echo "3. Create labels:"
    echo "   cd $OUTPUT_DIR && ./create_labels.sh"
    echo ""
    echo "4. Create milestones:"
    echo "   ./create_milestones.sh"
    echo ""
    echo "5. Create issues:"
    echo "   ./create_issues.sh"
    echo ""
    echo -e "${BLUE}💡 Tip:${NC} Review and modify the scripts before running if needed"
}

# Run main function
main "$@"