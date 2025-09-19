// JIRA Webhook Handler for Real-time Sync
// This script can be deployed as a serverless function or Express endpoint
// to handle JIRA webhooks and sync to GitHub

const https = require('https');
const crypto = require('crypto');

// Configuration (set these as environment variables)
const config = {
    GITHUB_TOKEN: process.env.GITHUB_TOKEN,
    GITHUB_OWNER: process.env.GITHUB_OWNER || 'your-org',
    GITHUB_REPO: process.env.GITHUB_REPO || 'mev-shield',
    JIRA_WEBHOOK_SECRET: process.env.JIRA_WEBHOOK_SECRET || 'your-webhook-secret',
    JIRA_BASE_URL: process.env.JIRA_BASE_URL || 'https://aurigraphdlt.atlassian.net'
};

// Status mapping from JIRA to GitHub
const statusMap = {
    'To Do': 'open',
    'In Progress': 'open',
    'In Review': 'open',
    'Done': 'closed',
    'Resolved': 'closed',
    'Closed': 'closed'
};

/**
 * Verify JIRA webhook signature
 */
function verifyWebhookSignature(payload, signature) {
    const hmac = crypto.createHmac('sha256', config.JIRA_WEBHOOK_SECRET);
    hmac.update(JSON.stringify(payload));
    const calculatedSignature = hmac.digest('hex');
    return calculatedSignature === signature;
}

/**
 * Create or update GitHub issue from JIRA ticket
 */
async function syncToGitHub(jiraIssue, event) {
    const jiraKey = jiraIssue.key;
    const summary = jiraIssue.fields.summary;
    const description = jiraIssue.fields.description || 'No description';
    const status = jiraIssue.fields.status.name;
    const priority = jiraIssue.fields.priority?.name || 'Medium';
    const assignee = jiraIssue.fields.assignee?.displayName || 'Unassigned';
    
    // Check if GitHub issue already exists
    const existingIssue = await findGitHubIssue(jiraKey);
    
    const issueData = {
        title: `[${jiraKey}] ${summary}`,
        body: `## JIRA Ticket: [${jiraKey}](${config.JIRA_BASE_URL}/browse/${jiraKey})

**Status:** ${status}
**Priority:** ${priority}
**Assignee:** ${assignee}
**Event:** ${event}

---

### Description
${description}

---
*Synced from JIRA webhook at ${new Date().toISOString()}*`,
        labels: ['jira-sync', jiraKey, `priority-${priority.toLowerCase()}`],
        state: statusMap[status] || 'open'
    };
    
    if (existingIssue) {
        // Update existing issue
        await updateGitHubIssue(existingIssue.number, issueData);
        console.log(`Updated GitHub issue #${existingIssue.number} for ${jiraKey}`);
    } else {
        // Create new issue
        const newIssue = await createGitHubIssue(issueData);
        console.log(`Created GitHub issue #${newIssue.number} for ${jiraKey}`);
        
        // Add comment back to JIRA with GitHub link
        await addJiraComment(jiraKey, `Created GitHub issue: ${newIssue.html_url}`);
    }
}

/**
 * Find GitHub issue by JIRA key
 */
async function findGitHubIssue(jiraKey) {
    return new Promise((resolve, reject) => {
        const options = {
            hostname: 'api.github.com',
            path: `/repos/${config.GITHUB_OWNER}/${config.GITHUB_REPO}/issues?state=all&labels=${jiraKey}`,
            method: 'GET',
            headers: {
                'Authorization': `token ${config.GITHUB_TOKEN}`,
                'Accept': 'application/vnd.github.v3+json',
                'User-Agent': 'JIRA-Webhook-Handler'
            }
        };
        
        https.request(options, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => {
                const issues = JSON.parse(data);
                resolve(issues.length > 0 ? issues[0] : null);
            });
        }).on('error', reject).end();
    });
}

/**
 * Create GitHub issue
 */
async function createGitHubIssue(issueData) {
    return new Promise((resolve, reject) => {
        const postData = JSON.stringify(issueData);
        
        const options = {
            hostname: 'api.github.com',
            path: `/repos/${config.GITHUB_OWNER}/${config.GITHUB_REPO}/issues`,
            method: 'POST',
            headers: {
                'Authorization': `token ${config.GITHUB_TOKEN}`,
                'Accept': 'application/vnd.github.v3+json',
                'Content-Type': 'application/json',
                'Content-Length': postData.length,
                'User-Agent': 'JIRA-Webhook-Handler'
            }
        };
        
        const req = https.request(options, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => resolve(JSON.parse(data)));
        });
        
        req.on('error', reject);
        req.write(postData);
        req.end();
    });
}

/**
 * Update GitHub issue
 */
async function updateGitHubIssue(issueNumber, issueData) {
    return new Promise((resolve, reject) => {
        const postData = JSON.stringify(issueData);
        
        const options = {
            hostname: 'api.github.com',
            path: `/repos/${config.GITHUB_OWNER}/${config.GITHUB_REPO}/issues/${issueNumber}`,
            method: 'PATCH',
            headers: {
                'Authorization': `token ${config.GITHUB_TOKEN}`,
                'Accept': 'application/vnd.github.v3+json',
                'Content-Type': 'application/json',
                'Content-Length': postData.length,
                'User-Agent': 'JIRA-Webhook-Handler'
            }
        };
        
        const req = https.request(options, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => resolve(JSON.parse(data)));
        });
        
        req.on('error', reject);
        req.write(postData);
        req.end();
    });
}

/**
 * Add comment to JIRA issue
 */
async function addJiraComment(jiraKey, comment) {
    // This would require JIRA API credentials
    // Implementation depends on your JIRA setup
    console.log(`Would add comment to ${jiraKey}: ${comment}`);
}

/**
 * Main webhook handler
 */
async function handleWebhook(payload, signature) {
    // Verify signature if configured
    if (config.JIRA_WEBHOOK_SECRET && !verifyWebhookSignature(payload, signature)) {
        throw new Error('Invalid webhook signature');
    }
    
    const { webhookEvent, issue, changelog } = payload;
    
    console.log(`Received JIRA webhook: ${webhookEvent}`);
    
    // Handle different webhook events
    switch (webhookEvent) {
        case 'jira:issue_created':
            await syncToGitHub(issue, 'created');
            break;
            
        case 'jira:issue_updated':
            // Check what was updated
            if (changelog) {
                const hasStatusChange = changelog.items.some(item => item.field === 'status');
                const hasDescriptionChange = changelog.items.some(item => item.field === 'description');
                const hasSummaryChange = changelog.items.some(item => item.field === 'summary');
                
                if (hasStatusChange || hasDescriptionChange || hasSummaryChange) {
                    await syncToGitHub(issue, 'updated');
                }
            }
            break;
            
        case 'jira:issue_deleted':
            // Optionally close GitHub issue
            const existingIssue = await findGitHubIssue(issue.key);
            if (existingIssue) {
                await updateGitHubIssue(existingIssue.number, {
                    state: 'closed',
                    body: existingIssue.body + '\n\n---\n⚠️ JIRA ticket was deleted'
                });
            }
            break;
            
        default:
            console.log(`Unhandled webhook event: ${webhookEvent}`);
    }
}

// Express.js endpoint example
if (require.main === module) {
    const express = require('express');
    const app = express();
    
    app.use(express.json());
    
    app.post('/webhook/jira', async (req, res) => {
        try {
            const signature = req.headers['x-hub-signature-256'];
            await handleWebhook(req.body, signature);
            res.status(200).json({ success: true });
        } catch (error) {
            console.error('Webhook error:', error);
            res.status(500).json({ error: error.message });
        }
    });
    
    const PORT = process.env.PORT || 3000;
    app.listen(PORT, () => {
        console.log(`JIRA webhook handler listening on port ${PORT}`);
    });
}

module.exports = { handleWebhook };