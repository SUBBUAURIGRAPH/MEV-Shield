// core/src/traits.rs
//! Service traits for MEV Shield components

use async_trait::async_trait;
use crate::types::*;
use crate::error::MEVShieldError;

/// Trait for transaction encryption services
#[async_trait]
pub trait EncryptionService: Send + Sync {
    /// Encrypt a transaction using threshold encryption
    async fn encrypt_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<EncryptedTransaction, MEVShieldError>;
    
    /// Decrypt a transaction using collected threshold shares
    async fn decrypt_transaction(
        &self,
        encrypted_tx: EncryptedTransaction,
    ) -> Result<Transaction, MEVShieldError>;
    
    /// Get transactions ready for decryption at given block height
    async fn get_ready_transactions(
        &self,
        block_height: u64,
    ) -> Result<Vec<EncryptedTransaction>, MEVShieldError>;
    
    /// Collect decryption shares from validators
    async fn collect_decryption_shares(
        &self,
        tx_id: TxHash,
        block_height: u64,
    ) -> Result<Vec<DecryptionShare>, MEVShieldError>;
}

/// Trait for fair transaction ordering services
#[async_trait]
pub trait OrderingService: Send + Sync {
    /// Create an ordering commitment for a transaction
    async fn create_ordering_commitment(
        &self,
        transaction: &EncryptedTransaction,
    ) -> Result<OrderingCommitment, MEVShieldError>;
    
    /// Order transactions using verifiable delay functions
    async fn order_transactions(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<Vec<Transaction>, MEVShieldError>;
    
    /// Verify ordering proof
    async fn verify_ordering_proof(
        &self,
        commitment: &OrderingCommitment,
        proof: &OrderingProof,
    ) -> Result<bool, MEVShieldError>;
}

/// Trait for MEV detection services
#[async_trait]
pub trait DetectionService: Send + Sync {
    /// Analyze a single transaction for MEV patterns
    async fn analyze_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<DetectionResult, MEVShieldError>;
    
    /// Analyze a batch of transactions for MEV patterns
    async fn analyze_transaction_batch(
        &self,
        transactions: &[Transaction],
    ) -> Result<DetectionResult, MEVShieldError>;
    
    /// Validate that a block is MEV-free
    async fn validate_block_mev_free(
        &self,
        block: &Block,
    ) -> Result<bool, MEVShieldError>;
    
    /// Get detection configuration
    fn get_detection_config(&self) -> DetectionConfig;
}

/// Trait for MEV redistribution services
#[async_trait]
pub trait RedistributionService: Send + Sync {
    /// Capture MEV from a block
    async fn capture_mev_from_block(
        &self,
        block: &Block,
        mev_data: &MEVData,
    ) -> Result<U256, MEVShieldError>;
    
    /// Distribute captured MEV to users
    async fn distribute_mev(&self) -> Result<DistributionResult, MEVShieldError>;
    
    /// Get pending rewards for a user
    async fn get_user_pending_rewards(
        &self,
        user_address: &Address,
    ) -> Result<U256, MEVShieldError>;
    
    /// Update user contributions
    async fn update_user_contributions(
        &self,
        block: &Block,
    ) -> Result<(), MEVShieldError>;
}

/// Trait for blockchain adapters
#[async_trait]
pub trait BlockchainAdapter: Send + Sync {
    /// Get the current block number
    async fn get_block_number(&self) -> Result<u64, MEVShieldError>;
    
    /// Get block by number
    async fn get_block(&self, block_number: u64) -> Result<Block, MEVShieldError>;
    
    /// Submit transaction to blockchain
    async fn submit_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TxHash, MEVShieldError>;
    
    /// Get transaction by hash
    async fn get_transaction(
        &self,
        tx_hash: TxHash,
    ) -> Result<Option<Transaction>, MEVShieldError>;
    
    /// Get chain ID
    fn chain_id(&self) -> ChainId;
    
    /// Subscribe to new blocks
    async fn subscribe_blocks(&self) -> Result<BlockStream, MEVShieldError>;
}

/// Trait for storage backends
#[async_trait]
pub trait StorageService: Send + Sync {
    /// Store encrypted transaction
    async fn store_encrypted_transaction(
        &self,
        tx: &EncryptedTransaction,
    ) -> Result<(), MEVShieldError>;
    
    /// Retrieve encrypted transaction
    async fn get_encrypted_transaction(
        &self,
        tx_id: &TxHash,
    ) -> Result<Option<EncryptedTransaction>, MEVShieldError>;
    
    /// Store transaction status
    async fn store_transaction_status(
        &self,
        status: &TransactionStatus,
    ) -> Result<(), MEVShieldError>;
    
    /// Get transaction status
    async fn get_transaction_status(
        &self,
        tx_id: &uuid::Uuid,
    ) -> Result<Option<TransactionStatus>, MEVShieldError>;
    
    /// Store MEV detection result
    async fn store_detection_result(
        &self,
        result: &DetectionResult,
    ) -> Result<(), MEVShieldError>;
    
    /// Store user contribution
    async fn store_user_contribution(
        &self,
        contribution: &UserContribution,
    ) -> Result<(), MEVShieldError>;
    
    /// Get user contribution
    async fn get_user_contribution(
        &self,
        address: &Address,
    ) -> Result<Option<UserContribution>, MEVShieldError>;
}

/// Supporting types for traits

/// Decryption share from threshold encryption
#[derive(Debug, Clone)]
pub struct DecryptionShare {
    pub validator_index: u32,
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Ordering proof for VDF verification
#[derive(Debug, Clone)]
pub struct OrderingProof {
    pub proof_hash: Hash,
    pub intermediate_values: Vec<num_bigint::BigUint>,
}

/// Detection configuration
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    pub sandwich_detection_enabled: bool,
    pub frontrun_detection_enabled: bool,
    pub arbitrage_detection_enabled: bool,
    pub window_size: std::time::Duration,
    pub max_history_size: usize,
    pub confidence_threshold: f64,
}

/// Block stream for real-time block updates
pub type BlockStream = tokio::sync::mpsc::Receiver<Block>;

/// Trait for pattern detectors
#[async_trait]
pub trait MEVPatternDetector: Send + Sync {
    /// Detect MEV patterns in transactions
    async fn detect_pattern(
        &self,
        transactions: &[Transaction],
    ) -> Result<Vec<MEVAlert>, MEVShieldError>;
    
    /// Get the pattern type this detector handles
    fn pattern_type(&self) -> MEVPatternType;
    
    /// Get confidence threshold for this detector
    fn confidence_threshold(&self) -> f64;
}

/// Trait for validators in threshold encryption
#[async_trait]
pub trait ValidatorService: Send + Sync {
    /// Request decryption share from validator
    async fn request_decryption_share(
        &self,
        validator_id: u32,
        encrypted_tx: &EncryptedTransaction,
    ) -> Result<DecryptionShare, MEVShieldError>;
    
    /// Verify validator signature
    async fn verify_validator_signature(
        &self,
        share: &DecryptionShare,
        public_key: &[u8],
    ) -> Result<bool, MEVShieldError>;
    
    /// Get validator public keys
    async fn get_validator_keys(&self) -> Result<Vec<Vec<u8>>, MEVShieldError>;
}

/// Trait for metrics collection
#[async_trait]
pub trait MetricsService: Send + Sync {
    /// Record transaction encryption time
    async fn record_encryption_time(&self, duration: std::time::Duration);
    
    /// Record MEV detection
    async fn record_mev_detection(&self, pattern_type: MEVPatternType);
    
    /// Record transaction throughput
    async fn record_transaction_throughput(&self, count: u64);
    
    /// Record MEV savings
    async fn record_mev_savings(&self, amount: &U256);
    
    /// Get metrics summary
    async fn get_metrics_summary(&self) -> Result<MetricsSummary, MEVShieldError>;
}

/// Metrics summary
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub total_transactions_protected: u64,
    pub total_mev_detected: u64,
    pub total_mev_savings: U256,
    pub average_protection_time: std::time::Duration,
    pub system_uptime: std::time::Duration,
}