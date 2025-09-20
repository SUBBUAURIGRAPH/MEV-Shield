import React, { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import {
  Box,
  Card,
  CardContent,
  Typography,
  TextField,
  Button,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Alert,
  CircularProgress,
  Chip,
  Grid,
  Divider,
  Switch,
  FormControlLabel,
  Tooltip,
  IconButton
} from '@mui/material';
import {
  SwapHoriz,
  Security,
  Info,
  Warning,
  CheckCircle,
  Timer,
  TrendingUp,
  LocalGasStation
} from '@mui/icons-material';

// Token list (simplified - in production, fetch from token list)
const TOKENS = [
  { symbol: 'ETH', address: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2', decimals: 18 },
  { symbol: 'USDC', address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', decimals: 6 },
  { symbol: 'DAI', address: '0x6B175474E89094C44Da98b954EedeAC495271d0F', decimals: 18 },
  { symbol: 'WBTC', address: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599', decimals: 8 },
  { symbol: 'USDT', address: '0xdAC17F958D2ee523a2206206994597C13D831ec7', decimals: 6 }
];

interface SwapQuote {
  expectedOutput: string;
  minimumOutput: string;
  priceImpact: number;
  estimatedGas: string;
  route: string[];
}

interface MEVAnalysis {
  risk: 'low' | 'medium' | 'high';
  factors: string[];
  recommendation: string;
}

const UniswapTrading: React.FC = () => {
  const [provider, setProvider] = useState<ethers.BrowserProvider | null>(null);
  const [signer, setSigner] = useState<ethers.Signer | null>(null);
  const [account, setAccount] = useState<string>('');
  
  // Swap state
  const [tokenIn, setTokenIn] = useState(TOKENS[0]);
  const [tokenOut, setTokenOut] = useState(TOKENS[1]);
  const [amountIn, setAmountIn] = useState('');
  const [quote, setQuote] = useState<SwapQuote | null>(null);
  const [mevAnalysis, setMevAnalysis] = useState<MEVAnalysis | null>(null);
  const [enableMEVProtection, setEnableMEVProtection] = useState(true);
  const [slippage, setSlippage] = useState(0.5); // 0.5%
  
  // UI state
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [pendingSwaps, setPendingSwaps] = useState<any[]>([]);

  useEffect(() => {
    initializeProvider();
  }, []);

  const initializeProvider = async () => {
    if (typeof window !== 'undefined' && (window as any).ethereum) {
      try {
        const provider = new ethers.BrowserProvider((window as any).ethereum);
        const accounts = await provider.send('eth_requestAccounts', []);
        const signer = await provider.getSigner();
        
        setProvider(provider);
        setSigner(signer);
        setAccount(accounts[0]);
      } catch (err) {
        setError('Failed to connect wallet');
        console.error(err);
      }
    } else {
      setError('Please install MetaMask');
    }
  };

  const getQuote = async () => {
    if (!amountIn || !provider) return;
    
    setLoading(true);
    setError(null);
    
    try {
      // In production, this would call the UniswapService
      // For demo, we'll simulate a quote
      const amountInWei = ethers.parseUnits(amountIn, tokenIn.decimals);
      const estimatedOutput = amountInWei * BigInt(1500); // Simulated exchange rate
      const priceImpact = Math.random() * 5; // Simulated price impact
      
      setQuote({
        expectedOutput: ethers.formatUnits(estimatedOutput, tokenOut.decimals),
        minimumOutput: ethers.formatUnits(estimatedOutput * BigInt(995) / BigInt(1000), tokenOut.decimals),
        priceImpact,
        estimatedGas: '150000',
        route: [tokenIn.symbol, tokenOut.symbol]
      });
      
      // Analyze MEV risk
      analyzeMEVRisk(amountInWei);
    } catch (err) {
      setError('Failed to get quote');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const analyzeMEVRisk = (amount: bigint) => {
    const factors: string[] = [];
    let risk: 'low' | 'medium' | 'high' = 'low';
    
    // Simple heuristic for demo
    const amountInEth = Number(ethers.formatEther(amount));
    
    if (amountInEth > 100) {
      factors.push('Large trade size increases sandwich attack risk');
      risk = 'high';
    } else if (amountInEth > 10) {
      factors.push('Medium trade size may attract MEV bots');
      risk = 'medium';
    }
    
    if (slippage > 1) {
      factors.push('High slippage tolerance increases MEV risk');
      risk = risk === 'low' ? 'medium' : risk;
    }
    
    const recommendation = risk === 'high' 
      ? 'Enable MEV protection and consider splitting the trade'
      : risk === 'medium'
      ? 'MEV protection recommended for this trade'
      : 'Standard protection should be sufficient';
    
    setMevAnalysis({ risk, factors, recommendation });
  };

  const executeSwap = async () => {
    if (!signer || !quote) return;
    
    setLoading(true);
    setError(null);
    setSuccess(null);
    
    try {
      // In production, this would interact with the smart contract
      // For demo, we'll simulate a transaction
      const tx = {
        to: '0x...', // MEV Shield contract
        data: '0x...',
        value: 0
      };
      
      // Simulate transaction
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      const swapId = ethers.hexlify(ethers.randomBytes(32));
      
      if (enableMEVProtection) {
        setPendingSwaps([...pendingSwaps, {
          id: swapId,
          tokenIn: tokenIn.symbol,
          tokenOut: tokenOut.symbol,
          amount: amountIn,
          status: 'pending',
          protectionDelay: 2
        }]);
        
        setSuccess(`Swap scheduled with MEV protection. ID: ${swapId.slice(0, 10)}...`);
        
        // Simulate execution after delay
        setTimeout(() => {
          setPendingSwaps(prev => prev.map(swap => 
            swap.id === swapId ? { ...swap, status: 'executed' } : swap
          ));
        }, 5000);
      } else {
        setSuccess(`Swap executed successfully. Hash: ${swapId.slice(0, 10)}...`);
      }
      
      // Reset form
      setAmountIn('');
      setQuote(null);
      setMevAnalysis(null);
    } catch (err) {
      setError('Failed to execute swap');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const getMEVRiskColor = (risk: string) => {
    switch (risk) {
      case 'high': return 'error';
      case 'medium': return 'warning';
      default: return 'success';
    }
  };

  return (
    <Box sx={{ maxWidth: 1200, margin: 'auto', padding: 3 }}>
      <Typography variant="h4" gutterBottom>
        <SwapHoriz sx={{ mr: 1, verticalAlign: 'middle' }} />
        Uniswap Trading with MEV Protection
      </Typography>

      <Grid container spacing={3}>
        {/* Swap Interface */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Swap Tokens
              </Typography>

              {/* Token In */}
              <Box sx={{ mb: 2 }}>
                <FormControl fullWidth>
                  <InputLabel>From</InputLabel>
                  <Select
                    value={tokenIn.symbol}
                    onChange={(e) => setTokenIn(TOKENS.find(t => t.symbol === e.target.value) || TOKENS[0])}
                    label="From"
                  >
                    {TOKENS.map(token => (
                      <MenuItem key={token.symbol} value={token.symbol}>
                        {token.symbol}
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>
                <TextField
                  fullWidth
                  type="number"
                  label="Amount"
                  value={amountIn}
                  onChange={(e) => setAmountIn(e.target.value)}
                  sx={{ mt: 2 }}
                />
              </Box>

              {/* Swap Icon */}
              <Box sx={{ display: 'flex', justifyContent: 'center', my: 2 }}>
                <IconButton 
                  onClick={() => {
                    const temp = tokenIn;
                    setTokenIn(tokenOut);
                    setTokenOut(temp);
                  }}
                >
                  <SwapHoriz />
                </IconButton>
              </Box>

              {/* Token Out */}
              <Box sx={{ mb: 3 }}>
                <FormControl fullWidth>
                  <InputLabel>To</InputLabel>
                  <Select
                    value={tokenOut.symbol}
                    onChange={(e) => setTokenOut(TOKENS.find(t => t.symbol === e.target.value) || TOKENS[1])}
                    label="To"
                  >
                    {TOKENS.map(token => (
                      <MenuItem key={token.symbol} value={token.symbol}>
                        {token.symbol}
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>
                {quote && (
                  <TextField
                    fullWidth
                    label="Expected Output"
                    value={quote.expectedOutput}
                    InputProps={{ readOnly: true }}
                    sx={{ mt: 2 }}
                  />
                )}
              </Box>

              {/* Settings */}
              <Box sx={{ mb: 3 }}>
                <Typography variant="subtitle2" gutterBottom>
                  Settings
                </Typography>
                <TextField
                  size="small"
                  type="number"
                  label="Slippage Tolerance (%)"
                  value={slippage}
                  onChange={(e) => setSlippage(Number(e.target.value))}
                  sx={{ width: 200, mr: 2 }}
                />
                <FormControlLabel
                  control={
                    <Switch
                      checked={enableMEVProtection}
                      onChange={(e) => setEnableMEVProtection(e.target.checked)}
                    />
                  }
                  label={
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <Security sx={{ mr: 1, fontSize: 20 }} />
                      MEV Protection
                    </Box>
                  }
                />
              </Box>

              {/* Action Buttons */}
              <Box sx={{ display: 'flex', gap: 2 }}>
                <Button
                  fullWidth
                  variant="outlined"
                  onClick={getQuote}
                  disabled={!amountIn || loading}
                >
                  {loading ? <CircularProgress size={20} /> : 'Get Quote'}
                </Button>
                <Button
                  fullWidth
                  variant="contained"
                  color="primary"
                  onClick={executeSwap}
                  disabled={!quote || loading}
                >
                  {loading ? <CircularProgress size={20} /> : 'Swap'}
                </Button>
              </Box>

              {/* Alerts */}
              {error && (
                <Alert severity="error" sx={{ mt: 2 }} onClose={() => setError(null)}>
                  {error}
                </Alert>
              )}
              {success && (
                <Alert severity="success" sx={{ mt: 2 }} onClose={() => setSuccess(null)}>
                  {success}
                </Alert>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* Quote Details & MEV Analysis */}
        <Grid item xs={12} md={6}>
          {quote && (
            <Card sx={{ mb: 2 }}>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Quote Details
                </Typography>
                <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">
                      Expected Output:
                    </Typography>
                    <Typography variant="body2">
                      {quote.expectedOutput} {tokenOut.symbol}
                    </Typography>
                  </Box>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">
                      Minimum Output:
                    </Typography>
                    <Typography variant="body2">
                      {quote.minimumOutput} {tokenOut.symbol}
                    </Typography>
                  </Box>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">
                      Price Impact:
                    </Typography>
                    <Chip 
                      label={`${quote.priceImpact.toFixed(2)}%`}
                      size="small"
                      color={quote.priceImpact > 3 ? 'warning' : 'success'}
                    />
                  </Box>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">
                      <LocalGasStation sx={{ fontSize: 16, verticalAlign: 'middle', mr: 0.5 }} />
                      Estimated Gas:
                    </Typography>
                    <Typography variant="body2">
                      {quote.estimatedGas}
                    </Typography>
                  </Box>
                </Box>
              </CardContent>
            </Card>
          )}

          {mevAnalysis && (
            <Card sx={{ mb: 2 }}>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  <Security sx={{ mr: 1, verticalAlign: 'middle' }} />
                  MEV Risk Analysis
                </Typography>
                
                <Box sx={{ mb: 2 }}>
                  <Chip 
                    label={`Risk Level: ${mevAnalysis.risk.toUpperCase()}`}
                    color={getMEVRiskColor(mevAnalysis.risk)}
                    sx={{ mb: 2 }}
                  />
                  
                  {mevAnalysis.factors.map((factor, index) => (
                    <Alert severity="info" sx={{ mb: 1 }} key={index}>
                      {factor}
                    </Alert>
                  ))}
                  
                  <Alert severity={mevAnalysis.risk === 'high' ? 'warning' : 'success'}>
                    <strong>Recommendation:</strong> {mevAnalysis.recommendation}
                  </Alert>
                </Box>
              </CardContent>
            </Card>
          )}

          {/* Pending Swaps */}
          {pendingSwaps.length > 0 && (
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  <Timer sx={{ mr: 1, verticalAlign: 'middle' }} />
                  Protected Swaps
                </Typography>
                {pendingSwaps.map((swap) => (
                  <Box 
                    key={swap.id}
                    sx={{ 
                      p: 2, 
                      mb: 1, 
                      border: '1px solid',
                      borderColor: 'divider',
                      borderRadius: 1
                    }}
                  >
                    <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                      <Typography variant="body2">
                        {swap.tokenIn} → {swap.tokenOut}
                      </Typography>
                      <Chip 
                        label={swap.status}
                        size="small"
                        color={swap.status === 'executed' ? 'success' : 'default'}
                        icon={swap.status === 'executed' ? <CheckCircle /> : <Timer />}
                      />
                    </Box>
                    <Typography variant="caption" color="text.secondary">
                      Amount: {swap.amount} | ID: {swap.id.slice(0, 10)}...
                    </Typography>
                    {swap.status === 'pending' && (
                      <Typography variant="caption" display="block" sx={{ mt: 1 }}>
                        MEV Protection: {swap.protectionDelay} blocks delay
                      </Typography>
                    )}
                  </Box>
                ))}
              </CardContent>
            </Card>
          )}
        </Grid>
      </Grid>
    </Box>
  );
};

export default UniswapTrading;