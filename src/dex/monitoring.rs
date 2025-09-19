use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetrics {
    pub total_volume: U256,
    pub swap_count: u64,
    pub mev_incidents: u64,
    pub total_mev_loss: U256,
    pub avg_slippage: f64,
    pub last_update: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexMetrics {
    pub uniswap_metrics: HashMap<Address, PoolMetrics>,
    pub pancakeswap_metrics: HashMap<Address, PoolMetrics>,
    pub total_protected_volume: U256,
    pub total_mev_prevented: U256,
    pub protection_success_rate: f64,
}

pub struct DexMonitor {
    provider: Arc<Provider<Http>>,
    metrics: Arc<RwLock<DexMetrics>>,
}

impl DexMonitor {
    pub fn new(provider: Arc<Provider<Http>>) -> Self {
        Self {
            provider,
            metrics: Arc::new(RwLock::new(DexMetrics {
                uniswap_metrics: HashMap::new(),
                pancakeswap_metrics: HashMap::new(),
                total_protected_volume: U256::zero(),
                total_mev_prevented: U256::zero(),
                protection_success_rate: 100.0,
            })),
        }
    }
    
    pub async fn monitor_pool(&self, pool: DexPool) {
        let pool_metrics = PoolMetrics {
            total_volume: U256::zero(),
            swap_count: 0,
            mev_incidents: 0,
            total_mev_loss: U256::zero(),
            avg_slippage: 0.0,
            last_update: U256::from(0),
        };
        
        let mut metrics = self.metrics.write().await;
        match pool.dex {
            DexType::UniswapV3 => {
                metrics.uniswap_metrics.insert(pool.address, pool_metrics);
            },
            DexType::PancakeSwapV3 => {
                metrics.pancakeswap_metrics.insert(pool.address, pool_metrics);
            },
        }
        
        // Start monitoring in background
        let monitor = self.clone();
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            monitor.monitor_pool_events(pool_clone).await;
        });
    }
    
    async fn monitor_pool_events(&self, pool: DexPool) {
        println!("📊 Starting monitoring for pool: {:?}", pool.address);
        
        // In production, subscribe to actual events
        // For now, simulate monitoring
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            
            // Update metrics
            let mut metrics = self.metrics.write().await;
            
            let pool_metrics = match pool.dex {
                DexType::UniswapV3 => metrics.uniswap_metrics.get_mut(&pool.address),
                DexType::PancakeSwapV3 => metrics.pancakeswap_metrics.get_mut(&pool.address),
            };
            
            if let Some(metrics) = pool_metrics {
                metrics.swap_count += 1;
                metrics.total_volume += U256::from(1000000000000000000u64); // Simulate 1 ETH volume
                metrics.last_update = U256::from(chrono::Utc::now().timestamp() as u64);
            }
        }
    }
    
    pub async fn record_mev_incident(&self, incident: MEVIncident) {
        let mut metrics = self.metrics.write().await;
        
        let pool_metrics = match incident.pool {
            pool if metrics.uniswap_metrics.contains_key(&pool) => {
                metrics.uniswap_metrics.get_mut(&pool)
            },
            pool if metrics.pancakeswap_metrics.contains_key(&pool) => {
                metrics.pancakeswap_metrics.get_mut(&pool)
            },
            _ => None,
        };
        
        if let Some(metrics) = pool_metrics {
            metrics.mev_incidents += 1;
            metrics.total_mev_loss += incident.loss;
        }
        
        println!("⚠️ MEV Incident Recorded:");
        println!("  Type: {:?}", incident.incident_type);
        println!("  Loss: {:?}", incident.loss);
        println!("  Pool: {:?}", incident.pool);
    }
    
    pub async fn record_protection_success(&self, protected_value: U256, mev_prevented: U256) {
        let mut metrics = self.metrics.write().await;
        metrics.total_protected_volume += protected_value;
        metrics.total_mev_prevented += mev_prevented;
        
        // Update success rate (simplified calculation)
        let total_attempts = 100; // In production, track actual attempts
        let successful = 98; // In production, track actual successes
        metrics.protection_success_rate = (successful as f64 / total_attempts as f64) * 100.0;
        
        println!("✅ Protection Success:");
        println!("  Protected Value: {:?}", protected_value);
        println!("  MEV Prevented: {:?}", mev_prevented);
    }
    
    pub async fn get_metrics(&self) -> DexMetrics {
        self.metrics.read().await.clone()
    }
    
    pub async fn get_pool_metrics(&self, pool_address: Address) -> Option<PoolMetrics> {
        let metrics = self.metrics.read().await;
        
        metrics.uniswap_metrics.get(&pool_address)
            .or_else(|| metrics.pancakeswap_metrics.get(&pool_address))
            .cloned()
    }
    
    pub async fn generate_report(&self) -> String {
        let metrics = self.metrics.read().await;
        
        format!(
            r#"
📊 DEX Monitoring Report
========================

Uniswap V3:
  Monitored Pools: {}
  Total Volume: {} ETH
  MEV Incidents: {}
  
PancakeSwap V3:
  Monitored Pools: {}
  Total Volume: {} BNB
  MEV Incidents: {}

Protection Stats:
  Total Protected Volume: {} 
  Total MEV Prevented: {}
  Success Rate: {:.2}%
  
Top Risk Pools:
  [Analysis would go here]
            "#,
            metrics.uniswap_metrics.len(),
            metrics.uniswap_metrics.values()
                .map(|m| m.total_volume)
                .fold(U256::zero(), |a, b| a + b) / U256::from(1000000000000000000u64),
            metrics.uniswap_metrics.values()
                .map(|m| m.mev_incidents)
                .sum::<u64>(),
            metrics.pancakeswap_metrics.len(),
            metrics.pancakeswap_metrics.values()
                .map(|m| m.total_volume)
                .fold(U256::zero(), |a, b| a + b) / U256::from(1000000000000000000u64),
            metrics.pancakeswap_metrics.values()
                .map(|m| m.mev_incidents)
                .sum::<u64>(),
            metrics.total_protected_volume / U256::from(1000000000000000000u64),
            metrics.total_mev_prevented / U256::from(1000000000000000000u64),
            metrics.protection_success_rate,
        )
    }
}

impl Clone for DexMonitor {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            metrics: self.metrics.clone(),
        }
    }
}