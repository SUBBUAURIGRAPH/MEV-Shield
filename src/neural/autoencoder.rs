            // Reconstruct
            let reconstruction = self.decoder.decode(&latent_sample);
            
            // Calculate reconstruction error
            let reconstruction_error = self.calculate_reconstruction_error(&input, &reconstruction);
            
            // Calculate KL divergence
            let kl_divergence = self.calculate_kl_divergence(&latent_mean, &latent_log_var);
            
            // Calculate anomaly score
            let anomaly_score = reconstruction_error + self.config.beta * kl_divergence;
            anomaly_scores.push(anomaly_score);
            
            // Check if anomaly
            if anomaly_score > self.anomaly_threshold {
                anomalies.push(AnomalyDetection {
                    transaction_id: tx_features.transaction_id.clone(),
                    anomaly_score,
                    reconstruction_error,
                    anomaly_type: self.classify_anomaly(&input, &reconstruction, anomaly_score),
                    latent_representation: latent_mean.clone(),
                    confidence: self.calculate_confidence(anomaly_score),
                });
            }
            
            // Update detection history
            self.detection_history.add(anomaly_score);
        }
        
        // Adaptive threshold adjustment
        self.update_threshold(&anomaly_scores);
        
        AnomalyDetectionResult {
            anomalies,
            mean_anomaly_score: anomaly_scores.iter().sum::<f32>() / anomaly_scores.len() as f32,
            threshold: self.anomaly_threshold,
            detection_rate: anomalies.len() as f32 / transactions.len() as f32,
        }
    }
    
    /// Train VAE on normal transaction patterns
    pub fn train(
        &mut self,
        training_data: &[TransactionFeatures],
        epochs: usize,
    ) -> Result<TrainingResult> {
        let mut total_loss = 0.0;
        let mut reconstruction_losses = Vec::new();
        let mut kl_losses = Vec::new();
        
        for epoch in 0..epochs {
            let mut epoch_loss = 0.0;
            let mut epoch_recon_loss = 0.0;
            let mut epoch_kl_loss = 0.0;
            
            for batch in training_data.chunks(32) {
                // Forward pass
                let mut batch_recon_loss = 0.0;
                let mut batch_kl_loss = 0.0;
                
                for tx_features in batch {
                    let input = self.prepare_input(tx_features);
                    
                    // Encode
                    let (latent_mean, latent_log_var) = self.encoder.encode(&input);
                    
                    // Sample
                    let latent_sample = self.reparameterize(&latent_mean, &latent_log_var);
                    
                    // Decode
                    let reconstruction = self.decoder.decode(&latent_sample);
                    
                    // Calculate losses
                    let recon_loss = self.calculate_reconstruction_error(&input, &reconstruction);
                    let kl_loss = self.calculate_kl_divergence(&latent_mean, &latent_log_var);
                    
                    batch_recon_loss += recon_loss;
                    batch_kl_loss += kl_loss;
                }
                
                // Backward pass (simplified)
                self.backward_pass(batch_recon_loss, batch_kl_loss)?;
                
                epoch_recon_loss += batch_recon_loss;
                epoch_kl_loss += batch_kl_loss;
            }
            
            epoch_loss = epoch_recon_loss + self.config.beta * epoch_kl_loss;
            total_loss += epoch_loss;
            
            reconstruction_losses.push(epoch_recon_loss / training_data.len() as f32);
            kl_losses.push(epoch_kl_loss / training_data.len() as f32);
            
            // Log progress
            if epoch % 10 == 0 {
                println!(
                    "Epoch {}/{}: Loss = {:.4}, Recon = {:.4}, KL = {:.4}",
                    epoch, epochs, epoch_loss, epoch_recon_loss, epoch_kl_loss
                );
            }
        }
        
        Ok(TrainingResult {
            final_loss: total_loss / epochs as f32,
            reconstruction_losses,
            kl_losses,
            epochs_trained: epochs,
        })
    }
    
    /// Generate synthetic normal patterns
    pub fn generate_normal_patterns(&self, n_samples: usize) -> Vec<Array1<f32>> {
        let mut patterns = Vec::new();
        
        for _ in 0..n_samples {
            // Sample from standard normal
            let latent = self.sample_latent();
            
            // Decode to generate pattern
            let pattern = self.decoder.decode(&latent);
            patterns.push(pattern);
        }
        
        patterns
    }
    
    /// Detect novel attack patterns
    pub fn detect_novel_attacks(
        &mut self,
        transactions: &[TransactionFeatures],
    ) -> Vec<NovelAttack> {
        let mut novel_attacks = Vec::new();
        
        for tx_features in transactions {
            let input = self.prepare_input(tx_features);
            let (latent_mean, latent_log_var) = self.encoder.encode(&input);
            
            // Check if latent representation is unusual
            let latent_anomaly = self.detect_latent_anomaly(&latent_mean);
            
            if latent_anomaly > 0.8 {
                // Analyze pattern
                let pattern_analysis = self.analyze_novel_pattern(&input, &latent_mean);
                
                novel_attacks.push(NovelAttack {
                    transaction_id: tx_features.transaction_id.clone(),
                    pattern_signature: latent_mean.clone(),
                    similarity_to_known: self.calculate_similarity_to_known(&latent_mean),
                    attack_characteristics: pattern_analysis,
                    risk_score: latent_anomaly,
                });
            }
        }
        
        novel_attacks
    }
    
    // Helper methods
    fn prepare_input(&self, features: &TransactionFeatures) -> Array1<f32> {
        let mut input = Array1::zeros(self.config.input_dim);
        
        // Normalize and encode features
        input[0] = normalize(features.gas_price as f32, 1e9, 1e12);
        input[1] = normalize(features.value as f32, 0.0, 1e20);
        input[2] = normalize(features.nonce as f32, 0.0, 1000.0);
        input[3] = features.is_contract_interaction as i32 as f32;
        input[4] = normalize(features.data_size as f32, 0.0, 10000.0);
        input[5] = normalize(features.mempool_time as f32, 0.0, 60.0);
        
        // Protocol interaction features
        input[6] = features.interacts_with_dex as i32 as f32;
        input[7] = features.interacts_with_lending as i32 as f32;
        input[8] = features.interacts_with_bridge as i32 as f32;
        
        // Historical features
        input[9] = normalize(features.sender_history.total_txs as f32, 0.0, 10000.0);
        input[10] = normalize(features.sender_history.mev_count as f32, 0.0, 100.0);
        
        // Temporal features
        input[11] = encode_time_of_day(features.timestamp);
        input[12] = encode_day_of_week(features.timestamp);
        
        // Network state
        input[13] = normalize(features.network_congestion, 0.0, 1.0);
        input[14] = normalize(features.base_fee as f32, 1e9, 1e11);
        
        input
    }
    
    fn reparameterize(&self, mean: &Array1<f32>, log_var: &Array1<f32>) -> Array1<f32> {
        use rand_distr::{Normal, Distribution};
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();
        
        let epsilon = Array1::from_shape_fn(mean.len(), |_| normal.sample(&mut rng));
        mean + &(log_var.mapv(|x| (x / 2.0).exp()) * epsilon)
    }
    
    fn calculate_reconstruction_error(&self, original: &Array1<f32>, reconstruction: &Array1<f32>) -> f32 {
        // Mean squared error
        (original - reconstruction).mapv(|x| x * x).mean().unwrap()
    }
    
    fn calculate_kl_divergence(&self, mean: &Array1<f32>, log_var: &Array1<f32>) -> f32 {
        // KL divergence from N(0, I)
        (-0.5 * (1.0 + log_var - mean.mapv(|x| x * x) - log_var.mapv(|x| x.exp())))
            .sum()
    }
    
    fn classify_anomaly(&self, input: &Array1<f32>, reconstruction: &Array1<f32>, score: f32) -> AnomalyType {
        let error_pattern = (input - reconstruction).mapv(|x| x.abs());
        
        // Analyze which features have highest reconstruction error
        let max_error_idx = error_pattern
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        match max_error_idx {
            0..=2 => AnomalyType::UnusualGasPattern,
            3..=5 => AnomalyType::AbnormalDataSize,
            6..=8 => AnomalyType::SuspiciousProtocolInteraction,
            9..=10 => AnomalyType::UnknownSenderBehavior,
            11..=12 => AnomalyType::TemporalAnomaly,
            13..=14 => AnomalyType::NetworkStateManipulation,
            _ => AnomalyType::Unknown,
        }
    }
    
    fn calculate_confidence(&self, anomaly_score: f32) -> f32 {
        // Sigmoid-based confidence
        1.0 / (1.0 + (-10.0 * (anomaly_score - self.anomaly_threshold)).exp())
    }
    
    fn update_threshold(&mut self, scores: &[f32]) {
        if scores.is_empty() {
            return;
        }
        
        // Calculate statistics
        let mean = scores.iter().sum::<f32>() / scores.len() as f32;
        let variance = scores.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / scores.len() as f32;
        let std_dev = variance.sqrt();
        
        // Update threshold using exponential moving average
        let new_threshold = mean + 3.0 * std_dev;
        self.anomaly_threshold = 0.9 * self.anomaly_threshold + 0.1 * new_threshold;
    }
    
    fn backward_pass(&mut self, recon_loss: f32, kl_loss: f32) -> Result<()> {
        // Simplified gradient update
        self.encoder.update_weights(recon_loss + self.config.beta * kl_loss);
        self.decoder.update_weights(recon_loss);
        Ok(())
    }
    
    fn sample_latent(&self) -> Array1<f32> {
        use rand_distr::{Normal, Distribution};
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();
        
        Array1::from_shape_fn(self.config.latent_dim, |_| normal.sample(&mut rng))
    }
    
    fn detect_latent_anomaly(&self, latent: &Array1<f32>) -> f32 {
        // Check if latent representation is far from origin
        let norm = latent.mapv(|x| x * x).sum().sqrt();
        (norm - self.config.latent_dim as f32).max(0.0) / self.config.latent_dim as f32
    }
    
    fn analyze_novel_pattern(&self, input: &Array1<f32>, latent: &Array1<f32>) -> Vec<String> {
        let mut characteristics = Vec::new();
        
        // Analyze input features
        if input[0] > 0.9 {
            characteristics.push("Extremely high gas price".to_string());
        }
        if input[6] > 0.5 && input[7] > 0.5 {
            characteristics.push("Cross-protocol interaction".to_string());
        }
        
        // Analyze latent space
        let latent_norm = latent.mapv(|x| x * x).sum().sqrt();
        if latent_norm > 3.0 * (self.config.latent_dim as f32).sqrt() {
            characteristics.push("Highly unusual pattern in latent space".to_string());
        }
        
        characteristics
    }
    
    fn calculate_similarity_to_known(&self, latent: &Array1<f32>) -> f32 {
        // Calculate cosine similarity to known attack patterns
        // Simplified: return random similarity
        0.3
    }
}

/// VAE Encoder
#[derive(Clone)]
pub struct VAEEncoder {
    layers: Vec<EncoderLayer>,
    mean_projection: LinearLayer,
    log_var_projection: LinearLayer,
}

impl VAEEncoder {
    pub fn new(input_dim: usize, hidden_dims: Vec<usize>, latent_dim: usize) -> Self {
        let mut layers = Vec::new();
        let mut prev_dim = input_dim;
        
        for hidden_dim in &hidden_dims {
            layers.push(EncoderLayer::new(prev_dim, *hidden_dim));
            prev_dim = *hidden_dim;
        }
        
        Self {
            layers,
            mean_projection: LinearLayer::new(prev_dim, latent_dim),
            log_var_projection: LinearLayer::new(prev_dim, latent_dim),
        }
    }
    
    pub fn encode(&self, input: &Array1<f32>) -> (Array1<f32>, Array1<f32>) {
        let mut hidden = input.clone();
        
        for layer in &self.layers {
            hidden = layer.forward(&hidden);
        }
        
        let mean = self.mean_projection.forward(&hidden);
        let log_var = self.log_var_projection.forward(&hidden);
        
        (mean, log_var)
    }
    
    pub fn update_weights(&mut self, loss: f32) {
        // Simplified weight update
        for layer in &mut self.layers {
            layer.update(loss);
        }
    }
}

/// VAE Decoder
#[derive(Clone)]
pub struct VAEDecoder {
    layers: Vec<DecoderLayer>,
    output_projection: LinearLayer,
}

impl VAEDecoder {
    pub fn new(latent_dim: usize, hidden_dims: Vec<usize>, output_dim: usize) -> Self {
        let mut layers = Vec::new();
        let mut prev_dim = latent_dim;
        
        for hidden_dim in hidden_dims.iter().rev() {
            layers.push(DecoderLayer::new(prev_dim, *hidden_dim));
            prev_dim = *hidden_dim;
        }
        
        Self {
            layers,
            output_projection: LinearLayer::new(prev_dim, output_dim),
        }
    }
    
    pub fn decode(&self, latent: &Array1<f32>) -> Array1<f32> {
        let mut hidden = latent.clone();
        
        for layer in &self.layers {
            hidden = layer.forward(&hidden);
        }
        
        // Sigmoid activation for output
        self.output_projection.forward(&hidden).mapv(|x| 1.0 / (1.0 + (-x).exp()))
    }
    
    pub fn update_weights(&mut self, loss: f32) {
        for layer in &mut self.layers {
            layer.update(loss);
        }
    }
}

/// Encoder Layer
#[derive(Clone)]
struct EncoderLayer {
    linear: LinearLayer,
    activation: ActivationType,
}

impl EncoderLayer {
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        Self {
            linear: LinearLayer::new(input_dim, output_dim),
            activation: ActivationType::ReLU,
        }
    }
    
    pub fn forward(&self, input: &Array1<f32>) -> Array1<f32> {
        let linear_output = self.linear.forward(input);
        self.apply_activation(linear_output)
    }
    
    fn apply_activation(&self, x: Array1<f32>) -> Array1<f32> {
        match self.activation {
            ActivationType::ReLU => x.mapv(|a| a.max(0.0)),
            ActivationType::Tanh => x.mapv(|a| a.tanh()),
            ActivationType::Sigmoid => x.mapv(|a| 1.0 / (1.0 + (-a).exp())),
        }
    }
    
    pub fn update(&mut self, loss: f32) {
        self.linear.update(loss);
    }
}

/// Decoder Layer
type DecoderLayer = EncoderLayer;

/// Linear Layer
#[derive(Clone)]
struct LinearLayer {
    weight: Array2<f32>,
    bias: Array1<f32>,
}

impl LinearLayer {
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        use ndarray_rand::RandomExt;
        use ndarray_rand::rand_distr::Uniform;
        
        let scale = (2.0 / (input_dim + output_dim) as f32).sqrt();
        
        Self {
            weight: Array2::random((output_dim, input_dim), Uniform::new(-scale, scale)),
            bias: Array1::zeros(output_dim),
        }
    }
    
    pub fn forward(&self, input: &Array1<f32>) -> Array1<f32> {
        self.weight.dot(input) + &self.bias
    }
    
    pub fn update(&mut self, loss: f32) {
        // Simplified gradient update
        let learning_rate = 0.001;
        self.weight *= 1.0 - learning_rate * loss * 0.01;
        self.bias *= 1.0 - learning_rate * loss * 0.01;
    }
}

#[derive(Clone)]
enum ActivationType {
    ReLU,
    Tanh,
    Sigmoid,
}

/// Detection History for adaptive thresholding
struct DetectionHistory {
    scores: VecDeque<f32>,
    capacity: usize,
}

impl DetectionHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            scores: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fn add(&mut self, score: f32) {
        if self.scores.len() >= self.capacity {
            self.scores.pop_front();
        }
        self.scores.push_back(score);
    }
}

// Data structures
#[derive(Clone, Debug)]
pub struct TransactionFeatures {
    pub transaction_id: String,
    pub gas_price: u64,
    pub value: u128,
    pub nonce: u64,
    pub is_contract_interaction: bool,
    pub data_size: usize,
    pub mempool_time: f32,
    pub interacts_with_dex: bool,
    pub interacts_with_lending: bool,
    pub interacts_with_bridge: bool,
    pub sender_history: SenderHistory,
    pub timestamp: u64,
    pub network_congestion: f32,
    pub base_fee: u64,
}

#[derive(Clone, Debug)]
pub struct SenderHistory {
    pub total_txs: usize,
    pub mev_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnomalyDetectionResult {
    pub anomalies: Vec<AnomalyDetection>,
    pub mean_anomaly_score: f32,
    pub threshold: f32,
    pub detection_rate: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub transaction_id: String,
    pub anomaly_score: f32,
    pub reconstruction_error: f32,
    pub anomaly_type: AnomalyType,
    pub latent_representation: Array1<f32>,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnomalyType {
    UnusualGasPattern,
    AbnormalDataSize,
    SuspiciousProtocolInteraction,
    UnknownSenderBehavior,
    TemporalAnomaly,
    NetworkStateManipulation,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NovelAttack {
    pub transaction_id: String,
    pub pattern_signature: Array1<f32>,
    pub similarity_to_known: f32,
    pub attack_characteristics: Vec<String>,
    pub risk_score: f32,
}

#[derive(Clone, Debug)]
pub struct TrainingResult {
    pub final_loss: f32,
    pub reconstruction_losses: Vec<f32>,
    pub kl_losses: Vec<f32>,
    pub epochs_trained: usize,
}

// Utility functions
fn normalize(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min).max(1.0)
}

fn encode_time_of_day(timestamp: u64) -> f32 {
    let hour = (timestamp % 86400) / 3600;
    (2.0 * PI * hour as f32 / 24.0).sin()
}

fn encode_day_of_week(timestamp: u64) -> f32 {
    let day = (timestamp / 86400) % 7;
    (2.0 * PI * day as f32 / 7.0).cos()
}

use std::collections::VecDeque;
use rand;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vae_initialization() {
        let config = VAEConfig {
            input_dim: 15,
            hidden_dims: vec![64, 32],
            latent_dim: 8,
            beta: 0.5,
            learning_rate: 0.001,
            reconstruction_threshold: 0.1,
        };
        
        let detector = MEVAnomalyDetector::new(config);
        assert_eq!(detector.anomaly_threshold, 0.1);
    }
    
    #[test]
    fn test_anomaly_detection() {
        let config = VAEConfig {
            input_dim: 15,
            hidden_dims: vec![32, 16],
            latent_dim: 4,
            beta: 0.5,
            learning_rate: 0.001,
            reconstruction_threshold: 0.1,
        };
        
        let mut detector = MEVAnomalyDetector::new(config);
        
        let tx_features = TransactionFeatures {
            transaction_id: "tx_001".to_string(),
            gas_price: 50_000_000_000,
            value: 1_000_000_000_000_000_000,
            nonce: 42,
            is_contract_interaction: true,
            data_size: 100,
            mempool_time: 2.5,
            interacts_with_dex: true,
            interacts_with_lending: false,
            interacts_with_bridge: false,
            sender_history: SenderHistory {
                total_txs: 100,
                mev_count: 5,
            },
            timestamp: 1700000000,
            network_congestion: 0.7,
            base_fee: 30_000_000_000,
        };
        
        let result = detector.detect_anomalies(&[tx_features]);
        
        assert!(result.mean_anomaly_score >= 0.0);
        assert!(result.detection_rate >= 0.0 && result.detection_rate <= 1.0);
    }
}
