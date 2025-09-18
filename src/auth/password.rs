//! Password Hashing and Validation
//!
//! Implements Argon2id password hashing with secure defaults
//! and password strength validation.

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use anyhow::{anyhow, Result};
use rand::rngs::OsRng;
use regex::Regex;
use thiserror::Error;

/// Password validation error types
#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Password is too short (minimum 12 characters)")]
    TooShort,
    #[error("Password must contain at least one uppercase letter")]
    NoUppercase,
    #[error("Password must contain at least one lowercase letter")]
    NoLowercase,
    #[error("Password must contain at least one digit")]
    NoDigit,
    #[error("Password must contain at least one special character")]
    NoSpecialChar,
    #[error("Password contains invalid characters")]
    InvalidCharacters,
    #[error("Password is too common or easily guessable")]
    TooCommon,
    #[error("Password hashing failed: {0}")]
    HashingError(String),
    #[error("Password verification failed: {0}")]
    VerificationError(String),
}

/// Password security configuration
#[derive(Debug, Clone)]
pub struct PasswordConfig {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special_char: bool,
    pub check_common_passwords: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special_char: true,
            check_common_passwords: true,
        }
    }
}

/// Password hashing and validation service
pub struct PasswordService {
    hasher: Argon2<'static>,
    config: PasswordConfig,
    common_passwords: Vec<&'static str>,
}

impl PasswordService {
    /// Create a new password service with default configuration
    pub fn new() -> Self {
        Self::with_config(PasswordConfig::default())
    }

    /// Create a new password service with custom configuration
    pub fn with_config(config: PasswordConfig) -> Self {
        // Use Argon2id with secure parameters
        let hasher = Argon2::default();
        
        // List of common passwords to reject
        let common_passwords = vec![
            "password",
            "123456",
            "123456789",
            "12345678",
            "12345",
            "1234567",
            "password123",
            "admin",
            "administrator",
            "welcome",
            "login",
            "passw0rd",
            "qwerty",
            "abc123",
            "letmein",
            "monkey",
            "1234567890",
            "dragon",
            "sunshine",
            "princess",
            "azerty",
            "trustno1",
            "000000",
        ];

        Self {
            hasher,
            config,
            common_passwords,
        }
    }

    /// Hash a password using Argon2id
    pub fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        // Validate password strength first
        self.validate_password_strength(password)?;

        // Generate a random salt
        let salt = SaltString::generate(&mut OsRng);
        
        // Hash the password
        let password_hash = self
            .hasher
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| PasswordError::HashingError(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, PasswordError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| PasswordError::VerificationError(e.to_string()))?;

        match self.hasher.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(PasswordError::VerificationError(e.to_string())),
        }
    }

    /// Validate password strength according to configuration
    pub fn validate_password_strength(&self, password: &str) -> Result<(), PasswordError> {
        // Check minimum length
        if password.len() < self.config.min_length {
            return Err(PasswordError::TooShort);
        }

        // Check for uppercase letters
        if self.config.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(PasswordError::NoUppercase);
        }

        // Check for lowercase letters
        if self.config.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(PasswordError::NoLowercase);
        }

        // Check for digits
        if self.config.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(PasswordError::NoDigit);
        }

        // Check for special characters
        if self.config.require_special_char {
            let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
            if !password.chars().any(|c| special_chars.contains(c)) {
                return Err(PasswordError::NoSpecialChar);
            }
        }

        // Check for invalid characters (only allow printable ASCII)
        if !password.chars().all(|c| c.is_ascii() && !c.is_control()) {
            return Err(PasswordError::InvalidCharacters);
        }

        // Check against common passwords
        if self.config.check_common_passwords {
            let password_lower = password.to_lowercase();
            if self.common_passwords.iter().any(|&common| {
                password_lower == common || password_lower.contains(common)
            }) {
                return Err(PasswordError::TooCommon);
            }
        }

        Ok(())
    }

    /// Generate a secure random password
    pub fn generate_password(&self, length: usize) -> String {
        use rand::Rng;
        
        let length = length.max(self.config.min_length);
        let mut rng = rand::thread_rng();
        
        let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let lowercase = "abcdefghijklmnopqrstuvwxyz";
        let digits = "0123456789";
        let special = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        
        let mut password = String::with_capacity(length);
        
        // Ensure at least one character from each required category
        if self.config.require_uppercase {
            password.push(uppercase.chars().nth(rng.gen_range(0..uppercase.len())).unwrap());
        }
        if self.config.require_lowercase {
            password.push(lowercase.chars().nth(rng.gen_range(0..lowercase.len())).unwrap());
        }
        if self.config.require_digit {
            password.push(digits.chars().nth(rng.gen_range(0..digits.len())).unwrap());
        }
        if self.config.require_special_char {
            password.push(special.chars().nth(rng.gen_range(0..special.len())).unwrap());
        }
        
        // Fill the rest with random characters
        let all_chars = format!("{}{}{}{}", uppercase, lowercase, digits, special);
        while password.len() < length {
            password.push(all_chars.chars().nth(rng.gen_range(0..all_chars.len())).unwrap());
        }
        
        // Shuffle the password characters
        let mut chars: Vec<char> = password.chars().collect();
        for i in (1..chars.len()).rev() {
            let j = rng.gen_range(0..=i);
            chars.swap(i, j);
        }
        
        chars.into_iter().collect()
    }

    /// Calculate password entropy (bits)
    pub fn calculate_entropy(&self, password: &str) -> f64 {
        let mut charset_size = 0;
        
        if password.chars().any(|c| c.is_ascii_lowercase()) {
            charset_size += 26;
        }
        if password.chars().any(|c| c.is_ascii_uppercase()) {
            charset_size += 26;
        }
        if password.chars().any(|c| c.is_ascii_digit()) {
            charset_size += 10;
        }
        if password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            charset_size += 23;
        }
        
        if charset_size == 0 {
            return 0.0;
        }
        
        password.len() as f64 * (charset_size as f64).log2()
    }

    /// Get password strength assessment
    pub fn assess_strength(&self, password: &str) -> PasswordStrength {
        let entropy = self.calculate_entropy(password);
        
        if entropy < 35.0 {
            PasswordStrength::Weak
        } else if entropy < 60.0 {
            PasswordStrength::Fair
        } else if entropy < 120.0 {
            PasswordStrength::Good
        } else {
            PasswordStrength::Strong
        }
    }
}

impl Default for PasswordService {
    fn default() -> Self {
        Self::new()
    }
}

/// Password strength levels
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordStrength {
    Weak,
    Fair,
    Good,
    Strong,
}

impl PasswordStrength {
    pub fn as_str(&self) -> &'static str {
        match self {
            PasswordStrength::Weak => "weak",
            PasswordStrength::Fair => "fair",
            PasswordStrength::Good => "good",
            PasswordStrength::Strong => "strong",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let service = PasswordService::new();
        let password = "SecurePass123!@#";
        
        let hash = service.hash_password(password).expect("Password hashing failed");
        assert!(!hash.is_empty());
        
        let is_valid = service.verify_password(password, &hash).expect("Verification failed");
        assert!(is_valid);
        
        let is_invalid = service.verify_password("WrongPassword", &hash).expect("Verification failed");
        assert!(!is_invalid);
    }

    #[test]
    fn test_password_validation() {
        let service = PasswordService::new();
        
        // Valid password
        assert!(service.validate_password_strength("SecurePass123!@#").is_ok());
        
        // Too short
        assert!(matches!(
            service.validate_password_strength("Short1!"),
            Err(PasswordError::TooShort)
        ));
        
        // No uppercase
        assert!(matches!(
            service.validate_password_strength("securepass123!@#"),
            Err(PasswordError::NoUppercase)
        ));
        
        // No lowercase
        assert!(matches!(
            service.validate_password_strength("SECUREPASS123!@#"),
            Err(PasswordError::NoLowercase)
        ));
        
        // No digit
        assert!(matches!(
            service.validate_password_strength("SecurePass!@#"),
            Err(PasswordError::NoDigit)
        ));
        
        // No special character
        assert!(matches!(
            service.validate_password_strength("SecurePass123"),
            Err(PasswordError::NoSpecialChar)
        ));
        
        // Common password
        assert!(matches!(
            service.validate_password_strength("Password123!"),
            Err(PasswordError::TooCommon)
        ));
    }

    #[test]
    fn test_password_generation() {
        let service = PasswordService::new();
        
        let password = service.generate_password(16);
        assert_eq!(password.len(), 16);
        assert!(service.validate_password_strength(&password).is_ok());
    }

    #[test]
    fn test_entropy_calculation() {
        let service = PasswordService::new();
        
        let weak_password = "password";
        let strong_password = "Tr0ub4dor&3";
        
        let weak_entropy = service.calculate_entropy(weak_password);
        let strong_entropy = service.calculate_entropy(strong_password);
        
        assert!(strong_entropy > weak_entropy);
    }

    #[test]
    fn test_strength_assessment() {
        let service = PasswordService::new();
        
        assert_eq!(service.assess_strength("weak"), PasswordStrength::Weak);
        assert_eq!(service.assess_strength("SecurePass123!@#"), PasswordStrength::Good);
    }
}