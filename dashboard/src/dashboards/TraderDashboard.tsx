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
  TextField,
  InputAdornment,
  Avatar,
  IconButton,
  Alert,
  List,
  ListItem,
  ListItemText,
  ListItemAvatar,
  Divider,
  LinearProgress,
  FormControl,
  Select,
  MenuItem,
  InputLabel,
  Tabs,
  Tab,
  ToggleButton,
  ToggleButtonGroup,
} from '@mui/material';
import {
  SwapHoriz,
  TrendingUp,
  TrendingDown,
  AttachMoney,
  Shield,
  Warning,
  CheckCircle,
  Refresh,
  Settings,
  Timeline,
  AccountBalanceWallet,
  Speed,
  Search,
  FilterList,
  ShowChart,
  CandlestickChart,
} from '@mui/icons-material';

interface Portfolio {
  totalValue: string;
  dailyPnL: string;
  dailyPnLPercent: number;
  protectedTrades: number;
  mevSaved: string;
}

interface Trade {
  id: string;
  pair: string;
  type: 'buy' | 'sell';
  amount: string;
  price: string;
  total: string;
  mevProtected: boolean;
  status: 'completed' | 'pending' | 'failed';
  timestamp: Date;
}

const TraderDashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);
  const [tradeType, setTradeType] = useState<'buy' | 'sell'>('buy');
  const [selectedPair, setSelectedPair] = useState('ETH/USDC');
  const [amount, setAmount] = useState('');
  const [slippage, setSlippage] = useState('0.5');

  const portfolio: Portfolio = {
    totalValue: '$124,580',
    dailyPnL: '+$2,450',
    dailyPnLPercent: 2.01,
    protectedTrades: 89,
    mevSaved: '$1,240',
  };

  const recentTrades: Trade[] = [
    {
      id: '1',
      pair: 'ETH/USDC',
      type: 'buy',
      amount: '2.5 ETH',
      price: '$2,150',
      total: '$5,375',
      mevProtected: true,
      status: 'completed',
      timestamp: new Date(),
    },
    {
      id: '2',
      pair: 'WBTC/ETH',
      type: 'sell',
      amount: '0.15 WBTC',
      price: '15.2 ETH',
      total: '2.28 ETH',
      mevProtected: true,
      status: 'completed',
      timestamp: new Date(Date.now() - 3600000),
    },
    {
      id: '3',
      pair: 'UNI/USDC',
      type: 'buy',
      amount: '500 UNI',
      price: '$6.25',
      total: '$3,125',
      mevProtected: false,
      status: 'completed',
      timestamp: new Date(Date.now() - 7200000),
    },
  ];

  const topPairs = [
    { pair: 'ETH/USDC', price: '$2,152.45', change: 2.3, volume: '$45.2M' },
    { pair: 'WBTC/ETH', price: '15.245', change: -0.8, volume: '$12.8M' },
    { pair: 'UNI/USDC', price: '$6.28', change: 4.2, volume: '$8.5M' },
    { pair: 'LINK/ETH', price: '0.0032', change: 1.5, volume: '$6.2M' },
  ];

  const handleTradeTypeChange = (event: React.MouseEvent<HTMLElement>, newType: string | null) => {
    if (newType !== null) {
      setTradeType(newType as 'buy' | 'sell');
    }
  };

  return (
    <Box sx={{ bgcolor: 'background.default', minHeight: '100vh', py: 3 }}>
      <Container maxWidth="xl">
        {/* Header */}
        <Box sx={{ mb: 4 }}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
            <Typography variant="h4" sx={{ fontWeight: 700, display: 'flex', alignItems: 'center' }}>
              <SwapHoriz sx={{ mr: 2, fontSize: 40, color: 'primary.main' }} />
              Trader Dashboard
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <Chip
                icon={<Shield />}
                label="MEV Protection Active"
                color="success"
                variant="outlined"
              />
              <IconButton>
                <Refresh />
              </IconButton>
              <IconButton>
                <Settings />
              </IconButton>
            </Box>
          </Box>
          
          <Alert severity="success" sx={{ mb: 2 }}>
            Welcome, Trader! Your trades are protected from MEV attacks. You've saved {portfolio.mevSaved} from MEV protection.
          </Alert>
        </Box>

        {/* Portfolio Metrics */}
        <Grid container spacing={3} sx={{ mb: 3 }}>
          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <AccountBalanceWallet color="primary" sx={{ mr: 1 }} />
                  <Typography color="text.secondary" variant="body2">
                    Portfolio Value
                  </Typography>
                </Box>
                <Typography variant="h4" sx={{ fontWeight: 600 }}>
                  {portfolio.totalValue}
                </Typography>
                <Box sx={{ display: 'flex', alignItems: 'center', mt: 1 }}>
                  <TrendingUp sx={{ fontSize: 16, color: 'success.main', mr: 0.5 }} />
                  <Typography variant="body2" color="success.main">
                    {portfolio.dailyPnL} ({portfolio.dailyPnLPercent}%)
                  </Typography>
                </Box>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <Shield color="success" sx={{ mr: 1 }} />
                  <Typography color="text.secondary" variant="body2">
                    Protected Trades
                  </Typography>
                </Box>
                <Typography variant="h4" sx={{ fontWeight: 600 }}>
                  {portfolio.protectedTrades}
                </Typography>
                <LinearProgress
                  variant="determinate"
                  value={89}
                  sx={{ mt: 1, height: 6, borderRadius: 3 }}
                  color="success"
                />
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <AttachMoney color="info" sx={{ mr: 1 }} />
                  <Typography color="text.secondary" variant="body2">
                    MEV Saved
                  </Typography>
                </Box>
                <Typography variant="h4" sx={{ fontWeight: 600 }}>
                  {portfolio.mevSaved}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  Lifetime savings
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <Speed color="warning" sx={{ mr: 1 }} />
                  <Typography color="text.secondary" variant="body2">
                    Avg Execution
                  </Typography>
                </Box>
                <Typography variant="h4" sx={{ fontWeight: 600 }}>
                  42ms
                </Typography>
                <Typography variant="body2" color="success.main" sx={{ mt: 1 }}>
                  Fast execution
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>

        {/* Main Content */}
        <Grid container spacing={3}>
          {/* Trading Panel */}
          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 3, height: '100%' }}>
              <Typography variant="h6" gutterBottom>
                Quick Trade
              </Typography>
              
              <ToggleButtonGroup
                value={tradeType}
                exclusive
                onChange={handleTradeTypeChange}
                fullWidth
                sx={{ mb: 2 }}
              >
                <ToggleButton value="buy" color="success">
                  Buy
                </ToggleButton>
                <ToggleButton value="sell" color="error">
                  Sell
                </ToggleButton>
              </ToggleButtonGroup>

              <FormControl fullWidth sx={{ mb: 2 }}>
                <InputLabel>Trading Pair</InputLabel>
                <Select
                  value={selectedPair}
                  onChange={(e) => setSelectedPair(e.target.value)}
                  label="Trading Pair"
                >
                  <MenuItem value="ETH/USDC">ETH/USDC</MenuItem>
                  <MenuItem value="WBTC/ETH">WBTC/ETH</MenuItem>
                  <MenuItem value="UNI/USDC">UNI/USDC</MenuItem>
                  <MenuItem value="LINK/ETH">LINK/ETH</MenuItem>
                </Select>
              </FormControl>

              <TextField
                fullWidth
                label="Amount"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                sx={{ mb: 2 }}
                InputProps={{
                  endAdornment: <InputAdornment position="end">ETH</InputAdornment>,
                }}
              />

              <TextField
                fullWidth
                label="Slippage Tolerance"
                value={slippage}
                onChange={(e) => setSlippage(e.target.value)}
                sx={{ mb: 2 }}
                InputProps={{
                  endAdornment: <InputAdornment position="end">%</InputAdornment>,
                }}
              />

              <Alert severity="info" sx={{ mb: 2 }}>
                <Typography variant="body2">
                  Estimated output: 2,145.50 USDC
                </Typography>
                <Typography variant="caption">
                  MEV Protection: Active
                </Typography>
              </Alert>

              <Button
                variant="contained"
                fullWidth
                size="large"
                color={tradeType === 'buy' ? 'success' : 'error'}
                startIcon={<Shield />}
              >
                {tradeType === 'buy' ? 'Protected Buy' : 'Protected Sell'}
              </Button>
            </Paper>
          </Grid>

          {/* Market Overview */}
          <Grid item xs={12} md={8}>
            <Paper sx={{ mb: 3 }}>
              <Tabs
                value={activeTab}
                onChange={(_, v) => setActiveTab(v)}
                sx={{ borderBottom: 1, borderColor: 'divider', px: 2 }}
              >
                <Tab label="Top Pairs" icon={<ShowChart />} iconPosition="start" />
                <Tab label="Recent Trades" icon={<Timeline />} iconPosition="start" />
                <Tab label="Price Alerts" icon={<Warning />} iconPosition="start" />
              </Tabs>

              {/* Top Pairs Tab */}
              {activeTab === 0 && (
                <Box sx={{ p: 2 }}>
                  <TableContainer>
                    <Table>
                      <TableHead>
                        <TableRow>
                          <TableCell>Pair</TableCell>
                          <TableCell align="right">Price</TableCell>
                          <TableCell align="right">24h Change</TableCell>
                          <TableCell align="right">Volume</TableCell>
                          <TableCell align="center">Action</TableCell>
                        </TableRow>
                      </TableHead>
                      <TableBody>
                        {topPairs.map((pair) => (
                          <TableRow key={pair.pair} hover>
                            <TableCell>
                              <Typography variant="body2" sx={{ fontWeight: 600 }}>
                                {pair.pair}
                              </Typography>
                            </TableCell>
                            <TableCell align="right">{pair.price}</TableCell>
                            <TableCell align="right">
                              <Chip
                                label={`${pair.change > 0 ? '+' : ''}${pair.change}%`}
                                size="small"
                                color={pair.change > 0 ? 'success' : 'error'}
                                icon={pair.change > 0 ? <TrendingUp /> : <TrendingDown />}
                              />
                            </TableCell>
                            <TableCell align="right">{pair.volume}</TableCell>
                            <TableCell align="center">
                              <Button size="small" variant="outlined">
                                Trade
                              </Button>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </TableContainer>
                </Box>
              )}

              {/* Recent Trades Tab */}
              {activeTab === 1 && (
                <Box sx={{ p: 2 }}>
                  <List>
                    {recentTrades.map((trade, index) => (
                      <React.Fragment key={trade.id}>
                        <ListItem>
                          <ListItemAvatar>
                            <Avatar sx={{ bgcolor: trade.type === 'buy' ? 'success.light' : 'error.light' }}>
                              <SwapHoriz />
                            </Avatar>
                          </ListItemAvatar>
                          <ListItemText
                            primary={
                              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                                <Typography variant="body1">
                                  {trade.type === 'buy' ? 'Bought' : 'Sold'} {trade.amount}
                                </Typography>
                                {trade.mevProtected && (
                                  <Chip
                                    icon={<Shield />}
                                    label="Protected"
                                    size="small"
                                    color="success"
                                  />
                                )}
                              </Box>
                            }
                            secondary={
                              <Typography variant="body2" color="text.secondary">
                                {trade.pair} • {trade.price} • {new Date(trade.timestamp).toLocaleString()}
                              </Typography>
                            }
                          />
                          <Typography variant="body2" sx={{ fontWeight: 600 }}>
                            {trade.total}
                          </Typography>
                        </ListItem>
                        {index < recentTrades.length - 1 && <Divider variant="inset" component="li" />}
                      </React.Fragment>
                    ))}
                  </List>
                </Box>
              )}

              {/* Price Alerts Tab */}
              {activeTab === 2 && (
                <Box sx={{ p: 2 }}>
                  <Alert severity="warning" sx={{ mb: 2 }}>
                    Set price alerts to never miss trading opportunities
                  </Alert>
                  
                  <List>
                    <ListItem>
                      <ListItemAvatar>
                        <Avatar sx={{ bgcolor: 'warning.light' }}>
                          <Warning />
                        </Avatar>
                      </ListItemAvatar>
                      <ListItemText
                        primary="ETH/USDC Alert"
                        secondary="Alert when price goes above $2,200"
                      />
                      <Chip label="Active" color="success" size="small" />
                    </ListItem>
                    <Divider variant="inset" component="li" />
                    <ListItem>
                      <ListItemAvatar>
                        <Avatar sx={{ bgcolor: 'info.light' }}>
                          <Warning />
                        </Avatar>
                      </ListItemAvatar>
                      <ListItemText
                        primary="WBTC/ETH Alert"
                        secondary="Alert when price drops below 15.0"
                      />
                      <Chip label="Active" color="success" size="small" />
                    </ListItem>
                  </List>
                  
                  <Button variant="outlined" fullWidth sx={{ mt: 2 }}>
                    Add Price Alert
                  </Button>
                </Box>
              )}
            </Paper>
          </Grid>
        </Grid>
      </Container>
    </Box>
  );
};

export default TraderDashboard;