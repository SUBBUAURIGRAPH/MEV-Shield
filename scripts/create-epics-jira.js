#!/usr/bin/env node

/**
 * MEV Shield - Create JIRA Epics with Logical Structure
 */

const axios = require('axios');

// JIRA Configuration from CLAUDE.md
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

// Epic Structure
const epics = [
  {
    name: 'MEVS-1000: Core Infrastructure & Architecture',
    summary: 'Core Infrastructure & Architecture',
    description: `Establish robust, scalable infrastructure foundation for MEV Shield platform.

**Key Results:**
• Development environment operational
• CI/CD pipeline automated
• Database architecture optimized
• Monitoring & alerting active

**Success Metrics:**
• Infrastructure uptime: 99.9%
• Deploy time: < 5 minutes
• Database query performance: < 100ms p95
• Cache hit rate: > 80%`,
    priority: 'Highest',
    labels: ['mev-shield', 'infrastructure', 'phase-1'],
    color: 'purple'
  },
  {
    name: 'MEVS-2000: Blockchain Integration Layer',
    summary: 'Blockchain Integration Layer',
    description: `Build multi-chain blockchain connectivity and data ingestion layer.

**Key Results:**
• Ethereum mainnet connected
• Multi-chain support (4+ chains)
• Real-time mempool access
• Historical data indexing

**Success Metrics:**
• Block processing latency: < 100ms
• Mempool coverage: > 95%
• Chain sync time: < 1 hour
• Data availability: 99.99%`,
    priority: 'Highest',
    labels: ['mev-shield', 'blockchain', 'phase-1'],
    color: 'blue'
  },
  {
    name: 'MEVS-3000: MEV Detection Engine',
    summary: 'MEV Detection Engine',
    description: `Develop ML-powered MEV detection system with high accuracy and low latency.

**Key Results:**
• Front-running detection operational
• Sandwich attack detection accurate
• Novel MEV pattern recognition
• Real-time alert system

**Success Metrics:**
• Detection accuracy: > 95%
• False positive rate: < 5%
• Detection latency: < 100ms
• Pattern coverage: 10+ types`,
    priority: 'Highest',
    labels: ['mev-shield', 'detection', 'ml', 'phase-1'],
    color: 'red'
  },
  {
    name: 'MEVS-4000: Protection & Mitigation System',
    summary: 'Protection & Mitigation System',
    description: `Implement comprehensive MEV protection mechanisms across multiple strategies.

**Key Results:**
• Private mempool operational
• Flashbots integration complete
• Smart routing engine active
• Protection success rate > 90%

**Success Metrics:**
• Protection success rate: > 90%
• Transaction inclusion time: < 2 blocks
• Gas savings: > 20%
• MEV captured: > $1M/month`,
    priority: 'Highest',
    labels: ['mev-shield', 'protection', 'phase-2'],
    color: 'green'
  },
  {
    name: 'MEVS-5000: User Experience & Dashboard',
    summary: 'User Experience & Dashboard',
    description: `Create intuitive, powerful dashboard for all user types with real-time insights.

**Key Results:**
• Web dashboard launched
• Mobile apps available
• Wallet integrations complete
• User satisfaction > 4.5/5

**Success Metrics:**
• Page load time: < 2 seconds
• User retention: > 60% (30-day)
• Daily active users: > 1,000
• Mobile app rating: > 4.5 stars`,
    priority: 'Highest',
    labels: ['mev-shield', 'frontend', 'ux', 'phase-2'],
    color: 'yellow'
  },
  {
    name: 'MEVS-6000: API & Developer Platform',
    summary: 'API & Developer Platform',
    description: `Build comprehensive API platform enabling third-party integrations.

**Key Results:**
• REST API v2 launched
• WebSocket API operational
• SDKs in 3+ languages
• Developer adoption > 100 apps

**Success Metrics:**
• API response time: < 200ms p95
• API uptime: > 99.9%
• SDK downloads: > 1,000/month
• Active developers: > 100`,
    priority: 'High',
    labels: ['mev-shield', 'api', 'developer', 'phase-2'],
    color: 'orange'
  },
  {
    name: 'MEVS-7000: Analytics & Reporting',
    summary: 'Analytics & Reporting',
    description: `Deliver comprehensive analytics platform with actionable insights.

**Key Results:**
• Real-time analytics dashboard
• Historical MEV analysis
• Custom report builder
• ML predictions active

**Success Metrics:**
• Report generation time: < 5 seconds
• Data accuracy: > 99%
• Prediction accuracy: > 80%
• User engagement: > 30 min/session`,
    priority: 'High',
    labels: ['mev-shield', 'analytics', 'phase-3'],
    color: 'teal'
  },
  {
    name: 'MEVS-8000: Security & Compliance',
    summary: 'Security & Compliance',
    description: `Ensure platform security, regulatory compliance, and user data protection.

**Key Results:**
• Security audit passed
• SOC2 compliance achieved
• GDPR compliant
• Zero security breaches

**Success Metrics:**
• Security score: A+ rating
• Compliance audits passed: 100%
• Incident response time: < 15 minutes
• Data breach count: 0`,
    priority: 'Highest',
    labels: ['mev-shield', 'security', 'compliance', 'phase-3'],
    color: 'dark_red'
  },
  {
    name: 'MEVS-9000: Performance & Scalability',
    summary: 'Performance & Scalability',
    description: `Optimize platform for high performance and horizontal scalability.

**Key Results:**
• 100K+ concurrent users supported
• Sub-second response times
• Auto-scaling operational
• 99.99% uptime achieved

**Success Metrics:**
• API response time: < 100ms p50
• Page load time: < 1 second
• Concurrent users: > 100K
• Infrastructure cost: < $0.10/user`,
    priority: 'High',
    labels: ['mev-shield', 'performance', 'phase-3'],
    color: 'dark_blue'
  },
  {
    name: 'MEVS-10000: Launch & Operations',
    summary: 'Launch & Operations',
    description: `Successfully launch MEV Shield to production with operational excellence.

**Key Results:**
• Production launch successful
• 10K+ users onboarded
• 24/7 support operational
• Market leader position

**Success Metrics:**
• Launch day users: > 1,000
• Month 1 users: > 10,000
• User satisfaction: > 4.5/5
• Revenue: > $50K MRR`,
    priority: 'Highest',
    labels: ['mev-shield', 'launch', 'phase-4'],
    color: 'dark_green'
  }
];

// Stories mapped to epics
const epicStories = {
  'MEVS-1000': [
    { summary: 'Set up Docker/K8s environment', points: 8 },
    { summary: 'Design PostgreSQL schema for MEV data', points: 13 },
    { summary: 'Configure CI/CD pipelines', points: 8 },
    { summary: 'Set up Prometheus/Grafana monitoring', points: 5 },
    { summary: 'Implement Redis caching layer', points: 5 },
    { summary: 'Create backup and disaster recovery', points: 8 },
    { summary: 'Implement secrets management', points: 5 }
  ],
  'MEVS-2000': [
    { summary: 'Connect Ethereum mainnet nodes', points: 13 },
    { summary: 'Integrate Polygon network', points: 8 },
    { summary: 'Integrate BSC network', points: 8 },
    { summary: 'Add Arbitrum/Optimism L2 support', points: 13 },
    { summary: 'Build mempool streaming service', points: 13 },
    { summary: 'Create blockchain data indexer', points: 8 },
    { summary: 'Implement cross-chain tracking', points: 8 }
  ],
  'MEVS-3000': [
    { summary: 'Implement front-running detection', points: 21 },
    { summary: 'Build sandwich attack recognition', points: 21 },
    { summary: 'Detect JIT liquidity attacks', points: 13 },
    { summary: 'Create ML model training pipeline', points: 13 },
    { summary: 'Build real-time MEV alerts', points: 8 },
    { summary: 'Implement MEV impact calculator', points: 8 },
    { summary: 'Add anomaly detection system', points: 13 }
  ],
  'MEVS-4000': [
    { summary: 'Build private transaction pool', points: 21 },
    { summary: 'Integrate Flashbots Protect', points: 13 },
    { summary: 'Create smart order routing', points: 21 },
    { summary: 'Implement transaction bundling', points: 13 },
    { summary: 'Add slippage protection', points: 8 },
    { summary: 'Build gas optimization engine', points: 8 },
    { summary: 'Create MEV redistribution system', points: 13 }
  ],
  'MEVS-5000': [
    { summary: 'Build responsive web dashboard', points: 21 },
    { summary: 'Integrate MetaMask/WalletConnect', points: 13 },
    { summary: 'Create portfolio overview', points: 13 },
    { summary: 'Build transaction history view', points: 8 },
    { summary: 'Develop iOS mobile app', points: 21 },
    { summary: 'Develop Android mobile app', points: 21 },
    { summary: 'Implement alerts system', points: 8 }
  ]
};

// Create Epic with stories
async function createEpic(epic) {
  try {
    // Get project - use MEV Shield project
    const projectsResponse = await jira.get('/project');
    const project = projectsResponse.data.find(p => 
      p.key === 'MEV' || p.name === 'MEV Shield'
    );
    
    if (!project) {
      throw new Error('MEV Shield project not found in JIRA');
    }
    
    const projectKey = project.key;
    console.log(`Using project: ${projectKey}`);
    
    // Get issue types
    const projectDetails = await jira.get(`/project/${projectKey}`);
    const epicType = projectDetails.data.issueTypes.find(t => 
      t.name.toLowerCase() === 'epic'
    );
    
    if (!epicType) {
      console.log('⚠️  Epic issue type not found, creating as Task');
    }
    
    // Create epic
    const epicData = {
      fields: {
        project: { key: projectKey },
        summary: `[MEV Shield] ${epic.summary}`,
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
        issuetype: { 
          id: epicType ? epicType.id : projectDetails.data.issueTypes[0].id 
        },
        labels: epic.labels
      }
    };
    
    // Remove priority field as it's not allowed in MEV project
    // Priority can be set manually after creation
    
    // Create the epic
    const response = await jira.post('/issue', epicData);
    const epicKey = response.data.key;
    console.log(`✅ Created Epic: ${epicKey} - ${epic.summary}`);
    
    // Create stories for this epic
    const stories = epicStories[epic.name.split(':')[0]];
    if (stories) {
      console.log(`   Creating ${stories.length} stories for ${epicKey}...`);
      for (const story of stories) {
        await createStory(story, epicKey, projectKey);
        await new Promise(resolve => setTimeout(resolve, 500)); // Rate limiting
      }
    }
    
    return epicKey;
    
  } catch (error) {
    console.error(`❌ Failed to create Epic: ${epic.summary}`);
    if (error.response) {
      console.error(`   Error: ${JSON.stringify(error.response.data)}`);
    } else {
      console.error(`   Error: ${error.message}`);
    }
    return null;
  }
}

// Create Story linked to Epic
async function createStory(story, epicKey, projectKey) {
  try {
    const storyData = {
      fields: {
        project: { key: projectKey },
        summary: story.summary,
        description: {
          type: 'doc',
          version: 1,
          content: [
            {
              type: 'paragraph',
              content: [
                {
                  type: 'text',
                  text: `Story Points: ${story.points}\nPart of Epic: ${epicKey}`
                }
              ]
            }
          ]
        },
        issuetype: { name: 'Task' },
        labels: ['mev-shield', 'story']
      }
    };
    
    const response = await jira.post('/issue', storyData);
    console.log(`   ✅ Story: ${response.data.key} - ${story.summary}`);
    return response.data.key;
    
  } catch (error) {
    console.error(`   ❌ Failed to create story: ${story.summary}`);
    return null;
  }
}

// Main execution
async function main() {
  console.log('🚀 Creating MEV Shield Epics in JIRA\n');
  
  if (!JIRA_API_TOKEN) {
    console.error('❌ JIRA_API_TOKEN not set!');
    console.log('\nGet your token at:');
    console.log('https://id.atlassian.com/manage-profile/security/api-tokens');
    console.log('\nThen run:');
    console.log('export JIRA_API_TOKEN="your-token-here"');
    process.exit(1);
  }
  
  console.log(`📋 Creating ${epics.length} Epics with stories...\n`);
  
  let epicCount = 0;
  let storyCount = 0;
  
  for (const epic of epics) {
    const epicKey = await createEpic(epic);
    if (epicKey) {
      epicCount++;
      const stories = epicStories[epic.name.split(':')[0]];
      if (stories) {
        storyCount += stories.length;
      }
    }
    console.log(''); // Blank line between epics
  }
  
  console.log('\n✅ Complete!');
  console.log(`   Created ${epicCount} Epics`);
  console.log(`   Created ${storyCount} Stories`);
  console.log(`\n🔗 View in JIRA: ${JIRA_URL}/projects`);
}

if (require.main === module) {
  main().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = { createEpic, epics, epicStories };