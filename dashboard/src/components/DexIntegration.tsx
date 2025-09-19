import React, { useState, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Button,
  Chip,
  Avatar,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Alert,
  LinearProgress,
  Tab,
  Tabs,
  IconButton,
  Tooltip,
  Badge,
  TextField,
  InputAdornment,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
} from '@mui/material';
import {
  SwapHoriz,
  Shield,
  Warning,
  CheckCircle,
  TrendingUp,
  Speed,
  AttachMoney,
  Refresh,
  Settings,
  FilterList,
  Search,
  AccountBalance,
  Security,
  Timeline,
  Error,
  MonitorHeart,
} from '@mui/icons-material';

interface PoolData {
  address: string;
  token0: string;
  token1: string;
  liquidity: string;
  volume24h: string;
  fee: number;
  mevRisk: 'low' | 'medium' | 'high';
  protected: boolean;
}

interface SwapTransaction {
  hash: string;
  dex: 'Uniswap' | 'PancakeSwap';
  pair: string;
  amount: string;
  protected: boolean;
  mevSaved: string;
  status: 'success' | 'pending' | 'failed';
  timestamp: Date;
}

interface MEVMetrics {
  totalProtectedVolume: string;
  mevPrevented: string;
  successRate: number;
  activeProtections: number;
}

const DexIntegration: React.FC = () => {
  const [selectedTab, setSelectedTab] = useState(0);
  const [selectedDex, setSelectedDex] = useState<'all' | 'uniswap' | 'pancakeswap'>('all');
  const [searchTerm, setSearchTerm] = useState('');
  const [loading, setLoading] = useState(false);
  const [autoProtect, setAutoProtect] = useState(true);

  // Mock data for pools
  const pools: PoolData[] = [
    {
      address: '0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8',
      token0: 'USDC',
      token1: 'WETH',
      liquidity: '125.4M',
      volume24h: '45.2M',
      fee: 3000,
      mevRisk: 'high',
      protected: true,
    },
    {
      address: '0x4e68Ccd3E89f51C3074ca5072bbAC773960dFa36',
      token0: 'WETH',
      token1: 'USDT',
      liquidity: '98.7M',
      volume24h: '32.1M',
      fee: 3000,
      mevRisk: 'medium',
      protected: true,
    },
    {
      address: '0x1d42064Fc4Beb5F8aAF85F4617AE8F8aAF85F4617AE3',
      token0: 'CAKE',
      token1: 'BNB',
      liquidity: '45.2M',
      volume24h: '18.5M',
      fee: 2500,
      mevRisk: 'high',
      protected: true,
    },
    {
      address: '0x2d42064Fc4Beb5F8aAF85F4617AE82d42064Fc4Beb5',
      token0: 'BUSD',
      token1: 'BNB',
      liquidity: '67.8M',
      volume24h: '24.3M',
      fee: 2500,
      mevRisk: 'low',
      protected: false,
    },
  ];

  // Mock data for recent swaps
  const recentSwaps: SwapTransaction[] = [
    {
      hash: '0x1234...5678',
      dex: 'Uniswap',
      pair: 'USDC/WETH',
      amount: '25,000 USDC',
      protected: true,
      mevSaved: '$124',
      status: 'success',
      timestamp: new Date(),
    },
    {
      hash: '0x8765...4321',
      dex: 'PancakeSwap',
      pair: 'CAKE/BNB',
      amount: '1,500 CAKE',
      protected: true,
      mevSaved: '$89',
      status: 'success',
      timestamp: new Date(),
    },
    {
      hash: '0xabcd...efgh',
      dex: 'Uniswap',
      pair: 'WETH/USDT',
      amount: '10.5 WETH',
      protected: false,
      mevSaved: '$0',
      status: 'pending',
      timestamp: new Date(),
    },
  ];

  const metrics: MEVMetrics = {
    totalProtectedVolume: '$124.5M',
    mevPrevented: '$2.3M',
    successRate: 98.5,
    activeProtections: 156,
  };

  const handleRefresh = () => {
    setLoading(true);
    setTimeout(() => setLoading(false), 2000);
  };

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'high': return 'error';
      case 'medium': return 'warning';
      case 'low': return 'success';
      default: return 'default';
    }
  };

  const getDexLogo = (dex: string) => {
    if (dex.toLowerCase().includes('uniswap')) {
      return '🦄';
    } else if (dex.toLowerCase().includes('pancake')) {
      return '🥞';
    }
    return '💱';
  };

  return (
    <Box>
      {/* Header */}
      <Box sx={{ mb: 3, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Typography variant="h5" sx={{ fontWeight: 600 }}>
          DEX Integration & MEV Protection
        </Typography>
        <Box sx={{ display: 'flex', gap: 2 }}>
          <FormControl size="small" sx={{ minWidth: 150 }}>
            <InputLabel>DEX Filter</InputLabel>
            <Select
              value={selectedDex}
              onChange={(e) => setSelectedDex(e.target.value as any)}
              label="DEX Filter"
            >
              <MenuItem value="all">All DEXs</MenuItem>
              <MenuItem value="uniswap">Uniswap V3</MenuItem>
              <MenuItem value="pancakeswap">PancakeSwap V3</MenuItem>
            </Select>
          </FormControl>
          <Button
            variant="outlined"
            startIcon={<Refresh />}
            onClick={handleRefresh}
            disabled={loading}
          >
            Refresh
          </Button>
          <Button
            variant="contained"
            color={autoProtect ? 'success' : 'default'}
            startIcon={<Shield />}
            onClick={() => setAutoProtect(!autoProtect)}
          >
            {autoProtect ? 'Auto-Protect ON' : 'Auto-Protect OFF'}
          </Button>
        </Box>
      </Box>

      {/* Metrics Cards */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <AccountBalance color="primary" sx={{ mr: 1 }} />
                <Typography color="text.secondary" variant="body2">
                  Protected Volume
                </Typography>
              </Box>
              <Typography variant="h4" sx={{ fontWeight: 600 }}>
                {metrics.totalProtectedVolume}
              </Typography>
              <Chip
                label="+12.5% 24h"
                size="small"
                color="success"
                sx={{ mt: 1 }}
              />
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Shield color="success" sx={{ mr: 1 }} />
                <Typography color="text.secondary" variant="body2">
                  MEV Prevented
                </Typography>
              </Box>
              <Typography variant="h4" sx={{ fontWeight: 600 }}>
                {metrics.mevPrevented}
              </Typography>
              <Chip
                label="+8.3% 24h"
                size="small"
                color="success"
                sx={{ mt: 1 }}
              />
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <CheckCircle color="info" sx={{ mr: 1 }} />
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
              />
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Speed color="warning" sx={{ mr: 1 }} />
                <Typography color="text.secondary" variant="body2">
                  Active Protections
                </Typography>
              </Box>
              <Typography variant="h4" sx={{ fontWeight: 600 }}>
                {metrics.activeProtections}
              </Typography>
              <Chip
                label="Real-time"
                size="small"
                color="warning"
                sx={{ mt: 1 }}
                icon={<MonitorHeart />}
              />
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Tabs */}
      <Paper sx={{ mb: 3 }}>
        <Tabs
          value={selectedTab}
          onChange={(_, v) => setSelectedTab(v)}
          sx={{ borderBottom: 1, borderColor: 'divider' }}
        >
          <Tab label="Monitored Pools" icon={<AccountBalance />} iconPosition="start" />
          <Tab label="Recent Swaps" icon={<SwapHoriz />} iconPosition="start" />
          <Tab label="MEV Analytics" icon={<Timeline />} iconPosition="start" />
          <Tab label="Protection Settings" icon={<Settings />} iconPosition="start" />
        </Tabs>

        {/* Monitored Pools Tab */}
        {selectedTab === 0 && (
          <Box sx={{ p: 3 }}>
            <Box sx={{ mb: 2 }}>
              <TextField
                size="small"
                placeholder="Search pools..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                InputProps={{
                  startAdornment: (
                    <InputAdornment position="start">
                      <Search />
                    </InputAdornment>
                  ),
                }}
                sx={{ width: 300 }}
              />
            </Box>
            
            <TableContainer>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>DEX</TableCell>
                    <TableCell>Pool</TableCell>
                    <TableCell>Liquidity</TableCell>
                    <TableCell>24h Volume</TableCell>
                    <TableCell>Fee Tier</TableCell>
                    <TableCell>MEV Risk</TableCell>
                    <TableCell>Protection</TableCell>
                    <TableCell>Actions</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {pools.map((pool, index) => (
                    <TableRow key={index} hover>
                      <TableCell>
                        <Chip
                          label={pool.address.startsWith('0x8') ? 'Uniswap' : 'PancakeSwap'}
                          size="small"
                          avatar={<Avatar>{getDexLogo(pool.address.startsWith('0x8') ? 'uniswap' : 'pancake')}</Avatar>}
                        />
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2" sx={{ fontWeight: 500 }}>
                          {pool.token0}/{pool.token1}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          {pool.address.slice(0, 8)}...
                        </Typography>
                      </TableCell>
                      <TableCell>{pool.liquidity}</TableCell>
                      <TableCell>{pool.volume24h}</TableCell>
                      <TableCell>
                        <Chip label={`${pool.fee / 10000}%`} size="small" />
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={pool.mevRisk.toUpperCase()}
                          size="small"
                          color={getRiskColor(pool.mevRisk)}
                          icon={pool.mevRisk === 'high' ? <Warning /> : undefined}
                        />
                      </TableCell>
                      <TableCell>
                        {pool.protected ? (
                          <Chip
                            label="Protected"
                            size="small"
                            color="success"
                            icon={<Shield />}
                          />
                        ) : (
                          <Chip
                            label="Unprotected"
                            size="small"
                            color="default"
                          />
                        )}
                      </TableCell>
                      <TableCell>
                        <Tooltip title="Configure Protection">
                          <IconButton size="small">
                            <Settings />
                          </IconButton>
                        </Tooltip>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          </Box>
        )}

        {/* Recent Swaps Tab */}
        {selectedTab === 1 && (
          <Box sx={{ p: 3 }}>
            <TableContainer>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>Transaction</TableCell>
                    <TableCell>DEX</TableCell>
                    <TableCell>Pair</TableCell>
                    <TableCell>Amount</TableCell>
                    <TableCell>MEV Protected</TableCell>
                    <TableCell>MEV Saved</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell>Time</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {recentSwaps.map((swap, index) => (
                    <TableRow key={index} hover>
                      <TableCell>
                        <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                          {swap.hash}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={swap.dex}
                          size="small"
                          avatar={<Avatar>{getDexLogo(swap.dex)}</Avatar>}
                        />
                      </TableCell>
                      <TableCell>{swap.pair}</TableCell>
                      <TableCell>{swap.amount}</TableCell>
                      <TableCell>
                        {swap.protected ? (
                          <CheckCircle color="success" />
                        ) : (
                          <Error color="error" />
                        )}
                      </TableCell>
                      <TableCell>
                        <Typography
                          variant="body2"
                          color={swap.protected ? 'success.main' : 'text.secondary'}
                        >
                          {swap.mevSaved}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={swap.status}
                          size="small"
                          color={
                            swap.status === 'success'
                              ? 'success'
                              : swap.status === 'pending'
                              ? 'warning'
                              : 'error'
                          }
                        />
                      </TableCell>
                      <TableCell>
                        {new Date(swap.timestamp).toLocaleTimeString()}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          </Box>
        )}

        {/* MEV Analytics Tab */}
        {selectedTab === 2 && (
          <Box sx={{ p: 3 }}>
            <Alert severity="info" sx={{ mb: 3 }}>
              Real-time MEV analytics across Uniswap V3 and PancakeSwap V3
            </Alert>
            
            <Grid container spacing={3}>
              <Grid item xs={12} md={6}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" gutterBottom>
                      MEV Attack Distribution
                    </Typography>
                    <Box sx={{ mt: 2 }}>
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                        <Typography variant="body2">Sandwich Attacks</Typography>
                        <Typography variant="body2" color="error.main">45%</Typography>
                      </Box>
                      <LinearProgress variant="determinate" value={45} color="error" sx={{ mb: 2 }} />
                      
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                        <Typography variant="body2">Frontrunning</Typography>
                        <Typography variant="body2" color="warning.main">30%</Typography>
                      </Box>
                      <LinearProgress variant="determinate" value={30} color="warning" sx={{ mb: 2 }} />
                      
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                        <Typography variant="body2">Arbitrage</Typography>
                        <Typography variant="body2" color="info.main">25%</Typography>
                      </Box>
                      <LinearProgress variant="determinate" value={25} color="info" />
                    </Box>
                  </CardContent>
                </Card>
              </Grid>
              
              <Grid item xs={12} md={6}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" gutterBottom>
                      Protection Effectiveness
                    </Typography>
                    <Box sx={{ mt: 2 }}>
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                        <Typography variant="body2">Flashbots Bundle</Typography>
                        <Typography variant="body2" color="success.main">95%</Typography>
                      </Box>
                      <LinearProgress variant="determinate" value={95} color="success" sx={{ mb: 2 }} />
                      
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                        <Typography variant="body2">Private Mempool</Typography>
                        <Typography variant="body2" color="success.main">92%</Typography>
                      </Box>
                      <LinearProgress variant="determinate" value={92} color="success" sx={{ mb: 2 }} />
                      
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                        <Typography variant="body2">Dynamic Slippage</Typography>
                        <Typography variant="body2" color="success.main">88%</Typography>
                      </Box>
                      <LinearProgress variant="determinate" value={88} color="success" />
                    </Box>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          </Box>
        )}

        {/* Protection Settings Tab */}
        {selectedTab === 3 && (
          <Box sx={{ p: 3 }}>
            <Alert severity="warning" sx={{ mb: 3 }}>
              Configure MEV protection strategies for different scenarios
            </Alert>
            
            <Grid container spacing={3}>
              <Grid item xs={12} md={6}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" gutterBottom>
                      Uniswap V3 Settings
                    </Typography>
                    <Box sx={{ mt: 2 }}>
                      <FormControl fullWidth sx={{ mb: 2 }}>
                        <InputLabel>Protection Strategy</InputLabel>
                        <Select defaultValue="flashbots" label="Protection Strategy">
                          <MenuItem value="flashbots">Flashbots Bundle</MenuItem>
                          <MenuItem value="private">Private Mempool</MenuItem>
                          <MenuItem value="dynamic">Dynamic Slippage</MenuItem>
                        </Select>
                      </FormControl>
                      
                      <TextField
                        fullWidth
                        label="Max Slippage (%)"
                        defaultValue="0.5"
                        type="number"
                        sx={{ mb: 2 }}
                      />
                      
                      <TextField
                        fullWidth
                        label="Gas Price Multiplier"
                        defaultValue="1.1"
                        type="number"
                        sx={{ mb: 2 }}
                      />
                      
                      <Button variant="contained" fullWidth>
                        Update Settings
                      </Button>
                    </Box>
                  </CardContent>
                </Card>
              </Grid>
              
              <Grid item xs={12} md={6}>
                <Card>
                  <CardContent>
                    <Typography variant="h6" gutterBottom>
                      PancakeSwap V3 Settings
                    </Typography>
                    <Box sx={{ mt: 2 }}>
                      <FormControl fullWidth sx={{ mb: 2 }}>
                        <InputLabel>Protection Strategy</InputLabel>
                        <Select defaultValue="split" label="Protection Strategy">
                          <MenuItem value="split">Split Transaction</MenuItem>
                          <MenuItem value="dynamic">Dynamic Slippage</MenuItem>
                          <MenuItem value="delay">Delayed Execution</MenuItem>
                        </Select>
                      </FormControl>
                      
                      <TextField
                        fullWidth
                        label="Max Slippage (%)"
                        defaultValue="0.3"
                        type="number"
                        sx={{ mb: 2 }}
                      />
                      
                      <TextField
                        fullWidth
                        label="Split Threshold (BNB)"
                        defaultValue="10"
                        type="number"
                        sx={{ mb: 2 }}
                      />
                      
                      <Button variant="contained" fullWidth>
                        Update Settings
                      </Button>
                    </Box>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          </Box>
        )}
      </Paper>

      {/* Loading overlay */}
      {loading && (
        <Box
          sx={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            bgcolor: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 9999,
          }}
        >
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Refreshing DEX Data...
              </Typography>
              <LinearProgress />
            </CardContent>
          </Card>
        </Box>
      )}
    </Box>
  );
};

export default DexIntegration;