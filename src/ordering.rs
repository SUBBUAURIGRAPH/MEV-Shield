// ordering/src/lib.rs
//! Fair Ordering Service for MEV Shield
//! 
//! Implements verifiable delay functions (VDF) for deterministic transaction ordering
//! that cannot be manipulated by block builders or validators.

use async_trait::async_trait;
use mev_shield_core::{
    config::OrderingConfig,
    error::{MEVShieldError, OrderingError},
    traits::{OrderingProof, OrderingService},
    types::*,
};
use num_bigint::BigUint;
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Fair ordering service implementation using VDF
pub struct FairOrderingService {
    config: OrderingConfig,
    vdf_params: VDFParameters,
    ordering_cache: Arc<RwLock<HashMap<OrderingKey, VDFOutput>>>,
    batch_processor: BatchProcessor,
    performance_monitor: PerformanceMonitor,
}

/// VDF parameters for ordering computation
#[derive(Debug, Clone)]
pub struct VDFParameters {
    pub difficulty: u64,
    pub security_param: u32,
    pub modulus: BigUint,
    pub batch_size: u32,
}

/// Key for ordering cache
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OrderingKey {
    pub commitment_hash: Hash,
    pub block_height: u64,
}

/// VDF computation output
#[derive(Debug, Clone)]
pub struct VDFOutput {
    pub output: BigUint,
    pub proof: VDFProof,
    pub computation_time: Duration,
    pub verified: bool,
}

/// VDF proof structure
#[derive(Debug, Clone)]
pub struct VDFProof {
    pub proof_hash: Hash,
    pub intermediate_values: Vec<BigUint>,
    pub witness: Vec<u8>,
}

/// Batch processor for VDF computations
pub struct BatchProcessor {
    max_batch_size: usize,
    pending_commitments: Arc<RwLock<Vec<OrderingCommitment>>>,
    processing_queue: Arc<RwLock<Vec<VDFTask>>>,
}

/// VDF computation task
#[derive(Debug, Clone)]
pub struct VDFTask {
    pub commitment: OrderingCommitment,
    pub start_time: Instant,
    pub priority: u32,
}

/// Performance monitoring for VDF operations
pub struct PerformanceMonitor {
    computation_times: Arc<RwLock<VecDeque<Duration>>>,
    success_rate: Arc<RwLock<f64>>,
    average_batch_size: Arc<RwLock<f64>>,
}

/// Priority computation engine
pub struct PriorityEngine {
    gas_weight: f64,
    time_weight: f64,
    type_weights: HashMap<TransactionType, f64>,
}

impl FairOrderingService {
    /// Create a new fair ordering service
    pub async fn new(config: OrderingConfig) -> Result<Self, MEVShieldError> {
        // Generate VDF parameters
        let vdf_params = Self::generate_vdf_parameters(&config)?;
        
        // Initialize batch processor
        let batch_processor = BatchProcessor::new(config.batch_size as usize);
        
        // Initialize performance monitor
        let performance_monitor = PerformanceMonitor::new();
        
        let service = Self {
            config,
            vdf_params,
            ordering_cache: Arc::new(RwLock::new(HashMap::new())),
            batch_processor,
            performance_monitor,
        };
        
        // Start background tasks
        service.start_batch_processing().await;
        service.start_performance_monitoring().await;
        
        Ok(service)
    }
    
    /// Generate VDF parameters based on configuration
    fn generate_vdf_parameters(config: &OrderingConfig) -> Result<VDFParameters, OrderingError> {
        // Generate a safe RSA modulus for VDF (simplified)
        // In production, this would use proper cryptographic parameter generation
        let modulus = Self::generate_safe_prime(config.vdf_security_param)?;
        
        Ok(VDFParameters {
            difficulty: config.vdf_difficulty,
            security_param: config.vdf_security_param,
            modulus,
            batch_size: config.batch_size,
        })
    }
    
    /// Generate a safe prime for VDF modulus (simplified)
    fn generate_safe_prime(bits: u32) -> Result<BigUint, OrderingError> {
        // Simplified - in production would use proper prime generation
        let base = BigUint::from(2u32);
        let exponent = BigUint::from(bits);
        let modulus = base.pow(exponent.try_into().unwrap_or(256)) + BigUint::from(1u32);
        Ok(modulus)
    }
    
    /// Start background batch processing
    async fn start_batch_processing(&self) {
        let batch_processor = self.batch_processor.clone();
        let vdf_params = self.vdf_params.clone();
        let cache = Arc::clone(&self.ordering_cache);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                // Process pending batches
                if let Err(e) = batch_processor.process_pending_batch(&vdf_params, &cache).await {
                    error!("Batch processing failed: {}", e);
                }
                
                // Cleanup old cache entries
                Self::cleanup_cache(&cache, Duration::from_secs(3600)).await;
            }
        });
    }
    
    /// Start performance monitoring
    async fn start_performance_monitoring(&self) {
        let monitor = self.performance_monitor.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                monitor.update_metrics().await;
            }
        });
    }
    
    /// Cleanup old cache entries
    async fn cleanup_cache(
        cache: &Arc<RwLock<HashMap<OrderingKey, VDFOutput>>>,
        max_age: Duration,
    ) {
        let mut cache_map = cache.write().await;
        let now = Instant::now();
        
        cache_map.retain(|_key, output| {
            now.duration_since(Instant::now() - output.computation_time) < max_age
        });
    }
}

#[async_trait]
impl OrderingService for FairOrderingService {
    async fn create_ordering_commitment(
        &self,
        transaction: &EncryptedTransaction,
    ) -> Result<OrderingCommitment, MEVShieldError> {
        info!("Creating ordering commitment for transaction: {}", transaction.id);
        
        // Create priority data
        let priority_data = self.create_priority_data(transaction)?;
        
        // Create deterministic commitment
        let commitment = self.create_deterministic_commitment(transaction, &priority_data)?;
        
        // Calculate priority score
        let priority = self.calculate_priority_score(transaction, &priority_data);
        
        let ordering_commitment = OrderingCommitment {
            transaction_hash: transaction.id,
            submission_time: transaction.submission_time,
            priority_data,
            commitment_hash: commitment,
            priority,
        };
        
        info!(
            "Created ordering commitment with priority {} for transaction: {}",
            priority, transaction.id
        );
        
        Ok(ordering_commitment)
    }
    
    async fn order_transactions(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<Vec<Transaction>, MEVShieldError> {
        info!("Ordering {} transactions using VDF", transactions.len());
        let start_time = Instant::now();
        
        // Create commitments for all transactions
        let commitments = self.create_commitments_for_transactions(&transactions).await?;
        
        // Compute VDF for each commitment
        let vdf_outputs = self.compute_vdf_batch(&commitments).await?;
        
        // Create ordered pairs
        let mut ordered_pairs: Vec<(Transaction, VDFOutput)> = transactions
            .into_iter()
            .zip(vdf_outputs.into_iter())
            .collect();
        
        // Sort by VDF output for deterministic ordering
        ordered_pairs.sort_by(|a, b| {
            // Primary sort: VDF output (deterministic)
            let vdf_cmp = a.1.output.cmp(&b.1.output);
            if vdf_cmp != std::cmp::Ordering::Equal {
                return vdf_cmp;
            }
            
            // Secondary sort: submission time (for equal VDF outputs)
            a.0.submission_time.cmp(&b.0.submission_time)
        });
        
        // Extract ordered transactions
        let ordered_transactions: Vec<Transaction> = ordered_pairs
            .into_iter()
            .map(|(tx, _)| tx)
            .collect();
        
        let ordering_time = start_time.elapsed();
        info!(
            "Ordered {} transactions in {:?}",
            ordered_transactions.len(),
            ordering_time
        );
        
        // Update performance metrics
        self.performance_monitor
            .record_ordering_time(ordering_time)
            .await;
        
        Ok(ordered_transactions)
    }
    
    async fn verify_ordering_proof(
        &self,
        commitment: &OrderingCommitment,
        proof: &OrderingProof,
    ) -> Result<bool, MEVShieldError> {
        info!("Verifying ordering proof for commitment: {}", commitment.commitment_hash);
        
        // Convert commitment hash to BigUint
        let input = BigUint::from_bytes_be(&commitment.commitment_hash.0);
        
        // Verify VDF computation
        let is_valid = Self::verify_vdf_computation(
            &input,
            &proof.intermediate_values,
            &self.vdf_params,
        )?;
        
        if is_valid {
            info!("Ordering proof verified successfully");
        } else {
            warn!("Ordering proof verification failed");
        }
        
        Ok(is_valid)
    }
}

impl FairOrderingService {
    /// Create priority data for a transaction
    fn create_priority_data(&self, tx: &EncryptedTransaction) -> Result<Vec<u8>, OrderingError> {
        let mut data = Vec::new();
        
        // Add gas price (higher priority)
        data.extend_from_slice(&tx.gas_price.to_bytes_be());
        
        // Add submission time (earlier submissions get slight priority)
        data.extend_from_slice(&tx.submission_time.timestamp().to_be_bytes());
        
        // Add priority level
        let priority_byte = match tx.priority {
            Priority::Low => 1u8,
            Priority::Medium => 2u8,
            Priority::High => 3u8,
        };
        data.push(priority_byte);
        
        // Add chain ID
        data.extend_from_slice(&tx.chain_id.to_be_bytes());
        
        Ok(data)
    }
    
    /// Create deterministic commitment hash
    fn create_deterministic_commitment(
        &self,
        tx: &EncryptedTransaction,
        priority_data: &[u8],
    ) -> Result<Hash, OrderingError> {
        let mut hasher = Keccak256::new();
        
        // Add transaction hash
        hasher.update(&tx.id.0);
        
        // Add priority data
        hasher.update(priority_data);
        
        // Add encrypted data hash for uniqueness
        hasher.update(&tx.encrypted_data);
        
        // Add submission time for temporal ordering
        hasher.update(&tx.submission_time.timestamp().to_be_bytes());
        
        let result = hasher.finalize();
        Ok(Hash::from(result.as_slice()))
    }
    
    /// Calculate priority score for ordering
    fn calculate_priority_score(&self, tx: &EncryptedTransaction, priority_data: &[u8]) -> u32 {
        let mut score = 0u32;
        
        // Gas price component (0-1000 points)
        let gas_price_score = std::cmp::min(
            (tx.gas_price.clone() / BigUint::from(1_000_000_000u64)).try_into().unwrap_or(0u32),
            1000u32,
        );
        score += gas_price_score;
        
        // Priority level component (0-300 points)
        let priority_score = match tx.priority {
            Priority::Low => 100u32,
            Priority::Medium => 200u32,
            Priority::High => 300u32,
        };
        score += priority_score;
        
        // Time component (newer transactions get slight boost)
        let time_since_submission = chrono::Utc::now() - tx.submission_time;
        let time_penalty = std::cmp::min(time_since_submission.num_seconds() as u32, 100u32);
        score = score.saturating_sub(time_penalty);
        
        score
    }
    
    /// Create commitments for a batch of transactions
    async fn create_commitments_for_transactions(
        &self,
        transactions: &[Transaction],
    ) -> Result<Vec<OrderingCommitment>, OrderingError> {
        let mut commitments = Vec::with_capacity(transactions.len());
        
        for tx in transactions {
            // Convert Transaction to EncryptedTransaction for commitment creation
            // In practice, this would come from the encrypted mempool
            let encrypted_tx = EncryptedTransaction {
                id: tx.hash(),
                encrypted_data: vec![0u8; 100], // Placeholder
                submission_time: tx.submission_time,
                time_lock: None,
                priority: Priority::Medium, // Would be determined by gas price
                gas_price: tx.gas_price.clone(),
                chain_id: tx.chain_id,
            };
            
            let commitment = self.create_ordering_commitment(&encrypted_tx).await
                .map_err(|e| OrderingError::CommitmentFailed(e.to_string()))?;
            
            commitments.push(commitment);
        }
        
        Ok(commitments)
    }
    
    /// Compute VDF for a batch of commitments
    async fn compute_vdf_batch(
        &self,
        commitments: &[OrderingCommitment],
    ) -> Result<Vec<VDFOutput>, OrderingError> {
        let batch_size = self.vdf_params.batch_size as usize;
        let mut results = Vec::with_capacity(commitments.len());
        
        // Process in batches to avoid overwhelming the system
        for batch in commitments.chunks(batch_size) {
            let batch_results = self.compute_vdf_batch_parallel(batch).await?;
            results.extend(batch_results);
        }
        
        Ok(results)
    }
    
    /// Compute VDF for a batch of commitments in parallel
    async fn compute_vdf_batch_parallel(
        &self,
        commitments: &[OrderingCommitment],
    ) -> Result<Vec<VDFOutput>, OrderingError> {
        let tasks: Vec<_> = commitments
            .iter()
            .map(|commitment| {
                let commitment = commitment.clone();
                let params = self.vdf_params.clone();
                
                tokio::spawn(async move {
                    Self::compute_vdf_single(&commitment, &params).await
                })
            })
            .collect();
        
        let mut results = Vec::with_capacity(commitments.len());
        for task in tasks {
            let result = task.await
                .map_err(|e| OrderingError::VDFComputationFailed(e.to_string()))?;
            results.push(result?);
        }
        
        Ok(results)
    }
    
    /// Compute VDF for a single commitment
    async fn compute_vdf_single(
        commitment: &OrderingCommitment,
        params: &VDFParameters,
    ) -> Result<VDFOutput, OrderingError> {
        let start_time = Instant::now();
        
        // Convert commitment hash to BigUint
        let input = BigUint::from_bytes_be(&commitment.commitment_hash.0);
        
        // Compute VDF: output = input^(2^difficulty) mod modulus
        let output = Self::compute_vdf_output(&input, params)?;
        
        // Generate proof
        let proof = Self::generate_vdf_proof(&input, &output, params)?;
        
        let computation_time = start_time.elapsed();
        
        Ok(VDFOutput {
            output,
            proof,
            computation_time,
            verified: false, // Will be verified separately
        })
    }
    
    /// Core VDF computation
    fn compute_vdf_output(
        input: &BigUint,
        params: &VDFParameters,
    ) -> Result<BigUint, OrderingError> {
        let mut result = input.clone();
        
        // Compute input^(2^difficulty) mod modulus using repeated squaring
        for i in 0..params.difficulty {
            result = (&result * &result) % &params.modulus;
            
            // Yield control periodically for long computations
            if i % 1000 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        Ok(result)
    }
    
    /// Generate VDF proof
    fn generate_vdf_proof(
        input: &BigUint,
        output: &BigUint,
        params: &VDFParameters,
    ) -> Result<VDFProof, OrderingError> {
        // Generate intermediate values for proof verification
        let mut intermediate_values = Vec::new();
        let mut current = input.clone();
        
        // Store intermediate values at regular intervals
        let checkpoint_interval = params.difficulty / 10; // 10 checkpoints
        
        for i in 0..params.difficulty {
            if i % checkpoint_interval == 0 {
                intermediate_values.push(current.clone());
            }
            current = (&current * &current) % &params.modulus;
        }
        
        // Create proof hash
        let proof_data = [
            input.to_bytes_be(),
            output.to_bytes_be(),
            params.difficulty.to_be_bytes().to_vec(),
        ].concat();
        
        let mut hasher = Keccak256::new();
        hasher.update(&proof_data);
        let proof_hash = Hash::from(hasher.finalize().as_slice());
        
        // Generate witness (simplified)
        let witness = proof_hash.0.to_vec();
        
        Ok(VDFProof {
            proof_hash,
            intermediate_values,
            witness,
        })
    }
    
    /// Verify VDF computation
    fn verify_vdf_computation(
        input: &BigUint,
        intermediate_values: &[BigUint],
        params: &VDFParameters,
    ) -> Result<bool, OrderingError> {
        // Verify each intermediate value
        let checkpoint_interval = params.difficulty / 10;
        let mut current = input.clone();
        
        for (i, expected_value) in intermediate_values.iter().enumerate() {
            // Compute up to this checkpoint
            let target_step = (i + 1) * (checkpoint_interval as usize);
            
            while (current != *expected_value) && (i * checkpoint_interval as usize) < target_step {
                current = (&current * &current) % &params.modulus;
            }
            
            if current != *expected_value {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

impl BatchProcessor {
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            max_batch_size,
            pending_commitments: Arc::new(RwLock::new(Vec::new())),
            processing_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn add_commitment(&self, commitment: OrderingCommitment) -> Result<(), OrderingError> {
        let mut pending = self.pending_commitments.write().await;
        pending.push(commitment);
        
        // Trigger batch processing if we have enough commitments
        if pending.len() >= self.max_batch_size {
            let batch = std::mem::take(&mut *pending);
            drop(pending);
            
            self.queue_batch_for_processing(batch).await?;
        }
        
        Ok(())
    }
    
    async fn queue_batch_for_processing(
        &self,
        commitments: Vec<OrderingCommitment>,
    ) -> Result<(), OrderingError> {
        let mut queue = self.processing_queue.write().await;
        
        for commitment in commitments {
            let task = VDFTask {
                commitment,
                start_time: Instant::now(),
                priority: 0, // Would be calculated based on commitment priority
            };
            queue.push(task);
        }
        
        // Sort by priority
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }
    
    pub async fn process_pending_batch(
        &self,
        params: &VDFParameters,
        cache: &Arc<RwLock<HashMap<OrderingKey, VDFOutput>>>,
    ) -> Result<(), OrderingError> {
        let tasks = {
            let mut queue = self.processing_queue.write().await;
            if queue.is_empty() {
                return Ok(());
            }
            
            // Take up to batch_size tasks
            let take_count = std::cmp::min(queue.len(), params.batch_size as usize);
            queue.drain(0..take_count).collect::<Vec<_>>()
        };
        
        if tasks.is_empty() {
            return Ok(());
        }
        
        info!("Processing batch of {} VDF tasks", tasks.len());
        
        // Process tasks in parallel
        let mut handles = Vec::new();
        for task in tasks {
            let params = params.clone();
            let cache = Arc::clone(cache);
            
            let handle = tokio::spawn(async move {
                let result = FairOrderingService::compute_vdf_single(&task.commitment, &params).await;
                
                match result {
                    Ok(output) => {
                        // Cache the result
                        let key = OrderingKey {
                            commitment_hash: task.commitment.commitment_hash,
                            block_height: 0, // Would be set appropriately
                        };
                        
                        let mut cache_map = cache.write().await;
                        cache_map.insert(key, output);
                    }
                    Err(e) => {
                        error!("VDF computation failed: {}", e);
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            if let Err(e) = handle.await {
                error!("VDF task failed: {}", e);
            }
        }
        
        Ok(())
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            computation_times: Arc::new(RwLock::new(VecDeque::new())),
            success_rate: Arc::new(RwLock::new(1.0)),
            average_batch_size: Arc::new(RwLock::new(1.0)),
        }
    }
    
    pub async fn record_ordering_time(&self, duration: Duration) {
        let mut times = self.computation_times.write().await;
        times.push_back(duration);
        
        // Keep only recent measurements
        while times.len() > 1000 {
            times.pop_front();
        }
    }
    
    pub async fn update_metrics(&self) {
        let times = self.computation_times.read().await;
        
        if !times.is_empty() {
            let total_duration: Duration = times.iter().sum();
            let average_duration = total_duration / times.len() as u32;
            
            info!(
                "VDF Performance: {} computations, avg time: {:?}",
                times.len(),
                average_duration
            );
        }
    }
    
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        let times = self.computation_times.read().await;
        let success_rate = *self.success_rate.read().await;
        let avg_batch_size = *self.average_batch_size.read().await;
        
        let (min_time, max_time, avg_time) = if times.is_empty() {
            (Duration::from_secs(0), Duration::from_secs(0), Duration::from_secs(0))
        } else {
            let min = times.iter().min().copied().unwrap_or_default();
            let max = times.iter().max().copied().unwrap_or_default();
            let avg = times.iter().sum::<Duration>() / times.len() as u32;
            (min, max, avg)
        };
        
        PerformanceStats {
            min_computation_time: min_time,
            max_computation_time: max_time,
            avg_computation_time: avg_time,
            success_rate,
            average_batch_size: avg_batch_size,
            total_computations: times.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub min_computation_time: Duration,
    pub max_computation_time: Duration,
    pub avg_computation_time: Duration,
    pub success_rate: f64,
    pub average_batch_size: f64,
    pub total_computations: usize,
}

impl PriorityEngine {
    pub fn new() -> Self {
        let mut type_weights = HashMap::new();
        type_weights.insert(TransactionType::Transfer, 1.0);
        type_weights.insert(TransactionType::DEXTrade, 1.2);
        type_weights.insert(TransactionType::ContractCall, 1.1);
        type_weights.insert(TransactionType::Deploy, 0.8);
        
        Self {
            gas_weight: 0.7,
            time_weight: 0.3,
            type_weights,
        }
    }
    
    pub fn calculate_priority(&self, tx: &Transaction) -> f64 {
        // Gas price component
        let gas_component = self.gas_weight * (tx.gas_price.clone().try_into().unwrap_or(0.0f64));
        
        // Time component (earlier = higher priority)
        let age = chrono::Utc::now() - tx.submission_time;
        let time_component = self.time_weight * (1.0 / (age.num_seconds() as f64 + 1.0));
        
        // Transaction type component
        let type_weight = self.type_weights
            .get(&tx.transaction_type())
            .copied()
            .unwrap_or(1.0);
        
        (gas_component + time_component) * type_weight
    }
}

use std::collections::VecDeque;

impl Clone for BatchProcessor {
    fn clone(&self) -> Self {
        Self {
            max_batch_size: self.max_batch_size,
            pending_commitments: Arc::clone(&self.pending_commitments),
            processing_queue: Arc::clone(&self.processing_queue),
        }
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            computation_times: Arc::clone(&self.computation_times),
            success_rate: Arc::clone(&self.success_rate),
            average_batch_size: Arc::clone(&self.average_batch_size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mev_shield_core::types::*;
    
    fn create_test_transaction() -> Transaction {
        Transaction {
            from: Address::from_str("0x742d35cc6465c3c962800060acea9d8ac2e7a0cf").unwrap(),
            to: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
            value: BigUint::from(1000000000000000000u64),
            gas: 21000,
            gas_price: BigUint::from(20000000000u64),
            gas_used: 21000,
            nonce: 42,
            data: vec![],
            chain_id: 1,
            submission_time: chrono::Utc::now(),
        }
    }
    
    #[tokio::test]
    async fn test_ordering_service_creation() {
        let config = OrderingConfig {
            vdf_difficulty: 1000, // Low for testing
            vdf_security_param: 128,
            batch_size: 10,
            computation_timeout: Duration::from_secs(30),
            verify_proofs: true,
        };
        
        let service = FairOrderingService::new(config).await.unwrap();
        assert!(service.vdf_params.difficulty > 0);
    }
    
    #[tokio::test]
    async fn test_transaction_ordering() {
        let config = OrderingConfig {
            vdf_difficulty: 100, // Very low for fast testing
            vdf_security_param: 64,
            batch_size: 5,
            computation_timeout: Duration::from_secs(10),
            verify_proofs: false, // Skip verification for speed
        };
        
        let service = FairOrderingService::new(config).await.unwrap();
        
        // Create test transactions with different gas prices
        let mut transactions = Vec::new();
        for i in 0..3 {
            let mut tx = create_test_transaction();
            tx.nonce = i;
            tx.gas_price = BigUint::from((30 - i * 5) * 1000000000u64); // Decreasing gas prices
            transactions.push(tx);
        }
        
        let original_order: Vec<_> = transactions.iter().map(|tx| tx.nonce).collect();
        
        // Order transactions
        let ordered = service.order_transactions(transactions).await.unwrap();
        let new_order: Vec<_> = ordered.iter().map(|tx| tx.nonce).collect();
        
        assert_eq!(ordered.len(), 3);
        
        // Ordering should be deterministic (may not match gas price order due to VDF)
        // Test that same input produces same output
        let transactions2 = (0..3).map(|i| {
            let mut tx = create_test_transaction();
            tx.nonce = i;
            tx.gas_price = BigUint::from((30 - i * 5) * 1000000000u64);
            tx
        }).collect();
        
        let ordered2 = service.order_transactions(transactions2).await.unwrap();
        let new_order2: Vec<_> = ordered2.iter().map(|tx| tx.nonce).collect();
        
        assert_eq!(new_order, new_order2); // Deterministic ordering
    }
    
    #[tokio::test]
    async fn test_ordering_commitment_creation() {
        let config = OrderingConfig {
            vdf_difficulty: 100,
            vdf_security_param: 64,
            batch_size: 5,
            computation_timeout: Duration::from_secs(10),
            verify_proofs: true,
        };
        
        let service = FairOrderingService::new(config).await.unwrap();
        
        let encrypted_tx = EncryptedTransaction {
            id: TxHash([1u8; 32]),
            encrypted_data: vec![1, 2, 3, 4],
            submission_time: chrono::Utc::now(),
            time_lock: None,
            priority: Priority::High,
            gas_price: BigUint::from(50000000000u64),
            chain_id: 1,
        };
        
        let commitment = service.create_ordering_commitment(&encrypted_tx).await.unwrap();
        
        assert_eq!(commitment.transaction_hash, encrypted_tx.id);
        assert!(commitment.priority > 0);
        assert!(!commitment.commitment_hash.0.iter().all(|&b| b == 0));
    }
    
    #[test]
    fn test_vdf_computation() {
        let params = VDFParameters {
            difficulty: 10, // Very small for testing
            security_param: 64,
            modulus: BigUint::from(2047u32), // Small prime for testing
            batch_size: 1,
        };
        
        let input = BigUint::from(123u32);
        let output = FairOrderingService::compute_vdf_output(&input, &params).unwrap();
        
        assert!(output < params.modulus);
        assert!(output > BigUint::from(0u32));
        
        // Test determinism
        let output2 = FairOrderingService::compute_vdf_output(&input, &params).unwrap();
        assert_eq!(output, output2);
    }
    
    #[test]
    fn test_priority_engine() {
        let engine = PriorityEngine::new();
        
        let mut tx1 = create_test_transaction();
        tx1.gas_price = BigUint::from(50000000000u64);
        
        let mut tx2 = create_test_transaction();
        tx2.gas_price = BigUint::from(20000000000u64);
        
        let priority1 = engine.calculate_priority(&tx1);
        let priority2 = engine.calculate_priority(&tx2);
        
        // Higher gas price should generally result in higher priority
        assert!(priority1 > priority2);
    }
    
    #[tokio::test]
    async fn test_batch_processor() {
        let processor = BatchProcessor::new(3);
        
        // Create test commitments
        for i in 0..5 {
            let commitment = OrderingCommitment {
                transaction_hash: TxHash([i; 32]),
                submission_time: chrono::Utc::now(),
                priority_data: vec![i],
                commitment_hash: Hash([i; 32]),
                priority: i as u32,
            };
            
            processor.add_commitment(commitment).await.unwrap();
        }
        
        // Check that batches were triggered
        let queue = processor.processing_queue.read().await;
        assert!(!queue.is_empty());
    }
}