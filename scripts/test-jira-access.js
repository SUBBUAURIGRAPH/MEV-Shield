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

async function testJiraAccess() {
  console.log('🔍 Testing JIRA Access...\n');
  
  if (!JIRA_API_TOKEN) {
    console.error('❌ JIRA_API_TOKEN not set!');
    console.log('\nTo create a JIRA API token:');
    console.log('1. Go to: https://id.atlassian.com/manage-profile/security/api-tokens');
    console.log('2. Click "Create API token"');
    console.log('3. Give it a name like "MEV Shield Integration"');
    console.log('4. Copy the token');
    console.log('5. Run: export JIRA_API_TOKEN="your-token-here"');
    console.log('6. Run this script again\n');
    process.exit(1);
  }
  
  try {
    // Test 1: Get current user
    console.log('1. Testing authentication...');
    const userResponse = await jira.get('/myself');
    console.log(`   ✅ Authenticated as: ${userResponse.data.emailAddress}`);
    console.log(`   Display Name: ${userResponse.data.displayName}`);
    
    // Test 2: Get projects
    console.log('\n2. Fetching available projects...');
    const projectsResponse = await jira.get('/project');
    console.log(`   ✅ Found ${projectsResponse.data.length} projects:`);
    
    projectsResponse.data.forEach((project, index) => {
      console.log(`   ${index + 1}. ${project.key} - ${project.name}`);
      console.log(`      ID: ${project.id}`);
      console.log(`      Type: ${project.projectTypeKey}`);
    });
    
    if (projectsResponse.data.length === 0) {
      console.log('\n⚠️  No projects found!');
      console.log('   You need to create a project first in JIRA');
      console.log('   Go to: https://aurigraphdlt.atlassian.net/projects');
      return;
    }
    
    // Test 3: Check issue types for first project
    const project = projectsResponse.data[0];
    console.log(`\n3. Checking issue types for project ${project.key}...`);
    
    const projectDetails = await jira.get(`/project/${project.key}`);
    const issueTypes = projectDetails.data.issueTypes || [];
    
    console.log(`   ✅ Available issue types:`);
    issueTypes.forEach(type => {
      console.log(`   - ${type.name} (ID: ${type.id})`);
    });
    
    // Test 4: Check permissions
    console.log('\n4. Checking permissions...');
    const permissions = await jira.get(`/mypermissions?projectKey=${project.key}`);
    const canCreateIssues = permissions.data.permissions.CREATE_ISSUES?.havePermission;
    const canCreateEpics = permissions.data.permissions.CREATE_EPIC?.havePermission;
    
    console.log(`   Can create issues: ${canCreateIssues ? '✅' : '❌'}`);
    console.log(`   Can create epics: ${canCreateEpics ? '✅' : '❌'}`);
    
    // Summary
    console.log('\n' + '='.repeat(50));
    console.log('✅ JIRA Connection Successful!');
    console.log('='.repeat(50));
    console.log('\nRecommended next steps:');
    console.log(`1. Use project: ${project.key} (${project.name})`);
    console.log('2. Create epics as "Task" or "Story" type if Epic type unavailable');
    console.log('3. Run the epic creation script with this project key');
    
    // Create environment variable suggestions
    console.log('\n📝 Set these environment variables:');
    console.log(`export JIRA_PROJECT_KEY="${project.key}"`);
    console.log(`export JIRA_API_TOKEN="${JIRA_API_TOKEN}"`);
    
  } catch (error) {
    console.error('\n❌ JIRA Connection Failed!');
    
    if (error.response) {
      console.error(`Status: ${error.response.status}`);
      console.error(`Error: ${JSON.stringify(error.response.data)}`);
      
      if (error.response.status === 401) {
        console.log('\n⚠️  Authentication failed!');
        console.log('   Check that your API token is correct');
        console.log('   You may need to create a new token');
      } else if (error.response.status === 403) {
        console.log('\n⚠️  Permission denied!');
        console.log('   You may not have access to this JIRA instance');
      }
    } else {
      console.error(`Error: ${error.message}`);
    }
    
    process.exit(1);
  }
}

// Run the test
testJiraAccess().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});