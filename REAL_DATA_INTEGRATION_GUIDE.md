# MEV Shield Real Blockchain Data Integration

This guide describes the comprehensive real blockchain data integration that has been implemented to replace all dummy/mock data with live blockchain data from actual APIs.

## 🎯 Overview

The MEV Shield platform now integrates with real blockchain data sources:

- **Ethereum Blockchain**: Real block data, gas prices, network statistics via Alchemy/Infura/Default providers
- **Token Prices**: Live prices from CoinGecko API with CoinMarketCap fallback
- **DEX Data**: Real pool data and swap quotes from Uniswap V3 and SushiSwap via The Graph
- **MEV Analytics**: Actual MEV metrics integration with Flashbots APIs
- **Wallet Data**: Real wallet balances and transaction history from blockchain
- **Real-time Updates**: WebSocket server for live price feeds and MEV alerts

## 🏗️ Architecture

### Backend Services

1. **BlockchainService.js** - Core blockchain data provider
   - Real Ethereum network statistics
   - Live gas price data
   - Block information and transaction details
   - Token contract interactions

2. **PriceService.js** - Token price data provider
   - CoinGecko API integration (primary)
   - CoinMarketCap API (fallback)
   - Historical price data
   - Token metrics and market data

3. **DEXDataService.js** - DEX protocol integration
   - Uniswap V3 pool data via The Graph
   - SushiSwap pool information
   - Real swap quotes and price calculations
   - Liquidity provider data

4. **MEVDataService.js** - MEV analytics and monitoring
   - Flashbots relay data integration
   - Real MEV threat detection
   - Builder reputation and statistics
   - MEV attack pattern analysis

5. **WalletService.js** - Wallet and portfolio management
   - Real wallet balance tracking
   - Transaction history analysis
   - Portfolio value calculations
   - Token allowance monitoring

### Real-time Features

- **WebSocket Server**: Live updates for prices, blocks, and MEV alerts
- **Caching System**: Intelligent caching to reduce API calls and improve performance
- **Rate Limiting**: Built-in protection against API rate limits
- **Fallback System**: Automatic fallback to mock data when APIs are unavailable

## 🚀 Getting Started

### 1. Install Dependencies

The real data server requires additional blockchain integration packages:

```bash
cd backend-mock
npm install
```

### 2. Configure API Keys

Copy the environment template and add your API keys:

```bash
cp .env.example .env
```

Edit `.env` with your API keys:

```env
# Essential for production use
ALCHEMY_API_KEY=your_alchemy_api_key_here
COINGECKO_API_KEY=your_coingecko_api_key_here
ETHERSCAN_API_KEY=your_etherscan_api_key_here

# Optional for enhanced functionality
INFURA_PROJECT_ID=your_infura_project_id_here
COINMARKETCAP_API_KEY=your_coinmarketcap_api_key_here
```

### 3. Start the Real Data Server

```bash
# Start with real blockchain data
npm run start:real

# Or for development with auto-restart
npm run dev:real
```

The server will start on port 8090 (configurable via PORT environment variable).

### 4. Verify Integration

Test the integration with these endpoints:

```bash
# Health check
curl http://localhost:8090/health

# Latest blockchain data
curl http://localhost:8090/api/blockchain/latest-block

# Current token prices
curl http://localhost:8090/api/prices/current

# DEX pool data
curl http://localhost:8090/api/dex/top-pools

# MEV metrics
curl http://localhost:8090/api/mev/metrics
```

## 📡 API Endpoints

### Blockchain Data

- `GET /api/blockchain/latest-block` - Latest Ethereum block
- `GET /api/blockchain/network-stats` - Network statistics and gas prices
- `GET /api/blockchain/gas-data?days=7` - Historical gas price data
- `GET /api/blockchain/transaction/:hash` - Transaction details

### Price Data

- `GET /api/prices/current?tokens=ethereum,bitcoin` - Current token prices
- `GET /api/prices/historical/:token?days=7` - Historical price data
- `GET /api/prices/metrics/:token` - Comprehensive token metrics

### DEX Integration

- `GET /api/dex/pools/:tokenA/:tokenB?dex=uniswap` - Pool data for token pair
- `POST /api/dex/quote` - Real swap quotes
- `GET /api/dex/top-pools?limit=10` - Top liquidity pools

### MEV Analytics

- `GET /api/mev/flashbots` - Flashbots relay data
- `POST /api/mev/analyze` - MEV threat analysis for transactions
- `GET /api/mev/metrics?timeframe=24h` - MEV statistics

### Wallet Data

- `GET /api/wallet/:address/overview` - Complete wallet overview
- `GET /api/wallet/:address/balance/:token?` - Token balances
- `GET /api/wallet/:address/transactions?limit=10` - Transaction history
- `GET /api/wallet/:address/activity?days=30` - Wallet activity analysis

## 🔌 WebSocket Real-time Updates

Connect to the WebSocket server for live updates:

```javascript
const ws = new WebSocket('ws://localhost:8081');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch(data.type) {
    case 'price_update':
      // Handle real-time price updates
      break;
    case 'new_block':
      // Handle new blockchain blocks
      break;
    case 'mev_alert':
      // Handle MEV attack alerts
      break;
  }
};

// Subscribe to specific channels
ws.send(JSON.stringify({
  type: 'subscribe',
  channels: ['prices', 'blocks', 'mev']
}));
```

## 🛡️ Rate Limiting and Caching

### Built-in Protection

- **API Rate Limiting**: 100 requests per minute per IP
- **Expensive Operations**: 10 requests per minute for complex operations
- **Intelligent Caching**: 
  - Prices: 1 minute cache
  - Blockchain data: 30 seconds cache
  - MEV data: 5 minutes cache

### Fallback Strategy

When APIs are rate-limited or unavailable:

1. **Primary Source Fails**: Automatically try fallback APIs
2. **All Sources Fail**: Return cached data if available
3. **No Cache**: Return realistic mock data with clear indicators

## 🔧 Configuration Options

### Environment Variables

```env
# Blockchain Providers
ALCHEMY_API_KEY=your_key_here
INFURA_PROJECT_ID=your_project_id
ETHERSCAN_API_KEY=your_key_here

# Price Data APIs
COINGECKO_API_KEY=your_key_here
COINMARKETCAP_API_KEY=your_key_here

# MEV Data Sources
FLASHBOTS_API_URL=https://relay.flashbots.net
FLASHBOTS_REPUTATION_URL=https://relay-analytics.flashbots.net

# DEX Data Sources (The Graph)
UNISWAP_V3_SUBGRAPH_URL=https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3
SUSHISWAP_SUBGRAPH_URL=https://api.thegraph.com/subgraphs/name/sushiswap/exchange

# Caching and Rate Limiting
CACHE_TTL_PRICES=60000          # 1 minute
CACHE_TTL_BLOCKCHAIN=30000      # 30 seconds
CACHE_TTL_MEV=300000            # 5 minutes
RATE_LIMIT_WINDOW_MS=60000      # 1 minute window
RATE_LIMIT_MAX_REQUESTS=100     # Max requests per window

# Server Configuration
PORT=8090                       # API server port
WS_PORT=8081                   # WebSocket server port
NODE_ENV=production
LOG_LEVEL=info
```

### Performance Tuning

- **Cache TTL**: Adjust cache durations based on your needs
- **Rate Limits**: Modify limits based on your API quotas
- **Update Intervals**: Configure real-time update frequencies

## 🚨 Error Handling

The system includes comprehensive error handling:

### API Failures
- Automatic retry with exponential backoff
- Fallback to alternative data sources
- Graceful degradation to cached/mock data

### Rate Limiting
- Automatic detection of 429 responses
- Intelligent request spacing
- Cache utilization during rate limit periods

### Network Issues
- Timeout handling for slow APIs
- Connection retry mechanisms
- Offline mode capabilities

## 📊 Monitoring and Logging

### Health Monitoring

```bash
# Check service health
curl http://localhost:8090/health

# Expected response:
{
  "status": "healthy",
  "timestamp": "2024-01-19T18:10:25.828Z",
  "services": {
    "blockchain": "connected",
    "prices": "connected", 
    "dex": "available",
    "mev": "available",
    "wallet": "available"
  },
  "version": "2.0.0"
}
```

### Log Levels

- **INFO**: Normal operation logs
- **WARN**: API rate limits, fallback usage
- **ERROR**: API failures, connection issues
- **DEBUG**: Detailed request/response data

## 🔄 Migration from Mock Data

### Automatic Migration

The real data server maintains API compatibility with the mock server:

1. **Same Endpoints**: All existing endpoints work unchanged
2. **Compatible Responses**: Response formats remain consistent
3. **Graceful Fallback**: Automatic fallback when real data unavailable

### Frontend Changes

No frontend changes required! The dashboard will automatically display real data when the real data server is running.

To switch between mock and real data:

```bash
# Mock data (existing)
npm start

# Real data (new)
npm run start:real
```

## 🎯 Production Deployment

### API Key Setup

For production use, obtain API keys from:

1. **Alchemy** - Ethereum blockchain data (free tier: 300M compute units/month)
2. **CoinGecko** - Token prices (free tier: 50 requests/minute)
3. **Etherscan** - Additional blockchain data (free tier: 5 calls/second)
4. **CoinMarketCap** - Price data fallback (optional)

### Security Considerations

- Store API keys in environment variables
- Use rate limiting to protect your quotas
- Monitor API usage and costs
- Implement proper error handling

### Scalability

- Use Redis for distributed caching
- Implement load balancing for multiple instances
- Monitor API quotas and usage patterns
- Consider upgrading to paid API tiers for high traffic

## 🛠️ Troubleshooting

### Common Issues

**Rate Limited APIs**
- Expected behavior for free tiers
- System automatically falls back to mock data
- Consider upgrading to paid API plans

**Slow Response Times**
- Due to external API calls
- Caching reduces subsequent calls
- Use WebSocket for real-time data

**Connection Errors**
- Check internet connectivity
- Verify API keys are correct
- Review firewall settings

### Debug Mode

Enable debug logging:

```bash
LOG_LEVEL=debug npm run start:real
```

This provides detailed information about API calls, caching, and error handling.

## 📈 What's New

### Real Data Integration

✅ **Live Ethereum Data**: Real blocks, gas prices, network stats
✅ **Token Prices**: Live prices from CoinGecko with 1-minute updates  
✅ **DEX Integration**: Real Uniswap V3 and SushiSwap pool data
✅ **MEV Analytics**: Actual MEV metrics and threat detection
✅ **Wallet Tracking**: Real wallet balances and transaction history
✅ **WebSocket Updates**: Live price feeds and MEV alerts
✅ **Smart Caching**: Reduces API calls while maintaining freshness
✅ **Rate Limiting**: Protects against API quota exhaustion
✅ **Fallback System**: Graceful degradation when APIs unavailable

### Enhanced Features

- **Historical Data**: 7-day, 30-day price and gas price history
- **Portfolio Tracking**: Real-time portfolio values and breakdowns  
- **MEV Protection**: Actual threat analysis for transactions
- **DEX Quotes**: Real swap quotes with price impact calculations
- **Network Monitoring**: Live Ethereum network statistics

---

## 🎉 Success!

Your MEV Shield platform now runs on **100% real blockchain data**! 

The system intelligently combines multiple data sources, implements smart caching, and provides graceful fallbacks to ensure consistent operation even when external APIs experience issues.

For support or questions about the real data integration, refer to the service logs or the individual service files in the `services/` directory.