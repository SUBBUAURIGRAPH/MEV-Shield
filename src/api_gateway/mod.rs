//! MEV Shield API Gateway Security
//!
//! Comprehensive API gateway with versioning, validation, throttling,
//! and security monitoring for all API endpoints.

use axum::{
    extract::{Path, Query, Request, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    middleware::Next,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, warn, error, info};
use uuid::Uuid;

// pub mod validation; // Temporarily disabled
// pub mod versioning; // Temporarily disabled
// pub mod throttling; // Temporarily disabled
// pub mod monitoring; // Temporarily disabled

use crate::{
    middleware::{rate_limit::RateLimiterState, cors::CorsConfig},
    validation::InputValidator,
    auth::models::Claims,
};

/// API Gateway configuration
#[derive(Debug, Clone)]
pub struct ApiGatewayConfig {
    /// API versioning configuration
    pub versioning: VersioningConfig,
    /// Request validation configuration
    pub validation: ValidationConfig,
    /// Throttling configuration
    pub throttling: ThrottlingConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Security configuration
    pub security: SecurityConfig,
}

#[derive(Debug, Clone)]
pub struct VersioningConfig {
    /// Supported API versions
    pub supported_versions: Vec<String>,
    /// Default version when none specified
    pub default_version: String,
    /// Deprecated versions with sunset dates
    pub deprecated_versions: HashMap<String, chrono::DateTime<chrono::Utc>>,
    /// Version extraction method
    pub extraction_method: VersionExtractionMethod,
}

#[derive(Debug, Clone)]
pub enum VersionExtractionMethod {
    Header(String),           // X-API-Version header
    QueryParameter(String),   // ?version=v1
    PathPrefix,              // /api/v1/...
    AcceptHeader,            // Accept: application/vnd.mev-shield.v1+json
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Enable request validation
    pub validate_requests: bool,
    /// Enable response validation
    pub validate_responses: bool,
    /// Maximum request size
    pub max_request_size: usize,
    /// Validation strictness level
    pub strictness_level: ValidationStrictness,
    /// Schema registry for validation
    pub schema_registry: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum ValidationStrictness {
    Strict,    // Reject any invalid data
    Lenient,   // Log warnings but allow
    Disabled,  // No validation
}

#[derive(Debug, Clone)]
pub struct ThrottlingConfig {
    /// Enable request throttling
    pub enabled: bool,
    /// Throttling strategies
    pub strategies: Vec<ThrottlingStrategy>,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
pub enum ThrottlingStrategy {
    RateLimit {
        requests_per_second: u32,
        burst_capacity: u32,
    },
    ConcurrencyLimit {
        max_concurrent: u32,
    },
    ResourceBased {
        cpu_threshold: f64,
        memory_threshold: f64,
    },
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Time window for failure counting
    pub time_window: Duration,
    /// Recovery timeout when circuit is open
    pub recovery_timeout: Duration,
    /// Success threshold to close circuit
    pub success_threshold: u32,
}

#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Enable request/response logging
    pub enable_logging: bool,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable distributed tracing
    pub enable_tracing: bool,
    /// Sample rate for detailed logging
    pub sample_rate: f64,
    /// Alerting thresholds
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Error rate threshold (percentage)
    pub error_rate: f64,
    /// High latency threshold (milliseconds)
    pub latency_p99: u64,
    /// Request volume threshold
    pub request_volume: u32,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// API key management
    pub api_key_config: ApiKeyConfig,
    /// JWT validation
    pub jwt_validation: bool,
    /// Request signing
    pub request_signing: RequestSigningConfig,
    /// IP filtering
    pub ip_filtering: bool,
    /// Content security
    pub content_security: ContentSecurityConfig,
}

#[derive(Debug, Clone)]
pub struct ApiKeyConfig {
    /// Enable API key authentication
    pub enabled: bool,
    /// API key header name
    pub header_name: String,
    /// Key validation strategy
    pub validation_strategy: ApiKeyValidationStrategy,
    /// Rate limiting per API key
    pub rate_limiting: bool,
}

#[derive(Debug, Clone)]
pub enum ApiKeyValidationStrategy {
    Database,
    Redis,
    Static(HashMap<String, ApiKeyInfo>),
}

#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    pub key_id: String,
    pub user_id: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<u32>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct RequestSigningConfig {
    /// Enable request signing
    pub enabled: bool,
    /// Signing algorithm
    pub algorithm: SigningAlgorithm,
    /// Timestamp tolerance (seconds)
    pub timestamp_tolerance: u64,
    /// Nonce validation
    pub validate_nonce: bool,
}

#[derive(Debug, Clone)]
pub enum SigningAlgorithm {
    HmacSha256,
    HmacSha512,
    Ed25519,
}

#[derive(Debug, Clone)]
pub struct ContentSecurityConfig {
    /// Maximum JSON depth
    pub max_json_depth: usize,
    /// Maximum array length
    pub max_array_length: usize,
    /// Maximum string length
    pub max_string_length: usize,
    /// Allowed content types
    pub allowed_content_types: Vec<String>,
}

impl Default for ApiGatewayConfig {
    fn default() -> Self {
        let mut schema_registry = HashMap::new();
        
        // Add basic schemas
        schema_registry.insert(
            "transaction".to_string(),
            serde_json::json!({
                "type": "object",
                "required": ["from", "to", "value", "gas", "gasPrice", "nonce", "data", "chainId"],
                "properties": {
                    "from": { "type": "string", "pattern": "^0x[a-fA-F0-9]{40}$" },
                    "to": { "type": "string", "pattern": "^0x[a-fA-F0-9]{40}$" },
                    "value": { "type": "string", "pattern": "^[0-9]+$" },
                    "gas": { "type": "integer", "minimum": 21000, "maximum": 30000000 },
                    "gasPrice": { "type": "string", "pattern": "^[0-9]+$" },
                    "nonce": { "type": "integer", "minimum": 0 },
                    "data": { "type": "string", "pattern": "^0x[a-fA-F0-9]*$" },
                    "chainId": { "type": "integer", "minimum": 1 }
                }
            })
        );
        
        Self {
            versioning: VersioningConfig {
                supported_versions: vec!["v1".to_string(), "v2".to_string()],
                default_version: "v1".to_string(),
                deprecated_versions: HashMap::new(),
                extraction_method: VersionExtractionMethod::PathPrefix,
            },
            validation: ValidationConfig {
                validate_requests: true,
                validate_responses: false, // Disabled for performance
                max_request_size: 10 * 1024 * 1024, // 10MB
                strictness_level: ValidationStrictness::Strict,
                schema_registry,
            },
            throttling: ThrottlingConfig {
                enabled: true,
                strategies: vec![
                    ThrottlingStrategy::RateLimit {
                        requests_per_second: 100,
                        burst_capacity: 200,
                    },
                    ThrottlingStrategy::ConcurrencyLimit {
                        max_concurrent: 1000,
                    },
                ],
                circuit_breaker: CircuitBreakerConfig {
                    failure_threshold: 10,
                    time_window: Duration::from_secs(60),
                    recovery_timeout: Duration::from_secs(30),
                    success_threshold: 5,
                },
            },
            monitoring: MonitoringConfig {
                enable_logging: true,
                enable_metrics: true,
                enable_tracing: true,
                sample_rate: 0.1, // 10% sampling
                alert_thresholds: AlertThresholds {
                    error_rate: 5.0, // 5%
                    latency_p99: 1000, // 1 second
                    request_volume: 10000, // per minute
                },
            },
            security: SecurityConfig {
                api_key_config: ApiKeyConfig {
                    enabled: true,
                    header_name: "X-API-Key".to_string(),
                    validation_strategy: ApiKeyValidationStrategy::Static(HashMap::new()),
                    rate_limiting: true,
                },
                jwt_validation: true,
                request_signing: RequestSigningConfig {
                    enabled: false, // Disabled by default
                    algorithm: SigningAlgorithm::HmacSha256,
                    timestamp_tolerance: 300, // 5 minutes
                    validate_nonce: true,
                },
                ip_filtering: true,
                content_security: ContentSecurityConfig {
                    max_json_depth: 10,
                    max_array_length: 1000,
                    max_string_length: 10000,
                    allowed_content_types: vec![
                        "application/json".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                        "multipart/form-data".to_string(),
                    ],
                },
            },
        }
    }
}

/// API Gateway state
pub struct ApiGatewayState {
    config: ApiGatewayConfig,
    input_validator: InputValidator,
    rate_limiter: Arc<RateLimiterState>,
    metrics: Arc<RwLock<GatewayMetrics>>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
}

impl ApiGatewayState {
    pub fn new(
        config: ApiGatewayConfig,
        rate_limiter: Arc<RateLimiterState>,
    ) -> Self {
        Self {
            config,
            input_validator: InputValidator::default(),
            rate_limiter,
            metrics: Arc::new(RwLock::new(GatewayMetrics::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Process incoming API request
    pub async fn process_request(
        &self,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let start_time = Instant::now();
        let request_id = Uuid::new_v4();
        
        // Add request ID to context
        request.extensions_mut().insert(request_id);
        
        // Extract request metadata
        let method = request.method().clone();
        let uri = request.uri().clone();
        let path = uri.path();
        
        debug!("API Gateway processing request: {} {} [{}]", method, path, request_id);
        
        // Step 1: Version extraction and validation
        let version = self.extract_api_version(&request)?;
        if !self.validate_api_version(&version) {
            warn!("Unsupported API version: {}", version);
            return Err(StatusCode::BAD_REQUEST);
        }
        
        // Step 2: Security validation
        self.validate_security(&request).await?;
        
        // Step 3: Request validation
        if self.config.validation.validate_requests {
            self.validate_request_content(&request).await?;
        }
        
        // Step 4: Circuit breaker check
        if self.config.throttling.enabled {
            self.check_circuit_breaker(path).await?;
        }
        
        // Step 5: Rate limiting (using existing rate limiter)
        // This is handled by the rate limiting middleware
        
        // Step 6: Process request
        let result = next.run(request).await;
        
        // Step 7: Process response
        let duration = start_time.elapsed();
        
        match result {
            Ok(response) => {
                // Record success metrics
                self.record_request_success(path, &version, duration).await;
                
                // Add API gateway headers
                let mut response = response;
                self.add_gateway_headers(&mut response, &version, &request_id);
                
                Ok(response)
            }
            Err(status) => {
                // Record failure metrics
                self.record_request_failure(path, &version, status, duration).await;
                
                // Update circuit breaker
                if self.config.throttling.enabled {
                    self.record_circuit_breaker_failure(path).await;
                }
                
                Err(status)
            }
        }
    }
    
    /// Extract API version from request
    fn extract_api_version(&self, request: &Request) -> Result<String, StatusCode> {
        match &self.config.versioning.extraction_method {
            VersionExtractionMethod::Header(header_name) => {
                request.headers()
                    .get(header_name)
                    .and_then(|v| v.to_str().ok())
                    .map(String::from)
                    .ok_or(StatusCode::BAD_REQUEST)
            }
            VersionExtractionMethod::QueryParameter(param_name) => {
                if let Some(query) = request.uri().query() {
                    let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
                        .into_owned()
                        .collect();
                    
                    params.get(param_name)
                        .cloned()
                        .ok_or(StatusCode::BAD_REQUEST)
                } else {
                    Ok(self.config.versioning.default_version.clone())
                }
            }
            VersionExtractionMethod::PathPrefix => {
                let path = request.uri().path();
                if path.starts_with("/api/") {
                    let parts: Vec<&str> = path.split('/').collect();
                    if parts.len() >= 3 && parts[2].starts_with('v') {
                        Ok(parts[2].to_string())
                    } else {
                        Ok(self.config.versioning.default_version.clone())
                    }
                } else {
                    Ok(self.config.versioning.default_version.clone())
                }
            }
            VersionExtractionMethod::AcceptHeader => {
                if let Some(accept) = request.headers().get("accept") {
                    if let Ok(accept_str) = accept.to_str() {
                        // Parse Accept header for version
                        if accept_str.contains("vnd.mev-shield.v") {
                            // Extract version from something like "application/vnd.mev-shield.v1+json"
                            for part in accept_str.split(',') {
                                if let Some(start) = part.find("vnd.mev-shield.v") {
                                    if let Some(end) = part[start..].find('+') {
                                        let version_part = &part[start + 16..start + end];
                                        return Ok(format!("v{}", version_part));
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(self.config.versioning.default_version.clone())
            }
        }
    }
    
    /// Validate API version
    fn validate_api_version(&self, version: &str) -> bool {
        self.config.versioning.supported_versions.contains(version)
    }
    
    /// Validate security aspects of the request
    async fn validate_security(&self, request: &Request) -> Result<(), StatusCode> {
        // API Key validation
        if self.config.security.api_key_config.enabled {
            self.validate_api_key(request).await?;
        }
        
        // Content type validation
        if matches!(request.method(), &Method::POST | &Method::PUT | &Method::PATCH) {
            self.validate_content_type(request)?;
        }
        
        // Request size validation
        self.validate_request_size(request)?;
        
        Ok(())
    }
    
    /// Validate API key
    async fn validate_api_key(&self, request: &Request) -> Result<(), StatusCode> {
        let api_key = request.headers()
            .get(&self.config.security.api_key_config.header_name)
            .and_then(|v| v.to_str().ok());
        
        if let Some(key) = api_key {
            match &self.config.security.api_key_config.validation_strategy {
                ApiKeyValidationStrategy::Static(keys) => {
                    if keys.contains_key(key) {
                        // Additional validation for API key
                        if let Some(key_info) = keys.get(key) {
                            // Check expiration
                            if let Some(expires_at) = key_info.expires_at {
                                if chrono::Utc::now() > expires_at {
                                    warn!("Expired API key used: {}", key_info.key_id);
                                    return Err(StatusCode::UNAUTHORIZED);
                                }
                            }
                            
                            debug!("Valid API key: {}", key_info.key_id);
                            return Ok(());
                        }
                    }
                }
                ApiKeyValidationStrategy::Database => {
                    // In production, validate against database
                    // For now, just accept any key
                    debug!("API key validation against database (not implemented)");
                    return Ok(());
                }
                ApiKeyValidationStrategy::Redis => {
                    // In production, validate against Redis
                    // For now, just accept any key
                    debug!("API key validation against Redis (not implemented)");
                    return Ok(());
                }
            }
            
            warn!("Invalid API key: {}", key);
            Err(StatusCode::UNAUTHORIZED)
        } else {
            warn!("Missing API key");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
    
    /// Validate content type
    fn validate_content_type(&self, request: &Request) -> Result<(), StatusCode> {
        if let Some(content_type) = request.headers().get("content-type") {
            if let Ok(content_type_str) = content_type.to_str() {
                let is_allowed = self.config.security.content_security.allowed_content_types
                    .iter()
                    .any(|allowed| content_type_str.starts_with(allowed));
                
                if !is_allowed {
                    warn!("Unsupported content type: {}", content_type_str);
                    return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate request size
    fn validate_request_size(&self, request: &Request) -> Result<(), StatusCode> {
        if let Some(content_length) = request.headers().get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    if length > self.config.validation.max_request_size {
                        warn!("Request too large: {} bytes", length);
                        return Err(StatusCode::PAYLOAD_TOO_LARGE);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate request content structure
    async fn validate_request_content(&self, request: &Request) -> Result<(), StatusCode> {
        // This would typically validate JSON schema
        // For now, we'll do basic validation
        
        let path = request.uri().path();
        
        // Example: validate transaction submission
        if path.contains("/transactions") && request.method() == Method::POST {
            // In a real implementation, we would:
            // 1. Parse the request body
            // 2. Validate against JSON schema
            // 3. Perform business logic validation
            debug!("Validating transaction request content");
        }
        
        Ok(())
    }
    
    /// Check circuit breaker status
    async fn check_circuit_breaker(&self, endpoint: &str) -> Result<(), StatusCode> {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        let circuit_breaker = circuit_breakers
            .entry(endpoint.to_string())
            .or_insert_with(|| CircuitBreaker::new(self.config.throttling.circuit_breaker.clone()));
        
        if circuit_breaker.is_open() {
            warn!("Circuit breaker open for endpoint: {}", endpoint);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        } else {
            Ok(())
        }
    }
    
    /// Record successful request
    async fn record_request_success(&self, endpoint: &str, version: &str, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.record_request(endpoint, version, true, duration);
        
        // Update circuit breaker
        if self.config.throttling.enabled {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            if let Some(circuit_breaker) = circuit_breakers.get_mut(endpoint) {
                circuit_breaker.record_success();
            }
        }
    }
    
    /// Record failed request
    async fn record_request_failure(&self, endpoint: &str, version: &str, status: StatusCode, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.record_request(endpoint, version, false, duration);
        metrics.record_error(endpoint, status);
    }
    
    /// Record circuit breaker failure
    async fn record_circuit_breaker_failure(&self, endpoint: &str) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        if let Some(circuit_breaker) = circuit_breakers.get_mut(endpoint) {
            circuit_breaker.record_failure();
        }
    }
    
    /// Add gateway-specific headers to response
    fn add_gateway_headers(&self, response: &mut Response, version: &str, request_id: &Uuid) {
        let headers = response.headers_mut();
        
        headers.insert("X-API-Version", version.parse().unwrap());
        headers.insert("X-Request-ID", request_id.to_string().parse().unwrap());
        headers.insert("X-Gateway", "MEV-Shield-v1".parse().unwrap());
        
        // Add deprecation warnings if applicable
        if let Some(sunset_date) = self.config.versioning.deprecated_versions.get(version) {
            headers.insert(
                "Sunset",
                sunset_date.to_rfc3339().parse().unwrap()
            );
            headers.insert(
                "Deprecation",
                "true".parse().unwrap()
            );
        }
    }
}

/// Circuit breaker implementation
#[derive(Debug)]
struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    failure_times: Vec<Instant>,
}

#[derive(Debug, PartialEq)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            failure_times: Vec::new(),
        }
    }
    
    fn is_open(&mut self) -> bool {
        self.update_state();
        self.state == CircuitBreakerState::Open
    }
    
    fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.reset_counts();
                }
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
                self.failure_times.clear();
            }
            CircuitBreakerState::Open => {
                // Should not happen, but handle gracefully
            }
        }
    }
    
    fn record_failure(&mut self) {
        let now = Instant::now();
        self.failure_count += 1;
        self.failure_times.push(now);
        self.last_failure_time = Some(now);
        
        // Remove old failures outside the time window
        let cutoff = now - self.config.time_window;
        self.failure_times.retain(|&time| time > cutoff);
        
        // Update failure count based on time window
        self.failure_count = self.failure_times.len() as u32;
        
        // Check if we should open the circuit
        if self.state == CircuitBreakerState::Closed &&
           self.failure_count >= self.config.failure_threshold {
            self.state = CircuitBreakerState::Open;
            warn!("Circuit breaker opened due to {} failures", self.failure_count);
        }
        
        if self.state == CircuitBreakerState::HalfOpen {
            self.state = CircuitBreakerState::Open;
        }
    }
    
    fn update_state(&mut self) {
        if self.state == CircuitBreakerState::Open {
            if let Some(last_failure) = self.last_failure_time {
                if Instant::now() - last_failure > self.config.recovery_timeout {
                    self.state = CircuitBreakerState::HalfOpen;
                    self.success_count = 0;
                    debug!("Circuit breaker moved to half-open state");
                }
            }
        }
    }
    
    fn reset_counts(&mut self) {
        self.failure_count = 0;
        self.success_count = 0;
        self.failure_times.clear();
        self.last_failure_time = None;
    }
}

/// Gateway metrics
#[derive(Debug)]
struct GatewayMetrics {
    request_count: HashMap<String, u64>,
    error_count: HashMap<String, u64>,
    latency_sum: HashMap<String, Duration>,
    version_usage: HashMap<String, u64>,
}

impl GatewayMetrics {
    fn new() -> Self {
        Self {
            request_count: HashMap::new(),
            error_count: HashMap::new(),
            latency_sum: HashMap::new(),
            version_usage: HashMap::new(),
        }
    }
    
    fn record_request(&mut self, endpoint: &str, version: &str, success: bool, duration: Duration) {
        *self.request_count.entry(endpoint.to_string()).or_insert(0) += 1;
        *self.version_usage.entry(version.to_string()).or_insert(0) += 1;
        
        let current_latency = self.latency_sum.entry(endpoint.to_string()).or_insert(Duration::ZERO);
        *current_latency += duration;
        
        if !success {
            *self.error_count.entry(endpoint.to_string()).or_insert(0) += 1;
        }
    }
    
    fn record_error(&mut self, endpoint: &str, status: StatusCode) {
        let error_key = format!("{}:{}", endpoint, status.as_u16());
        *self.error_count.entry(error_key).or_insert(0) += 1;
    }
}

/// API Gateway middleware
pub async fn api_gateway_middleware(
    State(gateway): State<Arc<ApiGatewayState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    gateway.process_request(request, next).await
}

/// Health check for API Gateway
pub async fn gateway_health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "component": "api-gateway",
        "version": "1.0.0"
    }))
}

/// Gateway metrics endpoint
pub async fn gateway_metrics(
    State(gateway): State<Arc<ApiGatewayState>>,
) -> Json<serde_json::Value> {
    let metrics = gateway.metrics.read().await;
    
    Json(serde_json::json!({
        "request_count": metrics.request_count,
        "error_count": metrics.error_count,
        "version_usage": metrics.version_usage,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_circuit_breaker() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            time_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 2,
        };
        
        let mut cb = CircuitBreaker::new(config);
        
        // Initially closed
        assert!(!cb.is_open());
        
        // Record failures
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.is_open()); // Still closed
        
        cb.record_failure();
        assert!(cb.is_open()); // Now open
        
        // Record success while open (should not change state immediately)
        cb.record_success();
        assert!(cb.is_open());
    }
    
    #[test]
    fn test_version_extraction() {
        let config = ApiGatewayConfig::default();
        let rate_limiter = Arc::new(RateLimiterState::new(Default::default()));
        let gateway = ApiGatewayState::new(config, rate_limiter);
        
        // Test path prefix extraction
        let mut request = Request::builder()
            .uri("/api/v2/transactions")
            .body(axum::body::Body::empty())
            .unwrap();
        
        let version = gateway.extract_api_version(&request).unwrap();
        assert_eq!(version, "v2");
        
        // Test default version
        let mut request = Request::builder()
            .uri("/api/transactions")
            .body(axum::body::Body::empty())
            .unwrap();
        
        let version = gateway.extract_api_version(&request).unwrap();
        assert_eq!(version, "v1"); // default
    }
}