    pub fn forward(&self, x: &Array2<f32>) -> Array1<f32> {
        let logits = self.linear.dot(&x.row(0).to_owned());
        softmax(&logits)
    }
}

/// MEV Analysis Result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MEVAnalysisResult {
    pub mev_value_estimation: f32,
    pub pattern_probabilities: Array1<f32>,
    pub attention_weights: Vec<Array2<f32>>,
    pub risk_score: f32,
    pub recommended_actions: Vec<String>,
}

/// MEV Prediction for future blocks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MEVPrediction {
    pub estimated_value: f32,
    pub probability_distribution: Array1<f32>,
    pub confidence: f32,
}

// Utility functions
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

fn apply_dropout_3d(x: &Array3<f32>, rate: f32) -> Array3<f32> {
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

fn apply_mask(scores: &Array2<f32>, mask: &Array2<bool>) -> Array2<f32> {
    let mut masked = scores.clone();
    for i in 0..scores.nrows() {
        for j in 0..scores.ncols() {
            if !mask[[i, j]] {
                masked[[i, j]] = f32::NEG_INFINITY;
            }
        }
    }
    masked
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transformer_config() {
        let config = TransformerConfig {
            d_model: 512,
            n_heads: 8,
            n_layers: 6,
            d_ff: 2048,
            max_seq_length: 100,
            dropout: 0.1,
            vocab_size: 10000,
        };
        
        let analyzer = TransformerMEVAnalyzer::new(config);
        let transactions = Array3::zeros((1, 10, 512));
        
        let result = analyzer.analyze_sequence(&transactions, None);
        
        assert!(result.mev_value_estimation >= 0.0);
        assert_eq!(result.pattern_probabilities.len(), 7);
        assert!(result.risk_score >= 0.0 && result.risk_score <= 1.0);
    }
    
    #[test]
    fn test_positional_encoding() {
        let pos_enc = PositionalEncoding::new(128, 100);
        let x = Array3::zeros((1, 10, 128));
        
        let encoded = pos_enc.encode(&x);
        
        assert_eq!(encoded.shape(), x.shape());
        // Check that positional encoding was added
        assert_ne!(encoded.sum(), 0.0);
    }
    
    #[test]
    fn test_multi_head_attention() {
        let mha = MultiHeadAttention::new(256, 8);
        let x = Array3::random((1, 10, 256), Uniform::new(-1.0, 1.0));
        
        let output = mha.forward(&x, &x, &x, None);
        
        assert_eq!(output.shape(), x.shape());
    }
}
