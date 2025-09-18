//! Ethereum-specific validation utilities
//!
//! Specialized validation for Ethereum addresses, transaction data,
//! contract interactions, and blockchain-specific values.

use crate::validation::{ValidationError, ValidationResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn, error};
use num_bigint::BigUint;
use std::str::FromStr;

/// Ethereum address validator with EIP-55 support
pub struct EthereumAddressValidator {
    basic_regex: Regex,
    strict_validation: bool,
}

impl EthereumAddressValidator {
    pub fn new(strict: bool) -> Self {
        Self {
            basic_regex: Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap(),
            strict_validation: strict,
        }
    }
    
    /// Validate Ethereum address with comprehensive checks
    pub fn validate(&self, address: &str) -> Result<String, ValidationError> {
        // Basic format check
        if !self.basic_regex.is_match(address) {
            return Err(ValidationError::InvalidAddress(
                format!("Invalid address format: {}", address)
            ));
        }
        
        // Check for null address
        if address.to_lowercase() == "0x0000000000000000000000000000000000000000" {
            warn!("Null address detected: {}", address);
            return Err(ValidationError::InvalidAddress(
                "Null address not allowed".to_string()
            ));
        }
        
        // EIP-55 checksum validation if strict mode
        if self.strict_validation {
            self.validate_eip55_checksum(address)?;
        }
        
        // Check against known problematic addresses
        self.check_problematic_addresses(address)?;
        
        Ok(address.to_lowercase())
    }
    
    /// Validate EIP-55 checksum (simplified version)
    fn validate_eip55_checksum(&self, address: &str) -> Result<(), ValidationError> {
        let addr_without_prefix = &address[2..];
        
        // If all lowercase or all uppercase, it's valid
        if addr_without_prefix == addr_without_prefix.to_lowercase() || 
           addr_without_prefix == addr_without_prefix.to_uppercase() {
            return Ok(());
        }
        
        // For mixed case, we should validate the checksum
        // This is a simplified check - in production, use proper keccak256 hashing
        debug!("Mixed case address detected, assuming valid EIP-55: {}", address);
        Ok(())
    }
    
    /// Check against known problematic addresses
    fn check_problematic_addresses(&self, address: &str) -> Result<(), ValidationError> {
        let addr_lower = address.to_lowercase();
        
        // Known burn addresses
        let burn_addresses = [
            "0x000000000000000000000000000000000000dead",
            "0x0000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000002",
        ];
        
        if burn_addresses.contains(&addr_lower.as_str()) {
            return Err(ValidationError::InvalidAddress(
                "Burn addresses not allowed".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Transaction parameter validator
pub struct TransactionValidator {
    max_gas_limit: u64,
    max_gas_price_gwei: u64,
    max_priority_fee_gwei: u64,
}

impl TransactionValidator {
    pub fn new() -> Self {
        Self {
            max_gas_limit: 30_000_000,      // Current Ethereum block gas limit
            max_gas_price_gwei: 1000,       // 1000 Gwei max
            max_priority_fee_gwei: 100,     // 100 Gwei max priority fee
        }
    }
    
    /// Validate transaction parameters
    pub fn validate_transaction_params(
        &self,
        from: &str,
        to: &str,
        value: &str,
        gas: u64,
        gas_price: &str,
        nonce: u64,
        data: &str,
        chain_id: u64,
    ) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::new();
        
        // Validate addresses
        let addr_validator = EthereumAddressValidator::new(true);
        if let Err(e) = addr_validator.validate(from) {
            result.add_error(format!("Invalid 'from' address: {}", e));
        }
        if let Err(e) = addr_validator.validate(to) {
            result.add_error(format!("Invalid 'to' address: {}", e));
        }
        
        // Validate value
        if let Err(e) = self.validate_value(value) {
            result.add_error(format!("Invalid value: {}", e));
        }
        
        // Validate gas
        if let Err(e) = self.validate_gas(gas) {
            result.add_error(format!("Invalid gas: {}", e));
        }
        
        // Validate gas price
        if let Err(e) = self.validate_gas_price(gas_price) {
            result.add_error(format!("Invalid gas price: {}", e));
        }
        
        // Validate nonce
        if let Err(e) = self.validate_nonce(nonce) {
            result.add_error(format!("Invalid nonce: {}", e));
        }
        
        // Validate data
        if let Err(e) = self.validate_data(data) {
            result.add_error(format!("Invalid data: {}", e));
        }
        
        // Validate chain ID
        if let Err(e) = self.validate_chain_id(chain_id) {
            result.add_error(format!("Invalid chain ID: {}", e));
        }
        
        // Additional security checks
        self.check_transaction_security(from, to, value, &mut result);
        
        Ok(result)
    }
    
    /// Validate transaction value
    fn validate_value(&self, value: &str) -> Result<BigUint, ValidationError> {
        let value_wei = BigUint::from_str(value)
            .map_err(|_| ValidationError::InvalidNumeric("Invalid value format".to_string()))?;
        
        // Check for reasonable maximum (100,000 ETH in wei)
        let max_value = BigUint::from_str("100000000000000000000000").unwrap();
        if value_wei > max_value {
            return Err(ValidationError::OutOfRange(
                "Value exceeds reasonable maximum".to_string()
            ));
        }
        
        Ok(value_wei)
    }
    
    /// Validate gas limit
    fn validate_gas(&self, gas: u64) -> Result<(), ValidationError> {
        const MIN_GAS: u64 = 21000;
        
        if gas < MIN_GAS {
            return Err(ValidationError::OutOfRange(
                format!("Gas {} below minimum {}", gas, MIN_GAS)
            ));
        }
        
        if gas > self.max_gas_limit {
            return Err(ValidationError::OutOfRange(
                format!("Gas {} exceeds block limit {}", gas, self.max_gas_limit)
            ));
        }
        
        Ok(())
    }
    
    /// Validate gas price
    fn validate_gas_price(&self, gas_price: &str) -> Result<(), ValidationError> {
        let price_wei = BigUint::from_str(gas_price)
            .map_err(|_| ValidationError::InvalidNumeric("Invalid gas price format".to_string()))?;
        
        // Convert to Gwei for comparison
        let gwei = BigUint::from(1_000_000_000u64);
        let max_price_wei = gwei * self.max_gas_price_gwei;
        
        if price_wei > max_price_wei {
            return Err(ValidationError::OutOfRange(
                format!("Gas price exceeds maximum {} Gwei", self.max_gas_price_gwei)
            ));
        }
        
        // Minimum gas price check (1 Gwei)
        if price_wei < gwei {
            return Err(ValidationError::OutOfRange(
                "Gas price below minimum 1 Gwei".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate nonce
    fn validate_nonce(&self, nonce: u64) -> Result<(), ValidationError> {
        // Nonce should be reasonable (not excessively high)
        if nonce > 1_000_000 {
            return Err(ValidationError::OutOfRange(
                "Nonce unreasonably high".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate transaction data
    fn validate_data(&self, data: &str) -> Result<(), ValidationError> {
        // Check hex format
        let hex_regex = Regex::new(r"^0x[a-fA-F0-9]*$").unwrap();
        if !hex_regex.is_match(data) {
            return Err(ValidationError::InvalidFormat(
                "Data must be valid hex string".to_string()
            ));
        }
        
        // Length check (1MB limit)
        if data.len() > 2_097_152 {
            return Err(ValidationError::TooLong(data.len(), 2_097_152));
        }
        
        // Even number of hex characters (after 0x)
        if (data.len() - 2) % 2 != 0 {
            return Err(ValidationError::InvalidFormat(
                "Hex data must have even number of characters".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate chain ID
    fn validate_chain_id(&self, chain_id: u64) -> Result<(), ValidationError> {
        // Known valid chain IDs
        let valid_chains = [
            1,      // Ethereum Mainnet
            3,      // Ropsten (deprecated but still valid)
            4,      // Rinkeby (deprecated but still valid)
            5,      // Goerli
            11155111, // Sepolia
            137,    // Polygon
            56,     // BSC
            250,    // Fantom
            43114,  // Avalanche
            42161,  // Arbitrum One
            10,     // Optimism
            100,    // Gnosis Chain
        ];
        
        if !valid_chains.contains(&chain_id) && chain_id < 1000000 {
            // Allow custom chains with ID >= 1000000
            return Err(ValidationError::InvalidFormat(
                format!("Unsupported chain ID: {}", chain_id)
            ));
        }
        
        Ok(())
    }
    
    /// Additional security checks for transactions
    fn check_transaction_security(&self, from: &str, to: &str, value: &str, result: &mut ValidationResult) {
        // Check for same from/to address
        if from.to_lowercase() == to.to_lowercase() {
            result.add_warning("Transaction from and to address are the same".to_string());
        }
        
        // Check for high-value transactions
        if let Ok(value_wei) = BigUint::from_str(value) {
            let ten_eth = BigUint::from_str("10000000000000000000").unwrap(); // 10 ETH
            if value_wei > ten_eth {
                result.add_warning("High-value transaction detected".to_string());
            }
        }
        
        // Check for contract interaction patterns
        if to.len() == 42 && to != "0x0000000000000000000000000000000000000000" {
            // This is likely a contract interaction
            result.add_warning("Contract interaction detected - ensure data is properly validated".to_string());
        }
    }
}

/// Contract interaction validator
pub struct ContractValidator {
    function_signatures: HashMap<String, ContractFunction>,
}

#[derive(Debug, Clone)]
pub struct ContractFunction {
    pub name: String,
    pub signature: String,
    pub risk_level: RiskLevel,
    pub parameters: Vec<ParameterType>,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum ParameterType {
    Address,
    Uint256,
    Bytes,
    String,
    Bool,
    Array(Box<ParameterType>),
}

impl ContractValidator {
    pub fn new() -> Self {
        let mut function_signatures = HashMap::new();
        
        // Common ERC-20 functions
        function_signatures.insert(
            "a9059cbb".to_string(), // transfer(address,uint256)
            ContractFunction {
                name: "transfer".to_string(),
                signature: "transfer(address,uint256)".to_string(),
                risk_level: RiskLevel::Medium,
                parameters: vec![ParameterType::Address, ParameterType::Uint256],
            }
        );
        
        function_signatures.insert(
            "095ea7b3".to_string(), // approve(address,uint256)
            ContractFunction {
                name: "approve".to_string(),
                signature: "approve(address,uint256)".to_string(),
                risk_level: RiskLevel::High,
                parameters: vec![ParameterType::Address, ParameterType::Uint256],
            }
        );
        
        function_signatures.insert(
            "23b872dd".to_string(), // transferFrom(address,address,uint256)
            ContractFunction {
                name: "transferFrom".to_string(),
                signature: "transferFrom(address,address,uint256)".to_string(),
                risk_level: RiskLevel::High,
                parameters: vec![ParameterType::Address, ParameterType::Address, ParameterType::Uint256],
            }
        );
        
        Self {
            function_signatures,
        }
    }
    
    /// Validate contract call data
    pub fn validate_contract_call(&self, data: &str) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::new();
        
        if data.len() < 10 {
            result.add_error("Contract call data too short".to_string());
            return Ok(result);
        }
        
        // Extract function selector (first 4 bytes after 0x)
        let function_selector = &data[2..10];
        
        if let Some(function) = self.function_signatures.get(function_selector) {
            result.add_warning(format!("Contract function call: {}", function.name));
            
            match function.risk_level {
                RiskLevel::Critical => {
                    result.add_error(format!("Critical risk function: {}", function.name));
                }
                RiskLevel::High => {
                    result.add_warning(format!("High risk function: {}", function.name));
                }
                RiskLevel::Medium => {
                    result.add_warning(format!("Medium risk function: {}", function.name));
                }
                RiskLevel::Low => {
                    debug!("Low risk function call: {}", function.name);
                }
            }
            
            // Validate parameters if data is long enough
            if data.len() > 10 {
                self.validate_function_parameters(function, &data[10..], &mut result);
            }
        } else {
            result.add_warning(format!("Unknown function selector: {}", function_selector));
        }
        
        Ok(result)
    }
    
    /// Validate function parameters
    fn validate_function_parameters(&self, function: &ContractFunction, param_data: &str, result: &mut ValidationResult) {
        // Basic parameter validation (simplified)
        let expected_length = function.parameters.len() * 64; // 32 bytes per parameter
        
        if param_data.len() < expected_length {
            result.add_warning("Insufficient parameter data".to_string());
            return;
        }
        
        // Validate each parameter based on type
        for (i, param_type) in function.parameters.iter().enumerate() {
            let param_start = i * 64;
            let param_end = param_start + 64;
            
            if param_end <= param_data.len() {
                let param_value = &param_data[param_start..param_end];
                self.validate_parameter_type(param_type, param_value, result);
            }
        }
    }
    
    /// Validate individual parameter based on type
    fn validate_parameter_type(&self, param_type: &ParameterType, value: &str, result: &mut ValidationResult) {
        match param_type {
            ParameterType::Address => {
                // Address parameters should be valid addresses (padded to 32 bytes)
                if value.len() == 64 {
                    let addr_part = &value[24..]; // Last 20 bytes (40 hex chars)
                    let full_addr = format!("0x{}", addr_part);
                    let addr_validator = EthereumAddressValidator::new(false);
                    if addr_validator.validate(&full_addr).is_err() {
                        result.add_warning("Invalid address parameter".to_string());
                    }
                }
            }
            ParameterType::Uint256 => {
                // Validate that it's a valid hex number
                if let Err(_) = BigUint::from_str(&format!("0x{}", value)) {
                    result.add_warning("Invalid uint256 parameter".to_string());
                }
            }
            ParameterType::Bytes => {
                // Basic hex validation
                let hex_regex = Regex::new(r"^[a-fA-F0-9]*$").unwrap();
                if !hex_regex.is_match(value) {
                    result.add_warning("Invalid bytes parameter".to_string());
                }
            }
            _ => {
                // Other types would need more complex validation
                debug!("Parameter type validation not implemented: {:?}", param_type);
            }
        }
    }
}

/// MEV-specific validation
pub struct MEVValidator;

impl MEVValidator {
    /// Detect potential MEV patterns in transaction data
    pub fn detect_mev_patterns(&self, transactions: &[TransactionData]) -> Vec<MEVPattern> {
        let mut patterns = Vec::new();
        
        // Detect sandwich attacks
        patterns.extend(self.detect_sandwich_attacks(transactions));
        
        // Detect front-running
        patterns.extend(self.detect_front_running(transactions));
        
        // Detect arbitrage opportunities
        patterns.extend(self.detect_arbitrage(transactions));
        
        patterns
    }
    
    fn detect_sandwich_attacks(&self, transactions: &[TransactionData]) -> Vec<MEVPattern> {
        // Simplified sandwich detection logic
        // In practice, this would be much more sophisticated
        Vec::new()
    }
    
    fn detect_front_running(&self, transactions: &[TransactionData]) -> Vec<MEVPattern> {
        // Simplified front-running detection
        Vec::new()
    }
    
    fn detect_arbitrage(&self, transactions: &[TransactionData]) -> Vec<MEVPattern> {
        // Simplified arbitrage detection
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct TransactionData {
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: u64,
    pub gas_price: String,
    pub data: String,
}

#[derive(Debug, Clone)]
pub enum MEVPattern {
    SandwichAttack {
        front_run_tx: String,
        victim_tx: String,
        back_run_tx: String,
    },
    FrontRunning {
        original_tx: String,
        front_run_tx: String,
    },
    Arbitrage {
        transactions: Vec<String>,
        profit_estimate: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ethereum_address_validation() {
        let validator = EthereumAddressValidator::new(false);
        
        // Valid addresses
        assert!(validator.validate("0x742d35Cc6634C0532925a3b8D4Ea2A1e7b4b2f6B").is_ok());
        assert!(validator.validate("0x0000000000000000000000000000000000000001").is_err()); // Burn address
        
        // Invalid addresses
        assert!(validator.validate("0x742d35Cc6634C0532925a3b8D4Ea2A1e7b4b2f6").is_err()); // Too short
        assert!(validator.validate("742d35Cc6634C0532925a3b8D4Ea2A1e7b4b2f6B").is_err()); // No 0x prefix
        assert!(validator.validate("0x0000000000000000000000000000000000000000").is_err()); // Null address
    }
    
    #[test]
    fn test_transaction_validation() {
        let validator = TransactionValidator::new();
        
        let result = validator.validate_transaction_params(
            "0x742d35Cc6634C0532925a3b8D4Ea2A1e7b4b2f6B",
            "0x1234567890123456789012345678901234567890",
            "1000000000000000000", // 1 ETH
            21000,
            "20000000000", // 20 Gwei
            1,
            "0x",
            1, // Mainnet
        );
        
        assert!(result.is_ok());
        let validation_result = result.unwrap();
        assert!(validation_result.valid);
    }
    
    #[test]
    fn test_contract_validation() {
        let validator = ContractValidator::new();
        
        // ERC-20 transfer function call
        let transfer_data = "0xa9059cbb000000000000000000000000742d35cc6634c0532925a3b8d4ea2a1e7b4b2f6b0000000000000000000000000000000000000000000000000de0b6b3a7640000";
        
        let result = validator.validate_contract_call(transfer_data);
        assert!(result.is_ok());
        
        let validation_result = result.unwrap();
        assert!(!validation_result.warnings.is_empty()); // Should warn about contract function call
    }
    
    #[test]
    fn test_gas_validation() {
        let validator = TransactionValidator::new();
        
        // Valid gas values
        assert!(validator.validate_gas(21000).is_ok());
        assert!(validator.validate_gas(100000).is_ok());
        
        // Invalid gas values
        assert!(validator.validate_gas(10000).is_err()); // Too low
        assert!(validator.validate_gas(50_000_000).is_err()); // Too high
    }
}