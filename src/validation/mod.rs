//! MEV Shield Input Validation Module
//!
//! Comprehensive input validation and sanitization to prevent injection attacks,
//! XSS, and other input-based vulnerabilities according to OWASP guidelines.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, warn, error};
use uuid::Uuid;
use num_bigint::BigUint;
use std::str::FromStr;

pub mod ethereum;
// pub mod sanitization; // Temporarily disabled due to compilation issues
pub mod sql_protection;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid input format: {0}")]
    InvalidFormat(String),
    
    #[error("Input too long: {0} exceeds maximum length {1}")]
    TooLong(usize, usize),
    
    #[error("Input too short: {0} below minimum length {1}")]
    TooShort(usize, usize),
    
    #[error("Invalid characters detected: {0}")]
    InvalidCharacters(String),
    
    #[error("Potential SQL injection detected: {0}")]
    SqlInjection(String),
    
    #[error("Potential XSS attack detected: {0}")]
    XssAttempt(String),
    
    #[error("Invalid Ethereum address: {0}")]
    InvalidAddress(String),
    
    #[error("Invalid numeric value: {0}")]
    InvalidNumeric(String),
    
    #[error("Value out of range: {0}")]
    OutOfRange(String),
    
    #[error("Invalid JSON format: {0}")]
    InvalidJson(String),
    
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(String),
    
    #[error("File upload validation failed: {0}")]
    FileUpload(String),
}

/// Configuration for validation rules
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_string_length: usize,
    pub min_password_length: usize,
    pub max_numeric_value: BigUint,
    pub allowed_file_types: Vec<String>,
    pub max_file_size: usize,
    pub enable_sql_injection_detection: bool,
    pub enable_xss_detection: bool,
    pub strict_address_validation: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_string_length: 1000,
            min_password_length: 12,
            max_numeric_value: BigUint::from_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap(), // 2^256 - 1
            allowed_file_types: vec!["json".to_string(), "csv".to_string()],
            max_file_size: 5 * 1024 * 1024, // 5MB
            enable_sql_injection_detection: true,
            enable_xss_detection: true,
            strict_address_validation: true,
        }
    }
}

/// Main validator struct
pub struct InputValidator {
    config: ValidationConfig,
    sql_patterns: Vec<Regex>,
    xss_patterns: Vec<Regex>,
    ethereum_address_regex: Regex,
    hex_regex: Regex,
}

impl InputValidator {
    pub fn new(config: ValidationConfig) -> Self {
        let sql_patterns = Self::compile_sql_patterns();
        let xss_patterns = Self::compile_xss_patterns();
        let ethereum_address_regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
        let hex_regex = Regex::new(r"^0x[a-fA-F0-9]*$").unwrap();
        
        Self {
            config,
            sql_patterns,
            xss_patterns,
            ethereum_address_regex,
            hex_regex,
        }
    }
    
    /// Compile SQL injection detection patterns
    fn compile_sql_patterns() -> Vec<Regex> {
        let patterns = [
            r"(?i)\b(union|select|insert|update|delete|drop|create|alter|exec|execute|sp_|xp_)\b",
            r"(?i)(\'\s*(;|--|\|)\s*)",
            r"(?i)(\'\s*or\s*\'\s*=\s*\')",
            r"(?i)(\'\s*and\s*\'\s*=\s*\')",
            r"(?i)(\/\*.*?\*\/)",
            r"(?i)(\bxp_cmdshell\b)",
            r"(?i)(\bsp_executesql\b)",
            r"(?i)(script\s*:)",
            r"(?i)(javascript\s*:)",
            r"(?i)(vbscript\s*:)",
        ];
        
        patterns.iter()
            .filter_map(|pattern| {
                match Regex::new(pattern) {
                    Ok(regex) => Some(regex),
                    Err(e) => {
                        error!("Failed to compile SQL pattern {}: {}", pattern, e);
                        None
                    }
                }
            })
            .collect()
    }
    
    /// Compile XSS detection patterns
    fn compile_xss_patterns() -> Vec<Regex> {
        let patterns = [
            r"(?i)<script[^>]*>.*?</script>",
            r"(?i)<iframe[^>]*>.*?</iframe>",
            r"(?i)<object[^>]*>.*?</object>",
            r"(?i)<embed[^>]*>.*?</embed>",
            r"(?i)<link[^>]*>",
            r"(?i)<meta[^>]*>",
            r"(?i)javascript\s*:",
            r"(?i)vbscript\s*:",
            r"(?i)data\s*:",
            r"(?i)on\w+\s*=",
            r"(?i)expression\s*\(",
            r"(?i)@import",
            r"(?i)&\#x[0-9a-f]+;",
            r"(?i)&\#[0-9]+;",
        ];
        
        patterns.iter()
            .filter_map(|pattern| {
                match Regex::new(pattern) {
                    Ok(regex) => Some(regex),
                    Err(e) => {
                        error!("Failed to compile XSS pattern {}: {}", pattern, e);
                        None
                    }
                }
            })
            .collect()
    }
    
    /// Validate general string input
    pub fn validate_string(&self, input: &str, field_name: &str) -> Result<String, ValidationError> {
        debug!("Validating string field: {}", field_name);
        
        // Length validation
        if input.len() > self.config.max_string_length {
            return Err(ValidationError::TooLong(input.len(), self.config.max_string_length));
        }
        
        // XSS detection
        if self.config.enable_xss_detection {
            self.detect_xss(input, field_name)?;
        }
        
        // SQL injection detection
        if self.config.enable_sql_injection_detection {
            self.detect_sql_injection(input, field_name)?;
        }
        
        // Control character detection
        self.detect_control_characters(input, field_name)?;
        
        Ok(input.to_string())
    }
    
    /// Validate Ethereum address with comprehensive checks
    pub fn validate_ethereum_address(&self, address: &str) -> Result<String, ValidationError> {
        debug!("Validating Ethereum address: {}", &address[..std::cmp::min(address.len(), 10)]);
        
        // Basic format validation
        if !self.ethereum_address_regex.is_match(address) {
            return Err(ValidationError::InvalidAddress(
                "Address must be 42 characters starting with 0x".to_string()
            ));
        }
        
        // Strict validation if enabled
        if self.config.strict_address_validation {
            self.validate_address_checksum(address)?;
        }
        
        // Additional security checks
        self.check_address_blacklist(address)?;
        
        Ok(address.to_lowercase())
    }
    
    /// Validate numeric values with overflow protection
    pub fn validate_amount(&self, amount: &str, field_name: &str) -> Result<BigUint, ValidationError> {
        debug!("Validating amount for field: {}", field_name);
        
        // Parse the value
        let value = BigUint::from_str(amount)
            .map_err(|_| ValidationError::InvalidNumeric(format!("Invalid numeric format: {}", amount)))?;
        
        // Range validation
        if value > self.config.max_numeric_value {
            return Err(ValidationError::OutOfRange(
                format!("Value {} exceeds maximum allowed", value)
            ));
        }
        
        // Check for potential overflow attacks
        if amount.len() > 78 { // Max reasonable length for uint256
            return Err(ValidationError::InvalidNumeric(
                "Number string too long, potential overflow attack".to_string()
            ));
        }
        
        Ok(value)
    }
    
    /// Validate gas values
    pub fn validate_gas(&self, gas: u64) -> Result<u64, ValidationError> {
        const MIN_GAS: u64 = 21000;
        const MAX_GAS: u64 = 30_000_000; // Current Ethereum block gas limit
        
        if gas < MIN_GAS {
            return Err(ValidationError::OutOfRange(
                format!("Gas {} below minimum {}", gas, MIN_GAS)
            ));
        }
        
        if gas > MAX_GAS {
            return Err(ValidationError::OutOfRange(
                format!("Gas {} exceeds maximum {}", gas, MAX_GAS)
            ));
        }
        
        Ok(gas)
    }
    
    /// Validate transaction data
    pub fn validate_transaction_data(&self, data: &str) -> Result<Vec<u8>, ValidationError> {
        debug!("Validating transaction data");
        
        // Validate hex format
        if !self.hex_regex.is_match(data) {
            return Err(ValidationError::InvalidFormat(
                "Transaction data must be valid hex string starting with 0x".to_string()
            ));
        }
        
        // Length validation (reasonable limit for transaction data)
        if data.len() > 1_048_576 { // 1MB limit
            return Err(ValidationError::TooLong(data.len(), 1_048_576));
        }
        
        // Convert to bytes
        let hex_data = if data.starts_with("0x") {
            &data[2..]
        } else {
            data
        };
        
        hex::decode(hex_data)
            .map_err(|_| ValidationError::InvalidFormat("Invalid hex encoding".to_string()))
    }
    
    /// Validate UUID
    pub fn validate_uuid(&self, uuid_str: &str) -> Result<Uuid, ValidationError> {
        Uuid::parse_str(uuid_str)
            .map_err(|_| ValidationError::InvalidUuid(format!("Invalid UUID format: {}", uuid_str)))
    }
    
    /// Validate JSON input
    pub fn validate_json(&self, json_str: &str, max_depth: usize) -> Result<serde_json::Value, ValidationError> {
        // Size check
        if json_str.len() > self.config.max_string_length {
            return Err(ValidationError::TooLong(json_str.len(), self.config.max_string_length));
        }
        
        // Parse JSON
        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))?;
        
        // Depth check to prevent JSON bomb attacks
        self.validate_json_depth(&value, max_depth, 0)?;
        
        Ok(value)
    }
    
    /// Validate file uploads
    pub fn validate_file_upload(&self, filename: &str, content: &[u8]) -> Result<(), ValidationError> {
        // File size check
        if content.len() > self.config.max_file_size {
            return Err(ValidationError::FileUpload(
                format!("File size {} exceeds limit {}", content.len(), self.config.max_file_size)
            ));
        }
        
        // File type validation
        let extension = filename.split('.').last().unwrap_or("").to_lowercase();
        if !self.config.allowed_file_types.contains(&extension) {
            return Err(ValidationError::FileUpload(
                format!("File type '{}' not allowed", extension)
            ));
        }
        
        // Content validation for known types
        match extension.as_str() {
            "json" => {
                serde_json::from_slice::<serde_json::Value>(content)
                    .map_err(|_| ValidationError::FileUpload("Invalid JSON content".to_string()))?;
            }
            _ => {} // Add more content validations as needed
        }
        
        Ok(())
    }
    
    /// Detect XSS attempts
    fn detect_xss(&self, input: &str, field_name: &str) -> Result<(), ValidationError> {
        for pattern in &self.xss_patterns {
            if pattern.is_match(input) {
                warn!("XSS attempt detected in field '{}': pattern matched", field_name);
                return Err(ValidationError::XssAttempt(
                    format!("Potential XSS in field '{}'", field_name)
                ));
            }
        }
        Ok(())
    }
    
    /// Detect SQL injection attempts
    fn detect_sql_injection(&self, input: &str, field_name: &str) -> Result<(), ValidationError> {
        for pattern in &self.sql_patterns {
            if pattern.is_match(input) {
                warn!("SQL injection attempt detected in field '{}': pattern matched", field_name);
                return Err(ValidationError::SqlInjection(
                    format!("Potential SQL injection in field '{}'", field_name)
                ));
            }
        }
        Ok(())
    }
    
    /// Detect control characters
    fn detect_control_characters(&self, input: &str, field_name: &str) -> Result<(), ValidationError> {
        for ch in input.chars() {
            if ch.is_control() && ch != '\t' && ch != '\n' && ch != '\r' {
                warn!("Control character detected in field '{}': {:?}", field_name, ch);
                return Err(ValidationError::InvalidCharacters(
                    format!("Control character detected in field '{}'", field_name)
                ));
            }
        }
        Ok(())
    }
    
    /// Validate Ethereum address checksum (EIP-55)
    fn validate_address_checksum(&self, address: &str) -> Result<(), ValidationError> {
        // For now, just do basic validation. Full EIP-55 would require keccak hashing
        // This is a placeholder for future enhancement
        if address != address.to_lowercase() && address != address.to_uppercase() {
            // Mixed case - should validate checksum properly
            // For now, we'll accept it but log a warning
            debug!("Mixed case address detected, should validate EIP-55 checksum: {}", address);
        }
        Ok(())
    }
    
    /// Check address against blacklist
    fn check_address_blacklist(&self, address: &str) -> Result<(), ValidationError> {
        // Known malicious/sanctioned addresses (example list)
        let blacklisted_addresses = [
            "0x7f367cc41522ce07553e823bf3be79a889debe1b", // Tornado Cash
            "0xd90e2f925da726b50c4ed8d0fb90ad053324f31b", // Tornado Cash
            // Add more as needed
        ];
        
        let addr_lower = address.to_lowercase();
        if blacklisted_addresses.contains(&addr_lower.as_str()) {
            warn!("Blacklisted address detected: {}", address);
            return Err(ValidationError::InvalidAddress(
                "Address is blacklisted".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate JSON depth to prevent bomb attacks
    fn validate_json_depth(&self, value: &serde_json::Value, max_depth: usize, current_depth: usize) -> Result<(), ValidationError> {
        if current_depth > max_depth {
            return Err(ValidationError::InvalidJson(
                format!("JSON depth {} exceeds maximum {}", current_depth, max_depth)
            ));
        }
        
        match value {
            serde_json::Value::Object(obj) => {
                for (_, v) in obj {
                    self.validate_json_depth(v, max_depth, current_depth + 1)?;
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    self.validate_json_depth(v, max_depth, current_depth + 1)?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
}

/// Default validator instance
impl Default for InputValidator {
    fn default() -> Self {
        Self::new(ValidationConfig::default())
    }
}

/// Helper function to sanitize string for safe output
pub fn sanitize_for_output(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

/// Validation result wrapper
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_validator() -> InputValidator {
        InputValidator::new(ValidationConfig::default())
    }
    
    #[test]
    fn test_string_validation() {
        let validator = create_test_validator();
        
        // Valid string
        assert!(validator.validate_string("Hello, World!", "test").is_ok());
        
        // Too long string
        let long_string = "a".repeat(2000);
        assert!(validator.validate_string(&long_string, "test").is_err());
    }
    
    #[test]
    fn test_xss_detection() {
        let validator = create_test_validator();
        
        // XSS attempts
        assert!(validator.validate_string("<script>alert('xss')</script>", "test").is_err());
        assert!(validator.validate_string("javascript:alert('xss')", "test").is_err());
        assert!(validator.validate_string("<iframe src='evil'></iframe>", "test").is_err());
    }
    
    #[test]
    fn test_sql_injection_detection() {
        let validator = create_test_validator();
        
        // SQL injection attempts
        assert!(validator.validate_string("'; DROP TABLE users; --", "test").is_err());
        assert!(validator.validate_string("' OR '1'='1", "test").is_err());
        assert!(validator.validate_string("UNION SELECT * FROM users", "test").is_err());
    }
    
    #[test]
    fn test_ethereum_address_validation() {
        let validator = create_test_validator();
        
        // Valid address
        assert!(validator.validate_ethereum_address("0x742d35Cc6634C0532925a3b8D4Ea2A1e7b4b2f6B").is_ok());
        
        // Invalid format
        assert!(validator.validate_ethereum_address("0x123").is_err());
        assert!(validator.validate_ethereum_address("742d35Cc6634C0532925a3b8D4Ea2A1e7b4b2f6B").is_err());
        assert!(validator.validate_ethereum_address("0xGGGd35Cc6634C0532925a3b8D4Ea2A1e7b4b2f6B").is_err());
    }
    
    #[test]
    fn test_amount_validation() {
        let validator = create_test_validator();
        
        // Valid amounts
        assert!(validator.validate_amount("1000000000000000000", "amount").is_ok()); // 1 ETH in wei
        assert!(validator.validate_amount("0", "amount").is_ok());
        
        // Invalid amounts
        assert!(validator.validate_amount("invalid", "amount").is_err());
        assert!(validator.validate_amount("-1", "amount").is_err());
    }
    
    #[test]
    fn test_gas_validation() {
        let validator = create_test_validator();
        
        // Valid gas values
        assert!(validator.validate_gas(21000).is_ok());
        assert!(validator.validate_gas(100000).is_ok());
        
        // Invalid gas values
        assert!(validator.validate_gas(10000).is_err()); // Too low
        assert!(validator.validate_gas(50_000_000).is_err()); // Too high
    }
    
    #[test]
    fn test_transaction_data_validation() {
        let validator = create_test_validator();
        
        // Valid transaction data
        assert!(validator.validate_transaction_data("0x").is_ok());
        assert!(validator.validate_transaction_data("0xa9059cbb000000000000000000000000742d35cc6634c0532925a3b8d4ea2a1e7b4b2f6b0000000000000000000000000000000000000000000000000de0b6b3a7640000").is_ok());
        
        // Invalid transaction data
        assert!(validator.validate_transaction_data("invalid").is_err());
        assert!(validator.validate_transaction_data("0xGG").is_err());
    }
    
    #[test]
    fn test_json_validation() {
        let validator = create_test_validator();
        
        // Valid JSON
        assert!(validator.validate_json(r#"{"key": "value"}"#, 10).is_ok());
        
        // Invalid JSON
        assert!(validator.validate_json(r#"{"key": "value""#, 10).is_err());
    }
    
    #[test]
    fn test_control_character_detection() {
        let validator = create_test_validator();
        
        // String with control characters
        let input_with_control = format!("hello{}world", char::from(7)); // Bell character
        assert!(validator.validate_string(&input_with_control, "test").is_err());
        
        // Valid string with allowed whitespace
        assert!(validator.validate_string("hello\tworld\n", "test").is_ok());
    }
}