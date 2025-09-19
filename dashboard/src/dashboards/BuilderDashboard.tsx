import React, { useState } from 'react';
import {
  Box,
  Container,
  Grid,
  Paper,
  Typography,
  Card,
  CardContent,
  Button,
  Chip,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  LinearProgress,
  Avatar,
  IconButton,
  Alert,
  List,
  ListItem,
  ListItemText,
  ListItemAvatar,
  Divider,
  Badge,
  Tabs,
  Tab,
} from '@mui/material';
import {
  Build,
  Block,
  Speed,
  AttachMoney,
  TrendingUp,
  Warning,
  CheckCircle,
  Refresh,
  Settings,
  Timeline,
  Memory,
  Storage,
  CloudQueue,
  Layers,
  Code,
} from '@mui/icons-material';

interface BlockMetrics {
  blocksBuilt: number;
  successRate: number;
  avgGasUsed: string;
  totalRevenue: string;
  mevCaptured: string;
}

const BuilderDashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);
  const [refreshing, setRefreshing] = useState(false);

  const metrics: BlockMetrics = {
    blocksBuilt: 1247,
    successRate: 94.3,
    avgGasUsed: '12.5M',
    totalRevenue: '45.8 ETH',
    mevCaptured: '12.3 ETH',
  };

  const recentBlocks = [
    {
      number: 18750432,
      hash: '0x1234...5678',
      transactions: 142,
      gasUsed: '12.8M',
      reward: '0.125 ETH',
      mevRevenue: '0.045 ETH',
      status: 'success',
      timestamp: new Date(),
    },
    {
      number: 18750431,
      hash: '0x8765...4321',
      transactions: 128,
      gasUsed: '11.2M',
      reward: '0.115 ETH',
      mevRevenue: '0.038 ETH',
      status: 'success',
      timestamp: new Date(Date.now() - 12000),
    },
    {
      number: 18750430,
      hash: '0xabcd...efgh',
      transactions: 156,
      gasUsed: '13.1M',
      reward: '0.132 ETH',
      mevRevenue: '0.052 ETH',
      status: 'success',
      timestamp: new Date(Date.now() - 24000),
    },
  ];

  const bundleQueue = [
    {
      id: '1',
      sender: '0xabc...def',
      txCount: 3,
      estimatedProfit: '0.025 ETH',
      priority: 'high',
      status: 'pending',
    },
    {
      id: '2',
      sender: '0xghi...jkl',
      txCount: 2,
      estimatedProfit: '0.018 ETH',
      priority: 'medium',
      status: 'pending',
    },
  ];

  const handleRefresh = () => {
    setRefreshing(true);
    setTimeout(() => setRefreshing(false), 2000);
  };

  return (
    <Box sx={{ bgcolor: 'background.default', minHeight: '100vh', py: 3 }}>
      <Container maxWidth="xl">
        {/* Header */}
        <Box sx={{ mb: 4 }}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
            <Typography variant="h4" sx={{ fontWeight: 700, display: 'flex', alignItems: 'center' }}>
              <Build sx={{ mr: 2, fontSize: 40, color: 'primary.main' }} />
              Builder Dashboard
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <Button
                variant="outlined"
                startIcon={<Refresh />}
                onClick={handleRefresh}
                disabled={refreshing}
              >
                Refresh
              </Button>
              <IconButton>
                <Settings />
              </IconButton>
            </Box>
          </Box>
          
          <Alert severity="info" sx={{ mb: 2 }}>
            Welcome, Builder! You have access to block building and MEV bundle management features.
          </Alert>
        </Box>

        {/* Metrics Cards */}
        <Grid container spacing={3} sx={{ mb: 3 }}>
          <Grid item xs={12} sm={6} md={2.4}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <Block color="primary" sx={{ mr: 1 }} />
                  <Typography color="text.secondary" variant="body2">
                    Blocks Built
                  </Typography>
                </Box>
                <Typography variant="h4" sx={{ fontWeight: 600 }}>
                  {metrics.blocksBuilt}
                </Typography>
                <Chip label="+12 today" size="small" color="success" sx={{ mt: 1 }} />
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={2.4}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <TrendingUp color="success" sx={{ mr: 1 }} />
                  <Typography color="text.secondary" variant="body2">
                    Success Rate
                  </Typography>
                </Box>
                <Typography variant="h4" sx={{ fontWeight: 600 }}>
                  {metrics.successRate}%
                </Typography>
                <LinearProgress
                  variant="determinate"
                  value={metrics.successRate}
                  sx={{ mt: 1, height: 6, borderRadius: 3 }}
                  color="success"
                />
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={2.4}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <Speed color="warning" sx={{ mr: 1 }} />
                  <Typography color="text.secondary" variant="body2">
                    Avg Gas Used
                  </Typography>
                </Box>
                <Typography variant="h4" sx={{ fontWeight: 600 }}>
                  {metrics.avgGasUsed}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  Per block
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={2.4}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <AttachMoney color="info" sx={{ mr: 1 }} />
                  <Typography color="text.secondary" variant="body2">
                    Total Revenue
                  </Typography>
                </Box>
                <Typography variant="h4" sx={{ fontWeight: 600 }}>
                  {metrics.totalRevenue}
                </Typography>
                <Chip label="+2.3 ETH" size="small" color="info" sx={{ mt: 1 }} />
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={2.4}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <Layers color="secondary" sx={{ mr: 1 }} />
                  <Typography color="text.secondary" variant="body2">
                    MEV Captured
                  </Typography>
                </Box>
                <Typography variant="h4" sx={{ fontWeight: 600 }}>
                  {metrics.mevCaptured}
                </Typography>
                <Typography variant="body2" color="success.main" sx={{ mt: 1 }}>
                  26.8% of revenue
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>

        {/* Tabs */}
        <Paper sx={{ mb: 3 }}>
          <Tabs
            value={activeTab}
            onChange={(_, v) => setActiveTab(v)}
            sx={{ borderBottom: 1, borderColor: 'divider', px: 2 }}
          >
            <Tab label="Recent Blocks" icon={<Block />} iconPosition="start" />
            <Tab label="Bundle Queue" icon={<Layers />} iconPosition="start" />
            <Tab label="MEV Opportunities" icon={<Timeline />} iconPosition="start" />
            <Tab label="Builder Settings" icon={<Settings />} iconPosition="start" />
          </Tabs>

          {/* Recent Blocks Tab */}
          {activeTab === 0 && (
            <Box sx={{ p: 3 }}>
              <TableContainer>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Block</TableCell>
                      <TableCell>Hash</TableCell>
                      <TableCell align="center">Transactions</TableCell>
                      <TableCell align="right">Gas Used</TableCell>
                      <TableCell align="right">Block Reward</TableCell>
                      <TableCell align="right">MEV Revenue</TableCell>
                      <TableCell align="center">Status</TableCell>
                      <TableCell>Time</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {recentBlocks.map((block) => (
                      <TableRow key={block.number} hover>
                        <TableCell>
                          <Typography variant="body2" sx={{ fontWeight: 600 }}>
                            {block.number.toLocaleString()}
                          </Typography>
                        </TableCell>
                        <TableCell>
                          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                            {block.hash}
                          </Typography>
                        </TableCell>
                        <TableCell align="center">{block.transactions}</TableCell>
                        <TableCell align="right">{block.gasUsed}</TableCell>
                        <TableCell align="right">
                          <Typography variant="body2" color="primary.main" sx={{ fontWeight: 500 }}>
                            {block.reward}
                          </Typography>
                        </TableCell>
                        <TableCell align="right">
                          <Typography variant="body2" color="success.main" sx={{ fontWeight: 500 }}>
                            {block.mevRevenue}
                          </Typography>
                        </TableCell>
                        <TableCell align="center">
                          <Chip
                            icon={<CheckCircle />}
                            label={block.status}
                            size="small"
                            color="success"
                          />
                        </TableCell>
                        <TableCell>
                          {new Date(block.timestamp).toLocaleTimeString()}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            </Box>
          )}

          {/* Bundle Queue Tab */}
          {activeTab === 1 && (
            <Box sx={{ p: 3 }}>
              <Alert severity="info" sx={{ mb: 2 }}>
                {bundleQueue.length} bundles in queue waiting for inclusion
              </Alert>
              
              <List>
                {bundleQueue.map((bundle, index) => (
                  <React.Fragment key={bundle.id}>
                    <ListItem>
                      <ListItemAvatar>
                        <Avatar sx={{ bgcolor: bundle.priority === 'high' ? 'error.light' : 'warning.light' }}>
                          <Layers />
                        </Avatar>
                      </ListItemAvatar>
                      <ListItemText
                        primary={
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <Typography variant="body1">Bundle from {bundle.sender}</Typography>
                            <Chip
                              label={bundle.priority}
                              size="small"
                              color={bundle.priority === 'high' ? 'error' : 'warning'}
                            />
                          </Box>
                        }
                        secondary={
                          <Box>
                            <Typography variant="body2" color="text.secondary">
                              {bundle.txCount} transactions • Estimated profit: {bundle.estimatedProfit}
                            </Typography>
                          </Box>
                        }
                      />
                      <Box>
                        <Button variant="contained" size="small" sx={{ mr: 1 }}>
                          Include
                        </Button>
                        <Button variant="outlined" size="small">
                          Reject
                        </Button>
                      </Box>
                    </ListItem>
                    {index < bundleQueue.length - 1 && <Divider variant="inset" component="li" />}
                  </React.Fragment>
                ))}
              </List>
            </Box>
          )}

          {/* MEV Opportunities Tab */}
          {activeTab === 2 && (
            <Box sx={{ p: 3 }}>
              <Alert severity="success" sx={{ mb: 2 }}>
                3 profitable MEV opportunities detected in the mempool
              </Alert>
              
              <Grid container spacing={2}>
                <Grid item xs={12} md={4}>
                  <Card>
                    <CardContent>
                      <Typography variant="h6" gutterBottom>
                        Arbitrage Opportunity
                      </Typography>
                      <Typography variant="body2" color="text.secondary" gutterBottom>
                        USDC/WETH price difference across DEXs
                      </Typography>
                      <Typography variant="h5" color="success.main">
                        +0.042 ETH
                      </Typography>
                      <Button variant="contained" fullWidth sx={{ mt: 2 }}>
                        Execute Bundle
                      </Button>
                    </CardContent>
                  </Card>
                </Grid>
                
                <Grid item xs={12} md={4}>
                  <Card>
                    <CardContent>
                      <Typography variant="h6" gutterBottom>
                        Liquidation Available
                      </Typography>
                      <Typography variant="body2" color="text.secondary" gutterBottom>
                        Aave position ready for liquidation
                      </Typography>
                      <Typography variant="h5" color="warning.main">
                        +0.028 ETH
                      </Typography>
                      <Button variant="contained" fullWidth sx={{ mt: 2 }}>
                        Execute Bundle
                      </Button>
                    </CardContent>
                  </Card>
                </Grid>
                
                <Grid item xs={12} md={4}>
                  <Card>
                    <CardContent>
                      <Typography variant="h6" gutterBottom>
                        Sandwich Opportunity
                      </Typography>
                      <Typography variant="body2" color="text.secondary" gutterBottom>
                        Large swap detected on Uniswap
                      </Typography>
                      <Typography variant="h5" color="info.main">
                        +0.035 ETH
                      </Typography>
                      <Button variant="outlined" fullWidth sx={{ mt: 2 }} disabled>
                        Protected by MEV Shield
                      </Button>
                    </CardContent>
                  </Card>
                </Grid>
              </Grid>
            </Box>
          )}

          {/* Builder Settings Tab */}
          {activeTab === 3 && (
            <Box sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                Builder Configuration
              </Typography>
              <Alert severity="warning" sx={{ mb: 2 }}>
                Adjusting these settings may affect your block building performance
              </Alert>
              
              <Grid container spacing={3}>
                <Grid item xs={12} md={6}>
                  <Paper sx={{ p: 2 }}>
                    <Typography variant="subtitle1" gutterBottom>
                      MEV Strategy Settings
                    </Typography>
                    <List>
                      <ListItem>
                        <ListItemText primary="Enable Arbitrage Bundles" secondary="Automatically include profitable arbitrage" />
                        <Badge color="success" variant="dot">
                          <Chip label="Enabled" color="success" size="small" />
                        </Badge>
                      </ListItem>
                      <ListItem>
                        <ListItemText primary="Enable Liquidations" secondary="Include liquidation transactions" />
                        <Badge color="success" variant="dot">
                          <Chip label="Enabled" color="success" size="small" />
                        </Badge>
                      </ListItem>
                      <ListItem>
                        <ListItemText primary="Enable Sandwich Protection" secondary="Protect users from sandwich attacks" />
                        <Badge color="success" variant="dot">
                          <Chip label="Enabled" color="success" size="small" />
                        </Badge>
                      </ListItem>
                    </List>
                  </Paper>
                </Grid>
                
                <Grid item xs={12} md={6}>
                  <Paper sx={{ p: 2 }}>
                    <Typography variant="subtitle1" gutterBottom>
                      Performance Settings
                    </Typography>
                    <List>
                      <ListItem>
                        <ListItemText primary="Max Gas per Block" secondary="12.5M gas" />
                        <Button size="small" variant="outlined">Edit</Button>
                      </ListItem>
                      <ListItem>
                        <ListItemText primary="Min Bundle Profit" secondary="0.01 ETH" />
                        <Button size="small" variant="outlined">Edit</Button>
                      </ListItem>
                      <ListItem>
                        <ListItemText primary="Priority Fee Threshold" secondary="2 Gwei" />
                        <Button size="small" variant="outlined">Edit</Button>
                      </ListItem>
                    </List>
                  </Paper>
                </Grid>
              </Grid>
            </Box>
          )}
        </Paper>
      </Container>
    </Box>
  );
};

export default BuilderDashboard;