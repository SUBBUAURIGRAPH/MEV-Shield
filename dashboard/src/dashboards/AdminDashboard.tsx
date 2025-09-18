import React, { useState, useEffect } from 'react';
import {
  Grid,
  Card,
  CardContent,
  Typography,
  Box,
  Paper,
  Button,
  IconButton,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Chip,
  LinearProgress,
  Alert,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Tabs,
  Tab,
  Avatar,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  Divider,
  Switch,
  FormControlLabel,
} from '@mui/material';
import {
  Dashboard,
  Shield,
  TrendingUp,
  Warning,
  Check,
  Block,
  Info,
  Speed,
  People,
  Settings,
  Refresh,
  Download,
  FilterList,
  Search,
  Notifications,
  Security,
  AccountBalance,
  Timeline,
  CloudUpload,
  Storage,
} from '@mui/icons-material';
import { Line, Bar, Doughnut, Radar } from 'react-chartjs-2';
import '../chartSetup'; // Import Chart.js setup
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDateFns } from '@mui/x-date-pickers/AdapterDateFns';

interface MetricCard {
  title: string;
  value: string | number;
  change: number;
  icon: React.ReactElement;
  color: string;
}

interface Transaction {
  id: string;
  hash: string;
  from: string;
  to: string;
  value: string;
  mevProtected: boolean;
  mevSaved: string;
  status: 'success' | 'pending' | 'failed';
  timestamp: Date;
  gasUsed: string;
  blockNumber: number;
}

interface Builder {
  address: string;
  reputation: number;
  blocksBuilt: number;
  stake: string;
  status: 'active' | 'inactive' | 'slashed';
  lastActive: Date;
}

const AdminDashboard: React.FC = () => {
  const [selectedTab, setSelectedTab] = useState(0);
  const [timeRange, setTimeRange] = useState('24h');
  const [refreshing, setRefreshing] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterStatus, setFilterStatus] = useState('all');
  const [notifications, setNotifications] = useState(5);

  // Mock data - replace with actual API calls
  const metrics: MetricCard[] = [
    {
      title: 'Total Protected Volume',
      value: '$45.8M',
      change: 12.5,
      icon: <Shield />,
      color: '#4CAF50',
    },
    {
      title: 'MEV Saved',
      value: '$2.3M',
      change: 8.3,
      icon: <AccountBalance />,
      color: '#2196F3',
    },
    {
      title: 'Active Users',
      value: '12,847',
      change: 15.2,
      icon: <People />,
      color: '#FF9800',
    },
    {
      title: 'Success Rate',
      value: '98.5%',
      change: 0.5,
      icon: <TrendingUp />,
      color: '#9C27B0',
    },
  ];

  const systemHealth = {
    api: { status: 'healthy', latency: 45, uptime: 99.99 },
    database: { status: 'healthy', connections: 85, queryTime: 12 },
    redis: { status: 'healthy', memory: 68, hitRate: 94.5 },
    blockchain: { status: 'synced', blockHeight: 18750432, peers: 128 },
  };

  const recentAlerts = [
    { id: 1, type: 'warning', message: 'High MEV activity detected on Uniswap', time: '5 min ago' },
    { id: 2, type: 'error', message: 'Builder 0x742d... missed block proposal', time: '12 min ago' },
    { id: 3, type: 'info', message: 'System update scheduled for 2:00 AM UTC', time: '1 hour ago' },
    { id: 4, type: 'success', message: 'Successfully prevented sandwich attack', time: '2 hours ago' },
  ];

  const handleRefresh = () => {
    setRefreshing(true);
    setTimeout(() => setRefreshing(false), 2000);
  };

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setSelectedTab(newValue);
  };

  // Chart configurations
  const volumeChartData = {
    labels: ['00:00', '04:00', '08:00', '12:00', '16:00', '20:00', '24:00'],
    datasets: [
      {
        label: 'Protected Volume',
        data: [2.1, 1.8, 3.2, 4.5, 3.8, 5.2, 4.8],
        borderColor: '#4CAF50',
        backgroundColor: 'rgba(76, 175, 80, 0.1)',
        tension: 0.4,
      },
      {
        label: 'MEV Saved',
        data: [0.3, 0.2, 0.5, 0.8, 0.6, 0.9, 0.7],
        borderColor: '#2196F3',
        backgroundColor: 'rgba(33, 150, 243, 0.1)',
        tension: 0.4,
      },
    ],
  };

  const mevTypeDistribution = {
    labels: ['Sandwich', 'Frontrun', 'Arbitrage', 'Liquidation', 'Other'],
    datasets: [
      {
        data: [35, 25, 20, 15, 5],
        backgroundColor: ['#FF6384', '#36A2EB', '#FFCE56', '#9C27B0', '#4CAF50'],
      },
    ],
  };

  const chainDistribution = {
    labels: ['Ethereum', 'Polygon', 'BSC', 'Arbitrum', 'Base'],
    datasets: [
      {
        label: 'Transaction Volume',
        data: [45, 20, 15, 12, 8],
        backgroundColor: 'rgba(75, 192, 192, 0.6)',
      },
      {
        label: 'MEV Saved',
        data: [52, 18, 12, 10, 8],
        backgroundColor: 'rgba(255, 99, 132, 0.6)',
      },
    ],
  };

  return (
    <LocalizationProvider dateAdapter={AdapterDateFns}>
      <Box sx={{ flexGrow: 1, p: 3, backgroundColor: '#f5f5f5', minHeight: '100vh' }}>
        {/* Header */}
        <Paper elevation={2} sx={{ p: 2, mb: 3 }}>
          <Grid container alignItems="center" justifyContent="space-between">
            <Grid item>
              <Typography variant="h4" component="h1" sx={{ fontWeight: 'bold' }}>
                MEV Shield Admin Dashboard
              </Typography>
              <Typography variant="body2" color="textSecondary">
                System monitoring and management console
              </Typography>
            </Grid>
            <Grid item>
              <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
                <Chip
                  icon={<Check />}
                  label="All Systems Operational"
                  color="success"
                  variant="outlined"
                />
                <IconButton onClick={handleRefresh} disabled={refreshing}>
                  <Refresh />
                </IconButton>
                <IconButton>
                  <Notifications />
                  {notifications > 0 && (
                    <Box
                      sx={{
                        position: 'absolute',
                        top: 8,
                        right: 8,
                        backgroundColor: 'red',
                        borderRadius: '50%',
                        width: 16,
                        height: 16,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                      }}
                    >
                      <Typography variant="caption" color="white">
                        {notifications}
                      </Typography>
                    </Box>
                  )}
                </IconButton>
                <IconButton>
                  <Settings />
                </IconButton>
              </Box>
            </Grid>
          </Grid>
        </Paper>

        {/* Metrics Cards */}
        <Grid container spacing={3} sx={{ mb: 3 }}>
          {metrics.map((metric, index) => (
            <Grid item xs={12} sm={6} md={3} key={index}>
              <Card elevation={2}>
                <CardContent>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <Box>
                      <Typography variant="body2" color="textSecondary">
                        {metric.title}
                      </Typography>
                      <Typography variant="h4" sx={{ fontWeight: 'bold', my: 1 }}>
                        {metric.value}
                      </Typography>
                      <Box sx={{ display: 'flex', alignItems: 'center' }}>
                        <TrendingUp
                          sx={{
                            fontSize: 16,
                            color: metric.change > 0 ? '#4CAF50' : '#f44336',
                            mr: 0.5,
                          }}
                        />
                        <Typography
                          variant="body2"
                          sx={{ color: metric.change > 0 ? '#4CAF50' : '#f44336' }}
                        >
                          {metric.change > 0 ? '+' : ''}{metric.change}%
                        </Typography>
                      </Box>
                    </Box>
                    <Avatar sx={{ bgcolor: metric.color, width: 56, height: 56 }}>
                      {metric.icon}
                    </Avatar>
                  </Box>
                </CardContent>
              </Card>
            </Grid>
          ))}
        </Grid>

        {/* Main Content Tabs */}
        <Paper elevation={2} sx={{ mb: 3 }}>
          <Tabs value={selectedTab} onChange={handleTabChange} sx={{ borderBottom: 1, borderColor: 'divider' }}>
            <Tab label="Overview" icon={<Dashboard />} iconPosition="start" />
            <Tab label="Transactions" icon={<Timeline />} iconPosition="start" />
            <Tab label="Builders" icon={<People />} iconPosition="start" />
            <Tab label="System Health" icon={<Speed />} iconPosition="start" />
            <Tab label="Security" icon={<Security />} iconPosition="start" />
            <Tab label="Settings" icon={<Settings />} iconPosition="start" />
          </Tabs>

          {/* Tab Panels */}
          {selectedTab === 0 && (
            <Box sx={{ p: 3 }}>
              <Grid container spacing={3}>
                {/* Volume Chart */}
                <Grid item xs={12} md={8}>
                  <Card>
                    <CardContent>
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 2 }}>
                        <Typography variant="h6">Protected Volume & MEV Saved</Typography>
                        <FormControl size="small" sx={{ minWidth: 120 }}>
                          <Select value={timeRange} onChange={(e) => setTimeRange(e.target.value)}>
                            <MenuItem value="1h">1 Hour</MenuItem>
                            <MenuItem value="24h">24 Hours</MenuItem>
                            <MenuItem value="7d">7 Days</MenuItem>
                            <MenuItem value="30d">30 Days</MenuItem>
                          </Select>
                        </FormControl>
                      </Box>
                      <Line data={volumeChartData} options={{ responsive: true, maintainAspectRatio: false }} height={300} />
                    </CardContent>
                  </Card>
                </Grid>

                {/* MEV Type Distribution */}
                <Grid item xs={12} md={4}>
                  <Card>
                    <CardContent>
                      <Typography variant="h6" sx={{ mb: 2 }}>MEV Attack Types</Typography>
                      <Doughnut data={mevTypeDistribution} options={{ responsive: true, maintainAspectRatio: false }} height={300} />
                    </CardContent>
                  </Card>
                </Grid>

                {/* Chain Distribution */}
                <Grid item xs={12} md={6}>
                  <Card>
                    <CardContent>
                      <Typography variant="h6" sx={{ mb: 2 }}>Chain Distribution</Typography>
                      <Bar data={chainDistribution} options={{ responsive: true, maintainAspectRatio: false }} height={250} />
                    </CardContent>
                  </Card>
                </Grid>

                {/* Recent Alerts */}
                <Grid item xs={12} md={6}>
                  <Card>
                    <CardContent>
                      <Typography variant="h6" sx={{ mb: 2 }}>Recent Alerts</Typography>
                      <List>
                        {recentAlerts.map((alert) => (
                          <React.Fragment key={alert.id}>
                            <ListItem>
                              <ListItemAvatar>
                                <Avatar
                                  sx={{
                                    bgcolor:
                                      alert.type === 'error' ? '#f44336' :
                                      alert.type === 'warning' ? '#ff9800' :
                                      alert.type === 'success' ? '#4caf50' : '#2196f3',
                                  }}
                                >
                                  {alert.type === 'error' ? <Block /> :
                                   alert.type === 'warning' ? <Warning /> :
                                   alert.type === 'success' ? <Check /> : <Info />}
                                </Avatar>
                              </ListItemAvatar>
                              <ListItemText
                                primary={alert.message}
                                secondary={alert.time}
                              />
                            </ListItem>
                            <Divider variant="inset" component="li" />
                          </React.Fragment>
                        ))}
                      </List>
                    </CardContent>
                  </Card>
                </Grid>
              </Grid>
            </Box>
          )}

          {selectedTab === 1 && (
            <Box sx={{ p: 3 }}>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
                <TextField
                  placeholder="Search transactions..."
                  variant="outlined"
                  size="small"
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  InputProps={{
                    startAdornment: <Search sx={{ mr: 1, color: 'text.secondary' }} />,
                  }}
                  sx={{ width: 300 }}
                />
                <Box sx={{ display: 'flex', gap: 2 }}>
                  <FormControl size="small" sx={{ minWidth: 120 }}>
                    <InputLabel>Status</InputLabel>
                    <Select value={filterStatus} onChange={(e) => setFilterStatus(e.target.value)} label="Status">
                      <MenuItem value="all">All</MenuItem>
                      <MenuItem value="success">Success</MenuItem>
                      <MenuItem value="pending">Pending</MenuItem>
                      <MenuItem value="failed">Failed</MenuItem>
                    </Select>
                  </FormControl>
                  <Button variant="outlined" startIcon={<Download />}>
                    Export
                  </Button>
                </Box>
              </Box>

              <TableContainer component={Paper}>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Transaction Hash</TableCell>
                      <TableCell>From</TableCell>
                      <TableCell>To</TableCell>
                      <TableCell align="right">Value</TableCell>
                      <TableCell align="center">MEV Protected</TableCell>
                      <TableCell align="right">MEV Saved</TableCell>
                      <TableCell align="center">Status</TableCell>
                      <TableCell align="right">Block</TableCell>
                      <TableCell>Time</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {/* Mock transaction data */}
                    {[...Array(10)].map((_, index) => (
                      <TableRow key={index} hover>
                        <TableCell>
                          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                            0x{Math.random().toString(16).substr(2, 8)}...
                          </Typography>
                        </TableCell>
                        <TableCell>
                          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                            0x{Math.random().toString(16).substr(2, 6)}...
                          </Typography>
                        </TableCell>
                        <TableCell>
                          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                            0x{Math.random().toString(16).substr(2, 6)}...
                          </Typography>
                        </TableCell>
                        <TableCell align="right">
                          ${(Math.random() * 10000).toFixed(2)}
                        </TableCell>
                        <TableCell align="center">
                          <Chip
                            icon={<Shield />}
                            label="Protected"
                            color="success"
                            size="small"
                          />
                        </TableCell>
                        <TableCell align="right">
                          ${(Math.random() * 100).toFixed(2)}
                        </TableCell>
                        <TableCell align="center">
                          <Chip
                            label={['Success', 'Pending', 'Failed'][Math.floor(Math.random() * 3)]}
                            color={['success', 'warning', 'error'][Math.floor(Math.random() * 3)] as any}
                            size="small"
                          />
                        </TableCell>
                        <TableCell align="right">
                          {18750432 - index}
                        </TableCell>
                        <TableCell>
                          {index + 1} min ago
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            </Box>
          )}

          {selectedTab === 3 && (
            <Box sx={{ p: 3 }}>
              <Grid container spacing={3}>
                {/* System Health Cards */}
                {Object.entries(systemHealth).map(([service, health]) => (
                  <Grid item xs={12} sm={6} md={3} key={service}>
                    <Card>
                      <CardContent>
                        <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                          <Avatar
                            sx={{
                              bgcolor: health.status === 'healthy' || health.status === 'synced' ? '#4CAF50' : '#f44336',
                              mr: 2,
                            }}
                          >
                            <Check />
                          </Avatar>
                          <Box>
                            <Typography variant="h6" sx={{ textTransform: 'capitalize' }}>
                              {service}
                            </Typography>
                            <Chip
                              label={health.status}
                              color={health.status === 'healthy' || health.status === 'synced' ? 'success' : 'error'}
                              size="small"
                            />
                          </Box>
                        </Box>
                        <Divider sx={{ my: 2 }} />
                        {service === 'api' && 'latency' in health && (
                          <>
                            <Typography variant="body2" color="textSecondary">
                              Latency: {health.latency}ms
                            </Typography>
                            <Typography variant="body2" color="textSecondary">
                              Uptime: {health.uptime}%
                            </Typography>
                          </>
                        )}
                        {service === 'database' && 'connections' in health && (
                          <>
                            <Typography variant="body2" color="textSecondary">
                              Connections: {health.connections}/100
                            </Typography>
                            <LinearProgress variant="determinate" value={health.connections} sx={{ mt: 1 }} />
                            <Typography variant="body2" color="textSecondary" sx={{ mt: 1 }}>
                              Query Time: {health.queryTime}ms
                            </Typography>
                          </>
                        )}
                        {service === 'redis' && 'memory' in health && (
                          <>
                            <Typography variant="body2" color="textSecondary">
                              Memory Usage: {health.memory}%
                            </Typography>
                            <LinearProgress variant="determinate" value={health.memory} sx={{ mt: 1 }} />
                            <Typography variant="body2" color="textSecondary" sx={{ mt: 1 }}>
                              Hit Rate: {health.hitRate}%
                            </Typography>
                          </>
                        )}
                        {service === 'blockchain' && 'blockHeight' in health && (
                          <>
                            <Typography variant="body2" color="textSecondary">
                              Block Height: {health.blockHeight.toLocaleString()}
                            </Typography>
                            <Typography variant="body2" color="textSecondary">
                              Peers: {health.peers}
                            </Typography>
                          </>
                        )}
                      </CardContent>
                    </Card>
                  </Grid>
                ))}
              </Grid>

              {/* Performance Metrics */}
              <Grid container spacing={3} sx={{ mt: 2 }}>
                <Grid item xs={12}>
                  <Card>
                    <CardContent>
                      <Typography variant="h6" sx={{ mb: 3 }}>Performance Metrics</Typography>
                      <Grid container spacing={2}>
                        <Grid item xs={12} md={3}>
                          <Box sx={{ textAlign: 'center' }}>
                            <Typography variant="h3" color="primary">
                              45ms
                            </Typography>
                            <Typography variant="body2" color="textSecondary">
                              Avg Response Time
                            </Typography>
                          </Box>
                        </Grid>
                        <Grid item xs={12} md={3}>
                          <Box sx={{ textAlign: 'center' }}>
                            <Typography variant="h3" color="primary">
                              12,847
                            </Typography>
                            <Typography variant="body2" color="textSecondary">
                              Requests/sec
                            </Typography>
                          </Box>
                        </Grid>
                        <Grid item xs={12} md={3}>
                          <Box sx={{ textAlign: 'center' }}>
                            <Typography variant="h3" color="primary">
                              99.99%
                            </Typography>
                            <Typography variant="body2" color="textSecondary">
                              Uptime
                            </Typography>
                          </Box>
                        </Grid>
                        <Grid item xs={12} md={3}>
                          <Box sx={{ textAlign: 'center' }}>
                            <Typography variant="h3" color="primary">
                              0
                            </Typography>
                            <Typography variant="body2" color="textSecondary">
                              Error Rate
                            </Typography>
                          </Box>
                        </Grid>
                      </Grid>
                    </CardContent>
                  </Card>
                </Grid>
              </Grid>
            </Box>
          )}
        </Paper>
      </Box>
    </LocalizationProvider>
  );
};

export default AdminDashboard;