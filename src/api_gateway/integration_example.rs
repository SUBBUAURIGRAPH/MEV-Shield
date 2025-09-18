//! MEV Shield API Gateway Integration Example
//!
//! Demonstrates how to integrate all Phase 2 security components
//! including validation, rate limiting, CORS, and API gateway security.

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    api_gateway::{ApiGatewayConfig, ApiGatewayState, api_gateway_middleware, gateway_health_check, gateway_metrics},
    middleware::{
        rate_limit::{RateLimitConfig, RateLimiterState, rate_limit_middleware},
        cors::{CorsConfig, cors_middleware},
        *,
    },
    auth::middleware::{AuthState, auth_middleware},
    validation::InputValidator,
};

/// Create a fully secured API router with all Phase 2 security features
pub fn create_secure_api_router() -> Router {
    // Initialize security components
    let rate_limit_config = create_production_rate_limit_config();
    let rate_limiter = Arc::new(RateLimiterState::new(rate_limit_config));
    
    let cors_config = create_production_cors_config();
    
    let api_gateway_config = create_production_api_gateway_config();
    let api_gateway = Arc::new(ApiGatewayState::new(api_gateway_config, rate_limiter.clone()));
    
    let auth_state = create_auth_state();
    
    // Create the secured router
    Router::new()
        // Public endpoints (minimal security)
        .route("/health", get(gateway_health_check))
        .route("/api/v1/health", get(gateway_health_check))
        
        // Protected API endpoints
        .route("/api/v1/transactions", post(submit_transaction_handler))
        .route("/api/v1/transactions/:id", get(get_transaction_handler))
        .route("/api/v1/analytics/mev", get(get_mev_analytics_handler))
        
        // Admin endpoints (highest security)
        .route("/api/v1/admin/metrics", get(gateway_metrics))
        .route("/api/v1/admin/config", get(get_admin_config_handler))
        
        // Add state for all middleware
        .with_state(api_gateway.clone())
        .with_state(rate_limiter.clone())
        .with_state(auth_state)
        
        // Apply middleware in security-focused order
        .layer(
            ServiceBuilder::new()
                // 1. Outermost: Error handling and logging
                .layer(middleware::from_fn(error_handling_middleware))
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(request_id_middleware))
                .layer(middleware::from_fn(timing_middleware))
                
                // 2. Early security checks
                .layer(middleware::from_fn(health_check_middleware))
                .layer(middleware::from_fn(malicious_request_detection_middleware))
                .layer(middleware::from_fn(user_agent_validation_middleware))
                .layer(middleware::from_fn(ip_filtering_middleware))
                
                // 3. Request validation and size limits
                .layer(middleware::from_fn(request_size_middleware))
                .layer(middleware::from_fn(request_validation_middleware))
                
                // 4. CORS handling
                .layer(middleware::from_fn(cors_middleware(cors_config)))
                
                // 5. Rate limiting
                .layer(middleware::from_fn_with_state(
                    rate_limiter.clone(),
                    rate_limit_middleware,
                ))
                
                // 6. API Gateway processing
                .layer(middleware::from_fn_with_state(
                    api_gateway,
                    api_gateway_middleware,
                ))
                
                // 7. Authentication (for protected routes)
                .layer(middleware::from_fn_with_state(
                    auth_state.clone(),
                    auth_middleware,
                ))
                
                // 8. Security headers (innermost)
                .layer(middleware::from_fn(security_headers_middleware))
        )
}

/// Create production-grade rate limiting configuration
fn create_production_rate_limit_config() -> RateLimitConfig {
    let mut config = RateLimitConfig::default();
    
    // Stricter limits for production
    config.ip_limits.requests_per_minute = 60; // Reduced from default
    config.ip_limits.requests_per_hour = 1000;
    config.ip_limits.concurrent_connections = 10;
    
    config.user_limits.authenticated_requests_per_minute = 120;
    config.user_limits.authenticated_requests_per_hour = 2000;
    config.user_limits.admin_requests_per_minute = 300;
    config.user_limits.admin_requests_per_hour = 5000;
    
    // Specific endpoint limits
    config.endpoint_limits.insert("/auth/login".to_string(), crate::middleware::rate_limit::EndpointLimit {
        requests_per_minute: 5,
        requests_per_hour: 20,
        concurrent_requests: 2,
        require_auth: false,
        admin_only: false,
    });
    
    config.endpoint_limits.insert("/api/v1/transactions".to_string(), crate::middleware::rate_limit::EndpointLimit {
        requests_per_minute: 30,
        requests_per_hour: 500,
        concurrent_requests: 5,
        require_auth: true,
        admin_only: false,
    });
    
    config
}

/// Create production-grade CORS configuration
fn create_production_cors_config() -> CorsConfig {
    CorsConfig::production(vec![
        "mev-shield.aurex.in".to_string(),
        "app.mev-shield.aurex.in".to_string(),
        "admin.mev-shield.aurex.in".to_string(),
    ])
}

/// Create production-grade API gateway configuration
fn create_production_api_gateway_config() -> ApiGatewayConfig {
    let mut config = ApiGatewayConfig::default();
    
    // Enable strict validation
    config.validation.strictness_level = crate::api_gateway::ValidationStrictness::Strict;
    config.validation.max_request_size = 5 * 1024 * 1024; // 5MB for production
    
    // Configure API keys
    let mut api_keys = HashMap::new();
    api_keys.insert(
        "mev_shield_prod_key_001".to_string(),
        crate::api_gateway::ApiKeyInfo {
            key_id: "prod_001".to_string(),
            user_id: "system".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            rate_limit: Some(1000),
            expires_at: None,
        }
    );
    
    config.security.api_key_config.validation_strategy = 
        crate::api_gateway::ApiKeyValidationStrategy::Static(api_keys);
    
    // Stricter content security
    config.security.content_security.max_json_depth = 5;
    config.security.content_security.max_array_length = 100;
    config.security.content_security.max_string_length = 1000;
    
    // Enhanced monitoring
    config.monitoring.sample_rate = 1.0; // 100% sampling in production for security
    config.monitoring.alert_thresholds.error_rate = 2.0; // 2% error rate threshold
    config.monitoring.alert_thresholds.latency_p99 = 500; // 500ms latency threshold
    
    config
}

/// Create authentication state
fn create_auth_state() -> AuthState {
    use crate::auth::jwt::{JwtService, JwtConfig};
    
    let jwt_config = JwtConfig {
        secret: "production_secret_key_change_in_production".to_string(),
        access_token_expiry: chrono::Duration::hours(1),
        refresh_token_expiry: chrono::Duration::days(7),
        issuer: "mev-shield-api".to_string(),
        audience: "mev-shield-users".to_string(),
    };
    
    let jwt_service = JwtService::new(jwt_config);
    AuthState::new(jwt_service)
}

// Example handlers with comprehensive input validation

use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use crate::validation::{InputValidator, ValidationError};

#[derive(Deserialize)]
pub struct SubmitTransactionRequest {
    pub transaction: TransactionData,
    pub protection: ProtectionConfig,
    pub chain_id: u64,
}

#[derive(Deserialize)]
pub struct TransactionData {
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: u64,
    pub gas_price: String,
    pub nonce: u64,
    pub data: String,
}

#[derive(Deserialize)]
pub struct ProtectionConfig {
    pub level: String,
    pub slippage_tolerance: Option<f64>,
}

#[derive(Serialize)]
pub struct TransactionResponse {
    pub success: bool,
    pub transaction_id: Option<String>,
    pub error: Option<String>,
}

/// Submit transaction with comprehensive validation
pub async fn submit_transaction_handler(
    State(validator): State<InputValidator>,
    Json(request): Json<SubmitTransactionRequest>,
) -> Result<Json<TransactionResponse>, StatusCode> {
    // Comprehensive input validation using our validation system
    match validate_transaction_request(&validator, &request) {
        Ok(validated_data) => {
            // Process the validated transaction
            let transaction_id = uuid::Uuid::new_v4().to_string();
            
            tracing::info!(
                "Transaction submitted successfully: {} -> {} (value: {}, gas: {})",
                validated_data.from,
                validated_data.to,
                validated_data.value,
                validated_data.gas
            );
            
            Ok(Json(TransactionResponse {
                success: true,
                transaction_id: Some(transaction_id),
                error: None,
            }))
        }
        Err(e) => {
            tracing::warn!("Transaction validation failed: {}", e);
            Ok(Json(TransactionResponse {
                success: false,
                transaction_id: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

/// Validate transaction request using our comprehensive validation system
fn validate_transaction_request(
    validator: &InputValidator,
    request: &SubmitTransactionRequest,
) -> Result<ValidatedTransactionData, ValidationError> {
    // Validate addresses
    let from = validator.validate_ethereum_address(&request.transaction.from)?;
    let to = validator.validate_ethereum_address(&request.transaction.to)?;
    
    // Validate amounts
    let value = validator.validate_amount(&request.transaction.value, "value")?;
    let gas_price = validator.validate_amount(&request.transaction.gas_price, "gas_price")?;
    
    // Validate gas
    let gas = validator.validate_gas(request.transaction.gas)?;
    
    // Validate transaction data
    let data = validator.validate_transaction_data(&request.transaction.data)?;
    
    // Validate nonce (basic range check)
    if request.transaction.nonce > 1_000_000 {
        return Err(ValidationError::OutOfRange(
            "Nonce too high".to_string()
        ));
    }
    
    // Validate chain ID
    let valid_chains = [1, 5, 11155111, 137, 56, 250, 43114, 42161, 10]; // Common chains
    if !valid_chains.contains(&request.chain_id) {
        return Err(ValidationError::InvalidFormat(
            format!("Unsupported chain ID: {}", request.chain_id)
        ));
    }
    
    // Validate protection level
    let valid_protection_levels = ["basic", "standard", "maximum"];
    if !valid_protection_levels.contains(&request.protection.level.as_str()) {
        return Err(ValidationError::InvalidFormat(
            format!("Invalid protection level: {}", request.protection.level)
        ));
    }
    
    // Validate slippage tolerance if provided
    if let Some(slippage) = request.protection.slippage_tolerance {
        if slippage < 0.0 || slippage > 50.0 {
            return Err(ValidationError::OutOfRange(
                "Slippage tolerance must be between 0% and 50%".to_string()
            ));
        }
    }
    
    Ok(ValidatedTransactionData {
        from,
        to,
        value,
        gas,
        gas_price,
        nonce: request.transaction.nonce,
        data,
        chain_id: request.chain_id,
        protection_level: request.protection.level.clone(),
        slippage_tolerance: request.protection.slippage_tolerance,
    })
}

#[derive(Debug)]
pub struct ValidatedTransactionData {
    pub from: String,
    pub to: String,
    pub value: num_bigint::BigUint,
    pub gas: u64,
    pub gas_price: num_bigint::BigUint,
    pub nonce: u64,
    pub data: Vec<u8>,
    pub chain_id: u64,
    pub protection_level: String,
    pub slippage_tolerance: Option<f64>,
}

/// Get transaction with validation
pub async fn get_transaction_handler(
    Path(id): Path<String>,
    State(validator): State<InputValidator>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate UUID format
    match validator.validate_uuid(&id) {
        Ok(transaction_id) => {
            tracing::debug!("Fetching transaction: {}", transaction_id);
            
            // Mock response
            Ok(Json(serde_json::json!({
                "success": true,
                "data": {
                    "id": transaction_id.to_string(),
                    "status": "completed",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            })))
        }
        Err(e) => {
            tracing::warn!("Invalid transaction ID format: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get MEV analytics with query validation
pub async fn get_mev_analytics_handler(
    Query(params): Query<HashMap<String, String>>,
    State(validator): State<InputValidator>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate query parameters
    let timeframe = params.get("timeframe")
        .map(|t| validator.sanitize_text(t, 20))
        .unwrap_or_else(|| "24h".to_string());
    
    // Validate timeframe options
    let valid_timeframes = ["1h", "24h", "7d", "30d"];
    if !valid_timeframes.contains(&timeframe.as_str()) {
        tracing::warn!("Invalid timeframe parameter: {}", timeframe);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    tracing::debug!("Getting MEV analytics for timeframe: {}", timeframe);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "timeframe": timeframe,
            "total_mev_captured": "15000000000000000000",
            "total_distributed": "12000000000000000000",
            "protected_transactions": 12567,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })))
}

/// Admin config endpoint with strict access control
pub async fn get_admin_config_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    // This endpoint requires admin authentication (handled by middleware)
    
    tracing::info!("Admin config requested");
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "version": env!("CARGO_PKG_VERSION"),
            "security_level": "production",
            "features": {
                "rate_limiting": true,
                "input_validation": true,
                "cors_protection": true,
                "api_gateway": true
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    
    #[tokio::test]
    async fn test_secure_api_integration() {
        let app = create_secure_api_router();
        let server = TestServer::new(app).unwrap();
        
        // Test health endpoint (should work without auth)
        let response = server.get("/health").await;
        assert_eq!(response.status_code(), 200);
        
        // Test protected endpoint without auth (should fail)
        let response = server.get("/api/v1/transactions/123").await;
        assert_eq!(response.status_code(), 401);
    }
    
    #[tokio::test]
    async fn test_input_validation() {
        let validator = InputValidator::default();
        
        let request = SubmitTransactionRequest {
            transaction: TransactionData {
                from: "0x742d35Cc6634C0532925a3b8D4Ea2A1e7b4b2f6B".to_string(),
                to: "0x1234567890123456789012345678901234567890".to_string(),
                value: "1000000000000000000".to_string(), // 1 ETH
                gas: 21000,
                gas_price: "20000000000".to_string(), // 20 Gwei
                nonce: 1,
                data: "0x".to_string(),
            },
            protection: ProtectionConfig {
                level: "standard".to_string(),
                slippage_tolerance: Some(1.0),
            },
            chain_id: 1,
        };
        
        let result = validate_transaction_request(&validator, &request);
        assert!(result.is_ok());
        
        let validated = result.unwrap();
        assert_eq!(validated.from, "0x742d35cc6634c0532925a3b8d4ea2a1e7b4b2f6b");
        assert_eq!(validated.chain_id, 1);
        assert_eq!(validated.protection_level, "standard");
    }
}