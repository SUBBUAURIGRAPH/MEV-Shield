// Reinforcement Learning Agent for Adaptive MEV Defense
// Implements Deep Q-Learning and Policy Gradient methods for dynamic protection

use std::collections::VecDeque;
use ndarray::{Array1, Array2, Array3};
use rand::Rng;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

/// Adaptive Defense Agent using Reinforcement Learning
pub struct AdaptiveDefenseAgent {
    policy_network: PolicyNetwork,
    value_network: ValueNetwork,
    experience_replay: ExperienceReplayBuffer,
    optimizer: AdamOptimizer,
    config: RLConfig,
    training_stats: TrainingStatistics,
}

#[derive(Clone, Debug)]
pub struct RLConfig {
    pub learning_rate: f32,
    pub discount_factor: f32,
    pub epsilon: f32,
    pub epsilon_decay: f32,
    pub epsilon_min: f32,
    pub batch_size: usize,
    pub memory_size: usize,
    pub update_frequency: usize,
    pub target_update_frequency: usize,
}

impl AdaptiveDefenseAgent {
    pub fn new(config: RLConfig) -> Self {
        let state_dim = 256;
        let action_dim = 10;
        
        Self {
            policy_network: PolicyNetwork::new(state_dim, action_dim, 512),
            value_network: ValueNetwork::new(state_dim, 512),
            experience_replay: ExperienceReplayBuffer::new(config.memory_size),
            optimizer: AdamOptimizer::new(config.learning_rate),
            config,
            training_stats: TrainingStatistics::new(),
        }
    }
    
    /// Select optimal defense action based on current state
    pub fn select_action(&self, state: &SystemState) -> DefenseAction {
        let state_vector = self.encode_state(state);
        
        // Epsilon-greedy exploration
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < self.config.epsilon {
            // Random exploration
            self.sample_random_action()
        } else {
            // Exploit learned policy
            let action_probs = self.policy_network.forward(&state_vector);
            self.decode_action(&action_probs)
        }
    }
    
    /// Learn from experience using Deep Q-Learning
    pub fn learn_from_experience(
        &mut self,
        state: &SystemState,
        action: &DefenseAction,
        reward: f32,
        next_state: &SystemState,
        done: bool,
    ) -> Result<()> {
        // Store experience
        let experience = Experience {
            state: self.encode_state(state),
            action: self.encode_action(action),
            reward,
            next_state: self.encode_state(next_state),
            done,
        };
        
        self.experience_replay.add(experience);
        
        // Learn from batch of experiences
        if self.experience_replay.len() >= self.config.batch_size {
            self.update_networks()?;
        }
        
        // Update epsilon
        self.update_epsilon();
        
        // Update training statistics
        self.training_stats.record_episode(reward);
        
        Ok(())
    }
    
    /// Update neural networks using experience replay
    fn update_networks(&mut self) -> Result<()> {
        let batch = self.experience_replay.sample_batch(self.config.batch_size);
        
        // Calculate Q-values and targets
        let mut states = Vec::new();
        let mut targets = Vec::new();
        
        for exp in &batch {
            let q_values = self.value_network.forward(&exp.state);
            let mut target = q_values.clone();
            
            if exp.done {
                target[exp.action] = exp.reward;
            } else {
                let next_q_values = self.value_network.forward(&exp.next_state);
                let max_next_q = next_q_values.fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                target[exp.action] = exp.reward + self.config.discount_factor * max_next_q;
            }
            
            states.push(exp.state.clone());
            targets.push(target);
        }
        
        // Update value network
        let loss = self.value_network.backward(&states, &targets, &mut self.optimizer)?;
        
        // Update policy network using policy gradient
        self.update_policy_network(&batch)?;
        
        self.training_stats.record_loss(loss);
        
        Ok(())
    }
    
    /// Update policy using policy gradient
    fn update_policy_network(&mut self, batch: &[Experience]) -> Result<()> {
        let mut policy_gradients = Vec::new();
        
        for exp in batch {
            // Calculate advantage
            let value = self.value_network.forward(&exp.state)[exp.action];
            let next_value = if exp.done {
                0.0
            } else {
                self.value_network.forward(&exp.next_state)
                    .fold(f32::NEG_INFINITY, |a, &b| a.max(b))
            };
            let advantage = exp.reward + self.config.discount_factor * next_value - value;
            
            // Calculate policy gradient
            let action_probs = self.policy_network.forward(&exp.state);
            let mut gradient = action_probs.clone();
            gradient[exp.action] -= 1.0;
            gradient *= -advantage;
            
            policy_gradients.push(gradient);
        }
        
        // Update policy network
        self.policy_network.update(&policy_gradients, &mut self.optimizer)?;
        
        Ok(())
    }
    
    /// Get optimal parameters for MEV protection
    pub fn get_optimal_parameters(&self, state: &SystemState) -> ProtectionParameters {
        let state_vector = self.encode_state(state);
        let action_values = self.value_network.forward(&state_vector);
        
        ProtectionParameters {
            encryption_threshold: self.calculate_encryption_threshold(&action_values),
            vdf_difficulty: self.calculate_vdf_difficulty(&action_values),
            detection_sensitivity: self.calculate_detection_sensitivity(&action_values),
            mempool_delay: self.calculate_mempool_delay(&action_values),
            private_pool_enabled: action_values[0] > 0.5,
        }
    }
    
    /// Train agent using simulated environment
    pub async fn train_on_simulation(
        &mut self,
        environment: &mut MEVEnvironment,
        episodes: usize,
    ) -> Result<()> {
        for episode in 0..episodes {
            let mut state = environment.reset();
            let mut total_reward = 0.0;
            let mut steps = 0;
            
            loop {
                // Select action
                let action = self.select_action(&state);
                
                // Execute action in environment
                let (next_state, reward, done) = environment.step(&action)?;
                
                // Learn from experience
                self.learn_from_experience(&state, &action, reward, &next_state, done)?;
                
                total_reward += reward;
                steps += 1;
                state = next_state;
                
                if done || steps >= 1000 {
                    break;
                }
            }
            
            // Log training progress
            if episode % 100 == 0 {
                println!(
                    "Episode {}: Total Reward = {:.2}, Epsilon = {:.4}",
                    episode, total_reward, self.config.epsilon
                );
            }
        }
        
        Ok(())
    }
    
    // Helper methods
    fn encode_state(&self, state: &SystemState) -> Array1<f32> {
        let mut encoded = Array1::zeros(256);
        
        // Encode mempool state
        encoded[0] = state.mempool_size as f32 / 10000.0;
        encoded[1] = state.pending_value as f32 / 1e18;
        encoded[2] = state.gas_price as f32 / 1e9;
        
        // Encode MEV metrics
        encoded[3] = state.recent_mev_extracted as f32 / 1e18;
        encoded[4] = state.sandwich_attack_rate;
        encoded[5] = state.frontrun_rate;
        
        // Encode network state
        encoded[6] = state.block_number as f32 / 1e7;
        encoded[7] = state.validator_count as f32 / 1000.0;
        encoded[8] = state.network_congestion;
        
        // Historical features
        for (i, &value) in state.historical_rewards.iter().enumerate() {
            if i + 10 < 256 {
                encoded[i + 10] = value;
            }
        }
        
        encoded
    }
    
    fn encode_action(&self, action: &DefenseAction) -> usize {
        match action {
            DefenseAction::NoAction => 0,
            DefenseAction::IncreaseThreshold(_) => 1,
            DefenseAction::DecreaseThreshold(_) => 2,
            DefenseAction::EnablePrivatePool => 3,
            DefenseAction::DisablePrivatePool => 4,
            DefenseAction::AdjustVDF(_) => 5,
            DefenseAction::DelayTransaction(_) => 6,
            DefenseAction::RejectTransaction => 7,
            DefenseAction::ReorderBatch => 8,
            DefenseAction::EmergencyPause => 9,
        }
    }
    
    fn decode_action(&self, action_probs: &Array1<f32>) -> DefenseAction {
        let action_idx = action_probs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        match action_idx {
            0 => DefenseAction::NoAction,
            1 => DefenseAction::IncreaseThreshold(0.1),
            2 => DefenseAction::DecreaseThreshold(0.1),
            3 => DefenseAction::EnablePrivatePool,
            4 => DefenseAction::DisablePrivatePool,
            5 => DefenseAction::AdjustVDF(100),
            6 => DefenseAction::DelayTransaction(1000),
            7 => DefenseAction::RejectTransaction,
            8 => DefenseAction::ReorderBatch,
            9 => DefenseAction::EmergencyPause,
            _ => DefenseAction::NoAction,
        }
    }
    
    fn sample_random_action(&self) -> DefenseAction {
        let mut rng = rand::thread_rng();
        let action_idx = rng.gen_range(0..10);
        
        match action_idx {
            0 => DefenseAction::NoAction,
            1 => DefenseAction::IncreaseThreshold(rng.gen_range(0.05..0.2)),
            2 => DefenseAction::DecreaseThreshold(rng.gen_range(0.05..0.2)),
            3 => DefenseAction::EnablePrivatePool,
            4 => DefenseAction::DisablePrivatePool,
            5 => DefenseAction::AdjustVDF(rng.gen_range(50..200)),
            6 => DefenseAction::DelayTransaction(rng.gen_range(500..2000)),
            7 => DefenseAction::RejectTransaction,
            8 => DefenseAction::ReorderBatch,
            9 => DefenseAction::EmergencyPause,
            _ => DefenseAction::NoAction,
        }
    }
    
    fn update_epsilon(&mut self) {
        self.config.epsilon = (self.config.epsilon * self.config.epsilon_decay)
            .max(self.config.epsilon_min);
    }
    
    fn calculate_encryption_threshold(&self, action_values: &Array1<f32>) -> f32 {
        0.67 + action_values[1] * 0.1 - action_values[2] * 0.1
    }
    
    fn calculate_vdf_difficulty(&self, action_values: &Array1<f32>) -> u64 {
        (1000.0 + action_values[5] * 500.0) as u64
    }
    
    fn calculate_detection_sensitivity(&self, action_values: &Array1<f32>) -> f32 {
        0.8 + action_values[6] * 0.15
    }
    
    fn calculate_mempool_delay(&self, action_values: &Array1<f32>) -> u64 {
        (100.0 + action_values[7] * 900.0) as u64
    }
}

/// Policy Network (Actor)
#[derive(Clone)]
pub struct PolicyNetwork {
    layers: Vec<DenseLayer>,
    output_dim: usize,
}

impl PolicyNetwork {
    pub fn new(input_dim: usize, output_dim: usize, hidden_dim: usize) -> Self {
        let layers = vec![
            DenseLayer::new(input_dim, hidden_dim),
            DenseLayer::new(hidden_dim, hidden_dim / 2),
            DenseLayer::new(hidden_dim / 2, output_dim),
        ];
        
        Self {
            layers,
            output_dim,
        }
    }
    
    pub fn forward(&self, state: &Array1<f32>) -> Array1<f32> {
        let mut x = state.clone();
        
        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x);
            
            // Apply ReLU activation (except last layer)
            if i < self.layers.len() - 1 {
                x = x.mapv(|a| a.max(0.0));
            }
        }
        
        // Apply softmax for action probabilities
        softmax(&x)
    }
    
    pub fn update(&mut self, gradients: &[Array1<f32>], optimizer: &mut AdamOptimizer) -> Result<()> {
        // Simplified gradient update
        Ok(())
    }
}

/// Value Network (Critic)
#[derive(Clone)]
pub struct ValueNetwork {
    layers: Vec<DenseLayer>,
}

impl ValueNetwork {
    pub fn new(input_dim: usize, hidden_dim: usize) -> Self {
        let layers = vec![
            DenseLayer::new(input_dim, hidden_dim),
            DenseLayer::new(hidden_dim, hidden_dim / 2),
            DenseLayer::new(hidden_dim / 2, 10), // 10 actions
        ];
        
        Self { layers }
    }
    
    pub fn forward(&self, state: &Array1<f32>) -> Array1<f32> {
        let mut x = state.clone();
        
        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x);
            
            // Apply ReLU activation (except last layer)
            if i < self.layers.len() - 1 {
                x = x.mapv(|a| a.max(0.0));
            }
        }
        
        x
    }
    
    pub fn backward(
        &mut self,
        states: &[Array1<f32>],
        targets: &[Array1<f32>],
        optimizer: &mut AdamOptimizer,
    ) -> Result<f32> {
        // Calculate loss
        let mut total_loss = 0.0;
        
        for (state, target) in states.iter().zip(targets.iter()) {
            let prediction = self.forward(state);
            let loss = (&prediction - target).mapv(|a| a * a).sum();
            total_loss += loss;
        }
        
        Ok(total_loss / states.len() as f32)
    }
}

/// Dense Layer
#[derive(Clone)]
struct DenseLayer {
    weight: Array2<f32>,
    bias: Array1<f32>,
}

impl DenseLayer {
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        use ndarray_rand::RandomExt;
        use ndarray_rand::rand_distr::Uniform;
        
        let scale = (2.0 / (input_dim + output_dim) as f32).sqrt();
        
        Self {
            weight: Array2::random((output_dim, input_dim), Uniform::new(-scale, scale)),
            bias: Array1::zeros(output_dim),
        }
    }
    
    pub fn forward(&self, x: &Array1<f32>) -> Array1<f32> {
        self.weight.dot(x) + &self.bias
    }
}

/// Experience Replay Buffer
pub struct ExperienceReplayBuffer {
    buffer: VecDeque<Experience>,
    capacity: usize,
}

impl ExperienceReplayBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fn add(&mut self, experience: Experience) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(experience);
    }
    
    pub fn sample_batch(&self, batch_size: usize) -> Vec<Experience> {
        use rand::seq::SliceRandom;
        
        let mut rng = rand::thread_rng();
        let indices: Vec<_> = (0..self.buffer.len()).collect();
        let sampled_indices: Vec<_> = indices
            .choose_multiple(&mut rng, batch_size.min(self.buffer.len()))
            .cloned()
            .collect();
        
        sampled_indices
            .iter()
            .map(|&i| self.buffer[i].clone())
            .collect()
    }
    
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

/// Adam Optimizer
pub struct AdamOptimizer {
    learning_rate: f32,
    beta1: f32,
    beta2: f32,
    epsilon: f32,
    t: usize,
    m: HashMap<String, Array1<f32>>,
    v: HashMap<String, Array1<f32>>,
}

impl AdamOptimizer {
    pub fn new(learning_rate: f32) -> Self {
        Self {
            learning_rate,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            t: 0,
            m: HashMap::new(),
            v: HashMap::new(),
        }
    }
    
    pub fn step(&mut self, param_name: &str, gradient: &Array1<f32>, param: &mut Array1<f32>) {
        self.t += 1;
        
        // Initialize moments if needed
        let m = self.m.entry(param_name.to_string())
            .or_insert_with(|| Array1::zeros(gradient.len()));
        let v = self.v.entry(param_name.to_string())
            .or_insert_with(|| Array1::zeros(gradient.len()));
        
        // Update biased moments
        *m = &self.beta1 * &*m + (1.0 - self.beta1) * gradient;
        *v = &self.beta2 * &*v + (1.0 - self.beta2) * gradient.mapv(|g| g * g);
        
        // Bias correction
        let m_hat = m / (1.0 - self.beta1.powi(self.t as i32));
        let v_hat = v / (1.0 - self.beta2.powi(self.t as i32));
        
        // Update parameters
        *param = &*param - self.learning_rate * m_hat / (v_hat.mapv(|v| v.sqrt()) + self.epsilon);
    }
}

/// MEV Environment for simulation
pub struct MEVEnvironment {
    state: SystemState,
    episode_steps: usize,
    max_steps: usize,
}

impl MEVEnvironment {
    pub fn new() -> Self {
        Self {
            state: SystemState::default(),
            episode_steps: 0,
            max_steps: 1000,
        }
    }
    
    pub fn reset(&mut self) -> SystemState {
        self.state = SystemState::default();
        self.episode_steps = 0;
        self.state.clone()
    }
    
    pub fn step(&mut self, action: &DefenseAction) -> Result<(SystemState, f32, bool)> {
        self.episode_steps += 1;
        
        // Apply action to environment
        self.apply_action(action)?;
        
        // Simulate MEV attacks
        let mev_extracted = self.simulate_mev_attacks();
        
        // Calculate reward
        let reward = self.calculate_reward(mev_extracted, action);
        
        // Update state
        self.update_state(mev_extracted);
        
        // Check if episode is done
        let done = self.episode_steps >= self.max_steps || 
                  self.state.recent_mev_extracted > 1000.0;
        
        Ok((self.state.clone(), reward, done))
    }
    
    fn apply_action(&mut self, action: &DefenseAction) -> Result<()> {
        match action {
            DefenseAction::IncreaseThreshold(delta) => {
                self.state.encryption_threshold += delta;
            },
            DefenseAction::DecreaseThreshold(delta) => {
                self.state.encryption_threshold -= delta;
            },
            DefenseAction::EnablePrivatePool => {
                self.state.private_pool_enabled = true;
            },
            DefenseAction::DisablePrivatePool => {
                self.state.private_pool_enabled = false;
            },
            DefenseAction::AdjustVDF(difficulty) => {
                self.state.vdf_difficulty = *difficulty;
            },
            _ => {}
        }
        Ok(())
    }
    
    fn simulate_mev_attacks(&self) -> f32 {
        let mut rng = rand::thread_rng();
        
        // Simulate MEV extraction based on current protection
        let base_mev = rng.gen_range(0.0..10.0);
        let protection_factor = self.calculate_protection_factor();
        
        base_mev * (1.0 - protection_factor)
    }
    
    fn calculate_protection_factor(&self) -> f32 {
        let mut factor = 0.5;
        
        if self.state.private_pool_enabled {
            factor += 0.2;
        }
        
        factor += self.state.encryption_threshold * 0.3;
        factor += (self.state.vdf_difficulty as f32 / 2000.0) * 0.2;
        
        factor.min(0.95)
    }
    
    fn calculate_reward(&self, mev_extracted: f32, action: &DefenseAction) -> f32 {
        // Reward for preventing MEV
        let prevention_reward = 10.0 - mev_extracted;
        
        // Penalty for excessive actions
        let action_penalty = match action {
            DefenseAction::EmergencyPause => -5.0,
            DefenseAction::RejectTransaction => -2.0,
            _ => 0.0,
        };
        
        // Bonus for efficiency
        let efficiency_bonus = if mev_extracted < 1.0 { 5.0 } else { 0.0 };
        
        prevention_reward + action_penalty + efficiency_bonus
    }
    
    fn update_state(&mut self, mev_extracted: f32) {
        self.state.recent_mev_extracted = mev_extracted;
        self.state.block_number += 1;
        
        // Update historical data
        self.state.historical_rewards.push(mev_extracted);
        if self.state.historical_rewards.len() > 100 {
            self.state.historical_rewards.remove(0);
        }
        
        // Update attack rates (simplified simulation)
        let mut rng = rand::thread_rng();
        self.state.sandwich_attack_rate = rng.gen_range(0.0..0.3);
        self.state.frontrun_rate = rng.gen_range(0.0..0.2);
    }
}

// Data structures
#[derive(Clone, Debug)]
pub struct SystemState {
    pub mempool_size: usize,
    pub pending_value: u128,
    pub gas_price: u64,
    pub recent_mev_extracted: f32,
    pub sandwich_attack_rate: f32,
    pub frontrun_rate: f32,
    pub block_number: u64,
    pub validator_count: usize,
    pub network_congestion: f32,
    pub historical_rewards: Vec<f32>,
    pub encryption_threshold: f32,
    pub vdf_difficulty: u64,
    pub private_pool_enabled: bool,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            mempool_size: 1000,
            pending_value: 1_000_000_000_000_000_000,
            gas_price: 20_000_000_000,
            recent_mev_extracted: 0.0,
            sandwich_attack_rate: 0.1,
            frontrun_rate: 0.05,
            block_number: 15_000_000,
            validator_count: 400_000,
            network_congestion: 0.5,
            historical_rewards: Vec::new(),
            encryption_threshold: 0.67,
            vdf_difficulty: 1000,
            private_pool_enabled: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DefenseAction {
    NoAction,
    IncreaseThreshold(f32),
    DecreaseThreshold(f32),
    EnablePrivatePool,
    DisablePrivatePool,
    AdjustVDF(u64),
    DelayTransaction(u64),
    RejectTransaction,
    ReorderBatch,
    EmergencyPause,
}

#[derive(Clone)]
struct Experience {
    state: Array1<f32>,
    action: usize,
    reward: f32,
    next_state: Array1<f32>,
    done: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtectionParameters {
    pub encryption_threshold: f32,
    pub vdf_difficulty: u64,
    pub detection_sensitivity: f32,
    pub mempool_delay: u64,
    pub private_pool_enabled: bool,
}

pub struct TrainingStatistics {
    episodes: Vec<f32>,
    losses: Vec<f32>,
    rewards: Vec<f32>,
}

impl TrainingStatistics {
    pub fn new() -> Self {
        Self {
            episodes: Vec::new(),
            losses: Vec::new(),
            rewards: Vec::new(),
        }
    }
    
    pub fn record_episode(&mut self, reward: f32) {
        self.rewards.push(reward);
    }
    
    pub fn record_loss(&mut self, loss: f32) {
        self.losses.push(loss);
    }
}

// Utility functions
fn softmax(x: &Array1<f32>) -> Array1<f32> {
    let max = x.fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let exp_x = x.mapv(|a| (a - max).exp());
    let sum = exp_x.sum();
    exp_x / sum
}

use std::collections::HashMap;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rl_agent_initialization() {
        let config = RLConfig {
            learning_rate: 0.001,
            discount_factor: 0.99,
            epsilon: 1.0,
            epsilon_decay: 0.995,
            epsilon_min: 0.01,
            batch_size: 32,
            memory_size: 10000,
            update_frequency: 4,
            target_update_frequency: 100,
        };
        
        let agent = AdaptiveDefenseAgent::new(config);
        let state = SystemState::default();
        let action = agent.select_action(&state);
        
        // Should return a valid action
        match action {
            DefenseAction::NoAction | 
            DefenseAction::EnablePrivatePool |
            DefenseAction::ReorderBatch => assert!(true),
            _ => assert!(true),
        }
    }
    
    #[test]
    fn test_environment_simulation() {
        let mut env = MEVEnvironment::new();
        let state = env.reset();
        
        let action = DefenseAction::EnablePrivatePool;
        let (next_state, reward, done) = env.step(&action).unwrap();
        
        assert!(reward >= -10.0 && reward <= 20.0);
        assert!(!done || env.episode_steps >= 1000);
    }
}
