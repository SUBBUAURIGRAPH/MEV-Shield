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
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Stepper,
  Step,
  StepLabel,
  ToggleButton,
  ToggleButtonGroup,
  Tooltip,
  Badge,
} from '@mui/material';
import {
  AccountBalanceWallet,
  Shield,
  TrendingUp,
  Send,
  History,
  Settings,
  Notifications,
  ContentCopy,
  CheckCircle,
  Warning,
  Info,
  AttachMoney,
  Speed,
  Security,
  Savings,
  QueryStats,
  ArrowUpward,
  ArrowDownward,
  SwapHoriz,
  Lock,
  Block,
  LockOpen,
  Timer,
  Verified,
  EmojiEvents,
} from '@mui/icons-material';
import { Line, Doughnut, Bar } from 'react-chartjs-2';
import QRCode from 'qrcode.react';

interface WalletInfo {
  address: string;
  balance: string;
  mevSaved: string;
  pendingRewards: string;
  protectionLevel: 'basic' | 'standard' | 'maximum';
  transactionCount: number;
}

interface Transaction {
  id: string;
  hash: string;
  type: 'send' | 'receive' | 'swap' | 'contract';
  amount: string;
  token: string;
  to: string;
  from: string;
  status: 'success' | 'pending' | 'failed';
  mevProtected: boolean;
  mevSaved: string;
  timestamp: Date;
  gasUsed: string;
  protectionType?: string;
}

interface ProtectionStats {
  totalProtected: number;
  mevPrevented: number;
  totalSaved: string;
  successRate: number;
}

const UserDashboard: React.FC = () => {
  const [selectedTab, setSelectedTab] = useState(0);
  const [protectionLevel, setProtectionLevel] = useState<'basic' | 'standard' | 'maximum'>('standard');
  const [showSendDialog, setShowSendDialog] = useState(false);
  const [showReceiveDialog, setShowReceiveDialog] = useState(false);
  const [timeRange, setTimeRange] = useState('7d');
  const [copied, setCopied] = useState(false);

  // Mock wallet data - replace with actual wallet integration
  const wallet: WalletInfo = {
    address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb8',
    balance: '12.458',
    mevSaved: '0.847',
    pendingRewards: '0.024',
    protectionLevel: 'standard',
    transactionCount: 156,
  };

  const protectionStats: ProtectionStats = {
    totalProtected: 156,
    mevPrevented: 23,
    totalSaved: '0.847',
    successRate: 98.5,
  };

  const recentTransactions: Transaction[] = [
    {
      id: '1',
      hash: '0xabc123...',
      type: 'swap',
      amount: '1000',
      token: 'USDC',
      to: '0xdef456...',
      from: wallet.address,
      status: 'success',
      mevProtected: true,
      mevSaved: '12.50',
      timestamp: new Date(Date.now() - 3600000),
      gasUsed: '0.002',
      protectionType: 'Sandwich Attack Prevention',
    },
    {
      id: '2',
      hash: '0xghi789...',
      type: 'send',
      amount: '0.5',
      token: 'ETH',
      to: '0xjkl012...',
      from: wallet.address,
      status: 'success',
      mevProtected: true,
      mevSaved: '0',
      timestamp: new Date(Date.now() - 7200000),
      gasUsed: '0.001',
      protectionType: 'Private Transaction',
    },
    {
      id: '3',
      hash: '0xmno345...',
      type: 'swap',
      amount: '5000',
      token: 'DAI',
      to: '0xpqr678...',
      from: wallet.address,
      status: 'pending',
      mevProtected: true,
      mevSaved: '0',
      timestamp: new Date(Date.now() - 10800000),
      gasUsed: '0.003',
      protectionType: 'Frontrun Protection',
    },
  ];

  const handleCopyAddress = () => {
    navigator.clipboard.writeText(wallet.address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleProtectionLevelChange = (event: React.MouseEvent<HTMLElement>, newLevel: string | null) => {
    if (newLevel !== null) {
      setProtectionLevel(newLevel as 'basic' | 'standard' | 'maximum');
    }
  };

  // Chart data
  const savingsChartData = {
    labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
    datasets: [
      {
        label: 'MEV Saved ($)',
        data: [15, 22, 18, 35, 28, 45, 38],
        borderColor: '#4CAF50',
        backgroundColor: 'rgba(76, 175, 80, 0.1)',
        tension: 0.4,
      },
    ],
  };

  const protectionDistribution = {
    labels: ['Sandwich Prevented', 'Frontrun Blocked', 'Arbitrage Protected', 'Private Tx'],
    datasets: [
      {
        data: [35, 30, 20, 15],
        backgroundColor: ['#4CAF50', '#2196F3', '#FF9800', '#9C27B0'],
      },
    ],
  };

  const protectionTrend = {
    labels: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun'],
    datasets: [
      {
        label: 'Protected Transactions',
        data: [12, 19, 23, 28, 32, 38],
        backgroundColor: 'rgba(76, 175, 80, 0.6)',
      },
      {
        label: 'MEV Attacks Prevented',
        data: [2, 3, 4, 5, 6, 8],
        backgroundColor: 'rgba(244, 67, 54, 0.6)',
      },
    ],
  };

  const protectionLevels = [
    {
      level: 'basic',
      name: 'Basic',
      description: 'Standard MEV protection',
      features: ['Sandwich protection', 'Basic frontrun defense'],
      cost: 'Free',
      color: '#9E9E9E',
    },
    {
      level: 'standard',
      name: 'Standard',
      description: 'Enhanced protection with fair ordering',
      features: ['All Basic features', 'Fair ordering', 'Private mempool', 'MEV redistribution'],
      cost: '0.05% fee',
      color: '#2196F3',
    },
    {
      level: 'maximum',
      name: 'Maximum',
      description: 'Complete protection with time-lock',
      features: ['All Standard features', 'Time-lock protection', 'Threshold encryption', 'Priority support'],
      cost: '0.1% fee',
      color: '#4CAF50',
    },
  ];

  return (
    <Box sx={{ flexGrow: 1, p: 3, backgroundColor: '#f5f5f5', minHeight: '100vh' }}>
      {/* Header with Wallet Info */}
      <Paper elevation={2} sx={{ p: 3, mb: 3 }}>
        <Grid container alignItems="center" justifyContent="space-between">
          <Grid item xs={12} md={6}>
            <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
              <Avatar sx={{ width: 56, height: 56, bgcolor: '#4CAF50', mr: 2 }}>
                <AccountBalanceWallet />
              </Avatar>
              <Box>
                <Typography variant="h5" sx={{ fontWeight: 'bold' }}>
                  MEV Shield Wallet
                </Typography>
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <Typography variant="body2" color="textSecondary" sx={{ fontFamily: 'monospace' }}>
                    {wallet.address.substring(0, 6)}...{wallet.address.substring(38)}
                  </Typography>
                  <IconButton size="small" onClick={handleCopyAddress}>
                    {copied ? <CheckCircle sx={{ fontSize: 16 }} /> : <ContentCopy sx={{ fontSize: 16 }} />}
                  </IconButton>
                  <Chip
                    icon={<Verified />}
                    label="Protected"
                    color="success"
                    size="small"
                    sx={{ ml: 1 }}
                  />
                </Box>
              </Box>
            </Box>
          </Grid>
          <Grid item xs={12} md={6}>
            <Box sx={{ display: 'flex', justifyContent: 'flex-end', gap: 2 }}>
              <Button
                variant="contained"
                startIcon={<Send />}
                onClick={() => setShowSendDialog(true)}
                sx={{ bgcolor: '#4CAF50' }}
              >
                Send
              </Button>
              <Button
                variant="outlined"
                startIcon={<SwapHoriz />}
                sx={{ borderColor: '#2196F3', color: '#2196F3' }}
              >
                Swap
              </Button>
              <IconButton>
                <Badge badgeContent={3} color="error">
                  <Notifications />
                </Badge>
              </IconButton>
              <IconButton>
                <Settings />
              </IconButton>
            </Box>
          </Grid>
        </Grid>

        {/* Balance Cards */}
        <Grid container spacing={3} sx={{ mt: 1 }}>
          <Grid item xs={12} sm={6} md={3}>
            <Card sx={{ bgcolor: '#1976d2', color: 'white' }}>
              <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <Box>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>
                      Total Balance
                    </Typography>
                    <Typography variant="h4" sx={{ fontWeight: 'bold' }}>
                      {wallet.balance} ETH
                    </Typography>
                    <Typography variant="body2" sx={{ opacity: 0.8 }}>
                      ≈ ${(parseFloat(wallet.balance) * 2250).toFixed(2)}
                    </Typography>
                  </Box>
                  <AccountBalanceWallet sx={{ fontSize: 48, opacity: 0.3 }} />
                </Box>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={12} sm={6} md={3}>
            <Card sx={{ bgcolor: '#4CAF50', color: 'white' }}>
              <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <Box>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>
                      MEV Saved
                    </Typography>
                    <Typography variant="h4" sx={{ fontWeight: 'bold' }}>
                      {wallet.mevSaved} ETH
                    </Typography>
                    <Typography variant="body2" sx={{ opacity: 0.8 }}>
                      ≈ ${(parseFloat(wallet.mevSaved) * 2250).toFixed(2)}
                    </Typography>
                  </Box>
                  <Shield sx={{ fontSize: 48, opacity: 0.3 }} />
                </Box>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={12} sm={6} md={3}>
            <Card sx={{ bgcolor: '#FF9800', color: 'white' }}>
              <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <Box>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>
                      Pending Rewards
                    </Typography>
                    <Typography variant="h4" sx={{ fontWeight: 'bold' }}>
                      {wallet.pendingRewards} ETH
                    </Typography>
                    <Button
                      size="small"
                      sx={{ color: 'white', borderColor: 'white', mt: 1 }}
                      variant="outlined"
                    >
                      Claim
                    </Button>
                  </Box>
                  <EmojiEvents sx={{ fontSize: 48, opacity: 0.3 }} />
                </Box>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={12} sm={6} md={3}>
            <Card sx={{ bgcolor: '#9C27B0', color: 'white' }}>
              <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <Box>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>
                      Protection Score
                    </Typography>
                    <Typography variant="h4" sx={{ fontWeight: 'bold' }}>
                      98.5%
                    </Typography>
                    <Typography variant="body2" sx={{ opacity: 0.8 }}>
                      {wallet.transactionCount} Protected
                    </Typography>
                  </Box>
                  <Security sx={{ fontSize: 48, opacity: 0.3 }} />
                </Box>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Paper>

      {/* Main Content Tabs */}
      <Paper elevation={2} sx={{ mb: 3 }}>
        <Tabs value={selectedTab} onChange={(e, v) => setSelectedTab(v)} sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <Tab label="Overview" icon={<QueryStats />} iconPosition="start" />
          <Tab label="Transactions" icon={<History />} iconPosition="start" />
          <Tab label="Protection" icon={<Shield />} iconPosition="start" />
          <Tab label="Rewards" icon={<EmojiEvents />} iconPosition="start" />
          <Tab label="Settings" icon={<Settings />} iconPosition="start" />
        </Tabs>

        {/* Overview Tab */}
        {selectedTab === 0 && (
          <Box sx={{ p: 3 }}>
            <Grid container spacing={3}>
              {/* MEV Savings Chart */}
              <Grid item xs={12} md={8}>
                <Card>
                  <CardContent>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 2 }}>
                      <Typography variant="h6">MEV Savings Over Time</Typography>
                      <FormControl size="small" sx={{ minWidth: 100 }}>
                        <Select value={timeRange} onChange={(e) => setTimeRange(e.target.value)}>
                          <MenuItem value="24h">24h</MenuItem>
                          <MenuItem value="7d">7 Days</MenuItem>
                          <MenuItem value="30d">30 Days</MenuItem>
                          <MenuItem value="all">All Time</MenuItem>
                        </Select>
                      </FormControl>
                    </Box>
                    <Line data={savingsChartData} options={{ responsive: true, maintainAspectRatio: false }} height={300} />
                  </CardContent>
                </Card>
              </Grid>

              {/* Protection Stats */}
              <Grid item xs={12} md={4}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 2 }}>Protection Stats</Typography>
                    <List>
                      <ListItem>
                        <ListItemAvatar>
                          <Avatar sx={{ bgcolor: '#4CAF50' }}>
                            <Shield />
                          </Avatar>
                        </ListItemAvatar>
                        <ListItemText
                          primary={protectionStats.totalProtected}
                          secondary="Total Protected"
                        />
                      </ListItem>
                      <ListItem>
                        <ListItemAvatar>
                          <Avatar sx={{ bgcolor: '#f44336' }}>
                            <Block />
                          </Avatar>
                        </ListItemAvatar>
                        <ListItemText
                          primary={protectionStats.mevPrevented}
                          secondary="MEV Attacks Prevented"
                        />
                      </ListItem>
                      <ListItem>
                        <ListItemAvatar>
                          <Avatar sx={{ bgcolor: '#2196F3' }}>
                            <AttachMoney />
                          </Avatar>
                        </ListItemAvatar>
                        <ListItemText
                          primary={`${protectionStats.totalSaved} ETH`}
                          secondary="Total Saved"
                        />
                      </ListItem>
                      <ListItem>
                        <ListItemAvatar>
                          <Avatar sx={{ bgcolor: '#FF9800' }}>
                            <TrendingUp />
                          </Avatar>
                        </ListItemAvatar>
                        <ListItemText
                          primary={`${protectionStats.successRate}%`}
                          secondary="Success Rate"
                        />
                      </ListItem>
                    </List>
                  </CardContent>
                </Card>
              </Grid>

              {/* Recent Activity */}
              <Grid item xs={12}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 2 }}>Recent Activity</Typography>
                    <TableContainer>
                      <Table>
                        <TableHead>
                          <TableRow>
                            <TableCell>Type</TableCell>
                            <TableCell>Amount</TableCell>
                            <TableCell>To/From</TableCell>
                            <TableCell>Protection</TableCell>
                            <TableCell>MEV Saved</TableCell>
                            <TableCell>Status</TableCell>
                            <TableCell>Time</TableCell>
                          </TableRow>
                        </TableHead>
                        <TableBody>
                          {recentTransactions.map((tx) => (
                            <TableRow key={tx.id}>
                              <TableCell>
                                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                                  {tx.type === 'send' && <ArrowUpward sx={{ mr: 1, color: '#f44336' }} />}
                                  {tx.type === 'receive' && <ArrowDownward sx={{ mr: 1, color: '#4CAF50' }} />}
                                  {tx.type === 'swap' && <SwapHoriz sx={{ mr: 1, color: '#2196F3' }} />}
                                  <Typography variant="body2" sx={{ textTransform: 'capitalize' }}>
                                    {tx.type}
                                  </Typography>
                                </Box>
                              </TableCell>
                              <TableCell>
                                <Typography variant="body2">
                                  {tx.amount} {tx.token}
                                </Typography>
                              </TableCell>
                              <TableCell>
                                <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                                  {tx.to.substring(0, 6)}...{tx.to.substring(38)}
                                </Typography>
                              </TableCell>
                              <TableCell>
                                {tx.mevProtected && (
                                  <Tooltip title={tx.protectionType}>
                                    <Chip
                                      icon={<Shield />}
                                      label="Protected"
                                      color="success"
                                      size="small"
                                    />
                                  </Tooltip>
                                )}
                              </TableCell>
                              <TableCell>
                                {parseFloat(tx.mevSaved) > 0 ? (
                                  <Typography variant="body2" color="success.main" sx={{ fontWeight: 'bold' }}>
                                    ${tx.mevSaved}
                                  </Typography>
                                ) : (
                                  '-'
                                )}
                              </TableCell>
                              <TableCell>
                                <Chip
                                  label={tx.status}
                                  color={
                                    tx.status === 'success' ? 'success' :
                                    tx.status === 'pending' ? 'warning' : 'error'
                                  }
                                  size="small"
                                />
                              </TableCell>
                              <TableCell>
                                <Typography variant="body2" color="textSecondary">
                                  {Math.round((Date.now() - tx.timestamp.getTime()) / 60000)} min ago
                                </Typography>
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </TableContainer>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          </Box>
        )}

        {/* Protection Tab */}
        {selectedTab === 2 && (
          <Box sx={{ p: 3 }}>
            <Grid container spacing={3}>
              {/* Protection Level Selector */}
              <Grid item xs={12}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 3 }}>Choose Your Protection Level</Typography>
                    <Grid container spacing={3}>
                      {protectionLevels.map((level) => (
                        <Grid item xs={12} md={4} key={level.level}>
                          <Card
                            sx={{
                              border: protectionLevel === level.level ? `2px solid ${level.color}` : '1px solid #e0e0e0',
                              cursor: 'pointer',
                              transition: 'all 0.3s',
                              '&:hover': {
                                boxShadow: 3,
                              },
                            }}
                            onClick={() => setProtectionLevel(level.level as any)}
                          >
                            <CardContent>
                              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                                <Typography variant="h5" sx={{ fontWeight: 'bold', color: level.color }}>
                                  {level.name}
                                </Typography>
                                {protectionLevel === level.level && (
                                  <CheckCircle sx={{ color: level.color }} />
                                )}
                              </Box>
                              <Typography variant="body2" color="textSecondary" sx={{ mb: 2 }}>
                                {level.description}
                              </Typography>
                              <Divider sx={{ my: 2 }} />
                              <List dense>
                                {level.features.map((feature, idx) => (
                                  <ListItem key={idx} sx={{ pl: 0 }}>
                                    <ListItemText
                                      primary={
                                        <Typography variant="body2">
                                          ✓ {feature}
                                        </Typography>
                                      }
                                    />
                                  </ListItem>
                                ))}
                              </List>
                              <Box sx={{ mt: 2, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                <Typography variant="h6" sx={{ fontWeight: 'bold' }}>
                                  {level.cost}
                                </Typography>
                                <Button
                                  variant={protectionLevel === level.level ? 'contained' : 'outlined'}
                                  size="small"
                                  sx={{
                                    bgcolor: protectionLevel === level.level ? level.color : 'transparent',
                                    borderColor: level.color,
                                    color: protectionLevel === level.level ? 'white' : level.color,
                                  }}
                                >
                                  {protectionLevel === level.level ? 'Current' : 'Select'}
                                </Button>
                              </Box>
                            </CardContent>
                          </Card>
                        </Grid>
                      ))}
                    </Grid>
                  </CardContent>
                </Card>
              </Grid>

              {/* Protection Analytics */}
              <Grid item xs={12} md={6}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 2 }}>Protection Distribution</Typography>
                    <Doughnut data={protectionDistribution} options={{ responsive: true, maintainAspectRatio: false }} height={250} />
                  </CardContent>
                </Card>
              </Grid>

              <Grid item xs={12} md={6}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 2 }}>Protection Trend</Typography>
                    <Bar data={protectionTrend} options={{ responsive: true, maintainAspectRatio: false }} height={250} />
                  </CardContent>
                </Card>
              </Grid>

              {/* Protection Features */}
              <Grid item xs={12}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 3 }}>Active Protection Features</Typography>
                    <Grid container spacing={2}>
                      <Grid item xs={12} sm={6} md={3}>
                        <Box sx={{ textAlign: 'center', p: 2 }}>
                          <Avatar sx={{ width: 80, height: 80, bgcolor: '#4CAF50', mx: 'auto', mb: 2 }}>
                            <Shield sx={{ fontSize: 40 }} />
                          </Avatar>
                          <Typography variant="h6">Sandwich Protection</Typography>
                          <Typography variant="body2" color="textSecondary">
                            Prevents sandwich attacks on DEX trades
                          </Typography>
                          <Chip label="Active" color="success" sx={{ mt: 1 }} />
                        </Box>
                      </Grid>
                      <Grid item xs={12} sm={6} md={3}>
                        <Box sx={{ textAlign: 'center', p: 2 }}>
                          <Avatar sx={{ width: 80, height: 80, bgcolor: '#2196F3', mx: 'auto', mb: 2 }}>
                            <Speed sx={{ fontSize: 40 }} />
                          </Avatar>
                          <Typography variant="h6">Frontrun Defense</Typography>
                          <Typography variant="body2" color="textSecondary">
                            Blocks frontrunning attempts
                          </Typography>
                          <Chip label="Active" color="success" sx={{ mt: 1 }} />
                        </Box>
                      </Grid>
                      <Grid item xs={12} sm={6} md={3}>
                        <Box sx={{ textAlign: 'center', p: 2 }}>
                          <Avatar sx={{ width: 80, height: 80, bgcolor: '#FF9800', mx: 'auto', mb: 2 }}>
                            <Lock sx={{ fontSize: 40 }} />
                          </Avatar>
                          <Typography variant="h6">Private Mempool</Typography>
                          <Typography variant="body2" color="textSecondary">
                            Transactions bypass public mempool
                          </Typography>
                          <Chip label="Active" color="success" sx={{ mt: 1 }} />
                        </Box>
                      </Grid>
                      <Grid item xs={12} sm={6} md={3}>
                        <Box sx={{ textAlign: 'center', p: 2 }}>
                          <Avatar sx={{ width: 80, height: 80, bgcolor: '#9C27B0', mx: 'auto', mb: 2 }}>
                            <Timer sx={{ fontSize: 40 }} />
                          </Avatar>
                          <Typography variant="h6">Fair Ordering</Typography>
                          <Typography variant="body2" color="textSecondary">
                            VDF-based transaction ordering
                          </Typography>
                          <Chip label={protectionLevel === 'maximum' ? 'Active' : 'Upgrade'} 
                                color={protectionLevel === 'maximum' ? 'success' : 'default'} 
                                sx={{ mt: 1 }} />
                        </Box>
                      </Grid>
                    </Grid>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          </Box>
        )}

        {/* Rewards Tab */}
        {selectedTab === 3 && (
          <Box sx={{ p: 3 }}>
            <Grid container spacing={3}>
              {/* Rewards Summary */}
              <Grid item xs={12}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 3 }}>Your MEV Rewards</Typography>
                    <Grid container spacing={3}>
                      <Grid item xs={12} md={4}>
                        <Box sx={{ textAlign: 'center' }}>
                          <Typography variant="h3" color="primary">
                            {wallet.pendingRewards} ETH
                          </Typography>
                          <Typography variant="body1" color="textSecondary" sx={{ mb: 2 }}>
                            Available to Claim
                          </Typography>
                          <Button variant="contained" color="success" size="large" fullWidth>
                            Claim Rewards
                          </Button>
                        </Box>
                      </Grid>
                      <Grid item xs={12} md={4}>
                        <Box sx={{ textAlign: 'center' }}>
                          <Typography variant="h3" color="primary">
                            {wallet.mevSaved} ETH
                          </Typography>
                          <Typography variant="body1" color="textSecondary">
                            Total Lifetime Rewards
                          </Typography>
                        </Box>
                      </Grid>
                      <Grid item xs={12} md={4}>
                        <Box sx={{ textAlign: 'center' }}>
                          <Typography variant="h3" color="primary">
                            12.5%
                          </Typography>
                          <Typography variant="body1" color="textSecondary">
                            Average APY
                          </Typography>
                        </Box>
                      </Grid>
                    </Grid>
                  </CardContent>
                </Card>
              </Grid>

              {/* Rewards History */}
              <Grid item xs={12}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" sx={{ mb: 2 }}>Rewards History</Typography>
                    <TableContainer>
                      <Table>
                        <TableHead>
                          <TableRow>
                            <TableCell>Date</TableCell>
                            <TableCell>Type</TableCell>
                            <TableCell>Transaction</TableCell>
                            <TableCell align="right">MEV Saved</TableCell>
                            <TableCell align="right">Reward</TableCell>
                            <TableCell>Status</TableCell>
                          </TableRow>
                        </TableHead>
                        <TableBody>
                          {[...Array(5)].map((_, index) => (
                            <TableRow key={index}>
                              <TableCell>
                                {new Date(Date.now() - index * 86400000).toLocaleDateString()}
                              </TableCell>
                              <TableCell>
                                <Chip label="MEV Protection" size="small" color="primary" />
                              </TableCell>
                              <TableCell>
                                <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                                  0x{Math.random().toString(16).substr(2, 8)}...
                                </Typography>
                              </TableCell>
                              <TableCell align="right">
                                ${(Math.random() * 100).toFixed(2)}
                              </TableCell>
                              <TableCell align="right">
                                {(Math.random() * 0.01).toFixed(4)} ETH
                              </TableCell>
                              <TableCell>
                                <Chip 
                                  label={index === 0 ? 'Pending' : 'Claimed'} 
                                  color={index === 0 ? 'warning' : 'success'} 
                                  size="small" 
                                />
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </TableContainer>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          </Box>
        )}
      </Paper>

      {/* Send Transaction Dialog */}
      <Dialog open={showSendDialog} onClose={() => setShowSendDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Send Transaction with MEV Protection</DialogTitle>
        <DialogContent>
          <Box sx={{ mt: 2 }}>
            <TextField
              fullWidth
              label="Recipient Address"
              variant="outlined"
              sx={{ mb: 2 }}
              placeholder="0x..."
            />
            <TextField
              fullWidth
              label="Amount"
              variant="outlined"
              sx={{ mb: 2 }}
              placeholder="0.0"
              InputProps={{
                endAdornment: <Typography>ETH</Typography>,
              }}
            />
            <FormControl fullWidth sx={{ mb: 2 }}>
              <InputLabel>Protection Level</InputLabel>
              <Select value={protectionLevel} label="Protection Level">
                <MenuItem value="basic">Basic (Free)</MenuItem>
                <MenuItem value="standard">Standard (0.05% fee)</MenuItem>
                <MenuItem value="maximum">Maximum (0.1% fee)</MenuItem>
              </Select>
            </FormControl>
            <Alert severity="info" sx={{ mb: 2 }}>
              Your transaction will be protected from MEV attacks using {protectionLevel} protection.
            </Alert>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowSendDialog(false)}>Cancel</Button>
          <Button variant="contained" color="success">
            Send Protected
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default UserDashboard;