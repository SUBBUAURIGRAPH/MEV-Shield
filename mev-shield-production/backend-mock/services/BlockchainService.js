const { ethers } = require('ethers');
const axios = require('axios');
const NodeCache = require('node-cache');

class BlockchainService {
  constructor() {
    this.cache = new NodeCache({ stdTTL: parseInt(process.env.CACHE_TTL_BLOCKCHAIN) || 30000 });
    this.provider = this.initializeProvider();
    this.initializeContracts();
  }

  initializeProvider() {
    const alchemyApiKey = process.env.ALCHEMY_API_KEY;
    const infuraProjectId = process.env.INFURA_PROJECT_ID;
    
    // Primary provider (Alchemy)
    if (alchemyApiKey && alchemyApiKey !== 'demo') {
      return new ethers.AlchemyProvider('mainnet', alchemyApiKey);
    }
    
    // Fallback provider (Infura)
    if (infuraProjectId && infuraProjectId !== 'demo') {
      return new ethers.InfuraProvider('mainnet', infuraProjectId);
    }
    
    // Default provider (rate limited)
    return ethers.getDefaultProvider('mainnet');
  }

  initializeContracts() {
    // Common token addresses on Ethereum mainnet
    this.tokens = {
      ETH: '0x0000000000000000000000000000000000000000',
      WETH: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
      USDC: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      USDT: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
      DAI: '0x6B175474E89094C44Da98b954EedeAC495271d0F',
      WBTC: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599',
    };

    // ERC-20 ABI for token contracts
    this.erc20Abi = [
      'function balanceOf(address owner) view returns (uint256)',
      'function decimals() view returns (uint8)',
      'function symbol() view returns (string)',
      'function name() view returns (string)',
      'function totalSupply() view returns (uint256)',
    ];
  }

  async getLatestBlock() {
    const cacheKey = 'latest-block';
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const block = await this.provider.getBlock('latest');
      const blockData = {
        number: block.number,
        hash: block.hash,
        timestamp: block.timestamp,
        gasLimit: block.gasLimit.toString(),
        gasUsed: block.gasUsed.toString(),
        baseFeePerGas: block.baseFeePerGas ? block.baseFeePerGas.toString() : '0',
        transactions: block.transactions.length,
      };

      this.cache.set(cacheKey, blockData, 15); // Cache for 15 seconds
      return blockData;
    } catch (error) {
      console.error('Error fetching latest block:', error);
      throw error;
    }
  }

  async getNetworkStats() {
    const cacheKey = 'network-stats';
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const [block, gasPrice, chainId] = await Promise.all([
        this.getLatestBlock(),
        this.provider.getFeeData(),
        this.provider.getNetwork(),
      ]);

      const stats = {
        chainId: chainId.chainId,
        blockNumber: block.number,
        gasPrice: {
          legacy: gasPrice.gasPrice ? ethers.formatUnits(gasPrice.gasPrice, 'gwei') : '0',
          maxFeePerGas: gasPrice.maxFeePerGas ? ethers.formatUnits(gasPrice.maxFeePerGas, 'gwei') : '0',
          maxPriorityFeePerGas: gasPrice.maxPriorityFeePerGas ? ethers.formatUnits(gasPrice.maxPriorityFeePerGas, 'gwei') : '0',
        },
        baseFeePerGas: block.baseFeePerGas ? ethers.formatUnits(block.baseFeePerGas, 'gwei') : '0',
        blockTime: 12, // Ethereum average block time
        tps: Math.round(block.transactions / 12), // Rough TPS calculation
      };

      this.cache.set(cacheKey, stats, 30); // Cache for 30 seconds
      return stats;
    } catch (error) {
      console.error('Error fetching network stats:', error);
      throw error;
    }
  }

  async getWalletBalance(address, tokenAddress = null) {
    const cacheKey = `balance-${address}-${tokenAddress || 'ETH'}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      let balance;
      let decimals = 18;
      let symbol = 'ETH';

      if (!tokenAddress || tokenAddress === this.tokens.ETH) {
        // ETH balance
        balance = await this.provider.getBalance(address);
      } else {
        // ERC-20 token balance
        const contract = new ethers.Contract(tokenAddress, this.erc20Abi, this.provider);
        [balance, decimals, symbol] = await Promise.all([
          contract.balanceOf(address),
          contract.decimals(),
          contract.symbol(),
        ]);
      }

      const balanceData = {
        address,
        tokenAddress: tokenAddress || this.tokens.ETH,
        balance: balance.toString(),
        formatted: ethers.formatUnits(balance, decimals),
        decimals,
        symbol,
      };

      this.cache.set(cacheKey, balanceData, 60); // Cache for 1 minute
      return balanceData;
    } catch (error) {
      console.error('Error fetching wallet balance:', error);
      throw error;
    }
  }

  async getTransaction(txHash) {
    const cacheKey = `tx-${txHash}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const [tx, receipt] = await Promise.all([
        this.provider.getTransaction(txHash),
        this.provider.getTransactionReceipt(txHash),
      ]);

      if (!tx) {
        throw new Error('Transaction not found');
      }

      const txData = {
        hash: tx.hash,
        from: tx.from,
        to: tx.to,
        value: tx.value.toString(),
        gasLimit: tx.gasLimit.toString(),
        gasPrice: tx.gasPrice ? tx.gasPrice.toString() : '0',
        maxFeePerGas: tx.maxFeePerGas ? tx.maxFeePerGas.toString() : '0',
        maxPriorityFeePerGas: tx.maxPriorityFeePerGas ? tx.maxPriorityFeePerGas.toString() : '0',
        nonce: tx.nonce,
        data: tx.data,
        blockNumber: tx.blockNumber,
        blockHash: tx.blockHash,
        transactionIndex: tx.index,
        status: receipt ? (receipt.status === 1 ? 'success' : 'failed') : 'pending',
        gasUsed: receipt ? receipt.gasUsed.toString() : null,
        effectiveGasPrice: receipt && receipt.effectiveGasPrice ? receipt.effectiveGasPrice.toString() : null,
        logs: receipt ? receipt.logs : [],
      };

      this.cache.set(cacheKey, txData, 300); // Cache for 5 minutes
      return txData;
    } catch (error) {
      console.error('Error fetching transaction:', error);
      throw error;
    }
  }

  async getRecentTransactions(count = 10) {
    const cacheKey = `recent-txs-${count}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const latestBlock = await this.provider.getBlock('latest', true);
      const transactions = latestBlock.transactions.slice(0, count).map(tx => ({
        hash: tx.hash,
        from: tx.from,
        to: tx.to,
        value: ethers.formatEther(tx.value),
        gasLimit: tx.gasLimit.toString(),
        gasPrice: tx.gasPrice ? ethers.formatUnits(tx.gasPrice, 'gwei') : '0',
        blockNumber: tx.blockNumber,
        timestamp: latestBlock.timestamp,
      }));

      this.cache.set(cacheKey, transactions, 30); // Cache for 30 seconds
      return transactions;
    } catch (error) {
      console.error('Error fetching recent transactions:', error);
      throw error;
    }
  }

  async getTokenInfo(tokenAddress) {
    const cacheKey = `token-info-${tokenAddress}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const contract = new ethers.Contract(tokenAddress, this.erc20Abi, this.provider);
      const [name, symbol, decimals, totalSupply] = await Promise.all([
        contract.name(),
        contract.symbol(),
        contract.decimals(),
        contract.totalSupply(),
      ]);

      const tokenInfo = {
        address: tokenAddress,
        name,
        symbol,
        decimals,
        totalSupply: totalSupply.toString(),
        formattedTotalSupply: ethers.formatUnits(totalSupply, decimals),
      };

      this.cache.set(cacheKey, tokenInfo, 3600); // Cache for 1 hour
      return tokenInfo;
    } catch (error) {
      console.error('Error fetching token info:', error);
      throw error;
    }
  }

  async estimateGas(transaction) {
    try {
      const gasEstimate = await this.provider.estimateGas(transaction);
      const feeData = await this.provider.getFeeData();

      return {
        gasLimit: gasEstimate.toString(),
        gasPrice: feeData.gasPrice ? feeData.gasPrice.toString() : '0',
        maxFeePerGas: feeData.maxFeePerGas ? feeData.maxFeePerGas.toString() : '0',
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas ? feeData.maxPriorityFeePerGas.toString() : '0',
        estimatedCost: feeData.gasPrice ? (gasEstimate * feeData.gasPrice).toString() : '0',
      };
    } catch (error) {
      console.error('Error estimating gas:', error);
      throw error;
    }
  }

  // Fetch historical gas prices using Etherscan API
  async getHistoricalGasData(days = 7) {
    const cacheKey = `gas-history-${days}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const etherscanKey = process.env.ETHERSCAN_API_KEY;
      if (!etherscanKey || etherscanKey === 'YourApiKeyToken') {
        // Return mock data if no API key
        return this.getMockGasHistory(days);
      }

      const endTime = Math.floor(Date.now() / 1000);
      const startTime = endTime - (days * 24 * 60 * 60);

      const response = await axios.get('https://api.etherscan.io/api', {
        params: {
          module: 'gastracker',
          action: 'gasoracle',
          apikey: etherscanKey,
        },
      });

      if (response.data.status === '1') {
        const gasData = {
          safe: response.data.result.SafeGasPrice,
          standard: response.data.result.ProposeGasPrice,
          fast: response.data.result.FastGasPrice,
          timestamp: Date.now(),
        };

        this.cache.set(cacheKey, gasData, 300); // Cache for 5 minutes
        return gasData;
      }

      throw new Error('Failed to fetch gas data from Etherscan');
    } catch (error) {
      console.error('Error fetching historical gas data:', error);
      return this.getMockGasHistory(days);
    }
  }

  getMockGasHistory(days) {
    const gasHistory = [];
    const now = Date.now();
    
    for (let i = days - 1; i >= 0; i--) {
      const timestamp = now - (i * 24 * 60 * 60 * 1000);
      gasHistory.push({
        timestamp,
        safe: Math.floor(Math.random() * 10) + 15, // 15-25 gwei
        standard: Math.floor(Math.random() * 15) + 25, // 25-40 gwei
        fast: Math.floor(Math.random() * 20) + 40, // 40-60 gwei
      });
    }

    return gasHistory;
  }
}

module.exports = BlockchainService;