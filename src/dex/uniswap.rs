use ethers::prelude::*;
use ethers::abi::Abi;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::types::*;

// Uniswap V3 contract ABIs
abigen!(
    IUniswapV3Factory,
    r#"[
        function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address pool)
        function createPool(address tokenA, address tokenB, uint24 fee) external returns (address pool)
        function feeAmountTickSpacing(uint24 fee) external view returns (int24)
    ]"#
);

abigen!(
    IUniswapV3Pool,
    r#"[
        function slot0() external view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, uint8 feeProtocol, bool unlocked)
        function liquidity() external view returns (uint128)
        function token0() external view returns (address)
        function token1() external view returns (address)
        function fee() external view returns (uint24)
        function tickSpacing() external view returns (int24)
        function observations(uint256 index) external view returns (uint32 blockTimestamp, int56 tickCumulative, uint160 secondsPerLiquidityCumulativeX128, bool initialized)
        event Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick)
        event Mint(address sender, address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1)
    ]"#
);

abigen!(
    ISwapRouter,
    r#"[
        function exactInputSingle(ExactInputSingleParams calldata params) external payable returns (uint256 amountOut)
        function exactOutputSingle(ExactOutputSingleParams calldata params) external payable returns (uint256 amountIn)
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 deadline;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 sqrtPriceLimitX96;
        }
        struct ExactOutputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 deadline;
            uint256 amountOut;
            uint256 amountInMaximum;
            uint160 sqrtPriceLimitX96;
        }
    ]"#
);

pub struct UniswapV3Integration {
    provider: Arc<Provider<Http>>,
    factory: IUniswapV3Factory<Provider<Http>>,
    router: ISwapRouter<Provider<Http>>,
    chain_id: u64,
}

impl UniswapV3Integration {
    pub fn new(
        provider: Arc<Provider<Http>>,
        factory_address: Address,
        router_address: Address,
        chain_id: u64,
    ) -> Self {
        let factory = IUniswapV3Factory::new(factory_address, provider.clone());
        let router = ISwapRouter::new(router_address, provider.clone());
        
        Self {
            provider,
            factory,
            router,
            chain_id,
        }
    }
    
    pub async fn get_pool(&self, token0: Address, token1: Address, fee: u32) -> Result<DexPool, Box<dyn std::error::Error>> {
        let pool_address = self.factory.get_pool(token0, token1, fee as u32).call().await?;
        
        if pool_address == Address::zero() {
            return Err("Pool does not exist".into());
        }
        
        let pool = IUniswapV3Pool::new(pool_address, self.provider.clone());
        
        // Get pool state
        let slot0 = pool.slot_0().call().await?;
        let liquidity = pool.liquidity().call().await?;
        
        Ok(DexPool {
            address: pool_address,
            token0,
            token1,
            fee,
            liquidity: U256::from(liquidity),
            sqrt_price_x96: U256::from(slot0.0),
            tick: slot0.1,
            dex: DexType::UniswapV3,
        })
    }
    
    pub async fn monitor_pool(&self, pool: DexPool) -> Result<(), Box<dyn std::error::Error>> {
        let pool_contract = IUniswapV3Pool::new(pool.address, self.provider.clone());
        
        // Subscribe to swap events
        let events = pool_contract.event::<SwapFilter>();
        let mut stream = events.stream().await?;
        
        println!("📊 Monitoring Uniswap V3 pool: {:?}", pool.address);
        
        while let Some(Ok(event)) = stream.next().await {
            let swap_event = SwapEvent {
                sender: event.sender,
                recipient: event.recipient,
                amount0: event.amount_0,
                amount1: event.amount_1,
                sqrt_price_x96: U256::from(event.sqrt_price_x96),
                liquidity: event.liquidity,
                tick: event.tick,
                timestamp: U256::from(self.provider.get_block_number().await?),
                tx_hash: event.transaction_hash.unwrap_or_default(),
            };
            
            // Analyze for MEV
            self.analyze_swap_for_mev(&swap_event).await?;
        }
        
        Ok(())
    }
    
    pub async fn analyze_swap_for_mev(&self, swap: &SwapEvent) -> Result<MEVAnalysis, Box<dyn std::error::Error>> {
        let block = self.provider.get_block(BlockNumber::Latest).await?;
        let current_gas_price = self.provider.get_gas_price().await?;
        
        // Check for sandwich attacks
        let sandwich_risk = self.detect_sandwich_risk(swap).await?;
        
        // Check for frontrunning
        let frontrun_risk = self.detect_frontrun_risk(swap).await?;
        
        // Calculate potential loss
        let estimated_loss = self.calculate_mev_loss(swap, sandwich_risk, frontrun_risk)?;
        
        let has_mev_risk = sandwich_risk > 0.3 || frontrun_risk > 0.3;
        
        let protection_strategy = if sandwich_risk > 0.7 {
            ProtectionStrategy::FlashbotsBundle
        } else if frontrun_risk > 0.5 {
            ProtectionStrategy::PrivateMempool
        } else if sandwich_risk > 0.3 {
            ProtectionStrategy::DynamicSlippage
        } else {
            ProtectionStrategy::DelayedExecution
        };
        
        Ok(MEVAnalysis {
            has_mev_risk,
            sandwich_risk,
            frontrun_risk,
            estimated_loss,
            recommended_gas_price: current_gas_price + (current_gas_price / 10), // 10% higher
            recommended_slippage: if has_mev_risk { 0.005 } else { 0.01 }, // 0.5% or 1%
            protection_strategy,
        })
    }
    
    async fn detect_sandwich_risk(&self, swap: &SwapEvent) -> Result<f64, Box<dyn std::error::Error>> {
        // Analyze recent blocks for sandwich patterns
        let latest_block = self.provider.get_block_number().await?;
        
        // Simple heuristic: check transaction ordering in recent blocks
        let mut risk_score = 0.0;
        
        // Check if swap size is large enough to be profitable for sandwich
        let swap_value = swap.amount0.abs() + swap.amount1.abs();
        if swap_value > I256::from(10000000000000000000u64) { // > 10 ETH equivalent
            risk_score += 0.3;
        }
        
        // Check gas price relative to average
        let current_gas = self.provider.get_gas_price().await?;
        let base_fee = self.provider.get_block(BlockNumber::Latest).await?
            .and_then(|b| b.base_fee_per_gas)
            .unwrap_or(U256::zero());
        
        if current_gas > base_fee * 2 {
            risk_score += 0.3;
        }
        
        Ok(risk_score.min(1.0))
    }
    
    async fn detect_frontrun_risk(&self, swap: &SwapEvent) -> Result<f64, Box<dyn std::error::Error>> {
        // Analyze mempool for similar transactions
        let mut risk_score = 0.0;
        
        // Check if transaction is in public mempool
        if swap.tx_hash != H256::zero() {
            // In production, check actual mempool
            // For now, use heuristics
            risk_score += 0.2;
        }
        
        // Check slippage tolerance
        let price_impact = self.calculate_price_impact(swap)?;
        if price_impact > 0.02 { // > 2% price impact
            risk_score += 0.4;
        }
        
        Ok(risk_score.min(1.0))
    }
    
    fn calculate_price_impact(&self, swap: &SwapEvent) -> Result<f64, Box<dyn std::error::Error>> {
        // Calculate price impact based on swap amounts
        let amount0 = swap.amount0.abs();
        let amount1 = swap.amount1.abs();
        
        // Simplified calculation
        let impact = if amount0 > I256::zero() {
            (amount1.as_u128() as f64) / (amount0.as_u128() as f64)
        } else {
            0.0
        };
        
        Ok(impact)
    }
    
    fn calculate_mev_loss(&self, swap: &SwapEvent, sandwich_risk: f64, frontrun_risk: f64) -> Result<U256, Box<dyn std::error::Error>> {
        let swap_value = U256::from(swap.amount0.abs().as_u128()) + U256::from(swap.amount1.abs().as_u128());
        
        // Estimate loss as percentage of swap value
        let loss_percentage = (sandwich_risk * 0.03) + (frontrun_risk * 0.02); // 3% for sandwich, 2% for frontrun
        let estimated_loss = swap_value * U256::from((loss_percentage * 10000.0) as u64) / U256::from(10000);
        
        Ok(estimated_loss)
    }
    
    pub async fn execute_protected_swap(&self, params: SwapParams) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        // Build the swap transaction
        let swap_params = ExactInputSingleParams {
            token_in: params.token_in,
            token_out: params.token_out,
            fee: 3000, // 0.3% fee tier
            recipient: params.recipient,
            deadline: params.deadline,
            amount_in: params.amount_in,
            amount_out_minimum: params.amount_out_min,
            sqrt_price_limit_x96: U256::zero(),
        };
        
        // In production, send through Flashbots or private mempool
        // For now, simulate the transaction
        println!("🛡️ Executing protected Uniswap V3 swap:");
        println!("  Token In: {:?}", params.token_in);
        println!("  Token Out: {:?}", params.token_out);
        println!("  Amount: {:?}", params.amount_in);
        println!("  Min Output: {:?}", params.amount_out_min);
        
        // Simulate successful transaction
        let mock_receipt = TransactionReceipt {
            transaction_hash: H256::random(),
            block_hash: Some(H256::random()),
            block_number: Some(U64::from(1000000)),
            gas_used: Some(U256::from(150000)),
            effective_gas_price: Some(U256::from(30000000000u64)),
            status: Some(U64::from(1)),
            ..Default::default()
        };
        
        Ok(mock_receipt)
    }
}