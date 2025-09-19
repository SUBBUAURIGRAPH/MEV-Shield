const express = require('express');
const cors = require('cors');
const jwt = require('jsonwebtoken');
const app = express();

// CORS configuration to allow specific origins with credentials
const corsOptions = {
  origin: function (origin, callback) {
    // Allow requests from localhost ports and no origin (for tools like Postman)
    const allowedOrigins = [
      'http://localhost:3000',
      'http://localhost:3001', 
      'http://localhost:3002',
      'http://localhost:3003',
      'http://localhost:3004',
      'http://localhost:3005'
    ];
    
    if (!origin || allowedOrigins.indexOf(origin) !== -1) {
      callback(null, true);
    } else {
      callback(new Error('Not allowed by CORS'));
    }
  },
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