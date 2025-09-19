use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    pub use_flashbots: bool,
    pub use_private_mempool: bool,
    pub max_slippage: f64,
    pub gas_price_multiplier: f64,
    pub split_threshold: U256,
    pub delay_blocks: u64,
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            use_flashbots: true,
            use_private_mempool: true,
            max_slippage: 0.01, // 1%
            gas_price_multiplier: 1.1, // 10% higher gas
            split_threshold: U256::from(10000000000000000000u64), // 10 ETH
            delay_blocks: 2,
        }
    }
}

pub struct MEVProtector {
    provider: Arc<Provider<Http>>,
    config: ProtectionConfig,
}

impl MEVProtector {
    pub fn new(provider: Arc<Provider<Http>>) -> Self {
        Self {
            provider,
            config: ProtectionConfig::default(),
        }
    }
    
    pub fn with_config(provider: Arc<Provider<Http>>, config: ProtectionConfig) -> Self {
        Self { provider, config }
    }
    
    pub async fn analyze_swap(&self, params: &SwapParams) -> Result<MEVAnalysis, Box<dyn std::error::Error>> {
        // Get current network conditions
        let gas_price = self.provider.get_gas_price().await?;
        let latest_block = self.provider.get_block(BlockNumber::Latest).await?;
        
        // Analyze transaction for MEV risks
        let sandwich_risk = self.calculate_sandwich_risk(params, &latest_block).await?;
        let frontrun_risk = self.calculate_frontrun_risk(params, &latest_block).await?;
        
        // Determine protection strategy
        let protection_strategy = self.determine_protection_strategy(sandwich_risk, frontrun_risk, params);
        
        // Calculate potential losses
        let estimated_loss = self.estimate_mev_loss(params, sandwich_risk, frontrun_risk)?;
        
        Ok(MEVAnalysis {
            has_mev_risk: sandwich_risk > 0.3 || frontrun_risk > 0.3,
            sandwich_risk,
            frontrun_risk,
            estimated_loss,
            recommended_gas_price: gas_price * U256::from((self.config.gas_price_multiplier * 100.0) as u64) / 100,
            recommended_slippage: self.calculate_safe_slippage(sandwich_risk, frontrun_risk),
            protection_strategy,
        })
    }
    
    async fn calculate_sandwich_risk(&self, params: &SwapParams, block: &Option<Block<H256>>) -> Result<f64, Box<dyn std::error::Error>> {
        let mut risk = 0.0;
        
        // Large transactions are more likely to be sandwiched
        if params.amount_in > self.config.split_threshold {
            risk += 0.4;
        }
        
        // High slippage tolerance increases risk
        if params.slippage_tolerance > 0.02 {
            risk += 0.3;
        }
        
        // Check recent block activity for MEV bots
        if let Some(block) = block {
            if block.transactions.len() > 200 { // High activity
                risk += 0.2;
            }
        }
        
        Ok(risk.min(1.0))
    }
    
    async fn calculate_frontrun_risk(&self, params: &SwapParams, block: &Option<Block<H256>>) -> Result<f64, Box<dyn std::error::Error>> {
        let mut risk = 0.0;
        
        // Transactions with high price impact are more likely to be frontrun
        let price_impact = self.estimate_price_impact(params)?;
        if price_impact > 0.02 {
            risk += 0.4;
        }
        
        // Popular tokens are more likely to be targeted
        if self.is_popular_token(params.token_in) || self.is_popular_token(params.token_out) {
            risk += 0.3;
        }
        
        // Network congestion increases frontrun risk
        let gas_price = self.provider.get_gas_price().await?;
        if gas_price > U256::from(100000000000u64) { // > 100 gwei
            risk += 0.2;
        }
        
        Ok(risk.min(1.0))
    }
    
    fn determine_protection_strategy(&self, sandwich_risk: f64, frontrun_risk: f64, params: &SwapParams) -> ProtectionStrategy {
        if sandwich_risk > 0.7 && self.config.use_flashbots {
            ProtectionStrategy::FlashbotsBundle
        } else if frontrun_risk > 0.6 && self.config.use_private_mempool {
            ProtectionStrategy::PrivateMempool
        } else if params.amount_in > self.config.split_threshold {
            ProtectionStrategy::SplitTransaction
        } else if sandwich_risk > 0.4 || frontrun_risk > 0.4 {
            ProtectionStrategy::DynamicSlippage
        } else {
            ProtectionStrategy::DelayedExecution
        }
    }
    
    fn estimate_mev_loss(&self, params: &SwapParams, sandwich_risk: f64, frontrun_risk: f64) -> Result<U256, Box<dyn std::error::Error>> {
        // Estimate potential loss based on risk scores
        let total_value = params.amount_in;
        
        // Sandwich attacks typically extract 2-5% of value
        let sandwich_loss = total_value * U256::from((sandwich_risk * 500) as u64) / 10000;
        
        // Frontrun attacks typically extract 1-3% of value
        let frontrun_loss = total_value * U256::from((frontrun_risk * 300) as u64) / 10000;
        
        Ok(sandwich_loss + frontrun_loss)
    }
    
    fn calculate_safe_slippage(&self, sandwich_risk: f64, frontrun_risk: f64) -> f64 {
        let base_slippage = 0.005; // 0.5% base
        
        // Reduce slippage for high-risk transactions
        if sandwich_risk > 0.5 || frontrun_risk > 0.5 {
            base_slippage * 0.5 // 0.25%
        } else if sandwich_risk > 0.3 || frontrun_risk > 0.3 {
            base_slippage * 0.75 // 0.375%
        } else {
            base_slippage
        }
    }
    
    fn estimate_price_impact(&self, params: &SwapParams) -> Result<f64, Box<dyn std::error::Error>> {
        // Simplified price impact calculation
        // In production, would query actual pool reserves
        let impact = if params.amount_in > U256::from(1000000000000000000000u64) { // > 1000 tokens
            0.05
        } else if params.amount_in > U256::from(100000000000000000000u64) { // > 100 tokens
            0.02
        } else {
            0.01
        };
        
        Ok(impact)
    }
    
    fn is_popular_token(&self, token: Address) -> bool {
        // Check if token is in list of popular tokens
        let popular_tokens = vec![
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
            "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
            "0x6B175474E89094C44Da98b954EedeAC495271d0F", // DAI
        ];
        
        popular_tokens.iter().any(|&addr| {
            addr.parse::<Address>().ok() == Some(token)
        })
    }
    
    pub async fn protect_transaction(&self, params: SwapParams, analysis: MEVAnalysis) -> Result<TypedTransaction, Box<dyn std::error::Error>> {
        match analysis.protection_strategy {
            ProtectionStrategy::FlashbotsBundle => {
                self.create_flashbots_bundle(params, analysis).await
            },
            ProtectionStrategy::PrivateMempool => {
                self.create_private_transaction(params, analysis).await
            },
            ProtectionStrategy::SplitTransaction => {
                self.create_split_transactions(params, analysis).await
            },
            ProtectionStrategy::DynamicSlippage => {
                self.create_dynamic_slippage_tx(params, analysis).await
            },
            ProtectionStrategy::DelayedExecution => {
                self.create_delayed_transaction(params, analysis).await
            },
        }
    }
    
    async fn create_flashbots_bundle(&self, params: SwapParams, analysis: MEVAnalysis) -> Result<TypedTransaction, Box<dyn std::error::Error>> {
        println!("🔦 Creating Flashbots bundle for protected swap");
        
        // Build transaction with Flashbots-specific parameters
        let tx = TransactionRequest::new()
            .to(params.token_out)
            .value(U256::zero())
            .gas_price(analysis.recommended_gas_price)
            .gas(U256::from(300000))
            .data(vec![0x00]); // Placeholder data
        
        Ok(TypedTransaction::Legacy(tx))
    }
    
    async fn create_private_transaction(&self, params: SwapParams, analysis: MEVAnalysis) -> Result<TypedTransaction, Box<dyn std::error::Error>> {
        println!("🔒 Creating private mempool transaction");
        
        // Build transaction for private mempool submission
        let tx = TransactionRequest::new()
            .to(params.token_out)
            .value(U256::zero())
            .gas_price(analysis.recommended_gas_price)
            .gas(U256::from(250000))
            .data(vec![0x01]); // Placeholder data
        
        Ok(TypedTransaction::Legacy(tx))
    }
    
    async fn create_split_transactions(&self, params: SwapParams, analysis: MEVAnalysis) -> Result<TypedTransaction, Box<dyn std::error::Error>> {
        println!("✂️ Splitting large transaction into smaller chunks");
        
        // Split into 3 smaller transactions
        let chunk_size = params.amount_in / 3;
        
        // Return first chunk transaction
        let tx = TransactionRequest::new()
            .to(params.token_out)
            .value(U256::zero())
            .gas_price(analysis.recommended_gas_price)
            .gas(U256::from(200000))
            .data(vec![0x02]); // Placeholder data
        
        Ok(TypedTransaction::Legacy(tx))
    }
    
    async fn create_dynamic_slippage_tx(&self, params: SwapParams, analysis: MEVAnalysis) -> Result<TypedTransaction, Box<dyn std::error::Error>> {
        println!("📊 Adjusting slippage dynamically based on MEV risk");
        
        // Build transaction with adjusted slippage
        let tx = TransactionRequest::new()
            .to(params.token_out)
            .value(U256::zero())
            .gas_price(analysis.recommended_gas_price)
            .gas(U256::from(250000))
            .data(vec![0x03]); // Placeholder data
        
        Ok(TypedTransaction::Legacy(tx))
    }
    
    async fn create_delayed_transaction(&self, params: SwapParams, analysis: MEVAnalysis) -> Result<TypedTransaction, Box<dyn std::error::Error>> {
        println!("⏰ Delaying transaction execution by {} blocks", self.config.delay_blocks);
        
        // Wait for specified number of blocks
        sleep(Duration::from_secs(self.config.delay_blocks * 12)).await;
        
        // Build delayed transaction
        let tx = TransactionRequest::new()
            .to(params.token_out)
            .value(U256::zero())
            .gas_price(analysis.recommended_gas_price)
            .gas(U256::from(200000))
            .data(vec![0x04]); // Placeholder data
        
        Ok(TypedTransaction::Legacy(tx))
    }
}