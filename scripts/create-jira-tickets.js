#!/usr/bin/env node

/**
 * MEV Shield - JIRA Ticket Creation Script
 * This script creates all tickets defined in the Sprint Breakdown document
 */

const axios = require('axios');
const fs = require('fs');
const path = require('path');

// JIRA Configuration
const JIRA_URL = 'https://aurigraphdlt.atlassian.net';
const JIRA_EMAIL = process.env.JIRA_EMAIL || 'subbu@aurigraph.io';
const JIRA_API_TOKEN = process.env.JIRA_API_TOKEN || '';
const PROJECT_KEY = 'MEVS'; // MEV Shield project key

// Axios instance with auth
const jiraApi = axios.create({
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

// Sprint and Epic definitions
const sprints = [
  {
    name: 'Sprint 1: Core Infrastructure & Setup',
    startDate: '2025-01-01',
    endDate: '2025-01-14',
    goal: 'Establish development environment and core infrastructure'
  },
  {
    name: 'Sprint 2: MEV Detection Engine Core',
    startDate: '2025-01-15',
    endDate: '2025-01-28',
    goal: 'Build foundation of MEV detection system'
  },
  {
    name: 'Sprint 3: Basic Protection Mechanisms',
    startDate: '2025-01-29',
    endDate: '2025-02-11',
    goal: 'Implement core protection features'
  },
  {
    name: 'Sprint 4: MVP Dashboard',
    startDate: '2025-02-12',
    endDate: '2025-02-25',
    goal: 'Create functional user interface'
  }
];

// Ticket definitions
const tickets = {
  'Sprint 1': [
    {
      key: 'MEVS-1001',
      summary: 'Set up development environment',
      description: 'Configure Docker, Kubernetes, CI/CD pipelines',
      storyPoints: 5,
      priority: 'Highest',
      components: ['Infrastructure'],
      labels: ['devops', 'setup'],
      epicKey: 'MEVS-1000'
    },
    {
      key: 'MEVS-1002',
      summary: 'Database architecture design',
      description: 'Design PostgreSQL schema, Redis caching strategy',
      storyPoints: 8,
      priority: 'Highest',
      components: ['Backend'],
      labels: ['database', 'architecture'],
      epicKey: 'MEVS-1000'
    },
    {
      key: 'MEVS-1003',
      summary: 'Blockchain node infrastructure',
      description: 'Set up Ethereum, BSC, Polygon nodes',
      storyPoints: 13,
      priority: 'Highest',
      components: ['Blockchain'],
      labels: ['blockchain', 'infrastructure'],
      epicKey: 'MEVS-1000'
    },
    {
      key: 'MEVS-1004',
      summary: 'Authentication service setup',
      description: 'JWT auth, wallet authentication, OAuth2',
      storyPoints: 8,
      priority: 'Highest',
      components: ['Backend', 'Security'],
      labels: ['auth', 'security'],
      epicKey: 'MEVS-1000'
    },
    {
      key: 'MEVS-1005',
      summary: 'Monitoring and logging setup',
      description: 'Prometheus, Grafana, ELK stack',
      storyPoints: 5,
      priority: 'High',
      components: ['Infrastructure'],
      labels: ['monitoring', 'logging'],
      epicKey: 'MEVS-1000'
    }
  ],
  'Sprint 2': [
    {
      key: 'MEVS-2001',
      summary: 'Mempool monitoring service',
      description: 'Real-time mempool data ingestion',
      storyPoints: 13,
      priority: 'Highest',
      components: ['Blockchain'],
      labels: ['mempool', 'monitoring'],
      epicKey: 'MEVS-2000'
    },
    {
      key: 'MEVS-2002',
      summary: 'Transaction analysis module',
      description: 'Parse and analyze transaction data',
      storyPoints: 8,
      priority: 'Highest',
      components: ['Backend'],
      labels: ['analysis', 'transactions'],
      epicKey: 'MEVS-2000'
    },
    {
      key: 'MEVS-2003',
      summary: 'Front-running detection algorithm',
      description: 'Detect front-running patterns',
      storyPoints: 13,
      priority: 'Highest',
      components: ['ML', 'Detection'],
      labels: ['ml', 'detection', 'front-running'],
      epicKey: 'MEVS-2000'
    },
    {
      key: 'MEVS-2004',
      summary: 'Sandwich attack detection',
      description: 'Identify sandwich attack patterns',
      storyPoints: 13,
      priority: 'Highest',
      components: ['ML', 'Detection'],
      labels: ['ml', 'detection', 'sandwich'],
      epicKey: 'MEVS-2000'
    },
    {
      key: 'MEVS-2005',
      summary: 'Detection API endpoints',
      description: 'REST API for detection results',
      storyPoints: 5,
      priority: 'Highest',
      components: ['Backend', 'API'],
      labels: ['api', 'rest'],
      epicKey: 'MEVS-2000'
    }
  ]
};

// Epic definitions
const epics = [
  {
    key: 'MEVS-1000',
    summary: 'Infrastructure Setup',
    description: 'Set up core infrastructure and development environment'
  },
  {
    key: 'MEVS-2000',
    summary: 'Detection Engine MVP',
    description: 'Build the core MEV detection engine'
  },
  {
    key: 'MEVS-3000',
    summary: 'Protection Engine MVP',
    description: 'Implement basic protection mechanisms'
  },
  {
    key: 'MEVS-4000',
    summary: 'User Dashboard MVP',
    description: 'Create the initial user interface'
  }
];

// Priority mapping
const priorityMap = {
  'Highest': { id: '1' },
  'High': { id: '2' },
  'Medium': { id: '3' },
  'Low': { id: '4' },
  'Lowest': { id: '5' }
};

// Create Epic
async function createEpic(epic) {
  try {
    const issueData = {
      fields: {
        project: { key: PROJECT_KEY },
        summary: epic.summary,
        description: {
          type: 'doc',
          version: 1,
          content: [
            {
              type: 'paragraph',
              content: [
                {
                  type: 'text',
                  text: epic.description
                }
              ]
            }
          ]
        },
        issuetype: { name: 'Epic' },
        customfield_10011: epic.key // Epic Name field
      }
    };

    const response = await jiraApi.post('/issue', issueData);
    console.log(`✅ Created Epic: ${epic.key} - ${epic.summary}`);
    return response.data.key;
  } catch (error) {
    console.error(`❌ Failed to create Epic ${epic.key}:`, error.response?.data || error.message);
    return null;
  }
}

// Create Story
async function createStory(ticket, sprintId) {
  try {
    const issueData = {
      fields: {
        project: { key: PROJECT_KEY },
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
            },
            {
              type: 'heading',
              attrs: { level: 3 },
              content: [
                {
                  type: 'text',
                  text: 'Acceptance Criteria'
                }
              ]
            },
            {
              type: 'bulletList',
              content: [
                {
                  type: 'listItem',
                  content: [
                    {
                      type: 'paragraph',
                      content: [
                        {
                          type: 'text',
                          text: 'Feature implemented according to specifications'
                        }
                      ]
                    }
                  ]
                },
                {
                  type: 'listItem',
                  content: [
                    {
                      type: 'paragraph',
                      content: [
                        {
                          type: 'text',
                          text: 'Unit tests written with >80% coverage'
                        }
                      ]
                    }
                  ]
                },
                {
                  type: 'listItem',
                  content: [
                    {
                      type: 'paragraph',
                      content: [
                        {
                          type: 'text',
                          text: 'Documentation updated'
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        },
        issuetype: { name: 'Story' },
        priority: priorityMap[ticket.priority],
        labels: ticket.labels,
        customfield_10016: ticket.storyPoints // Story Points field
      }
    };

    // Add epic link if available
    if (ticket.epicKey) {
      issueData.fields.customfield_10014 = ticket.epicKey; // Epic Link field
    }

    const response = await jiraApi.post('/issue', issueData);
    const createdKey = response.data.key;
    console.log(`✅ Created Story: ${createdKey} - ${ticket.summary}`);

    // Add to sprint if sprintId provided
    if (sprintId) {
      await addToSprint(createdKey, sprintId);
    }

    return createdKey;
  } catch (error) {
    console.error(`❌ Failed to create Story ${ticket.key}:`, error.response?.data || error.message);
    return null;
  }
}

// Create Sprint
async function createSprint(sprint) {
  try {
    // Note: Creating sprints requires Jira Software API
    // This would need the board ID which we need to fetch first
    console.log(`ℹ️ Sprint creation requires board ID. Please create sprint manually: ${sprint.name}`);
    return null;
  } catch (error) {
    console.error(`❌ Failed to create Sprint:`, error.message);
    return null;
  }
}

// Add issue to sprint
async function addToSprint(issueKey, sprintId) {
  try {
    // This would require the Jira Software API
    console.log(`ℹ️ Adding ${issueKey} to sprint ${sprintId} (manual action required)`);
  } catch (error) {
    console.error(`❌ Failed to add to sprint:`, error.message);
  }
}

// Main execution
async function main() {
  console.log('🚀 MEV Shield JIRA Ticket Creation Script');
  console.log('==========================================\n');

  if (!JIRA_API_TOKEN) {
    console.error('❌ Error: JIRA_API_TOKEN environment variable not set');
    console.log('Please set your JIRA API token:');
    console.log('export JIRA_API_TOKEN="your-token-here"');
    process.exit(1);
  }

  // Create Epics
  console.log('📋 Creating Epics...\n');
  for (const epic of epics) {
    await createEpic(epic);
  }

  console.log('\n📝 Creating Stories...\n');

  // Create Stories for each sprint
  for (const sprintName of Object.keys(tickets)) {
    console.log(`\n--- ${sprintName} ---`);
    const sprintTickets = tickets[sprintName];
    
    for (const ticket of sprintTickets) {
      await createStory(ticket, null);
      // Add delay to avoid rate limiting
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }

  console.log('\n✅ Ticket creation complete!');
  console.log('\n📊 Summary:');
  console.log(`- Epics: ${epics.length}`);
  console.log(`- Stories: ${Object.values(tickets).flat().length}`);
  console.log('\n🔗 View in JIRA: ' + JIRA_URL + '/browse/' + PROJECT_KEY);
}

// Check if running directly
if (require.main === module) {
  main().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { createEpic, createStory, tickets, epics };