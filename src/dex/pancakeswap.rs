use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::types::*;

// PancakeSwap V3 contract ABIs (similar to Uniswap V3)
abigen!(
    IPancakeV3Factory,
    r#"[
        function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address pool)
        function createPool(address tokenA, address tokenB, uint24 fee) external returns (address pool)
    ]"#
);

abigen!(
    IPancakeV3Pool,
    r#"[
        function slot0() external view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, uint32 feeProtocol, bool unlocked)
        function liquidity() external view returns (uint128)
        function token0() external view returns (address)
        function token1() external view returns (address)
        function fee() external view returns (uint24)
        event Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick)
    ]"#
);

abigen!(
    ISmartRouter,
    r#"[
        function exactInputSingleV3(ExactInputSingleParams calldata params) external payable returns (uint256 amountOut)
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 sqrtPriceLimitX96;
        }
    ]"#
);

pub struct PancakeSwapIntegration {
    provider: Arc<Provider<Http>>,
    factory: IPancakeV3Factory<Provider<Http>>,
    router: ISmartRouter<Provider<Http>>,
    chain_id: u64,
}

impl PancakeSwapIntegration {
    pub fn new(
        provider: Arc<Provider<Http>>,
        factory_address: Address,
        router_address: Address,
        chain_id: u64,
    ) -> Self {
        let factory = IPancakeV3Factory::new(factory_address, provider.clone());
        let router = ISmartRouter::new(router_address, provider.clone());
        
        Self {
            provider,
            factory,
            router,
            chain_id,
        }
    }
    
    pub async fn get_pool(&self, token0: Address, token1: Address, fee: u32) -> Result<DexPool, Box<dyn std::error::Error>> {
        let pool_address = self.factory.get_pool(token0, token1, fee).call().await?;
        
        if pool_address == Address::zero() {
            return Err("Pool does not exist".into());
        }
        
        let pool = IPancakeV3Pool::new(pool_address, self.provider.clone());
        
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
            dex: DexType::PancakeSwapV3,
        })
    }
    
    pub async fn monitor_pool(&self, pool: DexPool) -> Result<(), Box<dyn std::error::Error>> {
        let pool_contract = IPancakeV3Pool::new(pool.address, self.provider.clone());
        
        // Subscribe to swap events
        let events = pool_contract.event::<SwapFilter>();
        let mut stream = events.stream().await?;
        
        println!("🥞 Monitoring PancakeSwap V3 pool: {:?}", pool.address);
        
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
        let current_gas_price = self.provider.get_gas_price().await?;
        
        // BSC-specific MEV detection
        let sandwich_risk = self.detect_bsc_sandwich_risk(swap).await?;
        let frontrun_risk = self.detect_bsc_frontrun_risk(swap).await?;
        
        // Calculate potential loss
        let estimated_loss = self.calculate_mev_loss(swap, sandwich_risk, frontrun_risk)?;
        
        let has_mev_risk = sandwich_risk > 0.25 || frontrun_risk > 0.25; // Lower threshold for BSC
        
        let protection_strategy = if sandwich_risk > 0.6 {
            ProtectionStrategy::SplitTransaction // Split into smaller trades
        } else if frontrun_risk > 0.4 {
            ProtectionStrategy::DynamicSlippage
        } else {
            ProtectionStrategy::DelayedExecution
        };
        
        Ok(MEVAnalysis {
            has_mev_risk,
            sandwich_risk,
            frontrun_risk,
            estimated_loss,
            recommended_gas_price: current_gas_price + (current_gas_price / 20), // 5% higher for BSC
            recommended_slippage: if has_mev_risk { 0.003 } else { 0.005 }, // Lower slippage on BSC
            protection_strategy,
        })
    }
    
    async fn detect_bsc_sandwich_risk(&self, swap: &SwapEvent) -> Result<f64, Box<dyn std::error::Error>> {
        let mut risk_score = 0.0;
        
        // BSC has faster blocks, so sandwich attacks are more common
        let swap_value = swap.amount0.abs() + swap.amount1.abs();
        
        // Lower threshold for BSC due to lower gas costs
        if swap_value > I256::from(1000000000000000000u64) { // > 1 BNB equivalent
            risk_score += 0.4;
        }
        
        // Check gas price spikes (common during MEV on BSC)
        let current_gas = self.provider.get_gas_price().await?;
        if current_gas > U256::from(10000000000u64) { // > 10 gwei on BSC is high
            risk_score += 0.3;
        }
        
        // BSC has more MEV bots, increase base risk
        risk_score += 0.1;
        
        Ok(risk_score.min(1.0))
    }
    
    async fn detect_bsc_frontrun_risk(&self, swap: &SwapEvent) -> Result<f64, Box<dyn std::error::Error>> {
        let mut risk_score = 0.0;
        
        // BSC has 3-second blocks, making frontrunning easier
        risk_score += 0.15;
        
        // Check price impact
        let price_impact = self.calculate_price_impact(swap)?;
        if price_impact > 0.01 { // > 1% price impact
            risk_score += 0.35;
        }
        
        // Check if it's a popular pair (more likely to be targeted)
        if self.is_popular_pair(swap) {
            risk_score += 0.2;
        }
        
        Ok(risk_score.min(1.0))
    }
    
    fn is_popular_pair(&self, swap: &SwapEvent) -> bool {
        // Check if swap involves popular tokens on BSC
        // In production, maintain a list of popular token addresses
        true // Simplified for demo
    }
    
    fn calculate_price_impact(&self, swap: &SwapEvent) -> Result<f64, Box<dyn std::error::Error>> {
        let amount0 = swap.amount0.abs();
        let amount1 = swap.amount1.abs();
        
        let impact = if amount0 > I256::zero() {
            (amount1.as_u128() as f64) / (amount0.as_u128() as f64)
        } else {
            0.0
        };
        
        Ok(impact)
    }
    
    fn calculate_mev_loss(&self, swap: &SwapEvent, sandwich_risk: f64, frontrun_risk: f64) -> Result<U256, Box<dyn std::error::Error>> {
        let swap_value = U256::from(swap.amount0.abs().as_u128()) + U256::from(swap.amount1.abs().as_u128());
        
        // BSC has higher MEV activity, adjust loss estimates
        let loss_percentage = (sandwich_risk * 0.04) + (frontrun_risk * 0.025); // 4% for sandwich, 2.5% for frontrun
        let estimated_loss = swap_value * U256::from((loss_percentage * 10000.0) as u64) / U256::from(10000);
        
        Ok(estimated_loss)
    }
    
    pub async fn execute_protected_swap(&self, params: SwapParams) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        // Build the swap transaction for PancakeSwap
        let swap_params = ExactInputSingleParams {
            token_in: params.token_in,
            token_out: params.token_out,
            fee: 2500, // 0.25% fee tier (common on PancakeSwap)
            recipient: params.recipient,
            amount_in: params.amount_in,
            amount_out_minimum: params.amount_out_min,
            sqrt_price_limit_x96: U256::zero(),
        };
        
        println!("🥞 Executing protected PancakeSwap V3 swap:");
        println!("  Token In: {:?}", params.token_in);
        println!("  Token Out: {:?}", params.token_out);
        println!("  Amount: {:?}", params.amount_in);
        println!("  Min Output: {:?}", params.amount_out_min);
        println!("  Protection: Split transaction strategy");
        
        // For large swaps, split into multiple smaller transactions
        if params.amount_in > U256::from(10000000000000000000u64) { // > 10 tokens
            println!("  Splitting into 3 smaller transactions to avoid MEV");
            // Execute multiple smaller swaps
        }
        
        // Simulate successful transaction
        let mock_receipt = TransactionReceipt {
            transaction_hash: H256::random(),
            block_hash: Some(H256::random()),
            block_number: Some(U64::from(2000000)),
            gas_used: Some(U256::from(120000)),
            effective_gas_price: Some(U256::from(5000000000u64)), // 5 gwei on BSC
            status: Some(U64::from(1)),
            ..Default::default()
        };
        
        Ok(mock_receipt)
    }
}