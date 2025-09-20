const axios = require('axios');
const NodeCache = require('node-cache');

class PriceService {
  constructor() {
    this.cache = new NodeCache({ stdTTL: parseInt(process.env.CACHE_TTL_PRICES) || 60000 });
    this.coinGeckoApiKey = process.env.COINGECKO_API_KEY;
    this.coinMarketCapApiKey = process.env.COINMARKETCAP_API_KEY;
    
    // Token mapping for CoinGecko IDs
    this.tokenMapping = {
      'ethereum': { symbol: 'ETH', address: '0x0000000000000000000000000000000000000000' },
      'wrapped-bitcoin': { symbol: 'WBTC', address: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599' },
      'usd-coin': { symbol: 'USDC', address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48' },
      'tether': { symbol: 'USDT', address: '0xdAC17F958D2ee523a2206206994597C13D831ec7' },
      'dai': { symbol: 'DAI', address: '0x6B175474E89094C44Da98b954EedeAC495271d0F' },
      'chainlink': { symbol: 'LINK', address: '0x514910771AF9Ca656af840dff83E8264EcF986CA' },
      'uniswap': { symbol: 'UNI', address: '0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984' },
      'aave': { symbol: 'AAVE', address: '0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9' },
      'maker': { symbol: 'MKR', address: '0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2' },
      'compound': { symbol: 'COMP', address: '0xc00e94Cb662C3520282E6f5717214004A7f26888' },
    };

    this.reverseTokenMapping = {};
    Object.keys(this.tokenMapping).forEach(cgId => {
      const token = this.tokenMapping[cgId];
      this.reverseTokenMapping[token.symbol] = cgId;
      this.reverseTokenMapping[token.address.toLowerCase()] = cgId;
    });
  }

  async getCurrentPrices(tokens = Object.keys(this.tokenMapping)) {
    const cacheKey = `prices-${tokens.join('-')}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      // Try CoinGecko first (free tier: 50 requests/minute)
      const prices = await this.fetchCoinGeckoPrices(tokens);
      this.cache.set(cacheKey, prices, 60); // Cache for 1 minute
      return prices;
    } catch (error) {
      console.error('Error fetching prices from CoinGecko:', error);
      try {
        // Fallback to CoinMarketCap
        const prices = await this.fetchCoinMarketCapPrices(tokens);
        this.cache.set(cacheKey, prices, 60);
        return prices;
      } catch (fallbackError) {
        console.error('Error fetching prices from CoinMarketCap:', fallbackError);
        // Return mock prices as last resort
        return this.getMockPrices(tokens);
      }
    }
  }

  async fetchCoinGeckoPrices(tokens) {
    const tokenIds = tokens.filter(token => this.tokenMapping[token] || this.reverseTokenMapping[token]);
    const ids = tokenIds.map(token => this.reverseTokenMapping[token] || token).join(',');
    
    const url = this.coinGeckoApiKey && this.coinGeckoApiKey !== 'demo' 
      ? 'https://pro-api.coingecko.com/api/v3/simple/price'
      : 'https://api.coingecko.com/api/v3/simple/price';

    const params = {
      ids,
      vs_currencies: 'usd',
      include_24hr_change: true,
      include_24hr_vol: true,
      include_market_cap: true,
      include_last_updated_at: true,
    };

    if (this.coinGeckoApiKey && this.coinGeckoApiKey !== 'demo') {
      params.x_cg_pro_api_key = this.coinGeckoApiKey;
    }

    const response = await axios.get(url, { params });
    
    const prices = {};
    Object.keys(response.data).forEach(cgId => {
      const tokenData = this.tokenMapping[cgId];
      if (tokenData) {
        prices[tokenData.symbol] = {
          symbol: tokenData.symbol,
          address: tokenData.address,
          price: response.data[cgId].usd,
          change24h: response.data[cgId].usd_24h_change || 0,
          volume24h: response.data[cgId].usd_24h_vol || 0,
          marketCap: response.data[cgId].usd_market_cap || 0,
          lastUpdated: response.data[cgId].last_updated_at ? new Date(response.data[cgId].last_updated_at * 1000).toISOString() : new Date().toISOString(),
        };
      }
    });

    return prices;
  }

  async fetchCoinMarketCapPrices(tokens) {
    if (!this.coinMarketCapApiKey || this.coinMarketCapApiKey === 'demo') {
      throw new Error('CoinMarketCap API key not configured');
    }

    const symbols = tokens.map(token => {
      const mapping = this.tokenMapping[token];
      return mapping ? mapping.symbol : token;
    }).join(',');

    const response = await axios.get('https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest', {
      headers: {
        'X-CMC_PRO_API_KEY': this.coinMarketCapApiKey,
      },
      params: {
        symbol: symbols,
      },
    });

    const prices = {};
    Object.values(response.data.data).forEach(token => {
      prices[token.symbol] = {
        symbol: token.symbol,
        address: this.getTokenAddress(token.symbol),
        price: token.quote.USD.price,
        change24h: token.quote.USD.percent_change_24h || 0,
        volume24h: token.quote.USD.volume_24h || 0,
        marketCap: token.quote.USD.market_cap || 0,
        lastUpdated: token.quote.USD.last_updated,
      };
    });

    return prices;
  }

  getTokenAddress(symbol) {
    const cgId = this.reverseTokenMapping[symbol];
    return cgId ? this.tokenMapping[cgId].address : '0x0000000000000000000000000000000000000000';
  }

  async getHistoricalPrices(token, days = 7) {
    const cacheKey = `historical-${token}-${days}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const cgId = this.reverseTokenMapping[token] || token;
      
      const url = this.coinGeckoApiKey && this.coinGeckoApiKey !== 'demo'
        ? 'https://pro-api.coingecko.com/api/v3/coins/{id}/market_chart'
        : 'https://api.coingecko.com/api/v3/coins/{id}/market_chart';

      const params = {
        vs_currency: 'usd',
        days,
        interval: days <= 1 ? 'hourly' : 'daily',
      };

      if (this.coinGeckoApiKey && this.coinGeckoApiKey !== 'demo') {
        params.x_cg_pro_api_key = this.coinGeckoApiKey;
      }

      const response = await axios.get(url.replace('{id}', cgId), { params });
      
      const historicalData = {
        prices: response.data.prices.map(([timestamp, price]) => ({
          timestamp: new Date(timestamp).toISOString(),
          price,
        })),
        volumes: response.data.total_volumes.map(([timestamp, volume]) => ({
          timestamp: new Date(timestamp).toISOString(),
          volume,
        })),
        marketCaps: response.data.market_caps.map(([timestamp, marketCap]) => ({
          timestamp: new Date(timestamp).toISOString(),
          marketCap,
        })),
      };

      this.cache.set(cacheKey, historicalData, 3600); // Cache for 1 hour
      return historicalData;
    } catch (error) {
      console.error('Error fetching historical prices:', error);
      return this.getMockHistoricalPrices(token, days);
    }
  }

  async getTokenMetrics(token) {
    const cacheKey = `metrics-${token}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const cgId = this.reverseTokenMapping[token] || token;
      
      const url = this.coinGeckoApiKey && this.coinGeckoApiKey !== 'demo'
        ? 'https://pro-api.coingecko.com/api/v3/coins/{id}'
        : 'https://api.coingecko.com/api/v3/coins/{id}';

      const params = {
        localization: false,
        tickers: false,
        market_data: true,
        community_data: false,
        developer_data: false,
        sparkline: true,
      };

      if (this.coinGeckoApiKey && this.coinGeckoApiKey !== 'demo') {
        params.x_cg_pro_api_key = this.coinGeckoApiKey;
      }

      const response = await axios.get(url.replace('{id}', cgId), { params });
      
      const data = response.data;
      const marketData = data.market_data;
      
      const metrics = {
        name: data.name,
        symbol: data.symbol.toUpperCase(),
        rank: data.market_cap_rank,
        price: marketData.current_price.usd,
        marketCap: marketData.market_cap.usd,
        volume24h: marketData.total_volume.usd,
        change24h: marketData.price_change_percentage_24h,
        change7d: marketData.price_change_percentage_7d,
        change30d: marketData.price_change_percentage_30d,
        high24h: marketData.high_24h.usd,
        low24h: marketData.low_24h.usd,
        ath: marketData.ath.usd,
        athDate: marketData.ath_date.usd,
        atl: marketData.atl.usd,
        atlDate: marketData.atl_date.usd,
        circulatingSupply: marketData.circulating_supply,
        totalSupply: marketData.total_supply,
        maxSupply: marketData.max_supply,
        sparkline: marketData.sparkline_7d?.price || [],
      };

      this.cache.set(cacheKey, metrics, 300); // Cache for 5 minutes
      return metrics;
    } catch (error) {
      console.error('Error fetching token metrics:', error);
      return this.getMockTokenMetrics(token);
    }
  }

  getMockPrices(tokens) {
    const mockPrices = {
      ETH: { price: 2500, change24h: 2.5 },
      WBTC: { price: 45000, change24h: 1.8 },
      USDC: { price: 1.00, change24h: 0.01 },
      USDT: { price: 1.00, change24h: -0.01 },
      DAI: { price: 1.00, change24h: 0.02 },
      LINK: { price: 15.50, change24h: 3.2 },
      UNI: { price: 8.75, change24h: -1.5 },
      AAVE: { price: 120.00, change24h: 4.1 },
      MKR: { price: 1800.00, change24h: -0.8 },
      COMP: { price: 85.00, change24h: 2.1 },
    };

    const prices = {};
    tokens.forEach(token => {
      const symbol = this.tokenMapping[token]?.symbol || token;
      const mock = mockPrices[symbol] || { price: 1, change24h: 0 };
      prices[symbol] = {
        symbol,
        address: this.getTokenAddress(symbol),
        price: mock.price * (1 + (Math.random() - 0.5) * 0.02), // Add small random variation
        change24h: mock.change24h,
        volume24h: mock.price * 1000000 * Math.random(),
        marketCap: mock.price * 1000000000 * Math.random(),
        lastUpdated: new Date().toISOString(),
      };
    });

    return prices;
  }

  getMockHistoricalPrices(token, days) {
    const prices = [];
    const volumes = [];
    const marketCaps = [];
    const now = Date.now();
    const interval = days <= 1 ? 60 * 60 * 1000 : 24 * 60 * 60 * 1000; // 1 hour or 1 day
    const points = days <= 1 ? 24 : days;

    let basePrice = 2500; // Default ETH price
    if (token === 'WBTC') basePrice = 45000;
    else if (token.includes('USD')) basePrice = 1;
    else if (token === 'LINK') basePrice = 15.50;

    for (let i = points - 1; i >= 0; i--) {
      const timestamp = new Date(now - (i * interval)).toISOString();
      const price = basePrice * (1 + (Math.random() - 0.5) * 0.1); // ±10% variation
      const volume = price * 1000000 * Math.random();
      const marketCap = price * 1000000000 * Math.random();

      prices.push({ timestamp, price });
      volumes.push({ timestamp, volume });
      marketCaps.push({ timestamp, marketCap });
    }

    return { prices, volumes, marketCaps };
  }

  getMockTokenMetrics(token) {
    const mockData = {
      ETH: { price: 2500, marketCap: 300000000000, rank: 2 },
      WBTC: { price: 45000, marketCap: 8000000000, rank: 15 },
      USDC: { price: 1.00, marketCap: 25000000000, rank: 6 },
      USDT: { price: 1.00, marketCap: 80000000000, rank: 3 },
      DAI: { price: 1.00, marketCap: 5000000000, rank: 20 },
    };

    const data = mockData[token] || mockData.ETH;
    
    return {
      name: token,
      symbol: token,
      rank: data.rank,
      price: data.price,
      marketCap: data.marketCap,
      volume24h: data.marketCap * 0.1,
      change24h: (Math.random() - 0.5) * 10,
      change7d: (Math.random() - 0.5) * 20,
      change30d: (Math.random() - 0.5) * 50,
      high24h: data.price * 1.05,
      low24h: data.price * 0.95,
      ath: data.price * 2,
      athDate: new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString(),
      atl: data.price * 0.1,
      atlDate: new Date(Date.now() - 1000 * 24 * 60 * 60 * 1000).toISOString(),
      circulatingSupply: data.marketCap / data.price,
      totalSupply: data.marketCap / data.price * 1.1,
      maxSupply: data.marketCap / data.price * 1.2,
      sparkline: Array.from({ length: 168 }, () => data.price * (1 + (Math.random() - 0.5) * 0.05)),
    };
  }

  // Get real-time price updates for WebSocket
  async subscribeToPrice(token, callback) {
    // This would implement WebSocket subscription to price feeds
    // For now, we'll use polling as a simple implementation
    const interval = setInterval(async () => {
      try {
        const prices = await this.getCurrentPrices([token]);
        if (prices[token]) {
          callback(prices[token]);
        }
      } catch (error) {
        console.error('Error in price subscription:', error);
      }
    }, 10000); // Update every 10 seconds

    return interval;
  }

  unsubscribeFromPrice(intervalId) {
    clearInterval(intervalId);
  }
}

module.exports = PriceService;