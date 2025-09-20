import React, { useState, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Chip,
  Alert,
  AlertTitle,
  LinearProgress,
  IconButton,
  Tooltip,
  Badge,
  Avatar,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  Divider,
  Button,
  Stack
} from '@mui/material';
import {
  Warning as WarningIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
  TrendingUp as TrendingUpIcon,
  Shield as ShieldIcon,
  Speed as SpeedIcon,
  Visibility as VisibilityIcon,
  Block as BlockIcon,
  LocalFireDepartment as FireIcon
} from '@mui/icons-material';
import { format } from 'date-fns';

interface MEVAlert {
  id: string;
  type: 'front-run' | 'sandwich' | 'jit' | 'arbitrage' | 'liquidation';
  severity: 'low' | 'medium' | 'high' | 'critical';
  timestamp: Date;
  transaction: string;
  value: number;
  chain: string;
  status: 'detected' | 'mitigated' | 'failed';
  description: string;
}

const RealTimeDetection: React.FC = () => {
  const [alerts, setAlerts] = useState<MEVAlert[]>([]);
  const [isScanning, setIsScanning] = useState(true);
  const [protectionStats, setProtectionStats] = useState({
    totalBlocked: 0,
    savedValue: 0,
    detectionRate: 95,
    activeProtections: 4
  });

  // Simulate real-time MEV detection
  useEffect(() => {
    const interval = setInterval(() => {
      if (Math.random() > 0.7) {
        const newAlert: MEVAlert = {
          id: `alert-${Date.now()}`,
          type: ['front-run', 'sandwich', 'jit', 'arbitrage', 'liquidation'][Math.floor(Math.random() * 5)] as any,
          severity: ['low', 'medium', 'high', 'critical'][Math.floor(Math.random() * 4)] as any,
          timestamp: new Date(),
          transaction: `0x${Math.random().toString(16).substring(2, 10)}...`,
          value: Math.random() * 10,
          chain: ['Ethereum', 'BSC', 'Polygon', 'Arbitrum'][Math.floor(Math.random() * 4)],
          status: ['detected', 'mitigated', 'failed'][Math.floor(Math.random() * 3)] as any,
          description: 'Potential MEV activity detected in mempool'
        };
        
        setAlerts(prev => [newAlert, ...prev].slice(0, 10));
        
        if (newAlert.status === 'mitigated') {
          setProtectionStats(prev => ({
            ...prev,
            totalBlocked: prev.totalBlocked + 1,
            savedValue: prev.savedValue + newAlert.value
          }));
        }
      }
    }, 3000);

    return () => clearInterval(interval);
  }, []);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'error';
      case 'high': return 'warning';
      case 'medium': return 'info';
      case 'low': return 'success';
      default: return 'default';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'front-run': return <SpeedIcon />;
      case 'sandwich': return <WarningIcon />;
      case 'jit': return <FireIcon />;
      case 'arbitrage': return <TrendingUpIcon />;
      case 'liquidation': return <ErrorIcon />;
      default: return <ShieldIcon />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'mitigated': return '#4CAF50';
      case 'detected': return '#FF9800';
      case 'failed': return '#F44336';
      default: return '#9E9E9E';
    }
  };

  return (
    <Box>
      <Grid container spacing={3}>
        {/* Protection Stats Cards */}
        <Grid item xs={12} md={3}>
          <Card sx={{ background: 'linear-gradient(45deg, #4CAF50 30%, #45a049 90%)' }}>
            <CardContent>
              <Box display="flex" alignItems="center" justifyContent="space-between">
                <Box>
                  <Typography color="white" variant="h4" fontWeight="bold">
                    {protectionStats.totalBlocked}
                  </Typography>
                  <Typography color="rgba(255,255,255,0.9)" variant="body2">
                    Attacks Blocked
                  </Typography>
                </Box>
                <BlockIcon sx={{ fontSize: 40, color: 'rgba(255,255,255,0.8)' }} />
              </Box>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={3}>
          <Card sx={{ background: 'linear-gradient(45deg, #2196F3 30%, #1976D2 90%)' }}>
            <CardContent>
              <Box display="flex" alignItems="center" justifyContent="space-between">
                <Box>
                  <Typography color="white" variant="h4" fontWeight="bold">
                    ${protectionStats.savedValue.toFixed(2)}
                  </Typography>
                  <Typography color="rgba(255,255,255,0.9)" variant="body2">
                    Value Saved
                  </Typography>
                </Box>
                <ShieldIcon sx={{ fontSize: 40, color: 'rgba(255,255,255,0.8)' }} />
              </Box>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={3}>
          <Card sx={{ background: 'linear-gradient(45deg, #9C27B0 30%, #7B1FA2 90%)' }}>
            <CardContent>
              <Box display="flex" alignItems="center" justifyContent="space-between">
                <Box>
                  <Typography color="white" variant="h4" fontWeight="bold">
                    {protectionStats.detectionRate}%
                  </Typography>
                  <Typography color="rgba(255,255,255,0.9)" variant="body2">
                    Detection Rate
                  </Typography>
                </Box>
                <VisibilityIcon sx={{ fontSize: 40, color: 'rgba(255,255,255,0.8)' }} />
              </Box>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={3}>
          <Card sx={{ background: 'linear-gradient(45deg, #FF9800 30%, #F57C00 90%)' }}>
            <CardContent>
              <Box display="flex" alignItems="center" justifyContent="space-between">
                <Box>
                  <Typography color="white" variant="h4" fontWeight="bold">
                    {protectionStats.activeProtections}
                  </Typography>
                  <Typography color="rgba(255,255,255,0.9)" variant="body2">
                    Active Protections
                  </Typography>
                </Box>
                <SpeedIcon sx={{ fontSize: 40, color: 'rgba(255,255,255,0.8)' }} />
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Real-time Detection Feed */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                <Typography variant="h6" fontWeight="bold">
                  Real-time MEV Detection
                </Typography>
                <Box display="flex" alignItems="center" gap={2}>
                  {isScanning && (
                    <Chip
                      icon={<Badge color="success" variant="dot" />}
                      label="Scanning Active"
                      color="success"
                      size="small"
                    />
                  )}
                  <Button
                    variant="outlined"
                    size="small"
                    onClick={() => setIsScanning(!isScanning)}
                  >
                    {isScanning ? 'Pause' : 'Resume'}
                  </Button>
                </Box>
              </Box>

              {isScanning && <LinearProgress sx={{ mb: 2 }} />}

              <List sx={{ maxHeight: 400, overflow: 'auto' }}>
                {alerts.length === 0 ? (
                  <Alert severity="info">
                    <AlertTitle>No MEV Activity Detected</AlertTitle>
                    The system is actively monitoring for MEV attacks...
                  </Alert>
                ) : (
                  alerts.map((alert, index) => (
                    <React.Fragment key={alert.id}>
                      <ListItem
                        sx={{
                          bgcolor: index === 0 ? 'action.hover' : 'transparent',
                          borderRadius: 1,
                          mb: 1
                        }}
                      >
                        <ListItemAvatar>
                          <Avatar sx={{ bgcolor: getStatusColor(alert.status) }}>
                            {getTypeIcon(alert.type)}
                          </Avatar>
                        </ListItemAvatar>
                        <ListItemText
                          primary={
                            <Box display="flex" alignItems="center" gap={1}>
                              <Typography variant="body1" fontWeight="bold">
                                {alert.type.charAt(0).toUpperCase() + alert.type.slice(1).replace('-', ' ')} Attack
                              </Typography>
                              <Chip
                                label={alert.severity}
                                size="small"
                                color={getSeverityColor(alert.severity) as any}
                              />
                              <Chip
                                label={alert.chain}
                                size="small"
                                variant="outlined"
                              />
                              <Chip
                                label={alert.status}
                                size="small"
                                sx={{
                                  bgcolor: getStatusColor(alert.status),
                                  color: 'white'
                                }}
                              />
                            </Box>
                          }
                          secondary={
                            <Box>
                              <Typography variant="body2" color="text.secondary">
                                {alert.description}
                              </Typography>
                              <Box display="flex" gap={2} mt={0.5}>
                                <Typography variant="caption" color="text.secondary">
                                  TX: {alert.transaction}
                                </Typography>
                                <Typography variant="caption" color="text.secondary">
                                  Value: ${alert.value.toFixed(4)}
                                </Typography>
                                <Typography variant="caption" color="text.secondary">
                                  {format(alert.timestamp, 'HH:mm:ss')}
                                </Typography>
                              </Box>
                            </Box>
                          }
                        />
                        <Box>
                          <IconButton size="small">
                            <VisibilityIcon fontSize="small" />
                          </IconButton>
                        </Box>
                      </ListItem>
                      {index < alerts.length - 1 && <Divider />}
                    </React.Fragment>
                  ))
                )}
              </List>
            </CardContent>
          </Card>
        </Grid>

        {/* Protection Status */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Typography variant="h6" fontWeight="bold" mb={2}>
                Active Protection Modules
              </Typography>
              <Grid container spacing={2}>
                <Grid item xs={12} md={3}>
                  <Alert severity="success" icon={<ShieldIcon />}>
                    <AlertTitle>Private Mempool</AlertTitle>
                    <Typography variant="body2">
                      Transactions routed through private channels
                    </Typography>
                  </Alert>
                </Grid>
                <Grid item xs={12} md={3}>
                  <Alert severity="success" icon={<SpeedIcon />}>
                    <AlertTitle>Flashbots Protect</AlertTitle>
                    <Typography variant="body2">
                      Connected to Flashbots RPC
                    </Typography>
                  </Alert>
                </Grid>
                <Grid item xs={12} md={3}>
                  <Alert severity="success" icon={<TrendingUpIcon />}>
                    <AlertTitle>Smart Routing</AlertTitle>
                    <Typography variant="body2">
                      AI-powered transaction optimization
                    </Typography>
                  </Alert>
                </Grid>
                <Grid item xs={12} md={3}>
                  <Alert severity="success" icon={<BlockIcon />}>
                    <AlertTitle>MEV Blocker</AlertTitle>
                    <Typography variant="body2">
                      Real-time MEV detection and prevention
                    </Typography>
                  </Alert>
                </Grid>
              </Grid>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
};

export default RealTimeDetection;