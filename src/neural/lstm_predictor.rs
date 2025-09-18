// LSTM-based MEV Attack Predictor
// Implements deep LSTM networks with attention mechanism for predicting MEV attacks

use std::collections::VecDeque;
use ndarray::{Array1, Array2, Array3, Axis, s};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

/// LSTM Cell Implementation
#[derive(Clone, Debug)]
pub struct LSTMCell {
    // Weight matrices
    weight_ih: Array2<f32>,  // Input-to-hidden weights
    weight_hh: Array2<f32>,  // Hidden-to-hidden weights
    bias_ih: Array1<f32>,    // Input-to-hidden bias
    bias_hh: Array1<f32>,    // Hidden-to-hidden bias
    
    // Cell dimensions
    input_size: usize,
    hidden_size: usize,
}

impl LSTMCell {
    pub fn new(input_size: usize, hidden_size: usize) -> Self {
        use ndarray_rand::RandomExt;
        use ndarray_rand::rand_distr::Uniform;
        
        let xavier_scale = (2.0 / (input_size + hidden_size) as f32).sqrt();
        
        Self {
            weight_ih: Array2::random((4 * hidden_size, input_size), Uniform::new(-xavier_scale, xavier_scale)),
            weight_hh: Array2::random((4 * hidden_size, hidden_size), Uniform::new(-xavier_scale, xavier_scale)),
            bias_ih: Array1::zeros(4 * hidden_size),
            bias_hh: Array1::zeros(4 * hidden_size),
            input_size,
            hidden_size,
        }
    }
    
    /// Forward pass through LSTM cell
    pub fn forward(
        &self,
        input: &Array1<f32>,
        hidden: &Array1<f32>,
        cell: &Array1<f32>,
    ) -> (Array1<f32>, Array1<f32>) {
        // Compute gates
        let gates = self.weight_ih.dot(input) + &self.bias_ih + 
                   self.weight_hh.dot(hidden) + &self.bias_hh;
        
        let hidden_size = self.hidden_size;
        
        // Split gates
        let input_gate = sigmoid(&gates.slice(s![0..hidden_size]).to_owned());
        let forget_gate = sigmoid(&gates.slice(s![hidden_size..2*hidden_size]).to_owned());
        let cell_gate = tanh(&gates.slice(s![2*hidden_size..3*hidden_size]).to_owned());
        let output_gate = sigmoid(&gates.slice(s![3*hidden_size..4*hidden_size]).to_owned());
        
        // Update cell state
        let new_cell = &forget_gate * cell + &input_gate * &cell_gate;
        
        // Update hidden state
        let new_hidden = &output_gate * tanh(&new_cell);
        
        (new_hidden, new_cell)
    }
}

/// Multi-layer LSTM with Attention
#[derive(Clone)]
pub struct AttentionLSTM {
    layers: Vec<LSTMCell>,
    attention: MultiHeadSelfAttention,
    dropout_rate: f32,
    output_projection: Array2<f32>,
}

impl AttentionLSTM {
    pub fn new(
        input_size: usize,
        hidden_size: usize,
        num_layers: usize,
        num_heads: usize,
        dropout_rate: f32,
    ) -> Self {
        let mut layers = Vec::new();
        
        // First layer
        layers.push(LSTMCell::new(input_size, hidden_size));
        
        // Hidden layers
        for _ in 1..num_layers {
            layers.push(LSTMCell::new(hidden_size, hidden_size));
        }
        
        // Attention mechanism
        let attention = MultiHeadSelfAttention::new(hidden_size, num_heads);
        
        // Output projection
        use ndarray_rand::RandomExt;
        use ndarray_rand::rand_distr::Uniform;
        let output_projection = Array2::random((5, hidden_size), Uniform::new(-0.1, 0.1));
        
        Self {
            layers,
            attention,
            dropout_rate,
            output_projection,
        }
    }
    
    /// Process sequence of transactions
    pub fn forward(&self, sequence: &Array3<f32>) -> MEVPredictionOutput {
        let (batch_size, seq_len, input_size) = sequence.dim();
        let hidden_size = self.layers[0].hidden_size;
        
        // Initialize hidden and cell states for all layers
        let mut hidden_states: Vec<Array1<f32>> = self.layers
            .iter()
            .map(|_| Array1::zeros(hidden_size))
            .collect();
        
        let mut cell_states: Vec<Array1<f32>> = self.layers
            .iter()
            .map(|_| Array1::zeros(hidden_size))
            .collect();
        
        let mut outputs = Vec::new();
        
        // Process sequence through LSTM layers
        for t in 0..seq_len {
            let mut input = sequence.slice(s![0, t, ..]).to_owned();
            
            // Pass through each LSTM layer
            for (layer_idx, layer) in self.layers.iter().enumerate() {
                let (new_hidden, new_cell) = layer.forward(
                    &input,
                    &hidden_states[layer_idx],
                    &cell_states[layer_idx],
                );
                
                hidden_states[layer_idx] = new_hidden.clone();
                cell_states[layer_idx] = new_cell;
                input = new_hidden;
            }
            
            // Apply dropout during training
            if self.dropout_rate > 0.0 {
                input = apply_dropout(input, self.dropout_rate);
            }
            
            outputs.push(input);
        }
        
        // Stack outputs for attention
        let output_matrix = stack_arrays(&outputs);
        
        // Apply attention mechanism
        let attended_output = self.attention.forward(&output_matrix);
        
        // Final projection to prediction classes
        let prediction = self.output_projection.dot(&attended_output);
        let probabilities = softmax(&prediction);
        
        MEVPredictionOutput {
            attack_probabilities: probabilities.clone(),
            attack_type: self.get_attack_type(&probabilities),
            confidence: self.calculate_confidence(&probabilities),
            attention_weights: self.attention.get_attention_weights(),
            hidden_states: outputs,
        }
    }
    
    fn get_attack_type(&self, probabilities: &Array1<f32>) -> MEVAttackType {
        let max_idx = probabilities
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        
        match max_idx {
            0 => MEVAttackType::NoAttack,
            1 => MEVAttackType::SandwichAttack,
            2 => MEVAttackType::FrontRunning,
            3 => MEVAttackType::Arbitrage,
            4 => MEVAttackType::Liquidation,
            _ => MEVAttackType::Unknown,
        }
    }
    
    fn calculate_confidence(&self, probabilities: &Array1<f32>) -> f32 {
        probabilities.iter().fold(0.0_f32, |a, b| a.max(*b))
    }
}

/// Multi-Head Self-Attention Mechanism
#[derive(Clone)]
pub struct MultiHeadSelfAttention {
    num_heads: usize,
    hidden_size: usize,
    head_dim: usize,
    
    query_projection: Array2<f32>,
    key_projection: Array2<f32>,
    value_projection: Array2<f32>,
    output_projection: Array2<f32>,
    
    attention_weights: Option<Array2<f32>>,
}

impl MultiHeadSelfAttention {
    pub fn new(hidden_size: usize, num_heads: usize) -> Self {
        use ndarray_rand::RandomExt;
        use ndarray_rand::rand_distr::Uniform;
        
        let head_dim = hidden_size / num_heads;
        let scale = (1.0 / hidden_size as f32).sqrt();
        
        Self {
            num_heads,
            hidden_size,
            head_dim,
            query_projection: Array2::random((hidden_size, hidden_size), Uniform::new(-scale, scale)),
            key_projection: Array2::random((hidden_size, hidden_size), Uniform::new(-scale, scale)),
            value_projection: Array2::random((hidden_size, hidden_size), Uniform::new(-scale, scale)),
            output_projection: Array2::random((hidden_size, hidden_size), Uniform::new(-scale, scale)),
            attention_weights: None,
        }
    }
    
    pub fn forward(&self, x: &Array2<f32>) -> Array1<f32> {
        let seq_len = x.nrows();
        
        // Compute Q, K, V
        let queries = x.dot(&self.query_projection);
        let keys = x.dot(&self.key_projection);
        let values = x.dot(&self.value_projection);
        
        // Reshape for multi-head attention
        let q_heads = self.reshape_for_heads(&queries);
        let k_heads = self.reshape_for_heads(&keys);
        let v_heads = self.reshape_for_heads(&values);
        
        // Scaled dot-product attention
        let scale = (self.head_dim as f32).sqrt();
        let scores = q_heads.dot(&k_heads.t()) / scale;
        
        // Apply softmax
        let attention_weights = softmax_2d(&scores);
        
        // Apply attention to values
        let attended = attention_weights.dot(&v_heads);
        
        // Reshape and project back
        let concatenated = self.reshape_from_heads(&attended);
        let output = concatenated.dot(&self.output_projection);
        
        // Return the last timestep (or pool)
        output.row(seq_len - 1).to_owned()
    }
    
    fn reshape_for_heads(&self, x: &Array2<f32>) -> Array2<f32> {
        // Simplified reshaping for multi-head attention
        x.clone()
    }
    
    fn reshape_from_heads(&self, x: &Array2<f32>) -> Array2<f32> {
        x.clone()
    }
    
    pub fn get_attention_weights(&self) -> Option<Array2<f32>> {
        self.attention_weights.clone()
    }
}

/// Time Series Feature Extractor
pub struct TimeSeriesFeatureExtractor {
    window_size: usize,
    history: VecDeque<TransactionFeatures>,
    feature_dim: usize,
}

impl TimeSeriesFeatureExtractor {
    pub fn new(window_size: usize, feature_dim: usize) -> Self {
        Self {
            window_size,
            history: VecDeque::with_capacity(window_size),
            feature_dim,
        }
    }
    
    pub fn extract_features(&mut self, transaction: &Transaction) -> Array1<f32> {
        let mut features = Array1::zeros(self.feature_dim);
        
        // Gas price features
        features[0] = normalize_gas_price(transaction.gas_price);
        features[1] = normalize_gas_price(transaction.max_priority_fee);
        
        // Value features
        features[2] = normalize_value(transaction.value);
        features[3] = (transaction.value > 1_000_000_000_000_000_000) as i32 as f32; // > 1 ETH
        
        // Time features
        features[4] = extract_time_feature(transaction.timestamp);
        features[5] = extract_block_feature(transaction.block_number);
        
        // Transaction type features
        features[6] = is_dex_transaction(transaction) as i32 as f32;
        features[7] = is_token_transfer(transaction) as i32 as f32;
        features[8] = is_contract_creation(transaction) as i32 as f32;
        
        // Network features
        features[9] = transaction.gas_used as f32 / transaction.gas_limit as f32;
        
        // Historical features
        if self.history.len() > 0 {
            features[10] = self.calculate_velocity();
            features[11] = self.calculate_acceleration();
            features[12] = self.detect_pattern_similarity();
        }
        
        // Update history
        self.history.push_back(TransactionFeatures {
            features: features.clone(),
            timestamp: transaction.timestamp,
        });
        
        if self.history.len() > self.window_size {
            self.history.pop_front();
        }
        
        features
    }
    
    fn calculate_velocity(&self) -> f32 {
        if self.history.len() < 2 {
            return 0.0;
        }
        
        let recent = &self.history[self.history.len() - 1];
        let previous = &self.history[self.history.len() - 2];
        
        let time_diff = (recent.timestamp - previous.timestamp) as f32;
        if time_diff > 0.0 {
            (recent.features[2] - previous.features[2]) / time_diff
        } else {
            0.0
        }
    }
    
    fn calculate_acceleration(&self) -> f32 {
        if self.history.len() < 3 {
            return 0.0;
        }
        
        // Calculate change in velocity
        0.0 // Simplified
    }
    
    fn detect_pattern_similarity(&self) -> f32 {
        // Detect if current pattern matches historical MEV attacks
        0.0 // Simplified
    }
}

/// Output structure for MEV predictions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MEVPredictionOutput {
    pub attack_probabilities: Array1<f32>,
    pub attack_type: MEVAttackType,
    pub confidence: f32,
    pub attention_weights: Option<Array2<f32>>,
    pub hidden_states: Vec<Array1<f32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MEVAttackType {
    NoAttack,
    SandwichAttack,
    FrontRunning,
    Arbitrage,
    Liquidation,
    Unknown,
}

#[derive(Clone)]
struct TransactionFeatures {
    features: Array1<f32>,
    timestamp: u64,
}

// Utility structures
#[derive(Clone, Debug)]
pub struct Transaction {
    pub gas_price: u64,
    pub max_priority_fee: u64,
    pub value: u128,
    pub timestamp: u64,
    pub block_number: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub to: String,
    pub data: Vec<u8>,
}

// Activation functions
fn sigmoid(x: &Array1<f32>) -> Array1<f32> {
    x.mapv(|a| 1.0 / (1.0 + (-a).exp()))
}

fn tanh(x: &Array1<f32>) -> Array1<f32> {
    x.mapv(|a| a.tanh())
}

fn softmax(x: &Array1<f32>) -> Array1<f32> {
    let max = x.fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let exp_x = x.mapv(|a| (a - max).exp());
    let sum = exp_x.sum();
    exp_x / sum
}

fn softmax_2d(x: &Array2<f32>) -> Array2<f32> {
    let mut result = x.clone();
    for i in 0..x.nrows() {
        let row = x.row(i);
        let max = row.fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let exp_row = row.mapv(|a| (a - max).exp());
        let sum = exp_row.sum();
        let softmax_row = exp_row / sum;
        result.row_mut(i).assign(&softmax_row);
    }
    result
}

fn apply_dropout(x: Array1<f32>, rate: f32) -> Array1<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    x.mapv(|a| {
        if rng.gen::<f32>() > rate {
            a / (1.0 - rate)
        } else {
            0.0
        }
    })
}

fn stack_arrays(arrays: &[Array1<f32>]) -> Array2<f32> {
    let rows = arrays.len();
    let cols = arrays[0].len();
    let mut result = Array2::zeros((rows, cols));
    
    for (i, arr) in arrays.iter().enumerate() {
        result.row_mut(i).assign(arr);
    }
    
    result
}

// Feature normalization helpers
fn normalize_gas_price(price: u64) -> f32 {
    (price as f32 / 1_000_000_000.0).min(1000.0) / 1000.0
}

fn normalize_value(value: u128) -> f32 {
    (value as f32 / 1e18).ln_1p() / 10.0
}

fn extract_time_feature(timestamp: u64) -> f32 {
    ((timestamp % 3600) as f32) / 3600.0
}

fn extract_block_feature(block_number: u64) -> f32 {
    ((block_number % 100) as f32) / 100.0
}

fn is_dex_transaction(tx: &Transaction) -> bool {
    // Check if transaction interacts with known DEX contracts
    tx.data.len() > 4 && (tx.data[0..4] == [0xa9, 0x05, 0x9c, 0xbb] || // transferFrom
                          tx.data[0..4] == [0x38, 0xed, 0x17, 0x39])   // swapExactTokensForTokens
}

fn is_token_transfer(tx: &Transaction) -> bool {
    tx.data.len() == 68 && tx.data[0..4] == [0xa9, 0x05, 0x9c, 0xbb]
}

fn is_contract_creation(tx: &Transaction) -> bool {
    tx.to.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lstm_cell_forward() {
        let cell = LSTMCell::new(10, 20);
        let input = Array1::zeros(10);
        let hidden = Array1::zeros(20);
        let cell_state = Array1::zeros(20);
        
        let (new_hidden, new_cell) = cell.forward(&input, &hidden, &cell_state);
        
        assert_eq!(new_hidden.len(), 20);
        assert_eq!(new_cell.len(), 20);
    }
    
    #[test]
    fn test_attention_lstm() {
        let lstm = AttentionLSTM::new(10, 20, 2, 4, 0.1);
        let sequence = Array3::zeros((1, 5, 10));
        
        let output = lstm.forward(&sequence);
        
        assert_eq!(output.attack_probabilities.len(), 5);
        assert!(output.confidence >= 0.0 && output.confidence <= 1.0);
    }
}
