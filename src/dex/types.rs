use ethers::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexPool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
    pub liquidity: U256,
    pub sqrt_price_x96: U256,
    pub tick: i32,
    pub dex: DexType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DexType {
    UniswapV3,
    PancakeSwapV3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapParams {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub amount_out_min: U256,
    pub recipient: Address,
    pub deadline: U256,
    pub dex: DexType,
    pub slippage_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MEVAnalysis {
    pub has_mev_risk: bool,
    pub sandwich_risk: f64,
    pub frontrun_risk: f64,
    pub estimated_loss: U256,
    pub recommended_gas_price: U256,
    pub recommended_slippage: f64,
    pub protection_strategy: ProtectionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtectionStrategy {
    PrivateMempool,
    FlashbotsBundle,
    DynamicSlippage,
    SplitTransaction,
    DelayedExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    pub reserves0: U256,
    pub reserves1: U256,
    pub block_timestamp_last: u32,
    pub cumulative_prices: Vec<U256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapEvent {
    pub sender: Address,
    pub recipient: Address,
    pub amount0: I256,
    pub amount1: I256,
    pub sqrt_price_x96: U256,
    pub liquidity: U128,
    pub tick: i32,
    pub timestamp: U256,
    pub tx_hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MEVIncident {
    pub incident_type: MEVType,
    pub victim_tx: H256,
    pub attacker_tx: Vec<H256>,
    pub profit: U256,
    pub loss: U256,
    pub timestamp: U256,
    pub pool: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MEVType {
    Sandwich,
    Frontrun,
    Backrun,
    JIT,
    Arbitrage,
}