const axios = require('axios');
const NodeCache = require('node-cache');
const { ethers } = require('ethers');

class DEXDataService {
  constructor(blockchainService) {
    this.cache = new NodeCache({ stdTTL: parseInt(process.env.CACHE_TTL_BLOCKCHAIN) || 30000 });
    this.blockchainService = blockchainService;
    
    // Uniswap V3 contract addresses
    this.uniswapV3 = {
      factory: '0x1F98431c8aD98523631AE4a59f267346ea31F984',
      router: '0xE592427A0AEce92De3Edee1F18E0157C05861564',
      quoter: '0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6',
      nftManager: '0xC36442b4a4522E871399CD717aBDD847Ab11FE88',
    };

    // SushiSwap contract addresses
    this.sushiSwap = {
      factory: '0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac',
      router: '0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F',
    };

    // Common token addresses
    this.tokens = {
      ETH: '0x0000000000000000000000000000000000000000',
      WETH: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
      USDC: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      USDT: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
      DAI: '0x6B175474E89094C44Da98b954EedeAC495271d0F',
      WBTC: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599',
      UNI: '0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984',
      LINK: '0x514910771AF9Ca656af840dff83E8264EcF986CA',
    };

    // Fee tiers for Uniswap V3
    this.feeTiers = [100, 500, 3000, 10000]; // 0.01%, 0.05%, 0.3%, 1%
  }

  async getPoolData(tokenA, tokenB, dex = 'uniswap') {
    const cacheKey = `pool-${tokenA}-${tokenB}-${dex}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      let poolData;
      
      if (dex === 'uniswap') {
        poolData = await this.getUniswapV3PoolData(tokenA, tokenB);
      } else if (dex === 'sushiswap') {
        poolData = await this.getSushiSwapPoolData(tokenA, tokenB);
      } else {
        throw new Error(`Unsupported DEX: ${dex}`);
      }

      this.cache.set(cacheKey, poolData, 60); // Cache for 1 minute
      return poolData;
    } catch (error) {
      console.error(`Error fetching ${dex} pool data:`, error);
      return this.getMockPoolData(tokenA, tokenB, dex);
    }
  }

  async getUniswapV3PoolData(tokenA, tokenB) {
    const query = `
      query($token0: String!, $token1: String!) {
        pools(
          where: {
            token0_in: [$token0, $token1]
            token1_in: [$token0, $token1]
          }
          orderBy: totalValueLockedUSD
          orderDirection: desc
          first: 5
        ) {
          id
          token0 {
            id
            symbol
            name
            decimals
          }
          token1 {
            id
            symbol
            name
            decimals
          }
          feeTier
          liquidity
          sqrtPrice
          tick
          token0Price
          token1Price
          volumeUSD
          totalValueLockedUSD
          txCount
          createdAtTimestamp
          poolHourData(
            first: 24
            orderBy: periodStartUnix
            orderDirection: desc
          ) {
            periodStartUnix
            high
            low
            open
            close
            volumeUSD
          }
        }
      }
    `;

    const variables = {
      token0: tokenA.toLowerCase(),
      token1: tokenB.toLowerCase(),
    };

    const response = await axios.post(process.env.UNISWAP_V3_SUBGRAPH_URL, {
      query,
      variables,
    });

    if (response.data.errors) {
      throw new Error('GraphQL query failed: ' + JSON.stringify(response.data.errors));
    }

    return response.data.data.pools.map(pool => ({
      address: pool.id,
      dex: 'uniswap-v3',
      token0: {
        address: pool.token0.id,
        symbol: pool.token0.symbol,
        name: pool.token0.name,
        decimals: parseInt(pool.token0.decimals),
      },
      token1: {
        address: pool.token1.id,
        symbol: pool.token1.symbol,
        name: pool.token1.name,
        decimals: parseInt(pool.token1.decimals),
      },
      feeTier: parseInt(pool.feeTier),
      liquidity: pool.liquidity,
      sqrtPriceX96: pool.sqrtPrice,
      tick: parseInt(pool.tick),
      token0Price: parseFloat(pool.token0Price),
      token1Price: parseFloat(pool.token1Price),
      volume24h: parseFloat(pool.volumeUSD),
      tvl: parseFloat(pool.totalValueLockedUSD),
      txCount: parseInt(pool.txCount),
      createdAt: new Date(parseInt(pool.createdAtTimestamp) * 1000).toISOString(),
      hourlyData: pool.poolHourData?.map(hour => ({
        timestamp: new Date(parseInt(hour.periodStartUnix) * 1000).toISOString(),
        high: parseFloat(hour.high),
        low: parseFloat(hour.low),
        open: parseFloat(hour.open),
        close: parseFloat(hour.close),
        volume: parseFloat(hour.volumeUSD),
      })) || [],
    }));
  }

  async getSushiSwapPoolData(tokenA, tokenB) {
    const query = `
      query($token0: String!, $token1: String!) {
        pairs(
          where: {
            or: [
              { and: [{ token0: $token0 }, { token1: $token1 }] }
              { and: [{ token0: $token1 }, { token1: $token0 }] }
            ]
          }
          orderBy: reserveUSD
          orderDirection: desc
          first: 5
        ) {
          id
          token0 {
            id
            symbol
            name
            decimals
          }
          token1 {
            id
            symbol
            name
            decimals
          }
          reserve0
          reserve1
          reserveUSD
          token0Price
          token1Price
          volumeUSD
          txCount
          createdAtTimestamp
        }
      }
    `;

    const variables = {
      token0: tokenA.toLowerCase(),
      token1: tokenB.toLowerCase(),
    };

    const response = await axios.post(process.env.SUSHISWAP_SUBGRAPH_URL, {
      query,
      variables,
    });

    if (response.data.errors) {
      throw new Error('GraphQL query failed: ' + JSON.stringify(response.data.errors));
    }

    return response.data.data.pairs.map(pair => ({
      address: pair.id,
      dex: 'sushiswap',
      token0: {
        address: pair.token0.id,
        symbol: pair.token0.symbol,
        name: pair.token0.name,
        decimals: parseInt(pair.token0.decimals),
      },
      token1: {
        address: pair.token1.id,
        symbol: pair.token1.symbol,
        name: pair.token1.name,
        decimals: parseInt(pair.token1.decimals),
      },
      feeTier: 3000, // SushiSwap uses 0.3% fee
      reserve0: pair.reserve0,
      reserve1: pair.reserve1,
      token0Price: parseFloat(pair.token0Price),
      token1Price: parseFloat(pair.token1Price),
      volume24h: parseFloat(pair.volumeUSD),
      tvl: parseFloat(pair.reserveUSD),
      txCount: parseInt(pair.txCount),
      createdAt: new Date(parseInt(pair.createdAtTimestamp) * 1000).toISOString(),
    }));
  }

  async getSwapQuote(tokenIn, tokenOut, amountIn, dex = 'uniswap') {
    const cacheKey = `quote-${tokenIn}-${tokenOut}-${amountIn}-${dex}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      let quote;
      
      if (dex === 'uniswap') {
        quote = await this.getUniswapV3Quote(tokenIn, tokenOut, amountIn);
      } else if (dex === 'sushiswap') {
        quote = await this.getSushiSwapQuote(tokenIn, tokenOut, amountIn);
      } else {
        throw new Error(`Unsupported DEX: ${dex}`);
      }

      this.cache.set(cacheKey, quote, 30); // Cache for 30 seconds
      return quote;
    } catch (error) {
      console.error(`Error getting ${dex} quote:`, error);
      return this.getMockSwapQuote(tokenIn, tokenOut, amountIn, dex);
    }
  }

  async getUniswapV3Quote(tokenIn, tokenOut, amountIn) {
    // For production, you would use the Uniswap Quoter contract
    // This is a simplified implementation using pool data
    const pools = await this.getPoolData(tokenIn, tokenOut, 'uniswap');
    
    if (pools.length === 0) {
      throw new Error('No pools found for this pair');
    }

    // Use the pool with highest liquidity
    const bestPool = pools[0];
    
    // Simple price calculation (in production, use the actual Quoter contract)
    const amountInBig = ethers.parseUnits(amountIn.toString(), bestPool.token0.decimals);
    const price = bestPool.token0.address.toLowerCase() === tokenIn.toLowerCase() 
      ? bestPool.token0Price 
      : bestPool.token1Price;
    
    const amountOut = parseFloat(amountIn) * price;
    const priceImpact = this.calculatePriceImpact(amountIn, bestPool.tvl);
    
    return {
      tokenIn,
      tokenOut,
      amountIn: amountIn.toString(),
      amountOut: amountOut.toString(),
      amountOutMin: (amountOut * 0.995).toString(), // 0.5% slippage
      priceImpact: priceImpact.toFixed(4),
      fee: bestPool.feeTier,
      pool: bestPool.address,
      route: [tokenIn, tokenOut],
      dex: 'uniswap-v3',
      estimatedGas: '180000',
    };
  }

  async getSushiSwapQuote(tokenIn, tokenOut, amountIn) {
    const pools = await this.getPoolData(tokenIn, tokenOut, 'sushiswap');
    
    if (pools.length === 0) {
      throw new Error('No pools found for this pair');
    }

    const bestPool = pools[0];
    
    // Calculate output using constant product formula: x * y = k
    const isToken0 = bestPool.token0.address.toLowerCase() === tokenIn.toLowerCase();
    const reserveIn = parseFloat(isToken0 ? bestPool.reserve0 : bestPool.reserve1);
    const reserveOut = parseFloat(isToken0 ? bestPool.reserve1 : bestPool.reserve0);
    
    const amountInWithFee = parseFloat(amountIn) * 0.997; // 0.3% fee
    const amountOut = (amountInWithFee * reserveOut) / (reserveIn + amountInWithFee);
    const priceImpact = this.calculatePriceImpact(amountIn, bestPool.tvl);
    
    return {
      tokenIn,
      tokenOut,
      amountIn: amountIn.toString(),
      amountOut: amountOut.toString(),
      amountOutMin: (amountOut * 0.995).toString(),
      priceImpact: priceImpact.toFixed(4),
      fee: 3000,
      pool: bestPool.address,
      route: [tokenIn, tokenOut],
      dex: 'sushiswap',
      estimatedGas: '150000',
    };
  }

  calculatePriceImpact(amountIn, tvl) {
    // Simplified price impact calculation
    const tradeSizeRatio = parseFloat(amountIn) / tvl;
    return Math.min(tradeSizeRatio * 100, 50); // Cap at 50%
  }

  async getTopPools(limit = 10) {
    const cacheKey = `top-pools-${limit}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const [uniswapPools, sushiswapPools] = await Promise.all([
        this.getTopUniswapPools(limit),
        this.getTopSushiswapPools(limit),
      ]);

      const allPools = [...uniswapPools, ...sushiswapPools]
        .sort((a, b) => b.tvl - a.tvl)
        .slice(0, limit);

      this.cache.set(cacheKey, allPools, 300); // Cache for 5 minutes
      return allPools;
    } catch (error) {
      console.error('Error fetching top pools:', error);
      return this.getMockTopPools(limit);
    }
  }

  async getTopUniswapPools(limit) {
    const query = `
      query($first: Int!) {
        pools(
          first: $first
          orderBy: totalValueLockedUSD
          orderDirection: desc
          where: { totalValueLockedUSD_gt: "1000000" }
        ) {
          id
          token0 { id symbol name }
          token1 { id symbol name }
          feeTier
          totalValueLockedUSD
          volumeUSD
          txCount
          apr
        }
      }
    `;

    const response = await axios.post(process.env.UNISWAP_V3_SUBGRAPH_URL, {
      query,
      variables: { first: limit },
    });

    return response.data.data.pools.map(pool => ({
      address: pool.id,
      dex: 'uniswap-v3',
      token0Symbol: pool.token0.symbol,
      token1Symbol: pool.token1.symbol,
      pairName: `${pool.token0.symbol}/${pool.token1.symbol}`,
      feeTier: pool.feeTier,
      tvl: parseFloat(pool.totalValueLockedUSD),
      volume24h: parseFloat(pool.volumeUSD),
      txCount: parseInt(pool.txCount),
      apr: parseFloat(pool.apr) || 0,
    }));
  }

  async getTopSushiswapPools(limit) {
    const query = `
      query($first: Int!) {
        pairs(
          first: $first
          orderBy: reserveUSD
          orderDirection: desc
          where: { reserveUSD_gt: "1000000" }
        ) {
          id
          token0 { id symbol name }
          token1 { id symbol name }
          reserveUSD
          volumeUSD
          txCount
        }
      }
    `;

    const response = await axios.post(process.env.SUSHISWAP_SUBGRAPH_URL, {
      query,
      variables: { first: limit },
    });

    return response.data.data.pairs.map(pair => ({
      address: pair.id,
      dex: 'sushiswap',
      token0Symbol: pair.token0.symbol,
      token1Symbol: pair.token1.symbol,
      pairName: `${pair.token0.symbol}/${pair.token1.symbol}`,
      feeTier: 3000,
      tvl: parseFloat(pair.reserveUSD),
      volume24h: parseFloat(pair.volumeUSD),
      txCount: parseInt(pair.txCount),
      apr: 0, // SushiSwap doesn't provide APR in this query
    }));
  }

  async getLiquidityPositions(userAddress) {
    const cacheKey = `positions-${userAddress}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      // This would query user's liquidity positions
      // Implementation depends on whether you're tracking positions
      return this.getMockLiquidityPositions(userAddress);
    } catch (error) {
      console.error('Error fetching liquidity positions:', error);
      return this.getMockLiquidityPositions(userAddress);
    }
  }

  // Mock data methods for fallback
  getMockPoolData(tokenA, tokenB, dex) {
    return [{
      address: '0x' + Math.random().toString(16).substr(2, 40),
      dex,
      token0: {
        address: tokenA,
        symbol: this.getTokenSymbol(tokenA),
        name: this.getTokenSymbol(tokenA),
        decimals: 18,
      },
      token1: {
        address: tokenB,
        symbol: this.getTokenSymbol(tokenB),
        name: this.getTokenSymbol(tokenB),
        decimals: 18,
      },
      feeTier: 3000,
      liquidity: '50000000000000000000000',
      token0Price: 2500 + Math.random() * 100,
      token1Price: 0.0004 + Math.random() * 0.0001,
      volume24h: 8500000 + Math.random() * 1000000,
      tvl: 125000000 + Math.random() * 10000000,
      txCount: 15000 + Math.floor(Math.random() * 1000),
      createdAt: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000).toISOString(),
    }];
  }

  getMockSwapQuote(tokenIn, tokenOut, amountIn, dex) {
    const mockRate = 2500; // ETH to USD rate
    const amountOut = parseFloat(amountIn) * mockRate * (0.98 + Math.random() * 0.04);
    
    return {
      tokenIn,
      tokenOut,
      amountIn: amountIn.toString(),
      amountOut: amountOut.toString(),
      amountOutMin: (amountOut * 0.995).toString(),
      priceImpact: (Math.random() * 2).toFixed(4),
      fee: 3000,
      pool: '0x' + Math.random().toString(16).substr(2, 40),
      route: [tokenIn, tokenOut],
      dex,
      estimatedGas: '150000',
    };
  }

  getMockTopPools(limit) {
    const pools = [];
    const tokens = ['ETH', 'USDC', 'USDT', 'DAI', 'WBTC', 'UNI', 'LINK'];
    
    for (let i = 0; i < limit; i++) {
      const token0 = tokens[Math.floor(Math.random() * tokens.length)];
      let token1 = tokens[Math.floor(Math.random() * tokens.length)];
      while (token1 === token0) {
        token1 = tokens[Math.floor(Math.random() * tokens.length)];
      }

      pools.push({
        address: '0x' + Math.random().toString(16).substr(2, 40),
        dex: Math.random() > 0.5 ? 'uniswap-v3' : 'sushiswap',
        token0Symbol: token0,
        token1Symbol: token1,
        pairName: `${token0}/${token1}`,
        feeTier: [500, 3000, 10000][Math.floor(Math.random() * 3)],
        tvl: Math.random() * 100000000 + 10000000,
        volume24h: Math.random() * 10000000 + 1000000,
        txCount: Math.floor(Math.random() * 10000) + 1000,
        apr: Math.random() * 50,
      });
    }

    return pools.sort((a, b) => b.tvl - a.tvl);
  }

  getMockLiquidityPositions(userAddress) {
    return [
      {
        id: 1,
        pool: '0x' + Math.random().toString(16).substr(2, 40),
        token0Symbol: 'ETH',
        token1Symbol: 'USDC',
        liquidity: '1500000000000000000',
        amount0: '0.5',
        amount1: '1250.0',
        uncollectedFees0: '0.002',
        uncollectedFees1: '5.0',
        inRange: true,
      },
    ];
  }

  getTokenSymbol(address) {
    const symbolMap = {
      [this.tokens.ETH]: 'ETH',
      [this.tokens.WETH]: 'WETH',
      [this.tokens.USDC]: 'USDC',
      [this.tokens.USDT]: 'USDT',
      [this.tokens.DAI]: 'DAI',
      [this.tokens.WBTC]: 'WBTC',
      [this.tokens.UNI]: 'UNI',
      [this.tokens.LINK]: 'LINK',
    };
    
    return symbolMap[address] || 'UNKNOWN';
  }
}

module.exports = DEXDataService;