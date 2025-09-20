#!/usr/bin/env node

/**
 * Push MEV Shield tickets to JIRA
 */

const axios = require('axios');

// JIRA Configuration
const JIRA_URL = 'https://aurigraphdlt.atlassian.net';
const JIRA_EMAIL = 'subbu@aurigraph.io';
const JIRA_API_TOKEN = process.env.JIRA_API_TOKEN || '';

// Create axios instance
const jira = axios.create({
  baseURL: `${JIRA_URL}/rest/api/3`,
  auth: {
    username: JIRA_EMAIL,
    password: JIRA_API_TOKEN
  },
  headers: {
    'Accept': 'application/json',
    'Content-Type': 'application/json'
  }
});

// All tickets to create
const tickets = [
  // Sprint 1: Infrastructure
  {
    summary: '[MEV Shield] Set up development environment',
    description: 'Configure Docker, Kubernetes, CI/CD pipelines for MEV Shield platform',
    issueType: 'Task',
    priority: 'Highest',
    labels: ['mev-shield', 'infrastructure', 'sprint-1']
  },
  {
    summary: '[MEV Shield] Database architecture design',
    description: 'Design PostgreSQL schema and Redis caching strategy for MEV data',
    issueType: 'Task',
    priority: 'Highest',
    labels: ['mev-shield', 'database', 'sprint-1']
  },
  {
    summary: '[MEV Shield] Blockchain node infrastructure',
    description: 'Set up Ethereum, BSC, Polygon nodes for mempool monitoring',
    issueType: 'Task',
    priority: 'Highest',
    labels: ['mev-shield', 'blockchain', 'sprint-1']
  },
  {
    summary: '[MEV Shield] Authentication service setup',
    description: 'Implement JWT auth, wallet authentication, OAuth2 for user management',
    issueType: 'Task',
    priority: 'Highest',
    labels: ['mev-shield', 'auth', 'sprint-1']
  },
  {
    summary: '[MEV Shield] Monitoring and logging setup',
    description: 'Configure Prometheus, Grafana, ELK stack for system monitoring',
    issueType: 'Task',
    priority: 'High',
    labels: ['mev-shield', 'monitoring', 'sprint-1']
  },
  
  // Sprint 2: Detection Engine
  {
    summary: '[MEV Shield] Mempool monitoring service',
    description: 'Build real-time mempool data ingestion and monitoring service',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'detection', 'sprint-2']
  },
  {
    summary: '[MEV Shield] Transaction analysis module',
    description: 'Parse and analyze transaction data for MEV detection patterns',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'detection', 'sprint-2']
  },
  {
    summary: '[MEV Shield] Front-running detection algorithm',
    description: 'Implement ML algorithm to detect front-running attacks in real-time',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'ml', 'sprint-2']
  },
  {
    summary: '[MEV Shield] Sandwich attack detection',
    description: 'Identify and prevent sandwich attack patterns using ML',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'ml', 'sprint-2']
  },
  {
    summary: '[MEV Shield] Detection API endpoints',
    description: 'Create REST API for MEV detection results and alerts',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'api', 'sprint-2']
  },
  
  // Sprint 3: Protection
  {
    summary: '[MEV Shield] Private transaction pool',
    description: 'Implement private mempool functionality for transaction protection',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'protection', 'sprint-3']
  },
  {
    summary: '[MEV Shield] Flashbots integration',
    description: 'Integrate Flashbots Protect API for MEV protection',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'flashbots', 'sprint-3']
  },
  {
    summary: '[MEV Shield] Transaction routing engine',
    description: 'Build smart routing engine for protected transactions',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'routing', 'sprint-3']
  },
  {
    summary: '[MEV Shield] Protection API endpoints',
    description: 'API endpoints for transaction protection services',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'api', 'sprint-3']
  },
  {
    summary: '[MEV Shield] Gas optimization module',
    description: 'Optimize gas usage for protected transactions',
    issueType: 'Story',
    priority: 'High',
    labels: ['mev-shield', 'optimization', 'sprint-3']
  },
  
  // Sprint 4: Dashboard
  {
    summary: '[MEV Shield] Dashboard UI framework',
    description: 'Set up React dashboard with routing and state management',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'frontend', 'sprint-4']
  },
  {
    summary: '[MEV Shield] Wallet connection integration',
    description: 'Integrate MetaMask and WalletConnect for user authentication',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'wallet', 'sprint-4']
  },
  {
    summary: '[MEV Shield] Portfolio overview page',
    description: 'Display wallet balances and MEV protection status',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'dashboard', 'sprint-4']
  },
  {
    summary: '[MEV Shield] Transaction history view',
    description: 'Show transaction history with MEV impact analysis',
    issueType: 'Story',
    priority: 'Highest',
    labels: ['mev-shield', 'dashboard', 'sprint-4']
  },
  {
    summary: '[MEV Shield] Real-time notifications',
    description: 'WebSocket implementation for real-time MEV alerts',
    issueType: 'Story',
    priority: 'High',
    labels: ['mev-shield', 'notifications', 'sprint-4']
  }
];

// Create ticket function
async function createTicket(ticket) {
  try {
    // Find project key
    const projectsResponse = await jira.get('/project');
    let projectKey = 'MEVS';
    
    // Try to find MEV Shield project or use first available
    const mevProject = projectsResponse.data.find(p => 
      p.key === 'MEVS' || 
      p.name.includes('MEV') ||
      p.key === 'HMS'
    );
    
    if (mevProject) {
      projectKey = mevProject.key;
    } else if (projectsResponse.data.length > 0) {
      projectKey = projectsResponse.data[0].key;
      console.log(`Using project: ${projectKey}`);
    }
    
    // Get issue types for project
    const issueTypesResponse = await jira.get(`/project/${projectKey}`);
    const issueTypes = issueTypesResponse.data.issueTypes || [];
    
    // Find matching issue type
    let issueTypeId = '10001'; // default task
    const matchingType = issueTypes.find(t => 
      t.name.toLowerCase() === ticket.issueType.toLowerCase() ||
      t.name === 'Task'
    );
    
    if (matchingType) {
      issueTypeId = matchingType.id;
    }
    
    // Create issue
    const issueData = {
      fields: {
        project: { key: projectKey },
        summary: ticket.summary,
        description: {
          type: 'doc',
          version: 1,
          content: [
            {
              type: 'paragraph',
              content: [
                {
                  type: 'text',
                  text: ticket.description
                }
              ]
            }
          ]
        },
        issuetype: { id: issueTypeId },
        labels: ticket.labels
      }
    };
    
    // Set priority if available
    if (ticket.priority) {
      const priorities = {
        'Highest': '1',
        'High': '2',
        'Medium': '3',
        'Low': '4',
        'Lowest': '5'
      };
      
      if (priorities[ticket.priority]) {
        issueData.fields.priority = { id: priorities[ticket.priority] };
      }
    }
    
    const response = await jira.post('/issue', issueData);
    console.log(`✅ Created: ${response.data.key} - ${ticket.summary}`);
    return response.data.key;
    
  } catch (error) {
    if (error.response) {
      console.error(`❌ Failed to create ticket: ${ticket.summary}`);
      console.error(`   Error: ${JSON.stringify(error.response.data.errors || error.response.data)}`);
    } else {
      console.error(`❌ Failed to create ticket: ${error.message}`);
    }
    return null;
  }
}

// Main function
async function main() {
  console.log('🚀 Pushing MEV Shield tickets to JIRA\n');
  
  if (!JIRA_API_TOKEN) {
    console.error('❌ JIRA_API_TOKEN not set!');
    console.log('\nTo get your JIRA API token:');
    console.log('1. Go to: https://id.atlassian.com/manage-profile/security/api-tokens');
    console.log('2. Create a new token');
    console.log('3. Run: export JIRA_API_TOKEN="your-token-here"');
    console.log('4. Run this script again\n');
    process.exit(1);
  }
  
  console.log(`📊 Creating ${tickets.length} tickets...\n`);
  
  let created = 0;
  let failed = 0;
  
  for (const ticket of tickets) {
    const key = await createTicket(ticket);
    if (key) {
      created++;
    } else {
      failed++;
    }
    
    // Rate limiting
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
  
  console.log('\n✅ Complete!');
  console.log(`   Created: ${created} tickets`);
  console.log(`   Failed: ${failed} tickets`);
  console.log(`\n🔗 View in JIRA: ${JIRA_URL}/projects`);
}

// Run if called directly
if (require.main === module) {
  main().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { createTicket, tickets };