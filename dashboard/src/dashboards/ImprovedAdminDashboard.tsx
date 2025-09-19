import React, { useState } from 'react';
import { styled } from '@mui/material/styles';
import {
  Box,
  Container,
  Grid,
  Paper,
  Typography,
  Card,
  CardContent,
  IconButton,
  Button,
  Tabs,
  Tab,
  Chip,
  Avatar,
  List,
  ListItem,
  ListItemText,
  ListItemAvatar,
  Divider,
  LinearProgress,
  Badge,
  Tooltip,
} from '@mui/material';
import {
  Dashboard,
  Shield,
  TrendingUp,
  Speed,
  Security,
  Group,
  AttachMoney,
  Warning,
  CheckCircle,
  Refresh,
  Notifications,
  Settings,
  BarChart,
  Timeline,
  PieChart,
  ShowChart,
  Business,
  SwapHoriz,
} from '@mui/icons-material';
import {
  MEVActivityChart,
  ValueProtectedChart,
  MEVDistributionChart,
  NetworkPerformanceChart,
  ExchangeVolumeChart,
} from '../components/charts/ImprovedCharts';
import { ExchangeIntegration } from '../components/ExchangeIntegration';
import DexIntegration from '../components/DexIntegration';

// Styled component for spinning animation
const SpinningIcon = styled(Refresh)`
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
  
  &.spinning {
    animation: spin 1s linear infinite;
  }
`;

// Metric Card Component
const MetricCard: React.FC<{
  title: string;
  value: string | number;
  change?: string;
  icon: React.ReactNode;
  color?: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info';
}> = ({ title, value, change, icon, color = 'primary' }) => (
  <Card sx={{ height: '100%', position: 'relative', overflow: 'visible' }}>
    <CardContent>
      <Box sx={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between' }}>
        <Box>
          <Typography color="text.secondary" gutterBottom variant="body2">
            {title}
          </Typography>
          <Typography variant="h4" component="div" sx={{ fontWeight: 600, mb: 1 }}>
            {value}
          </Typography>
          {change && (
            <Chip
              label={change}
              size="small"
              color={change.includes('+') ? 'success' : 'error'}
              sx={{ fontWeight: 500 }}
            />
          )}
        </Box>
        <Avatar
          sx={{
            bgcolor: `${color}.light`,
            color: `${color}.main`,
            width: 56,
            height: 56,
          }}
        >
          {icon}
        </Avatar>
      </Box>
    </CardContent>
    <LinearProgress
      variant="determinate"
      value={Math.random() * 100}
      sx={{
        position: 'absolute',
        bottom: 0,
        left: 0,
        right: 0,
        height: 3,
        bgcolor: 'grey.200',
        '& .MuiLinearProgress-bar': {
          bgcolor: `${color}.main`,
        },
      }}
    />
  </Card>
);

const ImprovedAdminDashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);
  const [refreshing, setRefreshing] = useState(false);

  const handleRefresh = () => {
    setRefreshing(true);
    setTimeout(() => setRefreshing(false), 2000);
  };

  const recentAlerts = [
    {
      id: 1,
      type: 'sandwich',
      severity: 'high',
      message: 'Sandwich attack prevented on USDC/ETH pair',
      time: '2 min ago',
      saved: '$4,250',
    },
    {
      id: 2,
      type: 'frontrun',
      severity: 'medium',
      message: 'Front-running attempt blocked',
      time: '8 min ago',
      saved: '$1,820',
    },
    {
      id: 3,
      type: 'arbitrage',
      severity: 'low',
      message: 'Arbitrage bot activity detected',
      time: '15 min ago',
      saved: '$890',
    },
  ];

  const connectedValidators = [
    { name: 'Validator Node #1', status: 'active', stake: '32 ETH', performance: 98.5 },
    { name: 'Validator Node #2', status: 'active', stake: '32 ETH', performance: 97.8 },
    { name: 'Validator Node #3', status: 'syncing', stake: '32 ETH', performance: 0 },
    { name: 'Validator Node #4', status: 'active', stake: '32 ETH', performance: 99.1 },
  ];

  return (
    <Box sx={{ bgcolor: 'background.default', minHeight: '100vh', py: 3 }}>
      <Container maxWidth="xl">
        {/* Header */}
        <Box sx={{ mb: 4 }}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
            <Typography variant="h4" sx={{ fontWeight: 700, display: 'flex', alignItems: 'center' }}>
              <Shield sx={{ mr: 2, fontSize: 40, color: 'primary.main' }} />
              MEV Shield Admin Dashboard
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <Tooltip title="Refresh Data">
                <IconButton onClick={handleRefresh} color="primary">
                  <SpinningIcon className={refreshing ? 'spinning' : ''} />
                </IconButton>
              </Tooltip>
              <IconButton color="default">
                <Badge badgeContent={3} color="error">
                  <Notifications />
                </Badge>
              </IconButton>
              <IconButton color="default">
                <Settings />
              </IconButton>
            </Box>
          </Box>
          
          <Tabs value={activeTab} onChange={(_, v) => setActiveTab(v)} sx={{ borderBottom: 1, borderColor: 'divider' }}>
            <Tab icon={<Dashboard />} label="Overview" iconPosition="start" />
            <Tab icon={<Timeline />} label="Analytics" iconPosition="start" />
            <Tab icon={<SwapHoriz />} label="DEX Protection" iconPosition="start" />
            <Tab icon={<Business />} label="Exchanges" iconPosition="start" />
            <Tab icon={<Security />} label="Security" iconPosition="start" />
            <Tab icon={<Group />} label="Validators" iconPosition="start" />
          </Tabs>
        </Box>

        {/* Overview Tab */}
        {activeTab === 0 && (
          <Grid container spacing={3}>
            {/* Metrics Cards */}
            <Grid item xs={12} sm={6} md={3}>
              <MetricCard
                title="Total Value Protected"
                value="$124.5M"
                change="+12.3%"
                icon={<AttachMoney />}
                color="success"
              />
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <MetricCard
                title="MEV Attacks Blocked"
                value="8,423"
                change="+5.2%"
                icon={<Shield />}
                color="primary"
              />
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <MetricCard
                title="Active Users"
                value="15.2K"
                change="+18.7%"
                icon={<Group />}
                color="info"
              />
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <MetricCard
                title="Avg Response Time"
                value="42ms"
                change="-8.3%"
                icon={<Speed />}
                color="warning"
              />
            </Grid>

            {/* Charts */}
            <Grid item xs={12} lg={8}>
              <MEVActivityChart height={400} />
            </Grid>
            <Grid item xs={12} lg={4}>
              <MEVDistributionChart height={400} />
            </Grid>

            {/* Recent Alerts */}
            <Grid item xs={12} md={6}>
              <Paper sx={{ p: 3, height: 400 }}>
                <Typography variant="h6" gutterBottom sx={{ fontWeight: 600 }}>
                  Recent MEV Alerts
                </Typography>
                <List>
                  {recentAlerts.map((alert, index) => (
                    <React.Fragment key={alert.id}>
                      <ListItem alignItems="flex-start">
                        <ListItemAvatar>
                          <Avatar
                            sx={{
                              bgcolor: 
                                alert.severity === 'high' ? 'error.light' :
                                alert.severity === 'medium' ? 'warning.light' : 'info.light',
                              color:
                                alert.severity === 'high' ? 'error.main' :
                                alert.severity === 'medium' ? 'warning.main' : 'info.main',
                            }}
                          >
                            <Warning />
                          </Avatar>
                        </ListItemAvatar>
                        <ListItemText
                          primary={
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                              <Typography variant="body1">{alert.message}</Typography>
                              <Chip label={alert.saved} size="small" color="success" />
                            </Box>
                          }
                          secondary={
                            <Typography variant="body2" color="text.secondary">
                              {alert.time}
                            </Typography>
                          }
                        />
                      </ListItem>
                      {index < recentAlerts.length - 1 && <Divider variant="inset" component="li" />}
                    </React.Fragment>
                  ))}
                </List>
              </Paper>
            </Grid>

            {/* Validator Status */}
            <Grid item xs={12} md={6}>
              <Paper sx={{ p: 3, height: 400 }}>
                <Typography variant="h6" gutterBottom sx={{ fontWeight: 600 }}>
                  Validator Network
                </Typography>
                <List>
                  {connectedValidators.map((validator, index) => (
                    <React.Fragment key={index}>
                      <ListItem>
                        <ListItemAvatar>
                          <Avatar sx={{ bgcolor: validator.status === 'active' ? 'success.light' : 'warning.light' }}>
                            {validator.status === 'active' ? <CheckCircle /> : <Warning />}
                          </Avatar>
                        </ListItemAvatar>
                        <ListItemText
                          primary={validator.name}
                          secondary={`Stake: ${validator.stake} | Performance: ${validator.performance}%`}
                        />
                        <Chip
                          label={validator.status}
                          color={validator.status === 'active' ? 'success' : 'warning'}
                          size="small"
                        />
                      </ListItem>
                      {index < connectedValidators.length - 1 && <Divider variant="inset" component="li" />}
                    </React.Fragment>
                  ))}
                </List>
              </Paper>
            </Grid>
          </Grid>
        )}

        {/* Analytics Tab */}
        {activeTab === 1 && (
          <Grid container spacing={3}>
            <Grid item xs={12} lg={6}>
              <ValueProtectedChart height={400} />
            </Grid>
            <Grid item xs={12} lg={6}>
              <NetworkPerformanceChart height={400} />
            </Grid>
            <Grid item xs={12}>
              <ExchangeVolumeChart height={350} />
            </Grid>
          </Grid>
        )}

        {/* DEX Protection Tab */}
        {activeTab === 2 && (
          <DexIntegration />
        )}

        {/* Exchanges Tab */}
        {activeTab === 3 && (
          <ExchangeIntegration />
        )}

        {/* Security Tab */}
        {activeTab === 4 && (
          <Grid container spacing={3}>
            <Grid item xs={12}>
              <Paper sx={{ p: 3 }}>
                <Typography variant="h5" gutterBottom>
                  Security Overview
                </Typography>
                <Typography variant="body1" color="text.secondary">
                  Real-time security monitoring and threat detection
                </Typography>
              </Paper>
            </Grid>
          </Grid>
        )}

        {/* Validators Tab */}
        {activeTab === 5 && (
          <Grid container spacing={3}>
            <Grid item xs={12}>
              <Paper sx={{ p: 3 }}>
                <Typography variant="h5" gutterBottom>
                  Validator Network Management
                </Typography>
                <Typography variant="body1" color="text.secondary">
                  Monitor and manage connected validator nodes
                </Typography>
              </Paper>
            </Grid>
          </Grid>
        )}
      </Container>
    </Box>
  );
};

export default ImprovedAdminDashboard;