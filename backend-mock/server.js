const express = require('express');
const cors = require('cors');
const jwt = require('jsonwebtoken');
const app = express();

// CORS configuration - completely open in production, restricted in development
const corsOptions = {
  origin: true, // Allow all origins
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization', 'X-Requested-With', 'Accept', 'Origin', 'Expires', 'Cache-Control'],
  exposedHeaders: ['Content-Range', 'X-Content-Range'],
  maxAge: 86400 // 24 hours
};

// Middleware
app.use(cors(corsOptions));
app.use(express.json());

// Security headers to fix CSP and font loading issues
app.use((req, res, next) => {
  res.setHeader('Content-Security-Policy', 
    "default-src 'self' 'unsafe-inline' 'unsafe-eval'; " +
    "font-src 'self' data: https://fonts.gstatic.com https://fonts.googleapis.com; " +
    "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; " +
    "img-src 'self' data: blob:; " +
    "connect-src 'self' http://localhost:* ws://localhost:*; " +
    "script-src 'self' 'unsafe-inline' 'unsafe-eval';"
  );
  res.setHeader('X-Content-Type-Options', 'nosniff');
  res.setHeader('X-Frame-Options', 'DENY');
  res.setHeader('X-XSS-Protection', '1; mode=block');
  next();
});

// Serve static files (fonts, assets)
app.use('/res', express.static('public/res'));

// Handle preflight requests
app.options('*', cors(corsOptions));

// JWT Secret
const JWT_SECRET = 'mev-shield-secret-key-2024';

// Mock users database
const users = [
  { id: '1', email: 'admin@mevshield.ai', password: 'admin123', role: 'Admin' },
  { id: '2', email: 'user@mevshield.ai', password: 'user123', role: 'User' },
  { id: '3', email: 'builder@mevshield.ai', password: 'builder123', role: 'Builder' },
  { id: '4', email: 'trader@mevshield.ai', password: 'trader123', role: 'Trader' },
];

// Mock MEV data
const mevStats = {
  totalBlocked: 15234,
  totalSaved: '$12.5M',
  activeProtections: 892,
  successRate: 99.9,
  recentAttacks: [
    { id: 1, type: 'Sandwich', amount: '$5,234', status: 'Blocked', timestamp: new Date() },
    { id: 2, type: 'Frontrun', amount: '$2,145', status: 'Blocked', timestamp: new Date() },
    { id: 3, type: 'Arbitrage', amount: '$8,921', status: 'Blocked', timestamp: new Date() },
  ],
  networkStats: {
    ethereum: { attacks: 8234, saved: '$6.2M' },
    bsc: { attacks: 4123, saved: '$3.1M' },
    polygon: { attacks: 2877, saved: '$3.2M' },
  }
};

// Authentication endpoints
app.post('/auth/login', (req, res) => {
  const { email, password } = req.body;
  const user = users.find(u => u.email === email && u.password === password);
  
  if (user) {
    const token = jwt.sign({ id: user.id, email: user.email, role: user.role }, JWT_SECRET, { expiresIn: '24h' });
    const refreshToken = jwt.sign({ id: user.id }, JWT_SECRET, { expiresIn: '7d' });
    
    res.json({
      success: true,
      data: {
        access_token: token,
        refresh_token: refreshToken,
        token_type: 'Bearer',
        expires_in: 86400,
        user: {
          id: user.id,
          email: user.email,
          role: user.role,
          lastLogin: new Date().toISOString()
        }
      }
    });
  } else {
    res.status(401).json({ success: false, error: 'Invalid credentials' });
  }
});

app.post('/auth/logout', (req, res) => {
  res.json({ success: true, message: 'Logged out successfully' });
});

app.post('/auth/refresh', (req, res) => {
  const { refresh_token } = req.body;
  
  if (!refresh_token) {
    return res.status(401).json({ success: false, error: 'No refresh token provided' });
  }
  
  try {
    const decoded = jwt.verify(refresh_token, JWT_SECRET);
    const user = users.find(u => u.id === decoded.id);
    
    if (user) {
      const newToken = jwt.sign({ id: user.id, email: user.email, role: user.role }, JWT_SECRET, { expiresIn: '24h' });
      res.json({
        success: true,
        data: {
          access_token: newToken
        }
      });
    } else {
      res.status(404).json({ success: false, error: 'User not found' });
    }
  } catch (error) {
    res.status(401).json({ success: false, error: 'Invalid refresh token' });
  }
});

app.get('/auth/me', (req, res) => {
  const token = req.headers.authorization?.replace('Bearer ', '');
  
  if (!token) {
    return res.status(401).json({ success: false, error: 'No token provided' });
  }
  
  try {
    const decoded = jwt.verify(token, JWT_SECRET);
    const user = users.find(u => u.id === decoded.id);
    
    if (user) {
      res.json({
        success: true,
        data: {
          id: user.id,
          email: user.email,
          role: user.role,
          lastLogin: new Date().toISOString()
        }
      });
    } else {
      res.status(404).json({ success: false, error: 'User not found' });
    }
  } catch (error) {
    res.status(401).json({ success: false, error: 'Invalid token' });
  }
});

app.post('/auth/register', (req, res) => {
  const { email, password, role = 'User' } = req.body;
  
  if (users.find(u => u.email === email)) {
    return res.status(400).json({ success: false, error: 'User already exists' });
  }
  
  const newUser = {
    id: String(users.length + 1),
    email,
    password,
    role
  };
  
  users.push(newUser);
  res.json({ success: true, message: 'User registered successfully' });
});

// API endpoints
app.get('/api/stats', (req, res) => {
  res.json(mevStats);
});

app.get('/api/mev/attacks', (req, res) => {
  res.json({
    attacks: mevStats.recentAttacks,
    total: mevStats.totalBlocked,
    page: 1,
    limit: 10
  });
});

app.get('/api/mev/savings', (req, res) => {
  res.json({
    total: mevStats.totalSaved,
    byNetwork: mevStats.networkStats,
    last24h: '$125,000',
    last7d: '$892,000'
  });
});

app.get('/api/protection/status', (req, res) => {
  res.json({
    active: true,
    protections: mevStats.activeProtections,
    successRate: mevStats.successRate,
    lastUpdated: new Date().toISOString()
  });
});

app.get('/api/users', (req, res) => {
  res.json({
    users: users.map(u => ({ id: u.id, email: u.email, role: u.role })),
    total: users.length
  });
});

app.get('/api/dashboard/:role', (req, res) => {
  const { role } = req.params;
  
  const dashboardData = {
    Admin: {
      totalUsers: users.length,
      systemStatus: 'Operational',
      ...mevStats
    },
    User: {
      myProtections: 42,
      mySavings: '$2,341',
      recentTransactions: mevStats.recentAttacks.slice(0, 5)
    },
    Builder: {
      blocksBuilt: 1247,
      successRate: 94.3,
      avgGasUsed: '12.5M',
      totalRevenue: '45.8 ETH',
      mevCaptured: '12.3 ETH'
    },
    Trader: {
      totalValue: '$124,580',
      dailyPnL: '+$2,450',
      dailyPnLPercent: 2.01,
      protectedTrades: 89,
      mevSaved: '$1,240'
    }
  };
  
  res.json(dashboardData[role] || dashboardData.User);
});

// Uniswap Integration Endpoints
app.post('/api/uniswap/quote', (req, res) => {
  const { tokenIn, tokenOut, amountIn } = req.body;
  
  if (!tokenIn || !tokenOut || !amountIn) {
    return res.status(400).json({ 
      success: false, 
      error: 'Missing required parameters' 
    });
  }
  
  // Simulate quote based on mock exchange rates
  const mockRates = {
    'WETH_USDC': 2500,
    'WETH_DAI': 2500,
    'USDC_DAI': 1,
    'USDC_WETH': 0.0004,
    'DAI_WETH': 0.0004,
    'DAI_USDC': 1
  };
  
  // Simplified token pair mapping
  const getPair = (from, to) => {
    const mapping = {
      '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2': 'WETH',
      '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48': 'USDC',
      '0x6B175474E89094C44Da98b954EedeAC495271d0F': 'DAI'
    };
    return `${mapping[from] || 'UNKNOWN'}_${mapping[to] || 'UNKNOWN'}`;
  };
  
  const pair = getPair(tokenIn, tokenOut);
  const rate = mockRates[pair] || 1;
  const expectedOutput = (BigInt(amountIn) * BigInt(Math.floor(rate * 1000)) / 1000n).toString();
  const priceImpact = Math.random() * 5; // 0-5% impact
  
  res.json({
    success: true,
    expectedOutput,
    minimumOutput: (BigInt(expectedOutput) * 995n / 1000n).toString(),
    priceImpact: priceImpact.toFixed(2),
    estimatedGas: '150000',
    route: [tokenIn, tokenOut],
    fee: 3000
  });
});

app.post('/api/mev/analyze', (req, res) => {
  const { tokenIn, tokenOut, amountIn } = req.body;
  
  if (!amountIn) {
    return res.status(400).json({ 
      success: false, 
      error: 'Missing amount parameter' 
    });
  }
  
  // Analyze MEV risk based on amount
  const amountInEth = Number(amountIn) / 1e18;
  let risk = 'low';
  let factors = [];
  
  if (amountInEth > 100) {
    risk = 'high';
    factors.push('Large trade size significantly increases sandwich attack risk');
    factors.push('Consider splitting into smaller trades');
  } else if (amountInEth > 10) {
    risk = 'medium';
    factors.push('Medium trade size may attract MEV bots');
    factors.push('MEV protection recommended');
  } else {
    factors.push('Small trade size reduces MEV profitability');
  }
  
  // Add more risk factors
  if (tokenIn === '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2') {
    factors.push('ETH pairs have higher MEV activity');
  }
  
  const recommendation = risk === 'high' 
    ? 'Enable MEV protection and consider trade splitting'
    : risk === 'medium'
    ? 'MEV protection recommended for optimal execution'
    : 'Standard protection should be sufficient';
  
  res.json({
    success: true,
    risk,
    factors,
    recommendation,
    estimatedMEVLoss: risk === 'high' ? '2.5%' : risk === 'medium' ? '1.2%' : '0.3%',
    sandwichRisk: risk,
    frontrunRisk: risk === 'high' ? 'medium' : 'low',
    metadata: {
      gasPrice: '30 gwei',
      blockNumber: 18500000,
      mempoolActivity: 'moderate'
    }
  });
});

app.post('/api/swap/protected', (req, res) => {
  const { 
    tokenIn, 
    tokenOut, 
    amountIn, 
    recipient, 
    enableMEVProtection,
    slippageTolerance 
  } = req.body;
  
  if (!tokenIn || !tokenOut || !amountIn || !recipient) {
    return res.status(400).json({ 
      success: false, 
      error: 'Missing required swap parameters' 
    });
  }
  
  const swapId = '0x' + Array(64).fill(0).map(() => 
    Math.floor(Math.random() * 16).toString(16)
  ).join('');
  
  const response = {
    success: true,
    swapId,
    status: enableMEVProtection ? 'scheduled' : 'pending',
    estimatedExecution: new Date(Date.now() + 12000).toISOString(), // ~12 seconds (2 blocks)
    transaction: {
      from: recipient,
      to: '0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45', // Uniswap router
      data: '0x...', // Encoded swap data
      value: '0',
      gasLimit: '180000',
      maxFeePerGas: '35000000000',
      maxPriorityFeePerGas: '2000000000'
    }
  };
  
  if (enableMEVProtection) {
    response.mevProtection = {
      enabled: true,
      delay: '2 blocks',
      sandwichRisk: 'low',
      expectedSavings: '$' + (Math.random() * 100).toFixed(2),
      protectionCost: '$2.50'
    };
  }
  
  // Simulate swap execution after delay
  if (enableMEVProtection) {
    setTimeout(() => {
      // In production, this would emit an event or update a database
      console.log(`Protected swap ${swapId} executed successfully`);
    }, 12000);
  }
  
  res.json(response);
});

app.get('/api/uniswap/pools/:tokenA/:tokenB', (req, res) => {
  const { tokenA, tokenB } = req.params;
  
  // Mock pool data
  res.json({
    success: true,
    pools: [
      {
        address: '0x' + Array(40).fill(0).map(() => Math.floor(Math.random() * 16).toString(16)).join(''),
        fee: 3000,
        liquidity: '50000000000000000000000',
        sqrtPriceX96: '1961422080730315273739897227',
        tick: 202163,
        token0: tokenA,
        token1: tokenB,
        tvl: '$125,000,000',
        volume24h: '$8,500,000',
        apy: '12.5%'
      },
      {
        address: '0x' + Array(40).fill(0).map(() => Math.floor(Math.random() * 16).toString(16)).join(''),
        fee: 500,
        liquidity: '25000000000000000000000',
        sqrtPriceX96: '1961422080730315273739897227',
        tick: 202163,
        token0: tokenA,
        token1: tokenB,
        tvl: '$45,000,000',
        volume24h: '$3,200,000',
        apy: '8.3%'
      }
    ]
  });
});

app.get('/api/uniswap/pending-swaps', (req, res) => {
  // Mock pending swaps
  res.json({
    success: true,
    swaps: [
      {
        id: '0xabc123',
        tokenIn: 'ETH',
        tokenOut: 'USDC',
        amountIn: '1000000000000000000',
        status: 'pending',
        scheduledBlock: 18500100,
        user: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb3'
      }
    ]
  });
});

// MEV Protection Endpoints
app.post('/api/mev/analyze', (req, res) => {
  const { tokenIn, tokenOut, amountIn, slippageTolerance } = req.body;
  
  // Simulate MEV analysis
  const threats = [];
  const riskScore = Math.random();
  
  if (riskScore > 0.7) {
    threats.push({
      id: Math.random().toString(36).substr(2, 9),
      type: 'sandwich',
      severity: 'high',
      timestamp: new Date(),
      potentialLoss: `$${(parseFloat(amountIn) * 0.03).toFixed(2)}`,
      status: 'detected',
      details: `Detected potential sandwich attack with ${(riskScore * 5).toFixed(1)}% price impact`
    });
  }
  
  if (riskScore > 0.5 && riskScore <= 0.7) {
    threats.push({
      id: Math.random().toString(36).substr(2, 9),
      type: 'frontrun',
      severity: 'medium',
      timestamp: new Date(),
      potentialLoss: `$${(parseFloat(amountIn) * 0.02).toFixed(2)}`,
      status: 'detected',
      details: 'Frontrun attempt detected in mempool'
    });
  }
  
  // Calculate estimated output with protection
  const estimatedOutput = parseFloat(amountIn || 0) * 
    (tokenIn === 'ETH' ? 2000 : 1) * 
    (1 - parseFloat(slippageTolerance || 0.5) / 100);
  
  res.json({
    success: true,
    threats,
    riskScore,
    estimatedOutput: estimatedOutput.toFixed(6),
    recommendation: threats.length > 0 ? 'USE_PROTECTION' : 'SAFE_TO_PROCEED',
    protectionStrategy: threats.length > 0 ? 'flashbots' : 'standard'
  });
});

app.post('/api/swap/protected', (req, res) => {
  const { tokenIn, tokenOut, amountIn, strategy, slippageTolerance, delay } = req.body;
  
  // Simulate protected swap execution
  const txHash = '0x' + Math.random().toString(36).substr(2, 64);
  const blockNumber = Math.floor(Math.random() * 1000000) + 18000000;
  
  res.json({
    success: true,
    txHash,
    blockNumber,
    strategy,
    executionTime: new Date().toISOString(),
    gasUsed: '150000',
    effectivePrice: (parseFloat(amountIn) * 1995).toFixed(6),
    savedFromMEV: `$${(parseFloat(amountIn) * 0.025).toFixed(2)}`,
    status: 'executed'
  });
});

app.get('/api/mev/strategies', (req, res) => {
  res.json({
    success: true,
    strategies: [
      {
        id: 'flashbots',
        name: 'Flashbots Protect',
        description: 'Private mempool submission to avoid MEV bots',
        gasOverhead: 0,
        successRate: 95,
        avgSavings: 2.5,
        supported: true
      },
      {
        id: 'commit-reveal',
        name: 'Commit-Reveal',
        description: 'Two-phase transaction with hidden parameters',
        gasOverhead: 25000,
        successRate: 98,
        avgSavings: 4.1,
        supported: true
      },
      {
        id: 'time-delay',
        name: 'Time Delay',
        description: 'Delayed execution to avoid MEV detection',
        gasOverhead: 5000,
        successRate: 85,
        avgSavings: 1.9,
        supported: true
      },
      {
        id: 'cowswap',
        name: 'CoW Swap',
        description: 'Coincidence of Wants for gasless trading',
        gasOverhead: 15000,
        successRate: 92,
        avgSavings: 3.2,
        supported: false
      }
    ]
  });
});

app.get('/api/mev/metrics', (req, res) => {
  res.json({
    success: true,
    metrics: {
      sandwichAttacksBlocked: 142,
      frontrunsPreventeded: 89,
      backrunsDetected: 67,
      totalValueProtected: 2450000,
      averageSlippageSaved: 2.3,
      totalTransactions: 298,
      last24h: {
        attacks: 12,
        saved: 34500,
        transactions: 45
      },
      last7d: {
        attacks: 67,
        saved: 245000,
        transactions: 298
      }
    }
  });
});

app.get('/api/mev/threats', (req, res) => {
  const threats = [];
  const types = ['sandwich', 'frontrun', 'backrun', 'jit'];
  const severities = ['low', 'medium', 'high', 'critical'];
  
  // Generate mock threats
  for (let i = 0; i < 10; i++) {
    threats.push({
      id: Math.random().toString(36).substr(2, 9),
      type: types[Math.floor(Math.random() * types.length)],
      severity: severities[Math.floor(Math.random() * severities.length)],
      timestamp: new Date(Date.now() - Math.random() * 86400000),
      potentialLoss: `$${(Math.random() * 5000).toFixed(2)}`,
      status: Math.random() > 0.3 ? 'blocked' : 'detected',
      details: `MEV attack detected and ${Math.random() > 0.3 ? 'blocked' : 'monitored'}`,
      txHash: '0x' + Math.random().toString(36).substr(2, 64)
    });
  }
  
  res.json({
    success: true,
    threats: threats.sort((a, b) => b.timestamp - a.timestamp)
  });
});

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'healthy', timestamp: new Date().toISOString() });
});

// WebSocket mock for real-time updates
const PORT = process.env.PORT || 8080;
const server = app.listen(PORT, () => {
  console.log(`✅ MEV Shield Mock Backend running on port ${PORT}`);
  console.log(`📊 API endpoints available at http://localhost:${PORT}`);
  console.log(`
  Available test accounts:
  - Admin: admin@mevshield.ai / admin123
  - User: user@mevshield.ai / user123
  - Builder: builder@mevshield.ai / builder123
  - Trader: trader@mevshield.ai / trader123
  `);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  server.close(() => {
    console.log('Server closed');
  });
});