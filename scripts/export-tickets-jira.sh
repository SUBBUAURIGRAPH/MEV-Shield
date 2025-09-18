#!/bin/bash
# Export Security Tickets to JIRA CSV Format
# MEV Shield Security Remediation Project

set -euo pipefail

OUTPUT_FILE="security_tickets_jira.csv"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Create CSV header for JIRA import
cat > "$OUTPUT_FILE" << 'EOF'
"Issue Type","Summary","Description","Priority","Story Points","Labels","Epic Link","Acceptance Criteria","Sprint","Assignee","Fix Version","Component"
EOF

# Phase 1 Critical Tickets
cat >> "$OUTPUT_FILE" << 'EOF'
"Task","MEVS-SEC-001: JWT Authentication Backend Implementation","Implement JWT-based authentication system for API endpoints

Technical Requirements:
- JWT token generation with 24-hour expiry
- Token validation middleware
- Refresh token mechanism
- Token blacklist for logout
- Rate limiting on auth endpoints","Critical","5","security,authentication,phase1","Security Hardening","- JWT token generation with 24-hour expiry
- Token validation middleware implemented
- Refresh token mechanism in place
- Token blacklist for logout functionality
- Rate limiting on authentication endpoints","Sprint 1","Backend Team","v1.1.0","Backend"
"Task","MEVS-SEC-002: Password Hashing Implementation","Implement Argon2 password hashing for user credentials

Requirements:
- Argon2id algorithm
- Password strength validation (min 12 chars)
- Password history tracking
- Secure password reset flow
- Migration script for existing passwords","Critical","2","security,authentication,phase1","Security Hardening","- Argon2id algorithm implemented
- Password strength validation (min 12 chars)
- Password history tracking (prevent reuse)
- Secure password reset flow
- Migration script for existing passwords","Sprint 1","Backend Team","v1.1.0","Backend"
"Task","MEVS-SEC-003: Frontend Authentication Context","Implement React authentication context and protected routes

Components:
- AuthContext with login/logout functionality
- Protected route wrapper component
- Token persistence in secure storage
- Automatic token refresh
- Session timeout handling","Critical","3","security,frontend,phase1","Security Hardening","- AuthContext with login/logout functionality
- Protected route wrapper component
- Token persistence in secure storage
- Automatic token refresh
- Session timeout handling","Sprint 2","Frontend Team","v1.1.0","Frontend"
"Task","MEVS-SEC-004: Login/Registration UI","Create secure login and registration interfaces

Requirements:
- Login form with validation
- Registration form with password requirements
- Two-factor authentication support
- Remember me functionality (secure)
- Password strength indicator","Critical","3","security,frontend,phase1","Security Hardening","- Login form with validation
- Registration form with password requirements
- Two-factor authentication support
- Remember me functionality (secure)
- Password strength indicator","Sprint 2","Frontend Team","v1.1.0","Frontend"
"Task","MEVS-SEC-005: Environment Variable Security","Restructure environment variables for security

Requirements:
- .env.template created with all variables
- Separate configs for dev/staging/prod
- No hardcoded secrets in codebase
- Documentation for secret rotation
- Validation script for env variables","Critical","1","security,configuration,phase1","Security Hardening","- .env.template created with all variables
- Separate configs for dev/staging/prod
- No hardcoded secrets in codebase
- Documentation for secret rotation
- Validation script for env variables","Sprint 1","DevOps Team","v1.1.0","Infrastructure"
"Task","MEVS-SEC-006: Docker Secrets Implementation","Implement Docker secrets for sensitive data

Requirements:
- Docker secrets configured for all passwords
- Secrets mounted as files not env vars
- Secret rotation mechanism
- Backup and recovery procedures
- Access control on secret files","Critical","3","security,docker,phase1","Security Hardening","- Docker secrets configured for all passwords
- Secrets mounted as files not env vars
- Secret rotation mechanism
- Backup and recovery procedures
- Access control on secret files","Sprint 2","DevOps Team","v1.1.0","Infrastructure"
"Task","MEVS-SEC-007: Secret Generation Automation","Automate secure secret generation

Requirements:
- Script generates cryptographically secure secrets
- Different secret strengths for different uses
- Secrets stored with correct permissions (600)
- Backup mechanism included
- Integration with CI/CD pipeline","Critical","1","security,automation,phase1","Security Hardening","- Script generates cryptographically secure secrets
- Different secret strengths for different uses
- Secrets stored with correct permissions (600)
- Backup mechanism included
- Integration with CI/CD pipeline","Sprint 1","DevOps Team","v1.1.0","Infrastructure"
"Task","MEVS-SEC-008: Non-Root Container Implementation","Configure all containers to run as non-root users

Requirements:
- All Dockerfiles use non-root users
- Proper file permissions set
- No privilege escalation possible
- Health checks work with non-root
- Logging works with non-root","Critical","2","security,docker,phase1","Security Hardening","- All Dockerfiles use non-root users
- Proper file permissions set
- No privilege escalation possible
- Health checks work with non-root
- Logging works with non-root","Sprint 2","DevOps Team","v1.1.0","Infrastructure"
"Task","MEVS-SEC-009: Container Security Constraints","Apply security constraints to all containers

Requirements:
- no-new-privileges flag enabled
- Capabilities dropped (except required)
- Read-only root filesystem
- Security profiles applied
- Resource limits configured","Critical","1","security,docker,phase1","Security Hardening","- no-new-privileges flag enabled
- Capabilities dropped (except required)
- Read-only root filesystem
- Security profiles applied
- Resource limits configured","Sprint 2","DevOps Team","v1.1.0","Infrastructure"
"Task","MEVS-SEC-010: Container Vulnerability Scanning","Implement automated container vulnerability scanning

Requirements:
- Trivy scanner integrated
- Scan on every build
- Block deployment if critical vulnerabilities
- Regular base image updates
- Vulnerability reports generated","Critical","2","security,docker,phase1","Security Hardening","- Trivy scanner integrated
- Scan on every build
- Block deployment if critical vulnerabilities
- Regular base image updates
- Vulnerability reports generated","Sprint 2","DevOps Team","v1.1.0","Infrastructure"
EOF

# Phase 2 High Priority Tickets
cat >> "$OUTPUT_FILE" << 'EOF'
"Task","MEVS-SEC-011: Frontend Input Sanitization","Implement DOMPurify for XSS prevention

Requirements:
- DOMPurify integrated in all user inputs
- Custom sanitization rules defined
- HTML content properly escaped
- Rich text editor security configured
- Unit tests for sanitization","High","3","security,frontend,phase2","Security Hardening","- DOMPurify integrated in all user inputs
- Custom sanitization rules defined
- HTML content properly escaped
- Rich text editor security configured
- Unit tests for sanitization","Sprint 3","Frontend Team","v1.2.0","Frontend"
"Task","MEVS-SEC-012: Backend Input Validation","Implement comprehensive input validation on API

Requirements:
- Validation schemas for all endpoints
- Ethereum address validation
- Amount/number validation
- String length limits
- SQL injection prevention","High","5","security,backend,phase2","Security Hardening","- Validation schemas for all endpoints
- Ethereum address validation
- Amount/number validation
- String length limits
- SQL injection prevention","Sprint 3","Backend Team","v1.2.0","Backend"
"Task","MEVS-SEC-013: File Upload Security","Secure file upload handling

Requirements:
- File type validation
- File size limits
- Virus scanning integration
- Secure file storage location
- File name sanitization","High","3","security,backend,phase2","Security Hardening","- File type validation
- File size limits
- Virus scanning integration
- Secure file storage location
- File name sanitization","Sprint 4","Backend Team","v1.2.0","Backend"
"Task","MEVS-SEC-014: Nginx Security Headers","Implement comprehensive security headers

Requirements:
- CSP header configured
- HSTS enabled with preload
- X-Frame-Options set to DENY
- X-Content-Type-Options nosniff
- Referrer-Policy configured","High","1","security,infrastructure,phase2","Security Hardening","- CSP header configured
- HSTS enabled with preload
- X-Frame-Options set to DENY
- X-Content-Type-Options nosniff
- Referrer-Policy configured","Sprint 3","DevOps Team","v1.2.0","Infrastructure"
"Task","MEVS-SEC-015: SSL/TLS Configuration","Implement strong TLS configuration

Requirements:
- TLS 1.2/1.3 only
- Strong cipher suites only
- Certificate automation with Let's Encrypt
- OCSP stapling enabled
- SSL Labs A+ rating achieved","High","3","security,infrastructure,phase2","Security Hardening","- TLS 1.2/1.3 only
- Strong cipher suites only
- Certificate automation with Let's Encrypt
- OCSP stapling enabled
- SSL Labs A+ rating achieved","Sprint 3","DevOps Team","v1.2.0","Infrastructure"
"Task","MEVS-SEC-016: Rate Limiting Implementation","Implement API rate limiting

Requirements:
- Per-user rate limits
- Per-IP rate limits
- Different limits for different endpoints
- Rate limit headers in responses
- DDoS protection configured","High","3","security,backend,phase2","Security Hardening","- Per-user rate limits
- Per-IP rate limits
- Different limits for different endpoints
- Rate limit headers in responses
- DDoS protection configured","Sprint 4","Backend Team","v1.2.0","Backend"
"Task","MEVS-SEC-017: Database Network Isolation","Isolate database from external network

Requirements:
- Databases on internal network only
- No external port exposure
- Access through application only
- Network policies configured
- Connection encryption enabled","High","2","security,infrastructure,phase2","Security Hardening","- Databases on internal network only
- No external port exposure
- Access through application only
- Network policies configured
- Connection encryption enabled","Sprint 3","DevOps Team","v1.2.0","Infrastructure"
"Task","MEVS-SEC-018: Restrictive CORS Policy","Implement restrictive CORS configuration

Requirements:
- Whitelist specific origins only
- Credentials support configured properly
- Preflight caching configured
- Methods restricted to required only
- Headers restricted to required only","High","1","security,backend,phase2","Security Hardening","- Whitelist specific origins only
- Credentials support configured properly
- Preflight caching configured
- Methods restricted to required only
- Headers restricted to required only","Sprint 4","Backend Team","v1.2.0","Backend"
"Task","MEVS-SEC-019: API Gateway Security","Implement API gateway with security features

Requirements:
- API versioning implemented
- Request/response validation
- API key management
- Throttling configured
- Logging and monitoring","High","3","security,infrastructure,phase2","Security Hardening","- API versioning implemented
- Request/response validation
- API key management
- Throttling configured
- Logging and monitoring","Sprint 4","DevOps Team","v1.2.0","Infrastructure"
EOF

echo "✅ JIRA export completed: $OUTPUT_FILE"
echo "📊 Total tickets exported: 19 (Phase 1 & 2)"
echo ""
echo "📋 Import Instructions:"
echo "1. Go to JIRA > System > Import & Export > External System Import"
echo "2. Select 'CSV' import option"
echo "3. Upload $OUTPUT_FILE"
echo "4. Map fields as needed"
echo "5. Run import"
echo ""
echo "💡 Note: Remaining Phase 3 & 4 tickets can be added after initial import"