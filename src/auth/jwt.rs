//! JWT Token Management
//!
//! Provides secure JWT token generation, validation, and refresh functionality
//! with support for access and refresh tokens.

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use crate::auth::models::{Claims, User, UserRole};

/// JWT-related errors
#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Token creation failed: {0}")]
    TokenCreation(String),
    #[error("Token validation failed: {0}")]
    TokenValidation(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid issuer")]
    InvalidIssuer,
    #[error("Invalid audience")]
    InvalidAudience,
    #[error("Token not yet valid")]
    TokenNotYetValid,
}

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub access_token_expiry: Duration,
    pub refresh_token_expiry: Duration,
    pub algorithm: Algorithm,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-jwt-key-change-this-in-production".to_string()),
            issuer: "mev-shield".to_string(),
            audience: "mev-shield-api".to_string(),
            access_token_expiry: Duration::hours(1),
            refresh_token_expiry: Duration::days(30),
            algorithm: Algorithm::HS256,
        }
    }
}

/// JWT token pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Refresh token claims
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,        // Subject (user ID)
    pub email: String,      // User email
    pub exp: i64,          // Expiration time
    pub iat: i64,          // Issued at
    pub iss: String,       // Issuer
    pub aud: String,       // Audience
    pub token_type: String, // "refresh"
}

/// JWT service for token management
pub struct JwtService {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtService {
    /// Create a new JWT service
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_ref());
        let decoding_key = DecodingKey::from_secret(config.secret.as_ref());
        
        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&[&config.issuer]);
        validation.set_audience(&[&config.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        
        Self {
            config,
            encoding_key,
            decoding_key,
            validation,
        }
    }

    /// Generate access and refresh tokens for a user
    pub fn generate_tokens(&self, user: &User) -> Result<TokenPair, JwtError> {
        let access_token = self.create_access_token(user)?;
        let refresh_token = self.create_refresh_token(user)?;
        
        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_expiry.num_seconds(),
        })
    }

    /// Create an access token
    fn create_access_token(&self, user: &User) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + self.config.access_token_expiry;
        
        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
        };

        encode(&Header::new(self.config.algorithm), &claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreation(e.to_string()))
    }

    /// Create a refresh token
    fn create_refresh_token(&self, user: &User) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + self.config.refresh_token_expiry;
        
        let claims = RefreshClaims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            token_type: "refresh".to_string(),
        };

        encode(&Header::new(self.config.algorithm), &claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreation(e.to_string()))
    }

    /// Validate and decode an access token
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => JwtError::InvalidIssuer,
                jsonwebtoken::errors::ErrorKind::InvalidAudience => JwtError::InvalidAudience,
                jsonwebtoken::errors::ErrorKind::ImmatureSignature => JwtError::TokenNotYetValid,
                _ => JwtError::TokenValidation(e.to_string()),
            })?;

        Ok(token_data.claims)
    }

    /// Validate and decode a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<RefreshClaims, JwtError> {
        let token_data = decode::<RefreshClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => JwtError::InvalidIssuer,
                jsonwebtoken::errors::ErrorKind::InvalidAudience => JwtError::InvalidAudience,
                _ => JwtError::TokenValidation(e.to_string()),
            })?;

        if token_data.claims.token_type != "refresh" {
            return Err(JwtError::InvalidToken);
        }

        Ok(token_data.claims)
    }

    /// Refresh an access token using a refresh token
    pub fn refresh_access_token(&self, refresh_token: &str, user: &User) -> Result<String, JwtError> {
        // Validate the refresh token
        let refresh_claims = self.validate_refresh_token(refresh_token)?;
        
        // Ensure the refresh token belongs to the user
        if refresh_claims.sub != user.id.to_string() {
            return Err(JwtError::InvalidToken);
        }

        // Create a new access token
        self.create_access_token(user)
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(&self, auth_header: &str) -> Result<String, JwtError> {
        if !auth_header.starts_with("Bearer ") {
            return Err(JwtError::InvalidToken);
        }

        let token = auth_header.trim_start_matches("Bearer ").trim();
        if token.is_empty() {
            return Err(JwtError::InvalidToken);
        }

        Ok(token.to_string())
    }

    /// Get token claims without validating expiration (for debugging)
    pub fn decode_token_unsafe(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::new(self.config.algorithm);
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.insecure_disable_signature_validation();
        
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| JwtError::TokenValidation(e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Check if a token is expired
    pub fn is_token_expired(&self, token: &str) -> bool {
        match self.decode_token_unsafe(token) {
            Ok(claims) => {
                let now = Utc::now().timestamp();
                claims.exp < now
            }
            Err(_) => true,
        }
    }

    /// Get token expiration time
    pub fn get_token_expiration(&self, token: &str) -> Result<chrono::DateTime<Utc>, JwtError> {
        let claims = self.decode_token_unsafe(token)?;
        Ok(chrono::DateTime::from_timestamp(claims.exp, 0)
            .ok_or(JwtError::InvalidToken)?)
    }
}

impl Default for JwtService {
    fn default() -> Self {
        Self::new(JwtConfig::default())
    }
}

/// In-memory token blacklist (in production, use Redis or database)
pub struct TokenBlacklist {
    tokens: HashMap<String, chrono::DateTime<Utc>>,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// Add a token to the blacklist
    pub fn add_token(&mut self, token: &str, expires_at: chrono::DateTime<Utc>) {
        self.tokens.insert(token.to_string(), expires_at);
        self.cleanup_expired();
    }

    /// Check if a token is blacklisted
    pub fn is_blacklisted(&self, token: &str) -> bool {
        self.tokens.contains_key(token)
    }

    /// Remove expired tokens from blacklist
    fn cleanup_expired(&mut self) {
        let now = Utc::now();
        self.tokens.retain(|_, expires_at| *expires_at > now);
    }
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::models::UserRole;

    fn create_test_user() -> User {
        User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            UserRole::User,
        )
    }

    #[test]
    fn test_token_generation_and_validation() {
        let jwt_service = JwtService::default();
        let user = create_test_user();

        let tokens = jwt_service.generate_tokens(&user).expect("Token generation failed");
        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert_eq!(tokens.token_type, "Bearer");

        let claims = jwt_service
            .validate_access_token(&tokens.access_token)
            .expect("Token validation failed");
        
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.email, user.email);
        assert_eq!(claims.role, user.role);
    }

    #[test]
    fn test_refresh_token_validation() {
        let jwt_service = JwtService::default();
        let user = create_test_user();

        let tokens = jwt_service.generate_tokens(&user).expect("Token generation failed");
        
        let refresh_claims = jwt_service
            .validate_refresh_token(&tokens.refresh_token)
            .expect("Refresh token validation failed");
        
        assert_eq!(refresh_claims.sub, user.id.to_string());
        assert_eq!(refresh_claims.token_type, "refresh");
    }

    #[test]
    fn test_access_token_refresh() {
        let jwt_service = JwtService::default();
        let user = create_test_user();

        let tokens = jwt_service.generate_tokens(&user).expect("Token generation failed");
        
        let new_access_token = jwt_service
            .refresh_access_token(&tokens.refresh_token, &user)
            .expect("Token refresh failed");
        
        assert!(!new_access_token.is_empty());
        assert_ne!(new_access_token, tokens.access_token);
        
        let claims = jwt_service
            .validate_access_token(&new_access_token)
            .expect("New token validation failed");
        
        assert_eq!(claims.sub, user.id.to_string());
    }

    #[test]
    fn test_token_header_extraction() {
        let jwt_service = JwtService::default();
        
        let token = jwt_service
            .extract_token_from_header("Bearer abc123def456")
            .expect("Token extraction failed");
        
        assert_eq!(token, "abc123def456");
        
        // Test invalid header
        assert!(jwt_service.extract_token_from_header("Invalid header").is_err());
        assert!(jwt_service.extract_token_from_header("Bearer ").is_err());
    }

    #[test]
    fn test_token_blacklist() {
        let mut blacklist = TokenBlacklist::new();
        let token = "test_token";
        let expires_at = Utc::now() + Duration::hours(1);
        
        assert!(!blacklist.is_blacklisted(token));
        
        blacklist.add_token(token, expires_at);
        assert!(blacklist.is_blacklisted(token));
    }

    #[test]
    fn test_expired_token_validation() {
        let mut config = JwtConfig::default();
        config.access_token_expiry = Duration::milliseconds(1); // Very short expiry
        
        let jwt_service = JwtService::new(config);
        let user = create_test_user();

        let tokens = jwt_service.generate_tokens(&user).expect("Token generation failed");
        
        // Wait for token to expire
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let result = jwt_service.validate_access_token(&tokens.access_token);
        assert!(matches!(result, Err(JwtError::TokenExpired)));
    }
}