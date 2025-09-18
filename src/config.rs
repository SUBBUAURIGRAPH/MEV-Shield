// core/src/config.rs
//! Configuration management for MEV Shield

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use anyhow::Result;

// Duration serialization helper module
mod serde_duration {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub encryption: EncryptionConfig,
    pub ordering: OrderingConfig,
    pub detection: DetectionConfig,
    pub redistribution: RedistributionConfig,
    pub block_builder: BlockBuilderConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub blockchain: BlockchainConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            encryption: EncryptionConfig::default(),
            ordering: OrderingConfig::default(),
            detection: DetectionConfig::default(),
            redistribution: RedistributionConfig::default(),
            block_builder: BlockBuilderConfig::default(),
            database: DatabaseConfig::default(),
            cache: CacheConfig::default(),
            blockchain: BlockchainConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

/// Block builder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockBuilderConfig {
    pub max_block_size: u32,
    pub target_block_time: Duration,
    pub min_transactions: u32,
}

impl Default for BlockBuilderConfig {
    fn default() -> Self {
        Self {
            max_block_size: 30_000_000,
            target_block_time: Duration::from_secs(12),
            min_transactions: 10,
        }
    }
}

/// Main configuration for MEV Shield
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MEVShieldConfig {
    /// Service configuration
    pub services: ServicesConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Cache configuration
    pub cache: CacheConfig,
    
    /// API configuration
    pub api: ApiConfig,
    
    /// Blockchain configuration
    pub blockchain: BlockchainConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    
    /// Performance tuning
    pub performance: PerformanceConfig,
}

impl Default for MEVShieldConfig {
    fn default() -> Self {
        Self {
            services: ServicesConfig::default(),
            database: DatabaseConfig::default(),
            cache: CacheConfig::default(),
            api: ApiConfig::default(),
            blockchain: BlockchainConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Configuration for all MEV Shield services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    /// Encryption service configuration
    pub encryption: EncryptionConfig,
    
    /// Ordering service configuration
    pub ordering: OrderingConfig,
    
    /// Detection service configuration
    pub detection: DetectionConfig,
    
    /// Redistribution service configuration
    pub redistribution: RedistributionConfig,
}

impl Default for ServicesConfig {
    fn default() -> Self {
        Self {
            encryption: EncryptionConfig::default(),
            ordering: OrderingConfig::default(),
            detection: DetectionConfig::default(),
            redistribution: RedistributionConfig::default(),
        }
    }
}

/// Encryption service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Threshold for decryption (e.g., 67% of validators)
    pub threshold: u32,
    
    /// Total number of validators
    pub total_validators: u32,
    
    /// Maximum size of encrypted mempool
    pub max_pool_size: u32,
    
    /// Cleanup interval for expired transactions
    #[serde(with = "serde_duration")]
    pub cleanup_interval: Duration,
    
    /// Timeout for encryption operations
    #[serde(with = "serde_duration")]
    pub encryption_timeout: Duration,
    
    /// Minimum age before transaction can be decrypted
    #[serde(with = "serde_duration")]
    pub minimum_age: Duration,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            threshold: 67, // 67% of validators
            total_validators: 100,
            max_pool_size: 10000,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            encryption_timeout: Duration::from_secs(30),
            minimum_age: Duration::from_secs(5),
        }
    }
}

/// Ordering service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderingConfig {
    /// VDF difficulty parameter
    pub vdf_difficulty: u64,
    
    /// VDF security parameter
    pub vdf_security_param: u32,
    
    /// Batch size for VDF computation
    pub batch_size: u32,
    
    /// Timeout for VDF computation
    #[serde(with = "serde_duration")]
    pub computation_timeout: Duration,
    
    /// Enable VDF proof verification
    pub verify_proofs: bool,
}

impl Default for OrderingConfig {
    fn default() -> Self {
        Self {
            vdf_difficulty: 100000, // Tuned for ~10 seconds
            vdf_security_param: 256,
            batch_size: 100,
            computation_timeout: Duration::from_secs(60),
            verify_proofs: true,
        }
    }
}

/// Detection service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    /// Enable sandwich attack detection
    pub sandwich_detection_enabled: bool,
    
    /// Enable front-running detection
    pub frontrun_detection_enabled: bool,
    
    /// Enable arbitrage detection
    pub arbitrage_detection_enabled: bool,
    
    /// Analysis window size
    #[serde(with = "serde_duration")]
    pub window_size: Duration,
    
    /// Maximum history size for analysis
    pub max_history_size: usize,
    
    /// Confidence threshold for alerts
    pub confidence_threshold: f64,
    
    /// Sandwich detection configuration
    pub sandwich: SandwichDetectionConfig,
    
    /// Front-running detection configuration
    pub frontrun: FrontrunDetectionConfig,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            sandwich_detection_enabled: true,
            frontrun_detection_enabled: true,
            arbitrage_detection_enabled: true,
            window_size: Duration::from_secs(60),
            max_history_size: 10000,
            confidence_threshold: 0.8,
            sandwich: SandwichDetectionConfig::default(),
            frontrun: FrontrunDetectionConfig::default(),
        }
    }
}

/// Sandwich attack detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandwichDetectionConfig {
    /// Maximum distance between sandwich components
    pub max_distance: usize,
    
    /// Minimum profit threshold to consider
    pub min_profit_threshold_wei: String, // As string to handle large numbers
    
    /// Require same token pair for sandwich
    pub same_token_required: bool,
    
    /// Enable DEX operation decoding
    pub decode_dex_operations: bool,
}

impl Default for SandwichDetectionConfig {
    fn default() -> Self {
        Self {
            max_distance: 5,
            min_profit_threshold_wei: "1000000000000000".to_string(), // 0.001 ETH
            same_token_required: true,
            decode_dex_operations: true,
        }
    }
}

/// Front-running detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontrunDetectionConfig {
    /// Maximum time difference to consider front-running
    #[serde(with = "serde_duration")]
    pub max_time_difference: Duration,
    
    /// Minimum gas price differential
    pub min_gas_price_differential: f64,
    
    /// Enable mempool monitoring
    pub mempool_monitoring: bool,
}

impl Default for FrontrunDetectionConfig {
    fn default() -> Self {
        Self {
            max_time_difference: Duration::from_secs(30),
            min_gas_price_differential: 1.1, // 10% higher gas price
            mempool_monitoring: true,
        }
    }
}

/// Redistribution service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedistributionConfig {
    /// Percentage of MEV to redistribute to users
    pub redistribution_percentage: f64,
    
    /// How often to distribute rewards
    #[serde(with = "serde_duration")]
    pub distribution_frequency: Duration,
    
    /// Minimum amount required for distribution
    pub minimum_distribution_wei: String,
    
    /// Percentage to reserve for gas costs
    pub gas_reserve_percentage: f64,
    
    /// Percentage share for validators
    pub validator_share: f64,
    
    /// Enable automatic distribution
    pub auto_distribution: bool,
}

impl Default for RedistributionConfig {
    fn default() -> Self {
        Self {
            redistribution_percentage: 80.0, // 80% to users
            distribution_frequency: Duration::from_secs(3600), // 1 hour
            minimum_distribution_wei: "100000000000000000".to_string(), // 0.1 ETH
            gas_reserve_percentage: 5.0, // 5% for gas
            validator_share: 15.0, // 15% to validators
            auto_distribution: true,
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    
    /// Maximum number of connections
    pub max_connections: u32,
    
    /// Minimum number of connections
    pub min_connections: u32,
    
    /// Connection timeout
    #[serde(with = "serde_duration")]
    pub connect_timeout: Duration,
    
    /// Query timeout
    #[serde(with = "serde_duration")]
    pub query_timeout: Duration,
    
    /// Enable SSL
    pub ssl_mode: bool,
    
    /// Auto-migration
    pub auto_migrate: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/mevshield".to_string(),
            max_connections: 100,
            min_connections: 5,
            connect_timeout: Duration::from_secs(30),
            query_timeout: Duration::from_secs(60),
            ssl_mode: true,
            auto_migrate: true,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Redis URL
    pub redis_url: String,
    
    /// Default TTL for cached items
    #[serde(with = "serde_duration")]
    pub default_ttl: Duration,
    
    /// Maximum local cache size
    pub max_local_cache_size: usize,
    
    /// Local cache TTL
    #[serde(with = "serde_duration")]
    pub local_cache_ttl: Duration,
    
    /// Enable compression
    pub compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_local_cache_size: 1000,
            local_cache_ttl: Duration::from_secs(300), // 5 minutes
            compression: true,
        }
    }
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Server bind address
    pub bind_address: String,
    
    /// Server port
    pub port: u16,
    
    /// Enable CORS
    pub cors_enabled: bool,
    
    /// Allowed origins for CORS
    pub cors_origins: Vec<String>,
    
    /// Request timeout
    #[serde(with = "serde_duration")]
    pub request_timeout: Duration,
    
    /// Enable request logging
    pub request_logging: bool,
    
    /// Rate limiting
    pub rate_limiting: RateLimitConfig,
    
    /// Authentication
    pub auth: AuthConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
            request_timeout: Duration::from_secs(30),
            request_logging: true,
            rate_limiting: RateLimitConfig::default(),
            auth: AuthConfig::default(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    
    /// Requests per window
    pub requests_per_window: u32,
    
    /// Window duration
    #[serde(with = "serde_duration")]
    pub window_duration: Duration,
    
    /// Burst limit
    pub burst_limit: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_window: 1000,
            window_duration: Duration::from_secs(3600), // 1 hour
            burst_limit: 100,
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable API key authentication
    pub api_key_auth: bool,
    
    /// JWT secret for token validation
    pub jwt_secret: Option<String>,
    
    /// Token expiration time
    #[serde(with = "serde_duration")]
    pub token_expiry: Duration,
    
    /// Enable admin endpoints
    pub admin_endpoints: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_key_auth: true,
            jwt_secret: None,
            token_expiry: Duration::from_secs(86400), // 24 hours
            admin_endpoints: false,
        }
    }
}

/// Blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    /// Supported networks
    pub networks: HashMap<u64, NetworkConfig>,
    
    /// Default network
    pub default_network: u64,
    
    /// Block confirmation count
    pub confirmation_blocks: u64,
    
    /// Enable mempool monitoring
    pub mempool_monitoring: bool,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        let mut networks = HashMap::new();
        
        // Ethereum mainnet
        networks.insert(
            1,
            NetworkConfig {
                name: "Ethereum".to_string(),
                rpc_url: "https://eth-mainnet.alchemyapi.io/v2/your-api-key".to_string(),
                ws_url: Some("wss://eth-mainnet.alchemyapi.io/v2/your-api-key".to_string()),
                chain_id: 1,
                gas_price_multiplier: 1.1,
                max_gas_price_gwei: 1000,
                min_gas_price_gwei: 1,
            },
        );
        
        Self {
            networks,
            default_network: 1,
            confirmation_blocks: 12,
            mempool_monitoring: true,
        }
    }
}

/// Network-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network name
    pub name: String,
    
    /// RPC endpoint URL
    pub rpc_url: String,
    
    /// WebSocket URL for real-time updates
    pub ws_url: Option<String>,
    
    /// Chain ID
    pub chain_id: u64,
    
    /// Gas price multiplier for priority
    pub gas_price_multiplier: f64,
    
    /// Maximum gas price in Gwei
    pub max_gas_price_gwei: u64,
    
    /// Minimum gas price in Gwei
    pub min_gas_price_gwei: u64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// TLS configuration
    pub tls: TlsConfig,
    
    /// Key management
    pub key_management: KeyManagementConfig,
    
    /// Validator configuration
    pub validators: ValidatorConfig,
    
    /// Security monitoring
    pub monitoring: SecurityMonitoringConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls: TlsConfig::default(),
            key_management: KeyManagementConfig::default(),
            validators: ValidatorConfig::default(),
            monitoring: SecurityMonitoringConfig::default(),
        }
    }
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,
    
    /// Certificate file path
    pub cert_file: Option<String>,
    
    /// Private key file path
    pub key_file: Option<String>,
    
    /// CA certificate file path
    pub ca_file: Option<String>,
    
    /// Minimum TLS version
    pub min_version: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cert_file: None,
            key_file: None,
            ca_file: None,
            min_version: "1.3".to_string(),
        }
    }
}

/// Key management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementConfig {
    /// Key storage backend
    pub storage_backend: String,
    
    /// Key rotation interval
    #[serde(with = "serde_duration")]
    pub rotation_interval: Duration,
    
    /// Enable hardware security module
    pub hsm_enabled: bool,
    
    /// Key derivation function parameters
    pub kdf_params: KdfParams,
}

impl Default for KeyManagementConfig {
    fn default() -> Self {
        Self {
            storage_backend: "file".to_string(),
            rotation_interval: Duration::from_secs(86400 * 30), // 30 days
            hsm_enabled: false,
            kdf_params: KdfParams::default(),
        }
    }
}

/// Key derivation function parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// Iteration count
    pub iterations: u32,
    
    /// Memory usage in KB
    pub memory_kb: u32,
    
    /// Parallelism factor
    pub parallelism: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            iterations: 100000,
            memory_kb: 64 * 1024, // 64 MB
            parallelism: 4,
        }
    }
}

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Validator endpoints
    pub endpoints: Vec<String>,
    
    /// Timeout for validator requests
    #[serde(with = "serde_duration")]
    pub request_timeout: Duration,
    
    /// Maximum retries
    pub max_retries: u32,
    
    /// Enable validator reputation tracking
    pub reputation_tracking: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            endpoints: vec![],
            request_timeout: Duration::from_secs(10),
            max_retries: 3,
            reputation_tracking: true,
        }
    }
}

/// Security monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMonitoringConfig {
    /// Enable intrusion detection
    pub intrusion_detection: bool,
    
    /// Enable anomaly detection
    pub anomaly_detection: bool,
    
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

impl Default for SecurityMonitoringConfig {
    fn default() -> Self {
        Self {
            intrusion_detection: true,
            anomaly_detection: true,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Failed authentication threshold
    pub failed_auth_threshold: u32,
    
    /// Suspicious transaction threshold
    pub suspicious_tx_threshold: u32,
    
    /// High MEV detection rate threshold
    pub high_mev_rate_threshold: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            failed_auth_threshold: 10,
            suspicious_tx_threshold: 100,
            high_mev_rate_threshold: 0.1, // 10% MEV detection rate
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub metrics_enabled: bool,
    
    /// Metrics endpoint
    pub metrics_endpoint: String,
    
    /// Enable tracing
    pub tracing_enabled: bool,
    
    /// Trace sample rate
    pub trace_sample_rate: f64,
    
    /// Log level
    pub log_level: String,
    
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            metrics_endpoint: "/metrics".to_string(),
            tracing_enabled: true,
            trace_sample_rate: 0.1, // 10%
            log_level: "info".to_string(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint
    pub endpoint: String,
    
    /// Check interval
    #[serde(with = "serde_duration")]
    pub interval: Duration,
    
    /// Timeout for health checks
    #[serde(with = "serde_duration")]
    pub timeout: Duration,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
        }
    }
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Worker thread count
    pub worker_threads: Option<usize>,
    
    /// Blocking thread count
    pub blocking_threads: Option<usize>,
    
    /// Enable thread pool optimization
    pub thread_pool_optimization: bool,
    
    /// Batch processing configuration
    pub batch_processing: BatchProcessingConfig,
    
    /// Memory limits
    pub memory_limits: MemoryLimitsConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: None, // Use default
            blocking_threads: None, // Use default
            thread_pool_optimization: true,
            batch_processing: BatchProcessingConfig::default(),
            memory_limits: MemoryLimitsConfig::default(),
        }
    }
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingConfig {
    /// Default batch size
    pub default_batch_size: usize,
    
    /// Maximum batch size
    pub max_batch_size: usize,
    
    /// Batch timeout
    #[serde(with = "serde_duration")]
    pub batch_timeout: Duration,
    
    /// Enable adaptive batching
    pub adaptive_batching: bool,
}

impl Default for BatchProcessingConfig {
    fn default() -> Self {
        Self {
            default_batch_size: 100,
            max_batch_size: 1000,
            batch_timeout: Duration::from_secs(10),
            adaptive_batching: true,
        }
    }
}

/// Memory limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimitsConfig {
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<usize>,
    
    /// Memory warning threshold
    pub warning_threshold_percent: f64,
    
    /// Enable memory pressure handling
    pub pressure_handling: bool,
}

impl Default for MemoryLimitsConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: None,
            warning_threshold_percent: 80.0, // 80%
            pressure_handling: true,
        }
    }
}

// Custom serde module for Duration
mod serde_duration {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Configuration loading utilities
impl MEVShieldConfig {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: MEVShieldConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = MEVShieldConfig::default();
        
        // Load from environment variables with prefix "MEV_SHIELD_"
        config::Config::builder()
            .add_source(config::Environment::with_prefix("MEV_SHIELD"))
            .build()?
            .try_deserialize_into(&mut config)?;
        
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate encryption configuration
        if self.services.encryption.threshold > self.services.encryption.total_validators {
            return Err("Threshold cannot be greater than total validators".into());
        }
        
        // Validate redistribution percentages
        let total_percentage = self.services.redistribution.redistribution_percentage
            + self.services.redistribution.gas_reserve_percentage
            + self.services.redistribution.validator_share;
        
        if total_percentage > 100.0 {
            return Err("Total redistribution percentages exceed 100%".into());
        }
        
        // Validate network configuration
        if self.blockchain.networks.is_empty() {
            return Err("At least one blockchain network must be configured".into());
        }
        
        if !self
            .blockchain
            .networks
            .contains_key(&self.blockchain.default_network)
        {
            return Err("Default network not found in configured networks".into());
        }
        
        Ok(())
    }
}