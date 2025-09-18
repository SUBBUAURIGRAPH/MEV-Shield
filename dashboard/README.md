# MEV Shield Dashboard

Comprehensive admin and user dashboards for MEV Shield protection platform.

## 📊 Features

### Admin Dashboard
- **System Monitoring**: Real-time health metrics for all services
- **Transaction Management**: View and manage protected transactions
- **Builder Network**: Monitor and manage block builders
- **Analytics**: Comprehensive MEV protection statistics
- **Security Monitoring**: Track and prevent MEV attacks
- **Alert System**: Real-time notifications for critical events

### User Dashboard  
- **Wallet Integration**: Connect and manage protected wallets
- **MEV Protection**: Choose protection levels (Basic, Standard, Maximum)
- **Transaction History**: View all protected transactions
- **Rewards Tracking**: Monitor MEV savings and claim rewards
- **Protection Analytics**: Visualize MEV protection effectiveness
- **Settings Management**: Customize protection preferences

## 🚀 Quick Start

### Prerequisites
- Node.js 18+ 
- npm or yarn
- MEV Shield API running (default: localhost:8080)

### Installation

```bash
# Navigate to dashboard directory
cd dashboard

# Install dependencies
npm install

# Start Admin Dashboard (port 3001)
npm run start:admin

# OR Start User Dashboard (port 3002)  
npm run start:user

# OR Start default (port 3000)
npm start
```

## 🎨 Dashboard Screenshots

### Admin Dashboard Features

#### System Overview
- Total protected volume: $45.8M
- MEV saved: $2.3M
- Active users: 12,847
- Success rate: 98.5%

#### Key Metrics
- Real-time transaction monitoring
- MEV attack type distribution
- Chain distribution analytics
- System health monitoring

#### Management Tools
- Transaction filtering and search
- Builder reputation management
- Alert configuration
- Performance metrics

### User Dashboard Features

#### Wallet Overview
- Total balance display
- MEV savings tracker
- Pending rewards
- Protection score

#### Protection Levels

**Basic (Free)**
- Standard MEV protection
- Sandwich attack prevention
- Basic frontrun defense

**Standard (0.05% fee)**
- All Basic features
- Fair ordering
- Private mempool
- MEV redistribution

**Maximum (0.1% fee)**
- All Standard features
- Time-lock protection
- Threshold encryption
- Priority support

#### Analytics Views
- MEV savings over time
- Protection distribution
- Transaction history
- Rewards tracking

## 🔧 Configuration

### Environment Variables

Create a `.env` file in the dashboard directory:

```env
# API Configuration
REACT_APP_API_URL=http://localhost:8080
REACT_APP_WS_URL=ws://localhost:8080

# Web3 Configuration
REACT_APP_ETHEREUM_RPC=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
REACT_APP_POLYGON_RPC=https://polygon-rpc.com
REACT_APP_BSC_RPC=https://bsc-dataseed.binance.org

# Features
REACT_APP_ENABLE_TESTNET=true
REACT_APP_ENABLE_MAINNET=false
```

### API Integration

The dashboard connects to the MEV Shield API for:
- Transaction data
- Protection metrics
- System health
- User authentication
- Rewards management

## 🏗️ Project Structure

```
dashboard/
├── admin/
│   └── AdminDashboard.tsx    # Admin interface
├── user/
│   └── UserDashboard.tsx     # User interface  
├── src/
│   ├── App.tsx               # Main app component
│   ├── components/           # Shared components
│   ├── hooks/               # Custom React hooks
│   ├── services/            # API services
│   ├── utils/               # Utility functions
│   └── types/               # TypeScript types
├── public/
│   └── index.html
├── package.json
└── README.md
```

## 🔌 API Endpoints Used

### Admin Endpoints
- `GET /api/v1/admin/metrics` - System metrics
- `GET /api/v1/admin/transactions` - All transactions
- `GET /api/v1/admin/builders` - Builder network
- `GET /api/v1/admin/alerts` - System alerts
- `GET /api/v1/admin/health` - Health status

### User Endpoints
- `GET /api/v1/user/wallet` - Wallet info
- `GET /api/v1/user/transactions` - User transactions
- `GET /api/v1/user/rewards` - Rewards data
- `POST /api/v1/transactions` - Submit protected transaction
- `GET /api/v1/protection/levels` - Available protection levels

## 📱 Responsive Design

Both dashboards are fully responsive and optimized for:
- Desktop (1920x1080+)
- Laptop (1366x768+)
- Tablet (768x1024)
- Mobile (375x667+)

## 🔐 Security Features

- **Secure wallet connection** via Web3 providers
- **Transaction signing** with hardware wallet support
- **API authentication** using JWT tokens
- **Rate limiting** on all API calls
- **Input validation** and sanitization

## 📈 Performance Optimization

- React.memo for component optimization
- Lazy loading for charts and heavy components
- WebSocket for real-time updates
- Efficient state management
- Optimized re-renders

## 🧪 Testing

```bash
# Run tests
npm test

# Run tests with coverage
npm test -- --coverage

# Run specific test file
npm test AdminDashboard.test.tsx
```

## 🚀 Production Build

```bash
# Create optimized production build
npm run build

# Serve production build locally
npx serve -s build
```

## 🐛 Troubleshooting

### Common Issues

**Issue**: Dashboard not loading
- Check if MEV Shield API is running
- Verify API URL in .env file
- Check browser console for errors

**Issue**: Charts not displaying
- Clear browser cache
- Reinstall chart.js dependencies
- Check data format from API

**Issue**: Wallet connection failing
- Ensure MetaMask or wallet is installed
- Check network configuration
- Verify Web3 provider settings

## 📝 License

Apache License 2.0 - see LICENSE file

## 🤝 Contributing

See CONTRIBUTING.md for contribution guidelines

---

Built with ❤️ by Aurigraph DLT