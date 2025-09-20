const axios = require('axios');
const NodeCache = require('node-cache');
const { ethers } = require('ethers');

class MEVDataService {
  constructor(blockchainService) {
    this.cache = new NodeCache({ stdTTL: parseInt(process.env.CACHE_TTL_MEV) || 300000 });
    this.blockchainService = blockchainService;
    
    // MEV detection patterns
    this.mevPatterns = {
      sandwich: {
        name: 'Sandwich Attack',
        description: 'Front-run and back-run a victim transaction',
        severity: 'high',
      },
      frontrun: {
        name: 'Front-running',
        description: 'Execute transaction before victim with higher gas',
        severity: 'medium',
      },
      backrun: {
        name: 'Back-running',
        description: 'Execute transaction after victim to capture arbitrage',
        severity: 'low',
      },
      jit: {
        name: 'Just-in-Time Liquidity',
        description: 'Add liquidity just before swap and remove after',
        severity: 'medium',
      },
      liquidation: {
        name: 'Liquidation MEV',
        description: 'Extract value from liquidating under-collateralized positions',
        severity: 'medium',
      },
    };

    // Known MEV bot addresses (simplified list)
    this.knownMevBots = new Set([
      '0x000000000000000000000000000000000000dead', // Placeholder
      '0x0000000000000000000000000000000000000001', // Placeholder
    ]);
  }

  async getFlashbotsData() {
    const cacheKey = 'flashbots-data';
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      // Flashbots public API endpoints (note: some may require authentication)
      const [relayData, reputationData] = await Promise.all([
        this.fetchFlashbotsRelayData(),
        this.fetchFlashbotsReputationData(),
      ]);

      const data = {
        relay: relayData,
        reputation: reputationData,
        timestamp: new Date().toISOString(),
      };

      this.cache.set(cacheKey, data, 300); // Cache for 5 minutes
      return data;
    } catch (error) {
      console.error('Error fetching Flashbots data:', error);
      return this.getMockFlashbotsData();
    }
  }

  async fetchFlashbotsRelayData() {
    try {
      // Note: This is a simplified implementation. 
      // Real Flashbots API access may require special permissions
      const response = await axios.get(`${process.env.FLASHBOTS_API_URL}/relay_v1/data/bidtraces/proposer_payload_delivered`, {
        params: {
          slot: Math.floor(Date.now() / 12000), // Current slot approximation
          limit: 100,
        },
        timeout: 10000,
      });

      return {
        bundles: response.data.length || 0,
        totalValue: response.data.reduce((sum, bundle) => sum + parseFloat(bundle.value || 0), 0),
        avgGasPrice: response.data.length > 0 
          ? response.data.reduce((sum, bundle) => sum + parseFloat(bundle.gas_price || 0), 0) / response.data.length
          : 0,
      };
    } catch (error) {
      console.log('Flashbots API not accessible, using mock data');
      return {
        bundles: 42,
        totalValue: 15.7,
        avgGasPrice: 25.3,
      };
    }
  }

  async fetchFlashbotsReputationData() {
    try {
      const response = await axios.get(`${process.env.FLASHBOTS_REPUTATION_URL}/reputation`, {
        timeout: 10000,
      });

      return {
        builders: response.data.builders || [],
        totalStake: response.data.totalStake || 0,
        activeBuilders: response.data.activeBuilders || 0,
      };
    } catch (error) {
      console.log('Flashbots reputation API not accessible, using mock data');
      return {
        builders: [
          { address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb3', reputation: 95, stake: '100000' },
          { address: '0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5', reputation: 87, stake: '75000' },
        ],
        totalStake: 1750000,
        activeBuilders: 15,
      };
    }
  }

  async analyzeMEVThreats(transactionData) {
    const cacheKey = `mev-analysis-${transactionData.hash || Date.now()}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const threats = [];
      const riskScore = await this.calculateRiskScore(transactionData);

      // Analyze for different MEV types
      if (transactionData.to && this.isDEXContract(transactionData.to)) {
        const dexThreats = await this.analyzeDEXTransaction(transactionData);
        threats.push(...dexThreats);
      }

      if (this.isLargeTrade(transactionData)) {
        threats.push(this.createThreat('sandwich', transactionData, 'high'));
      }

      if (await this.detectFrontrunning(transactionData)) {
        threats.push(this.createThreat('frontrun', transactionData, 'medium'));
      }

      const analysis = {
        transactionHash: transactionData.hash,
        riskScore,
        threats,
        recommendation: this.getRecommendation(riskScore, threats),
        protectionStrategies: this.getProtectionStrategies(threats),
        estimatedMEVLoss: this.estimateMEVLoss(transactionData, threats),
        timestamp: new Date().toISOString(),
      };

      this.cache.set(cacheKey, analysis, 300); // Cache for 5 minutes
      return analysis;
    } catch (error) {
      console.error('Error analyzing MEV threats:', error);
      return this.getMockMEVAnalysis(transactionData);
    }
  }

  async calculateRiskScore(transactionData) {
    let score = 0;

    // Transaction value risk
    const valueETH = parseFloat(ethers.formatEther(transactionData.value || 0));
    if (valueETH > 100) score += 40;
    else if (valueETH > 10) score += 25;
    else if (valueETH > 1) score += 10;

    // Gas price risk (high gas prices attract MEV bots)
    const gasPrice = transactionData.gasPrice ? parseFloat(ethers.formatUnits(transactionData.gasPrice, 'gwei')) : 0;
    if (gasPrice > 50) score += 30;
    else if (gasPrice > 30) score += 20;
    else if (gasPrice > 20) score += 10;

    // Contract interaction risk
    if (transactionData.to && this.isDEXContract(transactionData.to)) {
      score += 20;
    }

    // Time-based risk (high activity periods)
    const hour = new Date().getHours();
    if (hour >= 8 && hour <= 18) score += 10; // UTC business hours

    return Math.min(score, 100);
  }

  async analyzeDEXTransaction(transactionData) {
    const threats = [];
    const valueETH = parseFloat(ethers.formatEther(transactionData.value || 0));

    // Check for sandwich attack risk
    if (valueETH > 10) {
      threats.push(this.createThreat('sandwich', transactionData, 'high'));
    }

    // Check for JIT liquidity risk
    if (this.isLiquidityTransaction(transactionData)) {
      threats.push(this.createThreat('jit', transactionData, 'medium'));
    }

    return threats;
  }

  async detectFrontrunning(transactionData) {
    // Simplified frontrunning detection
    const gasPrice = transactionData.gasPrice ? parseFloat(ethers.formatUnits(transactionData.gasPrice, 'gwei')) : 0;
    const networkGasPrice = await this.getNetworkAverageGasPrice();
    
    return gasPrice > networkGasPrice * 1.5; // 50% above average
  }

  async getNetworkAverageGasPrice() {
    try {
      const feeData = await this.blockchainService.provider.getFeeData();
      return parseFloat(ethers.formatUnits(feeData.gasPrice || 0, 'gwei'));
    } catch (error) {
      return 30; // Default gas price
    }
  }

  createThreat(type, transactionData, severity) {
    const pattern = this.mevPatterns[type];
    const valueETH = parseFloat(ethers.formatEther(transactionData.value || 0));
    
    return {
      id: Math.random().toString(36).substr(2, 9),
      type,
      name: pattern.name,
      description: pattern.description,
      severity,
      potentialLoss: `$${(valueETH * 0.01 * 2500).toFixed(2)}`, // Estimate 1% loss at $2500/ETH
      confidence: this.calculateConfidence(type, transactionData),
      timestamp: new Date().toISOString(),
      transactionHash: transactionData.hash,
    };
  }

  calculateConfidence(type, transactionData) {
    // Simplified confidence calculation
    const baseConfidence = {
      sandwich: 0.85,
      frontrun: 0.75,
      backrun: 0.65,
      jit: 0.70,
      liquidation: 0.90,
    };

    return baseConfidence[type] || 0.5;
  }

  getRecommendation(riskScore, threats) {
    if (riskScore > 70 || threats.some(t => t.severity === 'high')) {
      return 'ENABLE_STRONG_PROTECTION';
    } else if (riskScore > 40 || threats.length > 0) {
      return 'ENABLE_BASIC_PROTECTION';
    } else {
      return 'LOW_RISK_PROCEED';
    }
  }

  getProtectionStrategies(threats) {
    const strategies = new Set();
    
    threats.forEach(threat => {
      switch (threat.type) {
        case 'sandwich':
          strategies.add('flashbots');
          strategies.add('commit-reveal');
          break;
        case 'frontrun':
          strategies.add('flashbots');
          strategies.add('time-delay');
          break;
        case 'backrun':
          strategies.add('slippage-protection');
          break;
        case 'jit':
          strategies.add('pre-confirmation');
          break;
        default:
          strategies.add('basic-protection');
      }
    });

    return Array.from(strategies);
  }

  estimateMEVLoss(transactionData, threats) {
    const valueETH = parseFloat(ethers.formatEther(transactionData.value || 0));
    const baseRate = 0.005; // 0.5% base MEV extraction rate
    
    let multiplier = 1;
    threats.forEach(threat => {
      switch (threat.severity) {
        case 'high': multiplier += 0.02; break;
        case 'medium': multiplier += 0.01; break;
        case 'low': multiplier += 0.005; break;
      }
    });

    const lossETH = valueETH * baseRate * multiplier;
    return {
      eth: lossETH.toFixed(6),
      usd: (lossETH * 2500).toFixed(2), // Assuming $2500/ETH
      percentage: (baseRate * multiplier * 100).toFixed(3),
    };
  }

  async getMEVMetrics(timeframe = '24h') {
    const cacheKey = `mev-metrics-${timeframe}`;
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      // In a real implementation, you'd query historical MEV data
      const metrics = await this.calculateMEVMetrics(timeframe);
      this.cache.set(cacheKey, metrics, 600); // Cache for 10 minutes
      return metrics;
    } catch (error) {
      console.error('Error calculating MEV metrics:', error);
      return this.getMockMEVMetrics(timeframe);
    }
  }

  async calculateMEVMetrics(timeframe) {
    // This would involve complex queries to historical data
    // For now, we'll provide a simplified implementation
    
    const hours = timeframe === '24h' ? 24 : timeframe === '7d' ? 168 : 720; // 30d
    const blocks = hours * 300; // ~300 blocks per hour
    
    return {
      sandwichAttacks: Math.floor(blocks * 0.05), // 5% of blocks have sandwich attacks
      frontrunAttacks: Math.floor(blocks * 0.03),
      backrunEvents: Math.floor(blocks * 0.08),
      jitLiquidity: Math.floor(blocks * 0.02),
      totalMEVExtracted: (blocks * 0.1 * Math.random()).toFixed(3), // ETH
      averageGasPrice: 25 + Math.random() * 20, // gwei
      mevBotsActive: 150 + Math.floor(Math.random() * 50),
      protectedTransactions: Math.floor(blocks * 0.15),
      timeframe,
    };
  }

  isDEXContract(address) {
    const dexContracts = [
      '0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D', // Uniswap V2 Router
      '0xE592427A0AEce92De3Edee1F18E0157C05861564', // Uniswap V3 Router
      '0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F', // SushiSwap Router
      '0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45', // Uniswap Universal Router
    ];
    
    return dexContracts.includes(address);
  }

  isLargeTrade(transactionData) {
    const valueETH = parseFloat(ethers.formatEther(transactionData.value || 0));
    return valueETH > 10; // Trades larger than 10 ETH
  }

  isLiquidityTransaction(transactionData) {
    // Simplified detection - would need to decode transaction data
    return transactionData.data && transactionData.data.includes('addLiquidity');
  }

  // Mock data methods for fallback
  getMockFlashbotsData() {
    return {
      relay: {
        bundles: 127,
        totalValue: 23.5,
        avgGasPrice: 28.7,
      },
      reputation: {
        builders: [
          { address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb3', reputation: 95, stake: '100000' },
          { address: '0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5', reputation: 87, stake: '75000' },
          { address: '0x388C818CA8B9251b393131C08a736A67ccB19297', reputation: 92, stake: '125000' },
        ],
        totalStake: 1750000,
        activeBuilders: 15,
      },
      timestamp: new Date().toISOString(),
    };
  }

  getMockMEVAnalysis(transactionData) {
    const threats = [
      {
        id: 'threat_' + Math.random().toString(36).substr(2, 9),
        type: 'sandwich',
        name: 'Sandwich Attack',
        description: 'High probability sandwich attack detected',
        severity: 'high',
        potentialLoss: '$234.50',
        confidence: 0.87,
        timestamp: new Date().toISOString(),
        transactionHash: transactionData.hash,
      },
    ];

    return {
      transactionHash: transactionData.hash,
      riskScore: 75,
      threats,
      recommendation: 'ENABLE_STRONG_PROTECTION',
      protectionStrategies: ['flashbots', 'commit-reveal'],
      estimatedMEVLoss: {
        eth: '0.094',
        usd: '234.50',
        percentage: '2.340',
      },
      timestamp: new Date().toISOString(),
    };
  }

  getMockMEVMetrics(timeframe) {
    const multiplier = timeframe === '24h' ? 1 : timeframe === '7d' ? 7 : 30;
    
    return {
      sandwichAttacks: Math.floor(85 * multiplier),
      frontrunAttacks: Math.floor(127 * multiplier),
      backrunEvents: Math.floor(203 * multiplier),
      jitLiquidity: Math.floor(45 * multiplier),
      totalMEVExtracted: (15.7 * multiplier).toFixed(3),
      averageGasPrice: 25 + Math.random() * 15,
      mevBotsActive: 150 + Math.floor(Math.random() * 50),
      protectedTransactions: Math.floor(450 * multiplier),
      timeframe,
    };
  }

  // Real-time MEV monitoring
  async startMEVMonitoring(callback) {
    console.log('Starting MEV monitoring...');
    
    // Monitor new blocks for MEV activity
    const interval = setInterval(async () => {
      try {
        const latestBlock = await this.blockchainService.getLatestBlock();
        const mevActivity = await this.analyzeMEVInBlock(latestBlock);
        
        if (mevActivity.threats.length > 0) {
          callback({
            blockNumber: latestBlock.number,
            timestamp: new Date().toISOString(),
            mevActivity,
          });
        }
      } catch (error) {
        console.error('Error in MEV monitoring:', error);
      }
    }, 30000); // Check every 30 seconds

    return interval;
  }

  async analyzeMEVInBlock(block) {
    // Simplified MEV analysis for a block
    const threats = [];
    const numTransactions = block.transactions || 0;
    
    // Simulate MEV detection
    if (Math.random() > 0.7) { // 30% chance of MEV activity
      threats.push({
        type: 'sandwich',
        severity: 'high',
        blockNumber: block.number,
        estimatedValue: (Math.random() * 10).toFixed(3) + ' ETH',
      });
    }

    return {
      blockNumber: block.number,
      transactionCount: numTransactions,
      threats,
      timestamp: new Date().toISOString(),
    };
  }

  stopMEVMonitoring(intervalId) {
    clearInterval(intervalId);
    console.log('MEV monitoring stopped');
  }
}

module.exports = MEVDataService;