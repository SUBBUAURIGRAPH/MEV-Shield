//! Error types for MEV Shield

use thiserror::Error;
use crate::detection::{DetectionResult, BatchAnalysisResult};

/// Main error type for MEV Shield operations
#[derive(Error, Debug)]
pub enum MEVShieldError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Invalid transaction errors
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    /// MEV detected in transaction
    #[error("MEV detected in transaction")]
    MEVDetected(DetectionResult),
    
    /// MEV detected in transaction batch
    #[error("MEV detected in transaction batch")]
    BatchMEVDetected(BatchAnalysisResult),
    
    /// Encryption/decryption errors
    #[error("Encryption error: {0}")]
    Encryption(#[from] EncryptionError),
    
    /// Ordering service errors
    #[error("Ordering error: {0}")]
    Ordering(#[from] OrderingError),
    
    /// Detection service errors
    #[error("Detection error: {0}")]
    Detection(#[from] DetectionError),
    
    /// Redistribution service errors
    #[error("Redistribution error: {0}")]
    Redistribution(#[from] RedistributionError),
    
    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    /// Blockchain adapter errors
    #[error("Blockchain error: {0}")]
    Blockchain(#[from] BlockchainError),
    
    /// Network/communication errors
    #[error("Network error: {0}")]
    Network(String),
    
    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// Resource exhaustion
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    /// Internal service errors
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// External service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Encryption-specific errors
#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Threshold not met: need {required}, got {actual}")]
    InsufficientShares { required: u32, actual: u32 },
    
    #[error("Invalid decryption share from validator {validator_id}")]
    InvalidShare { validator_id: u32 },
    
    #[error("Transaction not found: {tx_id}")]
    TransactionNotFound { tx_id: String },
    
    #[error("Transaction not ready for decryption")]
    NotReadyForDecryption,
    
    #[error("Encryption timeout")]
    EncryptionTimeout,
    
    #[error("Key generation failed: {reason}")]
    KeyGenerationFailed { reason: String },
    
    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),
}

/// Ordering service errors
#[derive(Error, Debug)]
pub enum OrderingError {
    #[error("VDF computation failed: {0}")]
    VDFComputationFailed(String),
    
    #[error("Invalid ordering proof")]
    InvalidProof,
    
    #[error("Ordering commitment creation failed: {0}")]
    CommitmentFailed(String),
    
    #[error("Batch processing failed: {0}")]
    BatchProcessingFailed(String),
    
    #[error("Invalid VDF parameters: {0}")]
    InvalidParameters(String),
}

/// Detection service errors
#[derive(Error, Debug)]
pub enum DetectionError {
    #[error("Pattern detection failed: {0}")]
    PatternDetectionFailed(String),
    
    #[error("Unsupported operation")]
    UnsupportedOperation,
    
    #[error("Analysis timeout")]
    AnalysisTimeout,
    
    #[error("Insufficient data for analysis")]
    InsufficientData,
    
    #[error("DEX operation decoding failed: {0}")]
    DecodingFailed(String),
    
    #[error("History update failed: {0}")]
    HistoryUpdateFailed(String),
}

/// Redistribution service errors
#[derive(Error, Debug)]
pub enum RedistributionError {
    #[error("MEV calculation failed: {0}")]
    MEVCalculationFailed(String),
    
    #[error("Distribution calculation failed: {0}")]
    DistributionCalculationFailed(String),
    
    #[error("Payment processing failed: {0}")]
    PaymentProcessingFailed(String),
    
    #[error("Insufficient funds for distribution")]
    InsufficientFunds,
    
    #[error("User contribution update failed: {0}")]
    ContributionUpdateFailed(String),
    
    #[error("Distribution not due")]
    DistributionNotDue,
}

/// Storage errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    
    #[error("Transaction rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("Cache operation failed: {0}")]
    CacheFailed(String),
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

/// Blockchain adapter errors
#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("RPC call failed: {0}")]
    RpcFailed(String),
    
    #[error("Transaction submission failed: {0}")]
    SubmissionFailed(String),
    
    #[error("Block not found: {block_number}")]
    BlockNotFound { block_number: u64 },
    
    #[error("Transaction not found: {tx_hash}")]
    TransactionNotFound { tx_hash: String },
    
    #[error("Network not supported: {chain_id}")]
    UnsupportedNetwork { chain_id: u64 },
    
    #[error("Gas estimation failed: {0}")]
    GasEstimationFailed(String),
    
    #[error("Nonce calculation failed: {0}")]
    NonceFailed(String),
    
    #[error("Connection timeout")]
    ConnectionTimeout,
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid address format: {address}")]
    InvalidAddress { address: String },
    
    #[error("Invalid transaction hash: {hash}")]
    InvalidTransactionHash { hash: String },
    
    #[error("Gas limit too high: {gas_limit}")]
    GasLimitTooHigh { gas_limit: u64 },
    
    #[error("Value too large: {value}")]
    ValueTooLarge { value: String },
    
    #[error("Invalid nonce: {nonce}")]
    InvalidNonce { nonce: u64 },
    
    #[error("Malicious recipient detected: {address}")]
    MaliciousRecipient { address: String },
    
    #[error("Suspicious transaction pattern detected")]
    SuspiciousPattern,
    
    #[error("Unsupported chain ID: {chain_id}")]
    UnsupportedChainId { chain_id: u64 },
}

/// Rate limiting errors
#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for client {client_id}")]
    ExceededLimit { client_id: String },
    
    #[error("Invalid rate limit configuration")]
    InvalidConfiguration,
    
    #[error("Rate limit storage error: {0}")]
    StorageError(String),
}

/// Authentication/authorization errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid API key")]
    InvalidApiKey,
    
    #[error("API key expired")]
    ApiKeyExpired,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Token validation failed: {0}")]
    TokenValidationFailed(String),
}

/// Batch processing errors
#[derive(Error, Debug)]
pub enum BatchError {
    #[error("Batch size exceeded: {size}")]
    SizeExceeded { size: usize },
    
    #[error("Batch processing timeout")]
    ProcessingTimeout,
    
    #[error("Partial batch failure: {succeeded}/{total}")]
    PartialFailure { succeeded: usize, total: usize },
    
    #[error("Batch validation failed: {0}")]
    ValidationFailed(String),
}

/// Monitoring and metrics errors
#[derive(Error, Debug)]
pub enum MonitoringError {
    #[error("Metrics collection failed: {0}")]
    MetricsCollectionFailed(String),
    
    #[error("Alert system failure: {0}")]
    AlertSystemFailure(String),
    
    #[error("Threshold detection failure: {0}")]
    ThresholdDetectionFailure(String),
    
    #[error("Metrics export failed: {0}")]
    MetricsExportFailed(String),
}

// Implement From conversions for common error types
impl From<sqlx::Error> for MEVShieldError {
    fn from(err: sqlx::Error) -> Self {
        MEVShieldError::Storage(StorageError::QueryFailed(err.to_string()))
    }
}

impl From<redis::RedisError> for MEVShieldError {
    fn from(err: redis::RedisError) -> Self {
        MEVShieldError::Storage(StorageError::CacheFailed(err.to_string()))
    }
}

impl From<serde_json::Error> for MEVShieldError {
    fn from(err: serde_json::Error) -> Self {
        MEVShieldError::Storage(StorageError::SerializationFailed(err.to_string()))
    }
}

impl From<tokio::time::error::Elapsed> for MEVShieldError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        MEVShieldError::Timeout(err.to_string())
    }
}

impl From<hex::FromHexError> for ValidationError {
    fn from(err: hex::FromHexError) -> Self {
        ValidationError::InvalidAddress {
            address: format!("Hex decode error: {}", err),
        }
    }
}

/// Result type alias for MEV Shield operations
pub type MEVResult<T> = Result<T, MEVShieldError>;

/// Result type for specific service operations
pub type EncryptionResult<T> = Result<T, EncryptionError>;
pub type OrderingResult<T> = Result<T, OrderingError>;
pub type DetectionResultType<T> = Result<T, DetectionError>;
pub type RedistributionResult<T> = Result<T, RedistributionError>;
pub type StorageResult<T> = Result<T, StorageError>;
pub type BlockchainResult<T> = Result<T, BlockchainError>;
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Error context helpers
impl MEVShieldError {
    pub fn with_context<C: std::fmt::Display>(self, context: C) -> Self {
        match self {
            MEVShieldError::Internal(msg) => {
                MEVShieldError::Internal(format!("{}: {}", context, msg))
            }
            other => other,
        }
    }
    
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            MEVShieldError::Network(_)
                | MEVShieldError::Timeout(_)
                | MEVShieldError::ServiceUnavailable(_)
                | MEVShieldError::Storage(StorageError::ConnectionFailed(_))
                | MEVShieldError::Blockchain(BlockchainError::ConnectionTimeout)
        )
    }
    
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MEVShieldError::MEVDetected(_) | MEVShieldError::BatchMEVDetected(_) => {
                ErrorSeverity::Critical
            }
            MEVShieldError::InvalidTransaction(_) | MEVShieldError::Validation(_) => {
                ErrorSeverity::Warning
            }
            MEVShieldError::Timeout(_) | MEVShieldError::Network(_) => ErrorSeverity::Warning,
            MEVShieldError::Internal(_) | MEVShieldError::Configuration(_) => {
                ErrorSeverity::Error
            }
            _ => ErrorSeverity::Info,
        }
    }
}

/// Error severity levels for logging and alerting
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}