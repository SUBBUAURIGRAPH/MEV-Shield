// core/src/types.rs
//! Core data types for MEV Shield

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// 256-bit unsigned integer type for large numbers
pub type U256 = num_bigint::BigUint;

/// Ethereum-style address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub fn zero() -> Self {
        Self([0u8; 20])
    }
    
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
    
    pub fn from_str(s: &str) -> Result<Self, hex::FromHexError> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let bytes = hex::decode(s)?;
        if bytes.len() != 20 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&bytes);
        Ok(Self(addr))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

/// Transaction hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxHash(pub [u8; 32]);

impl TxHash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

/// Generic hash type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn from(bytes: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes[..32.min(bytes.len())]);
        Self(hash)
    }
}

/// Chain identifier
pub type ChainId = u64;

/// Timestamp type
pub type Timestamp = DateTime<Utc>;

/// Transaction type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas: u64,
    pub gas_price: U256,
    pub gas_used: u64,
    pub nonce: u64,
    pub data: Vec<u8>,
    pub chain_id: ChainId,
    pub submission_time: Timestamp,
}

impl Transaction {
    /// Calculate transaction hash
    pub fn hash(&self) -> TxHash {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        
        // Simplified hash calculation - in production would use proper RLP encoding
        hasher.update(&self.from.0);
        hasher.update(&self.to.0);
        hasher.update(self.nonce.to_be_bytes());
        hasher.update(&self.data);
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        TxHash(hash)
    }
    
    /// Serialize transaction for encryption
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(serde_json::to_vec(self)?)
    }
    
    /// Get transaction type
    pub fn transaction_type(&self) -> TransactionType {
        if self.data.is_empty() {
            TransactionType::Transfer
        } else if self.data.len() >= 4 {
            // Check function selector for common patterns
            match &self.data[0..4] {
                [0xa9, 0x05, 0x9c, 0xbb] => TransactionType::DEXTrade, // swapExactTokensForTokens
                [0x87, 0x86, 0x44, 0x56] => TransactionType::DEXTrade, // swapTokensForExactTokens
                _ => TransactionType::ContractCall,
            }
        } else {
            TransactionType::ContractCall
        }
    }
}

/// Transaction types for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    DEXTrade,
    ContractCall,
    Deploy,
}

/// Encrypted transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTransaction {
    pub id: TxHash,
    pub encrypted_data: Vec<u8>, // Simplified - would use proper threshold encryption
    pub submission_time: Timestamp,
    pub time_lock: Option<TimeLock>,
    pub priority: Priority,
    pub gas_price: U256,
    pub chain_id: ChainId,
}

impl EncryptedTransaction {
    pub fn hash(&self) -> TxHash {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(&self.encrypted_data);
        hasher.update(self.submission_time.timestamp().to_be_bytes());
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        TxHash(hash)
    }
}

/// Time lock for delayed execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLock {
    pub unlock_time: Timestamp,
    pub created_at: Timestamp,
}

/// Transaction priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
}

/// Protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    pub level: ProtectionLevel,
    pub private_pool: bool,
    pub time_lock_duration: Option<std::time::Duration>,
    pub max_slippage: Option<f64>,
}

/// Protection levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtectionLevel {
    Basic,
    Standard,
    Maximum,
    Enterprise,
}

/// Ordering commitment for fair sequencing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderingCommitment {
    pub transaction_hash: TxHash,
    pub submission_time: Timestamp,
    pub priority_data: Vec<u8>,
    pub commitment_hash: Hash,
    pub priority: u32,
}

/// Result of submitting a protected transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedTransactionResult {
    pub transaction_id: Uuid,
    pub original_hash: TxHash,
    pub encrypted_hash: TxHash,
    pub protection_applied: bool,
    pub execution_schedule: ExecutionSchedule,
    pub mev_detected: bool,
    pub estimated_savings: U256,
}

/// Execution schedule information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSchedule {
    pub estimated_execution_time: Timestamp,
    pub time_lock_duration: std::time::Duration,
    pub ordering_priority: u32,
}

/// Transaction execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub id: Uuid,
    pub status: ExecutionStatus,
    pub block_number: Option<u64>,
    pub block_hash: Option<Hash>,
    pub transaction_hash: Option<TxHash>,
    pub protection_details: ProtectionDetails,
}

/// Execution status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Encrypted,
    Ordered,
    Executed,
    Failed,
}

/// Protection details and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionDetails {
    pub mev_detected: bool,
    pub savings_amount: U256,
    pub execution_time: Option<Timestamp>,
    pub protection_methods: Vec<ProtectionMethod>,
}

/// Types of protection applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtectionMethod {
    Encryption,
    FairOrdering,
    TimeDelay,
    PrivatePool,
    MEVDetection,
}

/// MEV detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub is_mev_free: bool,
    pub alerts: Vec<MEVAlert>,
    pub analysis_time: Timestamp,
    pub transaction_count: usize,
}

impl DetectionResult {
    pub fn is_mev_detected(&self) -> bool {
        !self.is_mev_free
    }
}

/// MEV alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MEVAlert {
    pub pattern_type: MEVPatternType,
    pub confidence: f64,
    pub affected_transactions: Vec<TxHash>,
    pub evidence: MEVEvidence,
    pub timestamp: Timestamp,
    pub severity: AlertSeverity,
}

/// Types of MEV patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MEVPatternType {
    SandwichAttack,
    FrontRunning,
    BackRunning,
    Arbitrage,
    JustInTimeProvision,
    MEVExtraction,
}

/// Evidence of MEV activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MEVEvidence {
    Sandwich {
        front_run_tx: TxHash,
        victim_tx: TxHash,
        back_run_tx: TxHash,
        profit_amount: U256,
        token_pair: (Address, Address),
    },
    FrontRun {
        attacker_tx: TxHash,
        victim_tx: TxHash,
        copied_data: Vec<u8>,
        time_advantage: std::time::Duration,
    },
    Arbitrage {
        arbitrage_txs: Vec<TxHash>,
        profit_amount: U256,
        exchanges: Vec<Address>,
    },
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub number: u64,
    pub hash: Hash,
    pub parent_hash: Hash,
    pub timestamp: Timestamp,
    pub base_fee: U256,
    pub transactions: Vec<Transaction>,
}

/// MEV data for redistribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MEVData {
    pub extracted_value: U256,
    pub builder_payment: U256,
    pub mev_type: MEVType,
    pub affected_transactions: Vec<TxHash>,
}

/// Types of MEV for classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MEVType {
    Arbitrage,
    Sandwich,
    FrontRun,
    Liquidation,
    Other,
}

/// User contribution tracking for redistribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContribution {
    pub address: Address,
    pub total_gas_used: U256,
    pub transaction_count: u64,
    pub value_contributed: U256,
    pub last_activity: Timestamp,
    pub accumulated_rewards: U256,
}

/// Distribution of MEV rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Distribution {
    pub recipient: Address,
    pub amount: U256,
    pub reason: DistributionReason,
    pub epoch: u64,
    pub timestamp: Timestamp,
}

/// Reasons for MEV distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionReason {
    GasContribution,
    ValueContribution,
    ValidatorReward,
    BuilderReward,
}

/// Result of MEV distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionResult {
    pub distributed: bool,
    pub total_amount: U256,
    pub recipient_count: usize,
    pub distributions: Vec<Distribution>,
}