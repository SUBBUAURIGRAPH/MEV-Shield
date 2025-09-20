import React, { useState, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Button,
  Grid,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Alert,
  AlertTitle,
  Chip,
  LinearProgress,
  Switch,
  FormControlLabel,
  Tabs,
  Tab,
  Paper,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  ListItemSecondaryAction,
  IconButton,
  Tooltip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Stepper,
  Step,
  StepLabel,
  StepContent,
  Collapse,
  Badge,
  Avatar,
  Divider,
  CircularProgress,
} from '@mui/material';
import {
  Shield,
  Security,
  Warning,
  CheckCircle,
  Error,
  SwapHoriz,
  Speed,
  TrendingUp,
  AttachMoney,
  Timer,
  Lock,
  LockOpen,
  Visibility,
  VisibilityOff,
  FlashOn,
  NetworkCheck,
  QueryStats,
  Dangerous,
  SafetyCheck,
  WarningAmber,
  Verified,
  Block,
  Analytics,
  Timeline,
  ArrowForward,
  ArrowDropUp,
  ArrowDropDown,
  Info,
  Close,
  CheckBox,
  IndeterminateCheckBox,
} from '@mui/icons-material';
import { alpha, useTheme } from '@mui/material';
import axios from 'axios';
import { Line, Bar } from 'react-chartjs-2';

interface ProtectionStrategy {
  id: string;
  name: string;
  description: string;
  gasOverhead: number;
  successRate: number;
  avgSavings: number;
  enabled: boolean;
  icon: any;
}

interface MEVMetrics {
  sandwichAttacksBlocked: number;
  frontrunsPreventeded: number;
  backrunsDetected: number;
  totalValueProtected: string;
  averageSlippageSaved: number;
  totalTransactions: number;
}

interface DetectedThreat {
  id: string;
  type: 'sandwich' | 'frontrun' | 'backrun' | 'jit';
  severity: 'low' | 'medium' | 'high' | 'critical';
  timestamp: Date;
  potentialLoss: string;
  status: 'blocked' | 'pending' | 'executed';
  details: string;
}

const DEXProtection: React.FC = () => {
  const theme = useTheme();
  const [activeTab, setActiveTab] = useState(0);
  const [protectionEnabled, setProtectionEnabled] = useState(true);
  const [autoProtect, setAutoProtect] = useState(true);
  const [selectedStrategy, setSelectedStrategy] = useState('flashbots');
  const [slippageTolerance, setSlippageTolerance] = useState('0.5');
  const [transactionDelay, setTransactionDelay] = useState('2');
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [analyzing, setAnalyzing] = useState(false);
  const [openStrategyDialog, setOpenStrategyDialog] = useState(false);
  
  // Swap parameters
  const [tokenIn, setTokenIn] = useState('ETH');
  const [tokenOut, setTokenOut] = useState('USDC');
  const [amountIn, setAmountIn] = useState('');
  const [estimatedOut, setEstimatedOut] = useState('');
  
  // Metrics
  const [metrics, setMetrics] = useState<MEVMetrics>({
    sandwichAttacksBlocked: 142,
    frontrunsPreventeded: 89,
    backrunsDetected: 67,
    totalValueProtected: '$2,450,000',
    averageSlippageSaved: 2.3,
    totalTransactions: 298,
  });
  
  // Detected threats
  const [threats, setThreats] = useState<DetectedThreat[]>([
    {
      id: '1',
      type: 'sandwich',
      severity: 'high',
      timestamp: new Date(),
      potentialLoss: '$1,250',
      status: 'blocked',
      details: 'Detected sandwich attack with 2.5% price impact'
    },
    {
      id: '2',
      type: 'frontrun',
      severity: 'medium',
      timestamp: new Date(Date.now() - 3600000),
      potentialLoss: '$450',
      status: 'blocked',
      details: 'Frontrun attempt detected, transaction protected'
    }
  ]);
  
  // Protection strategies
  const [strategies] = useState<ProtectionStrategy[]>([
    {
      id: 'flashbots',
      name: 'Flashbots Protect',
      description: 'Private mempool submission to avoid MEV bots',
      gasOverhead: 0,
      successRate: 95,
      avgSavings: 2.5,
      enabled: true,
      icon: FlashOn,
    },
    {
      id: 'commit-reveal',
      name: 'Commit-Reveal',
      description: 'Two-phase transaction with hidden parameters',
      gasOverhead: 25000,
      successRate: 98,
      avgSavings: 4.1,
      enabled: false,
      icon: Lock,
    },
    {
      id: 'time-delay',
      name: 'Time Delay',
      description: 'Delayed execution to avoid MEV detection',
      gasOverhead: 5000,
      successRate: 85,
      avgSavings: 1.9,
      enabled: false,
      icon: Timer,
    },
    {
      id: 'cowswap',
      name: 'CoW Swap',
      description: 'Coincidence of Wants for gasless trading',
      gasOverhead: 15000,
      successRate: 92,
      avgSavings: 3.2,
      enabled: false,
      icon: SwapHoriz,
    }
  ]);

  // Analyze transaction for MEV
  const analyzeTransaction = async () => {
    setAnalyzing(true);
    
    try {
      const response = await axios.post('/api/mev/analyze', {
        tokenIn,
        tokenOut,
        amountIn,
        slippageTolerance
      });
      
      if (response.data.threats.length > 0) {
        setThreats([...response.data.threats, ...threats]);
      }
      
      setEstimatedOut(response.data.estimatedOutput);
    } catch (error) {
      console.error('Analysis failed:', error);
    }
    
    setAnalyzing(false);
  };

  // Execute protected swap
  const executeProtectedSwap = async () => {
    if (!amountIn || !tokenIn || !tokenOut) {
      alert('Please fill in all swap parameters');
      return;
    }
    
    try {
      const response = await axios.post('/api/swap/protected', {
        tokenIn,
        tokenOut,
        amountIn,
        strategy: selectedStrategy,
        slippageTolerance,
        delay: transactionDelay
      });
      
      if (response.data.success) {
        alert(`Protected swap executed! Tx: ${response.data.txHash}`);
      }
    } catch (error) {
      console.error('Swap failed:', error);
      alert('Protected swap failed. Please try again.');
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return theme.palette.error.main;
      case 'high': return theme.palette.warning.main;
      case 'medium': return theme.palette.info.main;
      case 'low': return theme.palette.success.main;
      default: return theme.palette.grey[500];
    }
  };

  const getThreatIcon = (type: string) => {
    switch (type) {
      case 'sandwich': return <Dangerous />;
      case 'frontrun': return <Speed />;
      case 'backrun': return <Timeline />;
      case 'jit': return <FlashOn />;
      default: return <Warning />;
    }
  };

  return (
    <Box sx={{ p: 3 }}>
      {/* Header */}
      <Box sx={{ mb: 3, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Shield sx={{ fontSize: 40, color: theme.palette.primary.main }} />
          <Box>
            <Typography variant="h4" fontWeight="bold">
              DEX Protection
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Advanced MEV protection for your DeFi trades
            </Typography>
          </Box>
        </Box>
        
        <Box sx={{ display: 'flex', gap: 2 }}>
          <FormControlLabel
            control={
              <Switch
                checked={protectionEnabled}
                onChange={(e) => setProtectionEnabled(e.target.checked)}
                color="primary"
              />
            }
            label="Protection Active"
          />
          <Chip
            icon={protectionEnabled ? <CheckCircle /> : <Error />}
            label={protectionEnabled ? 'Protected' : 'Unprotected'}
            color={protectionEnabled ? 'success' : 'error'}
            variant="outlined"
          />
        </Box>
      </Box>

      {/* Protection Status Alert */}
      {protectionEnabled ? (
        <Alert severity="success" sx={{ mb: 3 }}>
          <AlertTitle>MEV Protection Active</AlertTitle>
          Your transactions are protected from sandwich attacks, frontrunning, and other MEV exploits.
          Current strategy: <strong>{selectedStrategy.toUpperCase()}</strong>
        </Alert>
      ) : (
        <Alert severity="warning" sx={{ mb: 3 }}>
          <AlertTitle>Protection Disabled</AlertTitle>
          Your transactions are vulnerable to MEV attacks. Enable protection to secure your trades.
        </Alert>
      )}

      {/* Metrics Overview */}
      <Grid container spacing={2} sx={{ mb: 3 }}>
        <Grid item xs={12} md={2}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Block color="error" />
                <Box>
                  <Typography variant="h6">{metrics.sandwichAttacksBlocked}</Typography>
                  <Typography variant="caption" color="text.secondary">
                    Sandwiches Blocked
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={2}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Speed color="warning" />
                <Box>
                  <Typography variant="h6">{metrics.frontrunsPreventeded}</Typography>
                  <Typography variant="caption" color="text.secondary">
                    Frontruns Prevented
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={2}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Timeline color="info" />
                <Box>
                  <Typography variant="h6">{metrics.backrunsDetected}</Typography>
                  <Typography variant="caption" color="text.secondary">
                    Backruns Detected
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={2}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <AttachMoney color="success" />
                <Box>
                  <Typography variant="h6">{metrics.totalValueProtected}</Typography>
                  <Typography variant="caption" color="text.secondary">
                    Value Protected
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={2}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <TrendingUp color="primary" />
                <Box>
                  <Typography variant="h6">{metrics.averageSlippageSaved}%</Typography>
                  <Typography variant="caption" color="text.secondary">
                    Avg Slippage Saved
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={2}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <SwapHoriz color="action" />
                <Box>
                  <Typography variant="h6">{metrics.totalTransactions}</Typography>
                  <Typography variant="caption" color="text.secondary">
                    Protected Swaps
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Main Content Tabs */}
      <Paper sx={{ mb: 3 }}>
        <Tabs value={activeTab} onChange={(e, v) => setActiveTab(v)}>
          <Tab label="Protected Swap" icon={<SwapHoriz />} iconPosition="start" />
          <Tab label="Protection Strategies" icon={<Security />} iconPosition="start" />
          <Tab label="Threat Monitor" icon={<Warning />} iconPosition="start" />
          <Tab label="Analytics" icon={<Analytics />} iconPosition="start" />
        </Tabs>
      </Paper>

      {/* Tab Content */}
      {activeTab === 0 && (
        <Grid container spacing={3}>
          <Grid item xs={12} md={8}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Execute Protected Swap
                </Typography>
                
                <Grid container spacing={2}>
                  <Grid item xs={12} md={6}>
                    <FormControl fullWidth>
                      <InputLabel>From Token</InputLabel>
                      <Select
                        value={tokenIn}
                        onChange={(e) => setTokenIn(e.target.value)}
                        label="From Token"
                      >
                        <MenuItem value="ETH">ETH</MenuItem>
                        <MenuItem value="USDC">USDC</MenuItem>
                        <MenuItem value="USDT">USDT</MenuItem>
                        <MenuItem value="DAI">DAI</MenuItem>
                        <MenuItem value="WBTC">WBTC</MenuItem>
                      </Select>
                    </FormControl>
                  </Grid>
                  
                  <Grid item xs={12} md={6}>
                    <TextField
                      fullWidth
                      label="Amount"
                      value={amountIn}
                      onChange={(e) => setAmountIn(e.target.value)}
                      type="number"
                      InputProps={{
                        endAdornment: <Typography>{tokenIn}</Typography>
                      }}
                    />
                  </Grid>
                  
                  <Grid item xs={12} md={6}>
                    <FormControl fullWidth>
                      <InputLabel>To Token</InputLabel>
                      <Select
                        value={tokenOut}
                        onChange={(e) => setTokenOut(e.target.value)}
                        label="To Token"
                      >
                        <MenuItem value="USDC">USDC</MenuItem>
                        <MenuItem value="ETH">ETH</MenuItem>
                        <MenuItem value="USDT">USDT</MenuItem>
                        <MenuItem value="DAI">DAI</MenuItem>
                        <MenuItem value="WBTC">WBTC</MenuItem>
                      </Select>
                    </FormControl>
                  </Grid>
                  
                  <Grid item xs={12} md={6}>
                    <TextField
                      fullWidth
                      label="Estimated Output"
                      value={estimatedOut}
                      disabled
                      InputProps={{
                        endAdornment: <Typography>{tokenOut}</Typography>
                      }}
                    />
                  </Grid>
                  
                  <Grid item xs={12}>
                    <Divider sx={{ my: 2 }} />
                  </Grid>
                  
                  {/* Protection Settings */}
                  <Grid item xs={12} md={6}>
                    <FormControl fullWidth>
                      <InputLabel>Protection Strategy</InputLabel>
                      <Select
                        value={selectedStrategy}
                        onChange={(e) => setSelectedStrategy(e.target.value)}
                        label="Protection Strategy"
                      >
                        <MenuItem value="flashbots">Flashbots Protect</MenuItem>
                        <MenuItem value="commit-reveal">Commit-Reveal</MenuItem>
                        <MenuItem value="time-delay">Time Delay</MenuItem>
                        <MenuItem value="cowswap">CoW Swap</MenuItem>
                      </Select>
                    </FormControl>
                  </Grid>
                  
                  <Grid item xs={12} md={6}>
                    <TextField
                      fullWidth
                      label="Max Slippage (%)"
                      value={slippageTolerance}
                      onChange={(e) => setSlippageTolerance(e.target.value)}
                      type="number"
                      inputProps={{ step: 0.1, min: 0.1, max: 50 }}
                    />
                  </Grid>
                  
                  <Grid item xs={12}>
                    <FormControlLabel
                      control={
                        <Switch
                          checked={showAdvanced}
                          onChange={(e) => setShowAdvanced(e.target.checked)}
                        />
                      }
                      label="Advanced Settings"
                    />
                  </Grid>
                  
                  <Collapse in={showAdvanced} sx={{ width: '100%' }}>
                    <Grid container spacing={2} sx={{ px: 2 }}>
                      <Grid item xs={12} md={6}>
                        <TextField
                          fullWidth
                          label="Transaction Delay (blocks)"
                          value={transactionDelay}
                          onChange={(e) => setTransactionDelay(e.target.value)}
                          type="number"
                          inputProps={{ min: 0, max: 10 }}
                        />
                      </Grid>
                      
                      <Grid item xs={12} md={6}>
                        <FormControlLabel
                          control={
                            <Switch
                              checked={autoProtect}
                              onChange={(e) => setAutoProtect(e.target.checked)}
                            />
                          }
                          label="Auto-protect all swaps"
                        />
                      </Grid>
                    </Grid>
                  </Collapse>
                  
                  <Grid item xs={12}>
                    <Box sx={{ display: 'flex', gap: 2 }}>
                      <Button
                        variant="outlined"
                        startIcon={analyzing ? <CircularProgress size={20} /> : <QueryStats />}
                        onClick={analyzeTransaction}
                        disabled={analyzing || !amountIn}
                      >
                        {analyzing ? 'Analyzing...' : 'Analyze for MEV'}
                      </Button>
                      
                      <Button
                        variant="contained"
                        startIcon={<Shield />}
                        onClick={executeProtectedSwap}
                        disabled={!protectionEnabled || !amountIn || !estimatedOut}
                      >
                        Execute Protected Swap
                      </Button>
                    </Box>
                  </Grid>
                </Grid>
              </CardContent>
            </Card>
          </Grid>
          
          <Grid item xs={12} md={4}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Recent Threats Blocked
                </Typography>
                
                <List>
                  {threats.slice(0, 5).map((threat) => (
                    <ListItem key={threat.id}>
                      <ListItemIcon>
                        <Avatar sx={{ bgcolor: alpha(getSeverityColor(threat.severity), 0.1) }}>
                          {getThreatIcon(threat.type)}
                        </Avatar>
                      </ListItemIcon>
                      <ListItemText
                        primary={
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <Typography variant="body2">
                              {threat.type.charAt(0).toUpperCase() + threat.type.slice(1)} Attack
                            </Typography>
                            <Chip
                              label={threat.severity}
                              size="small"
                              sx={{
                                bgcolor: alpha(getSeverityColor(threat.severity), 0.1),
                                color: getSeverityColor(threat.severity)
                              }}
                            />
                          </Box>
                        }
                        secondary={
                          <Box>
                            <Typography variant="caption" display="block">
                              Potential Loss: {threat.potentialLoss}
                            </Typography>
                            <Typography variant="caption" color="text.secondary">
                              {new Date(threat.timestamp).toLocaleTimeString()}
                            </Typography>
                          </Box>
                        }
                      />
                      <ListItemSecondaryAction>
                        <Chip
                          icon={threat.status === 'blocked' ? <Block /> : <Warning />}
                          label={threat.status}
                          size="small"
                          color={threat.status === 'blocked' ? 'success' : 'warning'}
                          variant="outlined"
                        />
                      </ListItemSecondaryAction>
                    </ListItem>
                  ))}
                </List>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}

      {activeTab === 1 && (
        <Grid container spacing={3}>
          {strategies.map((strategy) => (
            <Grid item xs={12} md={6} key={strategy.id}>
              <Card
                sx={{
                  border: strategy.enabled ? `2px solid ${theme.palette.primary.main}` : 'none',
                  position: 'relative'
                }}
              >
                {strategy.enabled && (
                  <Chip
                    label="Active"
                    color="primary"
                    size="small"
                    sx={{ position: 'absolute', top: 10, right: 10 }}
                  />
                )}
                
                <CardContent>
                  <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 2 }}>
                    <Avatar sx={{ bgcolor: alpha(theme.palette.primary.main, 0.1) }}>
                      <strategy.icon color="primary" />
                    </Avatar>
                    
                    <Box sx={{ flex: 1 }}>
                      <Typography variant="h6">{strategy.name}</Typography>
                      <Typography variant="body2" color="text.secondary" paragraph>
                        {strategy.description}
                      </Typography>
                      
                      <Grid container spacing={2}>
                        <Grid item xs={4}>
                          <Typography variant="caption" color="text.secondary">
                            Success Rate
                          </Typography>
                          <Typography variant="h6">{strategy.successRate}%</Typography>
                        </Grid>
                        <Grid item xs={4}>
                          <Typography variant="caption" color="text.secondary">
                            Avg Savings
                          </Typography>
                          <Typography variant="h6">{strategy.avgSavings}%</Typography>
                        </Grid>
                        <Grid item xs={4}>
                          <Typography variant="caption" color="text.secondary">
                            Gas Overhead
                          </Typography>
                          <Typography variant="h6">
                            {strategy.gasOverhead.toLocaleString()}
                          </Typography>
                        </Grid>
                      </Grid>
                      
                      <Box sx={{ mt: 2 }}>
                        <Button
                          variant={strategy.enabled ? 'outlined' : 'contained'}
                          size="small"
                          onClick={() => setSelectedStrategy(strategy.id)}
                        >
                          {strategy.enabled ? 'Selected' : 'Select Strategy'}
                        </Button>
                      </Box>
                    </Box>
                  </Box>
                </CardContent>
              </Card>
            </Grid>
          ))}
        </Grid>
      )}

      {activeTab === 2 && (
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Real-Time Threat Monitor
            </Typography>
            
            <List>
              {threats.map((threat) => (
                <ListItem key={threat.id} divider>
                  <ListItemIcon>
                    <Avatar sx={{ bgcolor: alpha(getSeverityColor(threat.severity), 0.1) }}>
                      {getThreatIcon(threat.type)}
                    </Avatar>
                  </ListItemIcon>
                  
                  <ListItemText
                    primary={
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                        <Typography variant="subtitle1">
                          {threat.type.toUpperCase()} Attack Detected
                        </Typography>
                        <Chip
                          label={threat.severity}
                          color={
                            threat.severity === 'critical' ? 'error' :
                            threat.severity === 'high' ? 'warning' :
                            threat.severity === 'medium' ? 'info' : 'success'
                          }
                          size="small"
                        />
                        <Chip
                          label={threat.status}
                          color={threat.status === 'blocked' ? 'success' : 'default'}
                          size="small"
                          variant="outlined"
                        />
                      </Box>
                    }
                    secondary={
                      <Box>
                        <Typography variant="body2" color="text.secondary">
                          {threat.details}
                        </Typography>
                        <Box sx={{ display: 'flex', gap: 3, mt: 1 }}>
                          <Typography variant="caption">
                            Potential Loss: <strong>{threat.potentialLoss}</strong>
                          </Typography>
                          <Typography variant="caption">
                            Time: {new Date(threat.timestamp).toLocaleString()}
                          </Typography>
                        </Box>
                      </Box>
                    }
                  />
                  
                  <ListItemSecondaryAction>
                    <Tooltip title="View Details">
                      <IconButton edge="end">
                        <Info />
                      </IconButton>
                    </Tooltip>
                  </ListItemSecondaryAction>
                </ListItem>
              ))}
            </List>
          </CardContent>
        </Card>
      )}

      {activeTab === 3 && (
        <Grid container spacing={3}>
          <Grid item xs={12}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Protection Performance Analytics
                </Typography>
                
                <Box sx={{ height: 400, mt: 3 }}>
                  {/* Chart placeholder - would use actual charting library */}
                  <Box
                    sx={{
                      height: '100%',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      bgcolor: alpha(theme.palette.primary.main, 0.05),
                      borderRadius: 2
                    }}
                  >
                    <Typography color="text.secondary">
                      Protection metrics chart would be displayed here
                    </Typography>
                  </Box>
                </Box>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}
    </Box>
  );
};

export default DEXProtection;