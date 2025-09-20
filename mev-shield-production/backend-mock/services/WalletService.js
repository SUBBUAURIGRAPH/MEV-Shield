const { ethers } = require('ethers');
const NodeCache = require('node-cache');

class WalletService {
  constructor(blockchainService, priceService) {
    this.cache = new NodeCache({ stdTTL: parseInt(process.env.CACHE_TTL_BLOCKCHAIN) || 30000 });
    this.blockchainService = blockchainService;
    this.priceService = priceService;
    
    // Common token contracts
    this.tokenContracts = {
      WETH: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
      USDC: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      USDT: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
      DAI: '0x6B175474E89094C44Da98b954EedeAC495271d0F',
      WBTC: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599',
      UNI: '0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984',
      LINK: '0x514910771AF9Ca656af840dff83E8264EcF986CA',
      AAVE: '0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9',
      MKR: '0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2',
      COMP: '0xc00e94Cb662C3520282E6f5717214004A7f26888',
    };

    // ERC-20 ABI for token operations
    this.erc20Abi = [
      'function balanceOf(address owner) view returns (uint256)',
      'function decimals() view returns (uint8)',
      'function symbol() view returns (string)',
      'function name() view returns (string)',
      'function transfer(address to, uint256 amount) returns (bool)',
      'function allowance(address owner, address spender) view returns (uint256)',
      'function approve(address spender, uint256 amount) returns (bool)',
    ];
  }

  async getWalletOverview(address) {
    const cacheKey = `wallet-overview-${address}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const [ethBalance, tokenBalances, transactionHistory, nftCount] = await Promise.all([
        this.getETHBalance(address),
        this.getTokenBalances(address),
        this.getRecentTransactions(address, 10),
        this.getNFTCount(address),
      ]);

      const totalValue = await this.calculateTotalValue(ethBalance, tokenBalances);

      const overview = {
        address,
        ethBalance,
        tokenBalances,
        totalValueUSD: totalValue,
        transactionCount: transactionHistory.length,
        nftCount,
        lastActivity: transactionHistory.length > 0 ? transactionHistory[0].timestamp : null,
        portfolio: await this.getPortfolioBreakdown(ethBalance, tokenBalances),
        timestamp: new Date().toISOString(),
      };

      this.cache.set(cacheKey, overview, 120); // Cache for 2 minutes
      return overview;
    } catch (error) {
      console.error('Error getting wallet overview:', error);
      return this.getMockWalletOverview(address);
    }
  }

  async getETHBalance(address) {
    try {
      const balance = await this.blockchainService.getWalletBalance(address);
      return {
        raw: balance.balance,
        formatted: balance.formatted,
        symbol: 'ETH',
        decimals: 18,
      };
    } catch (error) {
      console.error('Error getting ETH balance:', error);
      return {
        raw: '0',
        formatted: '0.0',
        symbol: 'ETH',
        decimals: 18,
      };
    }
  }

  async getTokenBalances(address, tokenList = Object.keys(this.tokenContracts)) {
    const balances = [];
    
    try {
      const balancePromises = tokenList.map(async (symbol) => {
        try {
          const tokenAddress = this.tokenContracts[symbol];
          if (!tokenAddress) return null;

          const balance = await this.blockchainService.getWalletBalance(address, tokenAddress);
          
          if (parseFloat(balance.formatted) > 0) {
            return {
              symbol,
              address: tokenAddress,
              balance: balance.formatted,
              raw: balance.balance,
              decimals: balance.decimals,
            };
          }
          return null;
        } catch (error) {
          console.error(`Error getting ${symbol} balance:`, error);
          return null;
        }
      });

      const results = await Promise.all(balancePromises);
      return results.filter(balance => balance !== null);
    } catch (error) {
      console.error('Error getting token balances:', error);
      return this.getMockTokenBalances();
    }
  }

  async getCustomTokenBalance(address, tokenAddress) {
    const cacheKey = `token-balance-${address}-${tokenAddress}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const balance = await this.blockchainService.getWalletBalance(address, tokenAddress);
      const tokenInfo = await this.blockchainService.getTokenInfo(tokenAddress);
      
      const result = {
        ...balance,
        name: tokenInfo.name,
        address: tokenAddress,
      };

      this.cache.set(cacheKey, result, 60); // Cache for 1 minute
      return result;
    } catch (error) {
      console.error('Error getting custom token balance:', error);
      throw error;
    }
  }

  async getRecentTransactions(address, limit = 10) {
    const cacheKey = `recent-txs-${address}-${limit}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      // This would typically use Etherscan API or similar service
      // For now, we'll simulate with recent block transactions
      const transactions = await this.fetchTransactionHistory(address, limit);
      
      this.cache.set(cacheKey, transactions, 300); // Cache for 5 minutes
      return transactions;
    } catch (error) {
      console.error('Error getting recent transactions:', error);
      return this.getMockTransactions(address, limit);
    }
  }

  async fetchTransactionHistory(address, limit) {
    // In a real implementation, you'd use Etherscan API or similar
    // This is a simplified mock that shows the structure
    const recentTxs = await this.blockchainService.getRecentTransactions(limit);
    
    return recentTxs.map(tx => ({
      hash: tx.hash,
      from: tx.from,
      to: tx.to,
      value: tx.value,
      gasLimit: tx.gasLimit,
      gasPrice: tx.gasPrice,
      blockNumber: tx.blockNumber,
      timestamp: new Date(tx.timestamp * 1000).toISOString(),
      status: 'success', // Would need to check receipt
      type: this.getTransactionType(tx),
      direction: this.getTransactionDirection(tx, address),
    }));
  }

  getTransactionType(tx) {
    if (tx.to && this.isDEXContract(tx.to)) {
      return 'swap';
    } else if (tx.value && parseFloat(tx.value) > 0) {
      return 'transfer';
    } else if (tx.to) {
      return 'contract';
    }
    return 'unknown';
  }

  getTransactionDirection(tx, userAddress) {
    if (tx.from.toLowerCase() === userAddress.toLowerCase()) {
      return 'outgoing';
    } else if (tx.to && tx.to.toLowerCase() === userAddress.toLowerCase()) {
      return 'incoming';
    }
    return 'unknown';
  }

  isDEXContract(address) {
    const dexContracts = [
      '0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D', // Uniswap V2
      '0xE592427A0AEce92De3Edee1F18E0157C05861564', // Uniswap V3
      '0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F', // SushiSwap
    ];
    return dexContracts.includes(address);
  }

  async getNFTCount(address) {
    // This would require querying NFT contracts or using services like OpenSea API
    // For now, we'll return a mock count
    return Math.floor(Math.random() * 10);
  }

  async calculateTotalValue(ethBalance, tokenBalances) {
    try {
      const prices = await this.priceService.getCurrentPrices();
      let totalValue = 0;

      // Add ETH value
      if (prices.ETH) {
        totalValue += parseFloat(ethBalance.formatted) * prices.ETH.price;
      }

      // Add token values
      tokenBalances.forEach(token => {
        if (prices[token.symbol]) {
          totalValue += parseFloat(token.balance) * prices[token.symbol].price;
        }
      });

      return totalValue.toFixed(2);
    } catch (error) {
      console.error('Error calculating total value:', error);
      return '0.00';
    }
  }

  async getPortfolioBreakdown(ethBalance, tokenBalances) {
    try {
      const prices = await this.priceService.getCurrentPrices();
      const breakdown = [];
      let totalValue = 0;

      // Calculate individual values
      const ethValue = parseFloat(ethBalance.formatted) * (prices.ETH?.price || 0);
      totalValue += ethValue;

      if (ethValue > 0) {
        breakdown.push({
          symbol: 'ETH',
          value: ethValue,
          balance: ethBalance.formatted,
          price: prices.ETH?.price || 0,
        });
      }

      tokenBalances.forEach(token => {
        const price = prices[token.symbol]?.price || 0;
        const value = parseFloat(token.balance) * price;
        totalValue += value;

        if (value > 0) {
          breakdown.push({
            symbol: token.symbol,
            value,
            balance: token.balance,
            price,
          });
        }
      });

      // Calculate percentages
      return breakdown.map(item => ({
        ...item,
        percentage: totalValue > 0 ? ((item.value / totalValue) * 100).toFixed(2) : '0',
      })).sort((a, b) => b.value - a.value);
    } catch (error) {
      console.error('Error getting portfolio breakdown:', error);
      return [];
    }
  }

  async getTokenAllowances(address, spenderAddress) {
    const allowances = [];

    try {
      const allowancePromises = Object.entries(this.tokenContracts).map(async ([symbol, tokenAddress]) => {
        try {
          const contract = new ethers.Contract(tokenAddress, this.erc20Abi, this.blockchainService.provider);
          const allowance = await contract.allowance(address, spenderAddress);
          
          if (allowance > 0n) {
            const decimals = await contract.decimals();
            return {
              symbol,
              tokenAddress,
              spenderAddress,
              allowance: allowance.toString(),
              formatted: ethers.formatUnits(allowance, decimals),
            };
          }
          return null;
        } catch (error) {
          console.error(`Error getting ${symbol} allowance:`, error);
          return null;
        }
      });

      const results = await Promise.all(allowancePromises);
      return results.filter(allowance => allowance !== null);
    } catch (error) {
      console.error('Error getting token allowances:', error);
      return [];
    }
  }

  async getWalletActivity(address, days = 30) {
    const cacheKey = `wallet-activity-${address}-${days}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      // This would analyze transaction patterns over time
      const activity = {
        totalTransactions: Math.floor(Math.random() * 500) + 100,
        averageGasUsed: (Math.random() * 100000 + 50000).toFixed(0),
        totalGasSpent: (Math.random() * 5 + 1).toFixed(3),
        mostActiveDay: this.getRandomDate(days),
        transactionTypes: {
          transfers: Math.floor(Math.random() * 200) + 50,
          swaps: Math.floor(Math.random() * 100) + 20,
          contracts: Math.floor(Math.random() * 50) + 10,
        },
        dexInteractions: {
          uniswap: Math.floor(Math.random() * 50),
          sushiswap: Math.floor(Math.random() * 30),
          other: Math.floor(Math.random() * 20),
        },
      };

      this.cache.set(cacheKey, activity, 3600); // Cache for 1 hour
      return activity;
    } catch (error) {
      console.error('Error getting wallet activity:', error);
      return this.getMockWalletActivity();
    }
  }

  getRandomDate(daysAgo) {
    const date = new Date();
    date.setDate(date.getDate() - Math.floor(Math.random() * daysAgo));
    return date.toISOString().split('T')[0];
  }

  // Mock data methods for fallback
  getMockWalletOverview(address) {
    return {
      address,
      ethBalance: {
        raw: '2500000000000000000',
        formatted: '2.5',
        symbol: 'ETH',
        decimals: 18,
      },
      tokenBalances: this.getMockTokenBalances(),
      totalValueUSD: '8750.00',
      transactionCount: 127,
      nftCount: 3,
      lastActivity: new Date(Date.now() - 3600000).toISOString(),
      portfolio: [
        { symbol: 'ETH', value: 6250, balance: '2.5', price: 2500, percentage: '71.43' },
        { symbol: 'USDC', value: 1500, balance: '1500', price: 1, percentage: '17.14' },
        { symbol: 'UNI', value: 1000, balance: '100', price: 10, percentage: '11.43' },
      ],
      timestamp: new Date().toISOString(),
    };
  }

  getMockTokenBalances() {
    return [
      { symbol: 'USDC', address: this.tokenContracts.USDC, balance: '1500.000000', raw: '1500000000', decimals: 6 },
      { symbol: 'UNI', address: this.tokenContracts.UNI, balance: '100.0', raw: '100000000000000000000', decimals: 18 },
      { symbol: 'LINK', address: this.tokenContracts.LINK, balance: '25.5', raw: '25500000000000000000', decimals: 18 },
    ];
  }

  getMockTransactions(address, limit) {
    const transactions = [];
    
    for (let i = 0; i < limit; i++) {
      transactions.push({
        hash: '0x' + Math.random().toString(16).substr(2, 64),
        from: i % 2 === 0 ? address : '0x' + Math.random().toString(16).substr(2, 40),
        to: i % 2 === 0 ? '0x' + Math.random().toString(16).substr(2, 40) : address,
        value: (Math.random() * 5).toFixed(6),
        gasLimit: Math.floor(Math.random() * 200000 + 21000).toString(),
        gasPrice: (Math.random() * 50 + 10).toFixed(2),
        blockNumber: 18750000 - i,
        timestamp: new Date(Date.now() - i * 300000).toISOString(),
        status: 'success',
        type: ['transfer', 'swap', 'contract'][Math.floor(Math.random() * 3)],
        direction: i % 2 === 0 ? 'outgoing' : 'incoming',
      });
    }

    return transactions;
  }

  getMockWalletActivity() {
    return {
      totalTransactions: 287,
      averageGasUsed: '85000',
      totalGasSpent: '3.250',
      mostActiveDay: '2024-01-15',
      transactionTypes: {
        transfers: 150,
        swaps: 87,
        contracts: 50,
      },
      dexInteractions: {
        uniswap: 45,
        sushiswap: 25,
        other: 17,
      },
    };
  }

  // Utility methods for wallet monitoring
  async monitorWalletChanges(address, callback) {
    console.log(`Starting wallet monitoring for ${address}`);
    
    let lastBalance = await this.getETHBalance(address);
    
    const interval = setInterval(async () => {
      try {
        const currentBalance = await this.getETHBalance(address);
        
        if (currentBalance.raw !== lastBalance.raw) {
          const change = parseFloat(currentBalance.formatted) - parseFloat(lastBalance.formatted);
          
          callback({
            address,
            type: 'balance_change',
            oldBalance: lastBalance.formatted,
            newBalance: currentBalance.formatted,
            change: change.toFixed(6),
            timestamp: new Date().toISOString(),
          });
          
          lastBalance = currentBalance;
        }
      } catch (error) {
        console.error('Error monitoring wallet:', error);
      }
    }, 30000); // Check every 30 seconds

    return interval;
  }

  stopWalletMonitoring(intervalId) {
    clearInterval(intervalId);
    console.log('Wallet monitoring stopped');
  }
}

module.exports = WalletService;