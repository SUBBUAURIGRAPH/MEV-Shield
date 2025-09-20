import React from 'react';
import {
  Box,
  Container,
  Typography,
  Grid,
  Card,
  CardContent,
  Paper,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Chip,
  Alert,
  Button,
  LinearProgress,
} from '@mui/material';
import {
  Security,
  Shield,
  Lock,
  VerifiedUser,
  Warning,
  CheckCircle,
  Block,
  VpnKey,
  AdminPanelSettings,
} from '@mui/icons-material';

const SecurityPage: React.FC = () => {
  const securityMetrics = {
    activeThreats: 0,
    blockedAttempts: 247,
    securityScore: 98,
    lastScan: new Date().toLocaleString(),
  };

  return (
    <Container maxWidth="xl" sx={{ py: 3 }}>
      <Box sx={{ mb: 3 }}>
        <Typography variant="h4" gutterBottom sx={{ display: 'flex', alignItems: 'center' }}>
          <Security sx={{ mr: 2, fontSize: 40 }} />
          Security Center
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Monitor and manage platform security
        </Typography>
      </Box>

      <Grid container spacing={3}>
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Shield color="success" sx={{ mr: 1 }} />
                <Typography variant="h6">Security Score</Typography>
              </Box>
              <Typography variant="h3" color="success.main">
                {securityMetrics.securityScore}%
              </Typography>
              <LinearProgress
                variant="determinate"
                value={securityMetrics.securityScore}
                color="success"
                sx={{ mt: 2, height: 8, borderRadius: 4 }}
              />
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Warning color="error" sx={{ mr: 1 }} />
                <Typography variant="h6">Active Threats</Typography>
              </Box>
              <Typography variant="h3">{securityMetrics.activeThreats}</Typography>
              <Chip label="All Clear" color="success" size="small" sx={{ mt: 1 }} />
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Block color="warning" sx={{ mr: 1 }} />
                <Typography variant="h6">Blocked Attempts</Typography>
              </Box>
              <Typography variant="h3">{securityMetrics.blockedAttempts}</Typography>
              <Typography variant="caption" color="text.secondary">
                Last 30 days
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <VerifiedUser color="info" sx={{ mr: 1 }} />
                <Typography variant="h6">Last Scan</Typography>
              </Box>
              <Typography variant="body2">{securityMetrics.lastScan}</Typography>
              <Button size="small" sx={{ mt: 1 }}>Run Scan</Button>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Security Features
            </Typography>
            <List>
              <ListItem>
                <ListItemIcon>
                  <Lock color="primary" />
                </ListItemIcon>
                <ListItemText
                  primary="Two-Factor Authentication"
                  secondary="Enabled for all admin accounts"
                />
                <Chip label="Active" color="success" size="small" />
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <VpnKey color="primary" />
                </ListItemIcon>
                <ListItemText
                  primary="End-to-End Encryption"
                  secondary="All transactions are encrypted"
                />
                <Chip label="Active" color="success" size="small" />
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <AdminPanelSettings color="primary" />
                </ListItemIcon>
                <ListItemText
                  primary="Role-Based Access Control"
                  secondary="Granular permissions system"
                />
                <Chip label="Active" color="success" size="small" />
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <Shield color="primary" />
                </ListItemIcon>
                <ListItemText
                  primary="MEV Protection"
                  secondary="Advanced protection against MEV attacks"
                />
                <Chip label="Active" color="success" size="small" />
              </ListItem>
            </List>
          </Paper>
        </Grid>

        <Grid item xs={12}>
          <Alert severity="info">
            <Typography variant="body2">
              <strong>Security Tip:</strong> Regular security audits and monitoring help maintain the highest level of protection for your platform.
            </Typography>
          </Alert>
        </Grid>
      </Grid>
    </Container>
  );
};

export default SecurityPage;