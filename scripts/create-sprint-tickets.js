#!/usr/bin/env node

/**
 * MEV Shield - Create Sprint Tickets in JIRA
 * This script creates all sprint tickets organized by epics
 */

const axios = require('axios');

// JIRA Configuration
const JIRA_URL = 'https://aurigraphdlt.atlassian.net';
const JIRA_EMAIL = 'subbu@aurigraph.io';
const JIRA_API_TOKEN = process.env.JIRA_API_TOKEN || '';

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

// Sprint tickets organized by sprint and epic
const sprintTickets = {
  'Sprint 1': {
    epic: 'Core Infrastructure & Architecture',
    tickets: [
      { summary: 'Set up Docker/K8s environment', points: 8, priority: 'Highest' },
      { summary: 'Design PostgreSQL schema for MEV data', points: 13, priority: 'Highest' },
      { summary: 'Configure CI/CD pipelines', points: 8, priority: 'High' },
      { summary: 'Set up Prometheus/Grafana monitoring', points: 5, priority: 'Medium' },
      { summary: 'Implement Redis caching layer', points: 5, priority: 'Medium' },
      { summary: 'Create backup and disaster recovery', points: 8, priority: 'High' },
      { summary: 'Implement secrets management', points: 5, priority: 'High' },
      { summary: 'Initialize project repositories', points: 8, priority: 'Highest' }
    ]
  },
  'Sprint 2': {
    epic: 'Blockchain Integration Layer',
    tickets: [
      { summary: 'Connect Ethereum mainnet nodes', points: 13, priority: 'Highest' },
      { summary: 'Build mempool streaming service', points: 13, priority: 'Highest' },
      { summary: 'Create blockchain data indexer', points: 8, priority: 'High' },
      { summary: 'Implement WebSocket connections', points: 8, priority: 'High' },
      { summary: 'Set up RPC endpoints', points: 5, priority: 'Medium' },
      { summary: 'Build block listener service', points: 8, priority: 'High' },
      { summary: 'Create transaction decoder', points: 8, priority: 'Medium' }
    ]
  },
  'Sprint 3': {
    epic: 'MEV Detection Engine',
    tickets: [
      { summary: 'Integrate Polygon network', points: 8, priority: 'High' },
      { summary: 'Integrate BSC network', points: 8, priority: 'High' },
      { summary: 'Implement cross-chain tracking', points: 8, priority: 'Medium' },
      { summary: 'Implement front-running detection', points: 21, priority: 'Highest' },
      { summary: 'Create ML model training pipeline', points: 13, priority: 'High' },
      { summary: 'Set up feature extraction', points: 7, priority: 'Medium' }
    ]
  },
  'Sprint 4': {
    epic: 'MEV Detection Engine',
    tickets: [
      { summary: 'Build sandwich attack recognition', points: 21, priority: 'Highest' },
      { summary: 'Detect JIT liquidity attacks', points: 13, priority: 'High' },
      { summary: 'Build real-time MEV alerts', points: 8, priority: 'High' },
      { summary: 'Implement MEV impact calculator', points: 8, priority: 'Medium' },
      { summary: 'Create detection API endpoints', points: 5, priority: 'Medium' },
      { summary: 'Build alert notification system', points: 7, priority: 'High' }
    ]
  },
  'Sprint 5': {
    epic: 'Protection & Mitigation System',
    tickets: [
      { summary: 'Build private transaction pool', points: 21, priority: 'Highest' },
      { summary: 'Integrate Flashbots Protect', points: 13, priority: 'Highest' },
      { summary: 'Implement transaction bundling', points: 13, priority: 'High' },
      { summary: 'Add slippage protection', points: 8, priority: 'High' },
      { summary: 'Create protection API', points: 8, priority: 'Medium' }
    ]
  },
  'Sprint 6': {
    epic: 'Protection & Mitigation System',
    tickets: [
      { summary: 'Create smart order routing', points: 21, priority: 'Highest' },
      { summary: 'Build gas optimization engine', points: 8, priority: 'High' },
      { summary: 'Create MEV redistribution system', points: 13, priority: 'High' },
      { summary: 'Build responsive web dashboard foundation', points: 10, priority: 'High' },
      { summary: 'Set up authentication system', points: 8, priority: 'Highest' }
    ]
  },
  'Sprint 7': {
    epic: 'User Experience & Dashboard',
    tickets: [
      { summary: 'Complete responsive web dashboard', points: 11, priority: 'Highest' },
      { summary: 'Integrate MetaMask/WalletConnect', points: 13, priority: 'Highest' },
      { summary: 'Create portfolio overview', points: 13, priority: 'High' },
      { summary: 'Build transaction history view', points: 8, priority: 'High' },
      { summary: 'Implement alerts system', points: 8, priority: 'Medium' },
      { summary: 'Create user settings', points: 5, priority: 'Medium' },
      { summary: 'Build notification center', points: 6, priority: 'Medium' }
    ]
  },
  'Sprint 8': {
    epic: 'API & Developer Platform',
    tickets: [
      { summary: 'Develop iOS mobile app', points: 21, priority: 'High' },
      { summary: 'Develop Android mobile app', points: 21, priority: 'High' },
      { summary: 'Build REST API v2', points: 8, priority: 'Highest' },
      { summary: 'Create API documentation', points: 5, priority: 'High' },
      { summary: 'Build rate limiting', points: 4, priority: 'Medium' },
      { summary: 'Implement API keys', points: 4, priority: 'High' }
    ]
  },
  'Sprint 9': {
    epic: 'API & Developer Platform',
    tickets: [
      { summary: 'Build WebSocket API', points: 13, priority: 'High' },
      { summary: 'Create JavaScript SDK', points: 8, priority: 'High' },
      { summary: 'Create Python SDK', points: 8, priority: 'Medium' },
      { summary: 'Build webhook system', points: 8, priority: 'Medium' },
      { summary: 'Create real-time analytics dashboard', points: 13, priority: 'High' },
      { summary: 'Build data aggregation pipeline', points: 8, priority: 'Medium' },
      { summary: 'Implement caching layer', points: 3, priority: 'Low' }
    ]
  },
  'Sprint 10': {
    epic: 'Analytics & Reporting',
    tickets: [
      { summary: 'Complete historical MEV analysis', points: 13, priority: 'High' },
      { summary: 'Create custom report builder', points: 13, priority: 'High' },
      { summary: 'Build ML predictions system', points: 8, priority: 'Medium' },
      { summary: 'Create data export functionality', points: 5, priority: 'Medium' },
      { summary: 'Build dashboard widgets', points: 8, priority: 'High' },
      { summary: 'Implement data visualization', points: 8, priority: 'High' },
      { summary: 'Create trend analysis', points: 5, priority: 'Medium' }
    ]
  },
  'Sprint 11': {
    epic: 'Security & Compliance',
    tickets: [
      { summary: 'Conduct security audit', points: 13, priority: 'Highest' },
      { summary: 'Implement security fixes', points: 8, priority: 'Highest' },
      { summary: 'SOC2 compliance preparation', points: 13, priority: 'High' },
      { summary: 'GDPR compliance implementation', points: 8, priority: 'High' },
      { summary: 'Performance optimization', points: 8, priority: 'High' },
      { summary: 'Implement auto-scaling', points: 8, priority: 'Medium' },
      { summary: 'Load testing & optimization', points: 7, priority: 'High' }
    ]
  },
  'Sprint 12': {
    epic: 'Launch & Operations',
    tickets: [
      { summary: 'Production deployment', points: 13, priority: 'Highest' },
      { summary: 'Launch marketing website', points: 8, priority: 'High' },
      { summary: 'Set up 24/7 support', points: 8, priority: 'Highest' },
      { summary: 'Create user onboarding', points: 8, priority: 'High' },
      { summary: 'Implement monitoring alerts', points: 5, priority: 'High' },
      { summary: 'Create documentation', points: 8, priority: 'Medium' },
      { summary: 'Launch beta program', points: 5, priority: 'High' },
      { summary: 'Go-live activities', points: 6, priority: 'Highest' }
    ]
  }
};

// Create a single ticket
async function createTicket(ticket, sprintName, epicName, projectKey) {
  try {
    const ticketData = {
      fields: {
        project: { key: projectKey },
        summary: `[${sprintName}] ${ticket.summary}`,
        description: {
          type: 'doc',
          version: 1,
          content: [
            {
              type: 'paragraph',
              content: [
                {
                  type: 'text',
                  text: `Sprint: ${sprintName}\nEpic: ${epicName}\nStory Points: ${ticket.points}\nPriority: ${ticket.priority}`
                }
              ]
            }
          ]
        },
        issuetype: { name: 'Task' },
        labels: ['mev-shield', sprintName.toLowerCase().replace(' ', '-'), 'sprint-ticket']
      }
    };
    
    const response = await jira.post('/issue', ticketData);
    console.log(`   ✅ ${response.data.key}: ${ticket.summary} (${ticket.points} pts)`);
    return response.data.key;
    
  } catch (error) {
    console.error(`   ❌ Failed: ${ticket.summary}`);
    if (error.response && error.response.data) {
      console.error(`      Error: ${JSON.stringify(error.response.data.errors || error.response.data)}`);
    }
    return null;
  }
}

// Main execution
async function main() {
  console.log('🚀 Creating MEV Shield Sprint Tickets in JIRA\n');
  
  if (!JIRA_API_TOKEN) {
    console.error('❌ JIRA_API_TOKEN not set!');
    console.log('\nSet the token with:');
    console.log('export JIRA_API_TOKEN="your-token-here"');
    process.exit(1);
  }
  
  try {
    // Get the project
    const projectsResponse = await jira.get('/project');
    const project = projectsResponse.data.find(p => 
      p.key === 'MEV' || p.name === 'MEV Shield' || p.name.includes('MEV')
    );
    
    if (!project) {
      console.error('❌ MEV project not found in JIRA');
      process.exit(1);
    }
    
    const projectKey = project.key;
    console.log(`📋 Using project: ${projectKey} - ${project.name}\n`);
    
    let totalTickets = 0;
    let totalPoints = 0;
    
    // Create tickets for each sprint
    for (const [sprintName, sprintData] of Object.entries(sprintTickets)) {
      console.log(`\n🏃 ${sprintName} - ${sprintData.epic}`);
      console.log('─'.repeat(50));
      
      for (const ticket of sprintData.tickets) {
        const ticketKey = await createTicket(ticket, sprintName, sprintData.epic, projectKey);
        if (ticketKey) {
          totalTickets++;
          totalPoints += ticket.points;
        }
        
        // Rate limiting
        await new Promise(resolve => setTimeout(resolve, 500));
      }
      
      const sprintPoints = sprintData.tickets.reduce((sum, t) => sum + t.points, 0);
      console.log(`   Sprint Total: ${sprintPoints} points`);
    }
    
    // Summary
    console.log('\n' + '='.repeat(60));
    console.log('✅ Sprint Ticket Creation Complete!');
    console.log('='.repeat(60));
    console.log(`📊 Statistics:`);
    console.log(`   • Total Tickets Created: ${totalTickets}`);
    console.log(`   • Total Story Points: ${totalPoints}`);
    console.log(`   • Sprints: 12`);
    console.log(`   • Average Points per Sprint: ${Math.round(totalPoints / 12)}`);
    console.log(`\n🔗 View in JIRA: ${JIRA_URL}/projects/${projectKey}`);
    console.log(`📋 Board: ${JIRA_URL}/projects/${projectKey}/boards`);
    
  } catch (error) {
    console.error('\n❌ Fatal error:', error.message);
    if (error.response) {
      console.error('Response:', error.response.data);
    }
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { sprintTickets, createTicket };