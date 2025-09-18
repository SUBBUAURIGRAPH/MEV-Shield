//! Block Builder Coordinator for MEV Shield
//!
//! Manages decentralized block building with reputation system

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};

use crate::{
    types::*,
    config::BlockBuilderConfig,
    error::MEVShieldError,
};

/// Decentralized Block Builder Coordinator
pub struct BlockBuilder {
    config: BlockBuilderConfig,
    builders: Arc<RwLock<HashMap<Address, BlockBuilder>>>,
    reputation_system: ReputationSystem,
    proposal_aggregator: ProposalAggregator,
}


#[derive(Clone, Debug)]
pub struct BuilderInfo {
    pub address: Address,
    pub reputation_score: f64,
    pub blocks_built: u64,
    pub blocks_accepted: u64,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub stake: U256,
    pub is_active: bool,
}

pub struct ReputationSystem {
    // Simplified for now
}

pub struct ProposalAggregator {
    // Simplified for now
}

impl BlockBuilder {
    /// Create a new block builder coordinator
    pub async fn new(config: BlockBuilderConfig) -> Result<Self> {
        Ok(Self {
            config,
            builders: Arc::new(RwLock::new(HashMap::new())),
            reputation_system: ReputationSystem {},
            proposal_aggregator: ProposalAggregator {},
        })
    }
    
    pub async fn start_coordinator(&self) -> Result<()> {
        info!("Starting block builder coordinator");
        Ok(())
    }
}

/// Reputation System for block builders
pub struct ReputationSystem {
    scores: Arc<RwLock<HashMap<Address, ReputationScore>>>,
}

#[derive(Clone, Debug)]
pub struct ReputationScore {
    pub builder: Address,
    pub score: f64,
    pub total_blocks: u64,
    pub successful_blocks: u64,
    pub failed_blocks: u64,
    pub mev_incidents: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Proposal Aggregator for block proposals
pub struct ProposalAggregator {
    proposals: Arc<RwLock<Vec<BlockProposal>>>,
}

#[derive(Clone, Debug)]
pub struct BlockProposal {
    pub builder: Address,
    pub block_hash: H256,
    pub transactions: Vec<Transaction>,
    pub mev_protection_proof: MEVProtectionProof,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct MEVProtectionProof {
    pub fair_ordering_proof: Vec<u8>,
    pub no_sandwich_proof: Vec<u8>,
    pub no_frontrun_proof: Vec<u8>,
    pub encryption_proof: Vec<u8>,
}

impl BlockBuilderCoordinator {
    pub async fn new(config: BlockBuilderConfig) -> Result<Self> {
        Ok(Self {
            config,
            builders: Arc::new(RwLock::new(HashMap::new())),
            reputation_system: ReputationSystem::new(),
            proposal_aggregator: ProposalAggregator::new(),
        })
    }

    /// Register a new block builder
    pub async fn register_builder(&self, builder: BlockBuilder) -> Result<()> {
        if builder.stake < self.config.slashing_amount {
            return Err(anyhow!("Insufficient stake for builder registration"));
        }

        let mut builders = self.builders.write().await;
        builders.insert(builder.address, builder.clone());

        // Initialize reputation
        self.reputation_system.initialize_reputation(builder.address).await?;

        info!("✅ Registered new block builder: {:?}", builder.address);
        Ok(())
    }

    /// Select builders for the next block
    pub async fn select_builders(&self) -> Result<Vec<Address>> {
        let builders = self.builders.read().await;
        let mut eligible_builders = Vec::new();

        for (address, builder) in builders.iter() {
            if builder.is_active && builder.reputation_score >= self.config.reputation_threshold {
                eligible_builders.push((*address, builder.reputation_score));
            }
        }

        // Sort by reputation score
        eligible_builders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Select top builders
        let selected: Vec<Address> = eligible_builders
            .iter()
            .take(self.config.min_builders)
            .map(|(addr, _)| *addr)
            .collect();

        if selected.len() < self.config.min_builders {
            return Err(anyhow!(
                "Insufficient active builders: {} < {}",
                selected.len(),
                self.config.min_builders
            ));
        }

        Ok(selected)
    }

    /// Rotate active builders
    pub async fn rotate_builders(&self) -> Result<()> {
        let selected_builders = self.select_builders().await?;
        let mut builders = self.builders.write().await;

        // Deactivate all builders
        for builder in builders.values_mut() {
            builder.is_active = false;
        }

        // Activate selected builders
        for address in selected_builders {
            if let Some(builder) = builders.get_mut(&address) {
                builder.is_active = true;
                info!("🔄 Activated builder: {:?}", address);
            }
        }

        Ok(())
    }

    /// Collect block proposals from builders
    pub async fn collect_proposals(&self, timeout: Duration) -> Result<Vec<BlockProposal>> {
        let proposals = self.proposal_aggregator.collect_proposals(timeout).await?;
        
        // Verify all proposals
        let mut valid_proposals = Vec::new();
        for proposal in proposals {
            if self.verify_proposal(&proposal).await? {
                valid_proposals.push(proposal);
            }
        }

        Ok(valid_proposals)
    }

    /// Verify a block proposal
    async fn verify_proposal(&self, proposal: &BlockProposal) -> Result<bool> {
        // Verify builder is active
        let builders = self.builders.read().await;
        let builder = builders.get(&proposal.builder)
            .ok_or_else(|| anyhow!("Unknown builder"))?;
        
        if !builder.is_active {
            return Ok(false);
        }

        // Verify MEV protection proofs
        if !self.verify_mev_protection_proof(&proposal.mev_protection_proof).await? {
            return Ok(false);
        }

        // Verify signature
        // TODO: Implement signature verification

        Ok(true)
    }

    /// Verify MEV protection proof
    async fn verify_mev_protection_proof(&self, proof: &MEVProtectionProof) -> Result<bool> {
        // TODO: Implement actual proof verification
        // For now, just check that proofs exist
        Ok(!proof.fair_ordering_proof.is_empty() &&
           !proof.no_sandwich_proof.is_empty() &&
           !proof.no_frontrun_proof.is_empty() &&
           !proof.encryption_proof.is_empty())
    }

    /// Select the best proposal
    pub async fn select_best_proposal(&self, proposals: Vec<BlockProposal>) -> Result<BlockProposal> {
        if proposals.is_empty() {
            return Err(anyhow!("No proposals to select from"));
        }

        // TODO: Implement proposal scoring
        // For now, return the first valid proposal
        Ok(proposals.into_iter().next().unwrap())
    }

    /// Finalize block with selected proposal
    pub async fn finalize_block(&self, proposal: BlockProposal) -> Result<Block> {
        // Update builder statistics
        {
            let mut builders = self.builders.write().await;
            if let Some(builder) = builders.get_mut(&proposal.builder) {
                builder.blocks_built += 1;
                builder.blocks_accepted += 1;
                builder.last_active = chrono::Utc::now();
            }
        }

        // Update reputation
        self.reputation_system.update_reputation(
            proposal.builder,
            true,
            false,
        ).await?;

        // Create final block
        let block = Block {
            hash: proposal.block_hash,
            number: 0, // Will be set by consensus
            timestamp: chrono::Utc::now().timestamp() as u64,
            transactions: proposal.transactions,
            gas_used: U256::zero(),
            gas_limit: U256::from(30_000_000),
            base_fee: U256::from(1_000_000_000),
            difficulty: U256::zero(),
            total_difficulty: U256::zero(),
            miner: proposal.builder,
            parent_hash: H256::zero(), // Will be set by consensus
            receipts_root: H256::zero(),
            state_root: H256::zero(),
            transactions_root: H256::zero(),
        };

        info!("✅ Finalized block with {} transactions", block.transactions.len());

        Ok(block)
    }

    /// Slash a malicious builder
    pub async fn slash_builder(&self, builder: Address, reason: &str) -> Result<()> {
        let mut builders = self.builders.write().await;
        
        if let Some(builder_data) = builders.get_mut(&builder) {
            // Deduct stake
            builder_data.stake = builder_data.stake.saturating_sub(self.config.slashing_amount);
            builder_data.is_active = false;
            
            // Update reputation
            self.reputation_system.slash_reputation(builder).await?;
            
            warn!("⚠️ Slashed builder {:?} for: {}", builder, reason);
            
            // Remove builder if stake is too low
            if builder_data.stake == U256::zero() {
                builders.remove(&builder);
                info!("🚫 Removed builder {:?} due to insufficient stake", builder);
            }
        }

        Ok(())
    }

    /// Start the coordinator service
    pub async fn start_coordinator(&self) -> Result<()> {
        let mut interval = interval(self.config.rotation_interval);

        loop {
            interval.tick().await;

            // Rotate builders
            if let Err(e) = self.rotate_builders().await {
                error!("Failed to rotate builders: {}", e);
            }

            // Clean up inactive builders
            if let Err(e) = self.cleanup_inactive_builders().await {
                error!("Failed to cleanup inactive builders: {}", e);
            }
        }
    }

    /// Clean up inactive builders
    async fn cleanup_inactive_builders(&self) -> Result<()> {
        let mut builders = self.builders.write().await;
        let now = chrono::Utc::now();
        let inactive_threshold = chrono::Duration::days(7);

        builders.retain(|address, builder| {
            let inactive_duration = now.signed_duration_since(builder.last_active);
            if inactive_duration > inactive_threshold {
                warn!("Removing inactive builder: {:?}", address);
                false
            } else {
                true
            }
        });

        Ok(())
    }
}

impl ReputationSystem {
    pub fn new() -> Self {
        Self {
            scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn initialize_reputation(&self, builder: Address) -> Result<()> {
        let mut scores = self.scores.write().await;
        scores.insert(builder, ReputationScore {
            builder,
            score: 50.0, // Start with neutral score
            total_blocks: 0,
            successful_blocks: 0,
            failed_blocks: 0,
            mev_incidents: 0,
            last_updated: chrono::Utc::now(),
        });
        Ok(())
    }

    pub async fn update_reputation(
        &self,
        builder: Address,
        success: bool,
        mev_detected: bool,
    ) -> Result<()> {
        let mut scores = self.scores.write().await;
        
        if let Some(score) = scores.get_mut(&builder) {
            score.total_blocks += 1;
            
            if success {
                score.successful_blocks += 1;
                score.score = (score.score * 0.95 + 100.0 * 0.05).min(100.0);
            } else {
                score.failed_blocks += 1;
                score.score = (score.score * 0.95 + 0.0 * 0.05).max(0.0);
            }
            
            if mev_detected {
                score.mev_incidents += 1;
                score.score = (score.score - 10.0).max(0.0);
            }
            
            score.last_updated = chrono::Utc::now();
        }

        Ok(())
    }

    pub async fn slash_reputation(&self, builder: Address) -> Result<()> {
        let mut scores = self.scores.write().await;
        
        if let Some(score) = scores.get_mut(&builder) {
            score.score = (score.score - 25.0).max(0.0);
            score.last_updated = chrono::Utc::now();
        }

        Ok(())
    }

    pub async fn get_reputation(&self, builder: Address) -> Result<f64> {
        let scores = self.scores.read().await;
        Ok(scores.get(&builder).map(|s| s.score).unwrap_or(0.0))
    }
}

impl ProposalAggregator {
    pub fn new() -> Self {
        Self {
            proposals: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_proposal(&self, proposal: BlockProposal) -> Result<()> {
        let mut proposals = self.proposals.write().await;
        proposals.push(proposal);
        Ok(())
    }

    pub async fn collect_proposals(&self, timeout: Duration) -> Result<Vec<BlockProposal>> {
        tokio::time::sleep(timeout).await;
        
        let mut proposals = self.proposals.write().await;
        let collected = proposals.clone();
        proposals.clear();
        
        Ok(collected)
    }
}

impl Default for BlockBuilderConfig {
    fn default() -> Self {
        Self {
            min_builders: 5,
            rotation_interval: Duration::from_secs(300), // 5 minutes
            proposal_timeout: Duration::from_secs(10),
            reputation_threshold: 30.0,
            slashing_amount: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
        }
    }
}
