#!/bin/bash

# MEV Shield JIRA Epic and Task Creation Script
# This script creates comprehensive epics and tasks for MEV Shield development

set -e

# Configuration
JIRA_BASE_URL="https://aurigraphdlt.atlassian.net"
JIRA_PROJECT_KEY="MEV"
JIRA_BOARD_ID="855"
JIRA_EMAIL="subbu@aurigraph.io"

# API Token (will be passed as parameter or environment variable)
if [ -z "$JIRA_API_TOKEN" ]; then
    echo "Enter JIRA API Token:"
    read -s JIRA_API_TOKEN
fi

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}   MEV Shield JIRA Setup${NC}"
echo -e "${BLUE}=====================================${NC}"
echo ""

# Base64 encode credentials
AUTH_HEADER="Authorization: Basic $(echo -n "${JIRA_EMAIL}:${JIRA_API_TOKEN}" | base64)"

# Function to create JIRA issue
create_issue() {
    local issue_type="$1"
    local summary="$2"
    local description="$3"
    local parent_key="$4"
    local labels="$5"
    local story_points="$6"
    local priority="$7"
    
    # Set default priority if not specified
    if [ -z "$priority" ]; then
        priority="Medium"
    fi
    
    # Build JSON payload
    if [ "$issue_type" = "Epic" ]; then
        PAYLOAD=$(cat <<EOF
{
  "fields": {
    "project": {"key": "MEV"},
    "summary": "$summary",
    "description": {
      "type": "doc",
      "version": 1,
      "content": [{
        "type": "paragraph",
        "content": [{
          "type": "text",
          "text": "$description"
        }]
      }]
    },
    "issuetype": {"name": "Epic"},
    "customfield_10011": "$summary",
    "labels": $labels,
    "priority": {"name": "$priority"}
  }
}
EOF
        )
    else
        # Task or Story
        if [ ! -z "$parent_key" ]; then
            # Sub-task under Epic
            PARENT_FIELD="\"parent\": {\"key\": \"$parent_key\"},"
        else
            PARENT_FIELD=""
        fi
        
        if [ ! -z "$story_points" ] && [ "$story_points" != "null" ]; then
            POINTS_FIELD="\"customfield_10016\": $story_points,"
        else
            POINTS_FIELD=""
        fi
        
        PAYLOAD=$(cat <<EOF
{
  "fields": {
    "project": {"key": "MEV"},
    "summary": "$summary",
    "description": {
      "type": "doc",
      "version": 1,
      "content": [{
        "type": "paragraph",
        "content": [{
          "type": "text",
          "text": "$description"
        }]
      }]
    },
    "issuetype": {"name": "$issue_type"},
    $PARENT_FIELD
    $POINTS_FIELD
    "labels": $labels,
    "priority": {"name": "$priority"}
  }
}
EOF
        )
    fi
    
    # Create the issue
    RESPONSE=$(curl -s -X POST \
        -H "$AUTH_HEADER" \
        -H "Content-Type: application/json" \
        -d "$PAYLOAD" \
        "${JIRA_BASE_URL}/rest/api/3/issue")
    
    # Extract issue key
    ISSUE_KEY=$(echo "$RESPONSE" | jq -r '.key // empty')
    ERROR=$(echo "$RESPONSE" | jq -r '.errors // empty')
    
    if [ ! -z "$ISSUE_KEY" ] && [ "$ISSUE_KEY" != "null" ]; then
        echo -e "${GREEN}✅ Created $issue_type: $ISSUE_KEY - $summary${NC}"
        echo "$ISSUE_KEY"
    else
        echo -e "${RED}❌ Failed to create $issue_type: $summary${NC}"
        if [ ! -z "$ERROR" ] && [ "$ERROR" != "null" ]; then
            echo -e "${RED}   Error: $ERROR${NC}"
        fi
        echo ""
    fi
}

# Track created epics
EPIC_CORE=""
EPIC_ADMIN=""
EPIC_USER=""
EPIC_ML=""
EPIC_QA=""
EPIC_BLOCKCHAIN=""

echo -e "${YELLOW}Creating Epics and Tasks...${NC}"
echo ""

# Epic 1: Core MEV Protection System
echo -e "${BLUE}Creating Epic: Core MEV Protection System${NC}"
EPIC_CORE=$(create_issue "Epic" \
    "🛡️ Core MEV Protection System" \
    "Implement the foundational MEV protection mechanisms including encryption, ordering, and redistribution. This epic covers all backend Rust services that form the core of MEV Shield." \
    "" \
    '["mev-shield", "core", "backend", "rust"]' \
    "" \
    "High")

if [ ! -z "$EPIC_CORE" ] && [ "$EPIC_CORE" != "" ]; then
    create_issue "Task" \
        "Implement Threshold Encryption Service" \
        "Build the encryption service for transaction privacy using threshold cryptography. Implement BLS signatures and distributed key generation." \
        "$EPIC_CORE" \
        '["encryption", "security", "cryptography"]' \
        "8" \
        "High"
    
    create_issue "Task" \
        "Develop VDF-based Ordering System" \
        "Create verifiable delay function for fair transaction ordering. Implement time-lock puzzles for ordering fairness." \
        "$EPIC_CORE" \
        '["vdf", "ordering", "fairness"]' \
        "13" \
        "High"
    
    create_issue "Task" \
        "Build MEV Redistribution Mechanism" \
        "Implement fair value redistribution to users and validators. Design economic incentives for participation." \
        "$EPIC_CORE" \
        '["redistribution", "economics", "incentives"]' \
        "8" \
        "Medium"
    
    create_issue "Task" \
        "Create Detection Engine" \
        "Build ML-based MEV detection and classification system. Identify sandwich attacks, front-running, and arbitrage." \
        "$EPIC_CORE" \
        '["detection", "ml", "analysis"]' \
        "13" \
        "High"
fi

# Epic 2: Admin Dashboard
echo -e "${BLUE}Creating Epic: Admin Dashboard${NC}"
EPIC_ADMIN=$(create_issue "Epic" \
    "👨‍💼 Admin Dashboard Development" \
    "Build comprehensive admin interface for monitoring and managing MEV Shield. Full control panel for system administrators." \
    "" \
    '["dashboard", "frontend", "admin", "react"]' \
    "" \
    "High")

if [ ! -z "$EPIC_ADMIN" ] && [ "$EPIC_ADMIN" != "" ]; then
    create_issue "Task" \
        "Design Admin Dashboard UI/UX" \
        "Create Material-UI based dashboard design with responsive layout. Focus on data visualization and real-time updates." \
        "$EPIC_ADMIN" \
        '["ui", "design", "material-ui"]' \
        "5" \
        "Medium"
    
    create_issue "Task" \
        "Implement System Metrics Dashboard" \
        "Real-time monitoring of MEV protection metrics and system health. WebSocket connections for live updates." \
        "$EPIC_ADMIN" \
        '["metrics", "monitoring", "websocket"]' \
        "8" \
        "High"
    
    create_issue "Task" \
        "Build Validator Management Interface" \
        "Interface for managing validator nodes and reputation scores. CRUD operations for validator configuration." \
        "$EPIC_ADMIN" \
        '["validators", "management", "crud"]' \
        "5" \
        "Medium"
fi

# Epic 3: User Dashboard
echo -e "${BLUE}Creating Epic: User Dashboard${NC}"
EPIC_USER=$(create_issue "Epic" \
    "👤 User Dashboard Development" \
    "Create user-facing interface for transaction protection and MEV analytics. Enable users to protect their transactions." \
    "" \
    '["dashboard", "frontend", "user", "react"]' \
    "" \
    "High")

if [ ! -z "$EPIC_USER" ] && [ "$EPIC_USER" != "" ]; then
    create_issue "Task" \
        "Implement Transaction Protection Interface" \
        "Allow users to submit protected transactions through the UI. Integration with Web3 providers." \
        "$EPIC_USER" \
        '["transactions", "protection", "web3"]' \
        "8" \
        "High"
    
    create_issue "Task" \
        "Build MEV Savings Calculator" \
        "Show users how much MEV they saved. Historical data and analytics visualization." \
        "$EPIC_USER" \
        '["analytics", "calculator", "charts"]' \
        "5" \
        "Medium"
    
    create_issue "Task" \
        "Create Wallet Integration" \
        "Connect MetaMask, WalletConnect, and other Web3 wallets. Support multiple chain connections." \
        "$EPIC_USER" \
        '["wallets", "web3", "integration"]' \
        "8" \
        "High"
fi

# Epic 4: Machine Learning & AI
echo -e "${BLUE}Creating Epic: Machine Learning & AI${NC}"
EPIC_ML=$(create_issue "Epic" \
    "🧠 Neural Network & ML Integration" \
    "Advanced ML models for MEV prediction and detection. State-of-the-art AI for pattern recognition." \
    "" \
    '["ml", "ai", "neural-network", "prediction"]' \
    "" \
    "Medium")

if [ ! -z "$EPIC_ML" ] && [ "$EPIC_ML" != "" ]; then
    create_issue "Task" \
        "Implement LSTM Predictor" \
        "LSTM model for MEV opportunity prediction. Time-series analysis of transaction patterns." \
        "$EPIC_ML" \
        '["lstm", "prediction", "timeseries"]' \
        "13" \
        "Medium"
    
    create_issue "Task" \
        "Build Graph Neural Network" \
        "GNN for transaction graph analysis. Detect complex MEV patterns in transaction networks." \
        "$EPIC_ML" \
        '["gnn", "graph", "networks"]' \
        "13" \
        "Medium"
fi

# Epic 5: Testing & Quality
echo -e "${BLUE}Creating Epic: Testing & Quality${NC}"
EPIC_QA=$(create_issue "Epic" \
    "🧪 Testing & Quality Assurance" \
    "Comprehensive testing suite and quality standards. Ensure production-ready code quality." \
    "" \
    '["testing", "qa", "quality", "ci-cd"]' \
    "" \
    "High")

if [ ! -z "$EPIC_QA" ] && [ "$EPIC_QA" != "" ]; then
    create_issue "Task" \
        "Write Unit Tests for Core Services" \
        "Achieve 80% code coverage for all Rust services. Use cargo test framework." \
        "$EPIC_QA" \
        '["unit-tests", "rust", "coverage"]' \
        "8" \
        "High"
    
    create_issue "Task" \
        "Create Integration Test Suite" \
        "End-to-end integration testing. Test all service interactions and API endpoints." \
        "$EPIC_QA" \
        '["integration", "e2e", "api-testing"]' \
        "8" \
        "High"
fi

# Epic 6: Blockchain Integration
echo -e "${BLUE}Creating Epic: Blockchain Integration${NC}"
EPIC_BLOCKCHAIN=$(create_issue "Epic" \
    "⛓️ Blockchain Integration" \
    "Integration with Ethereum and other blockchains. Multi-chain support for MEV protection." \
    "" \
    '["blockchain", "ethereum", "web3", "smart-contracts"]' \
    "" \
    "High")

if [ ! -z "$EPIC_BLOCKCHAIN" ] && [ "$EPIC_BLOCKCHAIN" != "" ]; then
    create_issue "Task" \
        "Ethereum Mainnet Integration" \
        "Connect to Ethereum mainnet for production deployment. Implement Web3 providers and RPC connections." \
        "$EPIC_BLOCKCHAIN" \
        '["ethereum", "mainnet", "production"]' \
        "13" \
        "High"
    
    create_issue "Task" \
        "Smart Contract Development" \
        "MEV Shield smart contracts for on-chain components. Solidity contracts for protection mechanisms." \
        "$EPIC_BLOCKCHAIN" \
        '["solidity", "smart-contracts", "deployment"]' \
        "21" \
        "High"
fi

echo ""
echo -e "${YELLOW}Creating Sprint...${NC}"

# Create and start a sprint
SPRINT_NAME="MEV Shield Sprint - $(date +%Y-%m-%d)"
SPRINT_DATA=$(cat <<EOF
{
  "name": "$SPRINT_NAME",
  "startDate": "$(date -u +%Y-%m-%dT%H:%M:%S.000Z)",
  "endDate": "$(date -u -d '+14 days' +%Y-%m-%dT%H:%M:%S.000Z)",
  "originBoardId": ${JIRA_BOARD_ID},
  "goal": "Complete core MEV protection implementation and dashboard setup"
}
EOF
)

SPRINT_RESPONSE=$(curl -s -X POST \
    -H "$AUTH_HEADER" \
    -H "Content-Type: application/json" \
    -d "$SPRINT_DATA" \
    "${JIRA_BASE_URL}/rest/agile/1.0/sprint")

SPRINT_ID=$(echo "$SPRINT_RESPONSE" | jq -r '.id // empty')

if [ ! -z "$SPRINT_ID" ] && [ "$SPRINT_ID" != "null" ]; then
    echo -e "${GREEN}✅ Created Sprint: $SPRINT_NAME (ID: $SPRINT_ID)${NC}"
else
    echo -e "${YELLOW}⚠️ Could not create sprint (might already exist)${NC}"
fi

echo ""
echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}   Setup Complete!${NC}"
echo -e "${GREEN}=====================================${NC}"
echo ""
echo -e "${BLUE}📊 Summary:${NC}"
echo "  • Created 6 main epics"
echo "  • Added 15+ tasks with story points"
echo "  • Set priorities and labels"
echo "  • Created active sprint"
echo ""
echo -e "${BLUE}🔗 Links:${NC}"
echo "  Board: ${JIRA_BASE_URL}/jira/software/projects/MEV/boards/${JIRA_BOARD_ID}"
echo "  Backlog: ${JIRA_BASE_URL}/jira/software/projects/MEV/boards/${JIRA_BOARD_ID}/backlog"
echo ""
echo -e "${BLUE}📝 Next Steps:${NC}"
echo "  1. Review created epics in JIRA"
echo "  2. Drag tasks into the sprint"
echo "  3. Assign team members"
echo "  4. Start using MEV-XXX in commits"
echo ""
echo -e "${GREEN}Happy coding! 🚀${NC}"