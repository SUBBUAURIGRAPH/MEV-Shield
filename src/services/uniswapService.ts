import { ethers } from 'ethers';
import { Token, CurrencyAmount, TradeType, Percent } from '@uniswap/sdk-core';
import { Pool, Route, Trade, SwapQuoter, SwapRouter, computePoolAddress, FeeAmount } from '@uniswap/v3-sdk';
import IUniswapV3PoolABI from '@uniswap/v3-core/artifacts/contracts/interfaces/IUniswapV3Pool.sol/IUniswapV3Pool.json';
import Quoter from '@uniswap/v3-periphery/artifacts/contracts/lens/Quoter.sol/Quoter.json';

// Uniswap V3 contract addresses (Ethereum Mainnet)
const ADDRESSES = {
  SWAP_ROUTER: '0xE592427A0AEce92De3Edee1F18E0157C05861564',
  QUOTER: '0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6',
  FACTORY: '0x1F98431c8aD98523631AE4a59f267346ea31F984',
  WETH: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
  USDC: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  DAI: '0x6B175474E89094C44Da98b954EedeAC495271d0F'
};

// MEV Shield configuration
const MEV_CONFIG = {
  MAX_SLIPPAGE: 0.05, // 5%
  PROTECTION_DELAY: 2, // blocks
  MIN_PROFIT_THRESHOLD: 0.001, // 0.1% minimum profit for MEV detection
  GAS_PRICE_MULTIPLIER: 1.2 // 20% gas price buffer
};

export interface SwapParams {
  tokenIn: string;
  tokenOut: string;
  amountIn: string;
  recipient: string;
  slippageTolerance?: number;
  deadline?: number;
}

export interface ProtectedSwapResult {
  swapId: string;
  expectedOutput: string;
  minimumOutput: string;
  priceImpact: number;
  estimatedGas: string;
  mevProtection: {
    enabled: boolean;
    delay: number;
    sandwichRisk: 'low' | 'medium' | 'high';
  };
}

export class UniswapService {
  private provider: ethers.Provider;
  private signer: ethers.Signer | undefined;
  private swapRouter: ethers.Contract;
  private quoter: ethers.Contract;
  private mevShieldContract: ethers.Contract | undefined;

  constructor(provider: ethers.Provider, signer?: ethers.Signer) {
    this.provider = provider;
    this.signer = signer;
    
    // Initialize contracts
    this.swapRouter = new ethers.Contract(
      ADDRESSES.SWAP_ROUTER,
      [
        'function exactInputSingle(tuple(address,address,uint24,address,uint256,uint256,uint256,uint160)) external payable returns (uint256)',
        'function exactOutputSingle(tuple(address,address,uint24,address,uint256,uint256,uint256,uint160)) external payable returns (uint256)'
      ],
      signer || provider
    );

    this.quoter = new ethers.Contract(
      ADDRESSES.QUOTER,
      Quoter.abi,
      provider
    );
  }

  /**
   * Initialize MEV Shield contract
   */
  async initializeMEVShield(contractAddress: string) {
    const abi = [
      'function scheduleProtectedSwap(tuple(address,address,uint24,address,uint256,uint256,uint160)) returns (bytes32)',
      'function executeProtectedSwap(bytes32)',
      'function getQuote(address,address,uint24,uint256) returns (uint256)',
      'function detectSandwichAttack(tuple(address,address,uint24,address,uint256,uint256,uint160)) returns (bool)'
    ];
    
    this.mevShieldContract = new ethers.Contract(
      contractAddress,
      abi,
      this.signer || this.provider
    );
  }

  /**
   * Get quote for token swap
   */
  async getQuote(params: SwapParams): Promise<string> {
    try {
      const amountOut = await this.quoter.quoteExactInputSingle.staticCall(
        params.tokenIn,
        params.tokenOut,
        FeeAmount.MEDIUM, // 0.3% fee tier
        params.amountIn,
        0 // sqrtPriceLimitX96
      );
      
      return amountOut.toString();
    } catch (error) {
      console.error('Error getting quote:', error);
      throw error;
    }
  }

  /**
   * Analyze MEV risk for a swap
   */
  async analyzeMEVRisk(params: SwapParams): Promise<{
    risk: 'low' | 'medium' | 'high';
    factors: string[];
    recommendation: string;
  }> {
    const factors: string[] = [];
    let riskScore = 0;

    // Check swap size
    const amountInEth = ethers.formatEther(params.amountIn);
    if (parseFloat(amountInEth) > 100) {
      factors.push('Large swap size increases sandwich attack risk');
      riskScore += 3;
    } else if (parseFloat(amountInEth) > 10) {
      factors.push('Medium swap size may attract MEV bots');
      riskScore += 2;
    }

    // Check gas price
    const gasPrice = (await this.provider.getFeeData()).gasPrice;
    const block = await this.provider.getBlock('latest');
    if (gasPrice && block?.baseFeePerGas) {
      const gasPriceGwei = Number(ethers.formatUnits(gasPrice, 'gwei'));
      const baseFeeGwei = Number(ethers.formatUnits(block.baseFeePerGas, 'gwei'));
      
      if (gasPriceGwei > baseFeeGwei * 1.5) {
        factors.push('High gas price indicates network congestion');
        riskScore += 2;
      }
    }

    // Check pool liquidity
    const poolLiquidity = await this.getPoolLiquidity(params.tokenIn, params.tokenOut);
    if (poolLiquidity < 1000000) { // Less than $1M liquidity
      factors.push('Low pool liquidity increases price impact');
      riskScore += 2;
    }

    // Determine risk level
    let risk: 'low' | 'medium' | 'high';
    let recommendation: string;

    if (riskScore >= 5) {
      risk = 'high';
      recommendation = 'Use MEV protection with longer delay or split the trade';
    } else if (riskScore >= 3) {
      risk = 'medium';
      recommendation = 'Enable MEV protection for this trade';
    } else {
      risk = 'low';
      recommendation = 'Trade can proceed with standard protection';
    }

    return { risk, factors, recommendation };
  }

  /**
   * Execute protected swap with MEV Shield
   */
  async executeProtectedSwap(params: SwapParams): Promise<ProtectedSwapResult> {
    if (!this.mevShieldContract) {
      throw new Error('MEV Shield contract not initialized');
    }

    // Get quote
    const expectedOutput = await this.getQuote(params);
    
    // Calculate minimum output with slippage
    const slippage = params.slippageTolerance || MEV_CONFIG.MAX_SLIPPAGE;
    const minimumOutput = BigInt(expectedOutput) * BigInt(Math.floor((1 - slippage) * 10000)) / BigInt(10000);

    // Analyze MEV risk
    const mevRisk = await this.analyzeMEVRisk(params);

    // Prepare swap parameters
    const swapParams = {
      tokenIn: params.tokenIn,
      tokenOut: params.tokenOut,
      fee: FeeAmount.MEDIUM,
      recipient: params.recipient,
      amountIn: params.amountIn,
      amountOutMinimum: minimumOutput.toString(),
      sqrtPriceLimitX96: 0
    };

    // Estimate gas
    const estimatedGas = await this.mevShieldContract.scheduleProtectedSwap.estimateGas(swapParams);

    // Schedule protected swap
    const tx = await this.mevShieldContract.scheduleProtectedSwap(swapParams);
    const receipt = await tx.wait();
    
    // Extract swap ID from events
    const event = receipt.logs.find((log: any) => log.eventName === 'SwapScheduled');
    const swapId = event?.args?.swapId || ethers.randomBytes(32);

    // Calculate price impact
    const priceImpact = await this.calculatePriceImpact(params.tokenIn, params.tokenOut, params.amountIn);

    return {
      swapId: ethers.hexlify(swapId),
      expectedOutput: expectedOutput,
      minimumOutput: minimumOutput.toString(),
      priceImpact,
      estimatedGas: estimatedGas.toString(),
      mevProtection: {
        enabled: true,
        delay: MEV_CONFIG.PROTECTION_DELAY,
        sandwichRisk: mevRisk.risk
      }
    };
  }

  /**
   * Direct swap without MEV protection (for comparison/testing)
   */
  async executeDirectSwap(params: SwapParams): Promise<string> {
    if (!this.signer) {
      throw new Error('Signer required for swap execution');
    }

    const deadline = params.deadline || Math.floor(Date.now() / 1000) + 60 * 20; // 20 minutes
    const slippage = params.slippageTolerance || MEV_CONFIG.MAX_SLIPPAGE;
    
    // Get quote
    const expectedOutput = await this.getQuote(params);
    const minimumOutput = BigInt(expectedOutput) * BigInt(Math.floor((1 - slippage) * 10000)) / BigInt(10000);

    // Execute swap
    const tx = await this.swapRouter.exactInputSingle({
      tokenIn: params.tokenIn,
      tokenOut: params.tokenOut,
      fee: FeeAmount.MEDIUM,
      recipient: params.recipient,
      deadline: deadline,
      amountIn: params.amountIn,
      amountOutMinimum: minimumOutput,
      sqrtPriceLimitX96: 0
    });

    const receipt = await tx.wait();
    return receipt.hash;
  }

  /**
   * Get pool liquidity
   */
  private async getPoolLiquidity(tokenA: string, tokenB: string): Promise<number> {
    try {
      const poolAddress = computePoolAddress({
        factoryAddress: ADDRESSES.FACTORY,
        tokenA: new Token(1, tokenA, 18),
        tokenB: new Token(1, tokenB, 18),
        fee: FeeAmount.MEDIUM
      });

      const poolContract = new ethers.Contract(
        poolAddress,
        IUniswapV3PoolABI.abi,
        this.provider
      );

      const liquidity = await poolContract.liquidity();
      return Number(ethers.formatUnits(liquidity, 18));
    } catch (error) {
      console.error('Error fetching pool liquidity:', error);
      return 0;
    }
  }

  /**
   * Calculate price impact
   */
  private async calculatePriceImpact(
    tokenIn: string,
    tokenOut: string,
    amountIn: string
  ): Promise<number> {
    try {
      // Get quote for small amount (to get current price)
      const smallAmount = ethers.parseUnits('1', 18);
      const smallQuote = await this.getQuote({
        tokenIn,
        tokenOut,
        amountIn: smallAmount.toString(),
        recipient: ethers.ZeroAddress
      });

      // Get quote for actual amount
      const actualQuote = await this.getQuote({
        tokenIn,
        tokenOut,
        amountIn,
        recipient: ethers.ZeroAddress
      });

      // Calculate price impact
      const expectedOutput = BigInt(amountIn) * BigInt(smallQuote) / smallAmount;
      const actualOutput = BigInt(actualQuote);
      
      const impact = Number((expectedOutput - actualOutput) * BigInt(10000) / expectedOutput) / 100;
      return Math.abs(impact);
    } catch (error) {
      console.error('Error calculating price impact:', error);
      return 0;
    }
  }

  /**
   * Monitor pending swaps
   */
  async monitorPendingSwaps(callback: (swap: any) => void) {
    if (!this.mevShieldContract) {
      throw new Error('MEV Shield contract not initialized');
    }

    // Listen for swap events
    this.mevShieldContract.on('SwapScheduled', (swapId, user, tokenIn, tokenOut, amountIn) => {
      callback({
        swapId: ethers.hexlify(swapId),
        user,
        tokenIn,
        tokenOut,
        amountIn: amountIn.toString(),
        status: 'scheduled'
      });
    });

    this.mevShieldContract.on('SwapExecuted', (swapId, user, amountOut) => {
      callback({
        swapId: ethers.hexlify(swapId),
        user,
        amountOut: amountOut.toString(),
        status: 'executed'
      });
    });

    this.mevShieldContract.on('MEVProtectionTriggered', (swapId, reason) => {
      callback({
        swapId: ethers.hexlify(swapId),
        reason,
        status: 'protected'
      });
    });
  }
}

export default UniswapService;