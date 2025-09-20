#!/usr/bin/env node

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

// Simplified epic list
const epics = [
  'MEV Shield - Core Infrastructure & Architecture',
  'MEV Shield - Blockchain Integration Layer',
  'MEV Shield - MEV Detection Engine',
  'MEV Shield - Protection & Mitigation System',
  'MEV Shield - User Experience & Dashboard',
  'MEV Shield - API & Developer Platform',
  'MEV Shield - Analytics & Reporting',
  'MEV Shield - Security & Compliance',
  'MEV Shield - Performance & Scalability',
  'MEV Shield - Launch & Operations'
];

async function createEpics() {
  console.log('🚀 Creating MEV Shield Epics\n');
  
  let created = 0;
  let failed = 0;
  
  for (const epicName of epics) {
    try {
      const issueData = {
        fields: {
          project: { key: 'MEV' },
          summary: epicName,
          description: {
            type: 'doc',
            version: 1,
            content: [
              {
                type: 'paragraph',
                content: [
                  {
                    type: 'text',
                    text: `Epic for: ${epicName}`
                  }
                ]
              }
            ]
          },
          issuetype: { name: 'Epic' },
          labels: ['mev-shield', 'epic']
        }
      };
      
      const response = await jira.post('/issue', issueData);
      console.log(`✅ Created: ${response.data.key} - ${epicName}`);
      created++;
      
    } catch (error) {
      // Try as Task if Epic fails
      try {
        const issueData = {
          fields: {
            project: { key: 'MEV' },
            summary: epicName,
            description: {
              type: 'doc',
              version: 1,
              content: [
                {
                  type: 'paragraph',
                  content: [
                    {
                      type: 'text',
                      text: `Epic for: ${epicName}`
                    }
                  ]
                }
              ]
            },
            issuetype: { name: 'Task' },
            labels: ['mev-shield', 'epic']
          }
        };
        
        const response = await jira.post('/issue', issueData);
        console.log(`✅ Created as Task: ${response.data.key} - ${epicName}`);
        created++;
        
      } catch (error2) {
        console.error(`❌ Failed: ${epicName}`);
        if (error2.response) {
          console.error(`   Error: ${JSON.stringify(error2.response.data)}`);
        }
        failed++;
      }
    }
    
    // Rate limiting
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  console.log(`\n✅ Complete!`);
  console.log(`   Created: ${created} epics`);
  console.log(`   Failed: ${failed} epics`);
  console.log(`\n🔗 View in JIRA: ${JIRA_URL}/projects/MEV`);
}

// Run
createEpics().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});