//! MEV Shield REST API Server
//! 
//! Provides HTTP/REST API endpoints for MEV Shield functionality including
//! transaction submission, status checking, analytics, and administration.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};
use uuid::Uuid;
use anyhow::Result;

use crate::{
    config::ApiConfig,
    core::MEVShieldCore,
    types::*,
};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    mev_shield: Arc<MEVShieldCore>,
    config: ApiConfig,
}

/// API server for MEV Shield
pub struct ApiServer {
    app_state: AppState,
    config: ApiConfig,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(
        mev_shield: Arc<MEVShieldCore>,
        config: ApiConfig,
    ) -> Self {
        let app_state = AppState {
            mev_shield,
            config: config.clone(),
        };
        
        Self {
            app_state,
            config,
        }
    }
    
    /// Run the API server
    pub async fn run(self) -> Result<()> {
        let app = self.create_router();
        
        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        info!("Starting MEV Shield API server on {}", addr);
        
        let listener = TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    /// Create the router with all endpoints
    fn create_router(&self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        
        Router::new()
            // Transaction endpoints
            .route("/api/v1/transactions", post(submit_transaction))
            .route("/api/v1/transactions/:id", get(get_transaction_status))
            .route("/api/v1/transactions", get(list_transactions))
            
            // Analytics endpoints
            .route("/api/v1/analytics/mev", get(get_mev_analytics))
            .route("/api/v1/analytics/user/:address", get(get_user_analytics))
            .route("/api/v1/analytics/network/:chain_id", get(get_network_analytics))
            
            // System endpoints
            .route("/api/v1/health", get(health_check))
            .route("/api/v1/status", get(system_status))
            .route("/api/v1/metrics", get(get_metrics))
            
            // Admin endpoints (if enabled)
            .route("/api/v1/admin/config", get(get_config))
            .route("/api/v1/admin/validators", get(get_validators))
            
            // Apply middleware
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(cors)
                    .layer(middleware::from_fn(timeout_middleware))
                    .layer(middleware::from_fn_with_state(
                        self.app_state.clone(),
                        auth_middleware,
                    )),
            )
            .with_state(self.app_state)
    }
}

// Request/Response types

/// Request to submit a protected transaction
#[derive(Debug, Deserialize)]
pub struct SubmitTransactionRequest {
    pub transaction: TransactionData,
    pub protection: ProtectionConfig,
    pub chain_id: u64,
}

/// Transaction data in the API
#[derive(Debug, Deserialize)]
pub struct TransactionData {
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: u64,
    #[serde(rename = "gasPrice")]
    pub gas_price: String,
    pub nonce: u64,
    pub data: String,
}

/// Response for transaction submission
#[derive(Debug, Serialize)]
pub struct SubmitTransactionResponse {
    pub success: bool,
    pub data: Option<TransactionSubmissionData>,
    pub error: Option<String>,
}

/// Transaction submission data
#[derive(Debug, Serialize)]
pub struct TransactionSubmissionData {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "encryptedHash")]
    pub encrypted_hash: String,
    #[serde(rename = "estimatedExecution")]
    pub estimated_execution: String,
    #[serde(rename = "protectionLevel")]
    pub protection_level: String,
    pub fees: TransactionFees,
}

/// Transaction fees information
#[derive(Debug, Serialize)]
pub struct TransactionFees {
    #[serde(rename = "protectionFee")]
    pub protection_fee: String,
    #[serde(rename = "estimatedGas")]
    pub estimated_gas: String,
}

/// Query parameters for transactions list
#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub status: Option<String>,
    pub chain_id: Option<u64>,
}

/// MEV analytics response
#[derive(Debug, Serialize)]
pub struct MEVAnalyticsResponse {
    pub success: bool,
    pub data: MEVAnalyticsData,
}

/// MEV analytics data
#[derive(Debug, Serialize)]
pub struct MEVAnalyticsData {
    pub timeframe: String,
    #[serde(rename = "totalMevCaptured")]
    pub total_mev_captured: String,
    #[serde(rename = "totalDistributed")]
    pub total_distributed: String,
    #[serde(rename = "protectedTransactions")]
    pub protected_transactions: u64,
    #[serde(rename = "averageSavings")]
    pub average_savings: String,
    #[serde(rename = "mevTypes")]
    pub mev_types: HashMap<String, String>,
}

/// System status response
#[derive(Debug, Serialize)]
pub struct SystemStatusResponse {
    pub success: bool,
    pub data: SystemStatusData,
}

/// System status data
#[derive(Debug, Serialize)]
pub struct SystemStatusData {
    pub version: String,
    pub uptime: String,
    pub status: String,
    pub services: HashMap<String, ServiceStatus>,
}

/// Service status
#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub status: String,
    pub latency: Option<String>,
    #[serde(rename = "lastCheck")]
    pub last_check: String,
}

// API Handler Functions

/// Submit a transaction for MEV protection
async fn submit_transaction(
    State(state): State<AppState>,
    Json(request): Json<SubmitTransactionRequest>,
) -> Result<Json<SubmitTransactionResponse>, StatusCode> {
    info!("Received transaction submission request");
    
    // Convert API request to internal types
    let transaction = match convert_transaction_data(request.transaction, request.chain_id) {
        Ok(tx) => tx,
        Err(e) => {
            error!("Invalid transaction data: {}", e);
            return Ok(Json(SubmitTransactionResponse {
                success: false,
                data: None,
                error: Some(format!("Invalid transaction: {}", e)),
            }));
        }
    };
    
    // Submit to MEV Shield
    match state.mev_shield
        .submit_protected_transaction(transaction, request.protection)
        .await
    {
        Ok(result) => {
            info!("Transaction submitted successfully: {}", result.transaction_id);
            
            Ok(Json(SubmitTransactionResponse {
                success: true,
                data: Some(TransactionSubmissionData {
                    transaction_id: result.transaction_id.to_string(),
                    encrypted_hash: result.encrypted_hash.to_string(),
                    estimated_execution: result.execution_schedule.estimated_execution_time.to_rfc3339(),
                    protection_level: format!("{:?}", request.protection.level),
                    fees: TransactionFees {
                        protection_fee: "5000000000000000".to_string(), // 0.005 ETH
                        estimated_gas: "23000".to_string(),
                    },
                }),
                error: None,
            }))
        }
        Err(e) => {
            error!("Transaction submission failed: {}", e);
            Ok(Json(SubmitTransactionResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

/// Get transaction status by ID
async fn get_transaction_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Getting transaction status for ID: {}", id);
    
    let transaction_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    match state.mev_shield.get_transaction_status(transaction_id).await {
        Ok(status) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "id": status.id.to_string(),
                "status": format!("{:?}", status.status),
                "blockNumber": status.block_number,
                "blockHash": status.block_hash.map(|h| h.to_string()),
                "transactionHash": status.transaction_hash.map(|h| h.to_string()),
                "protection": {
                    "mevDetected": status.protection_details.mev_detected,
                    "savingsAmount": status.protection_details.savings_amount.to_string(),
                    "executionTime": status.protection_details.execution_time.map(|t| t.to_rfc3339()),
                }
            }
        }))),
        Err(e) => {
            error!("Failed to get transaction status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List transactions with filtering
async fn list_transactions(
    State(_state): State<AppState>,
    Query(params): Query<TransactionQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Listing transactions with params: {:?}", params);
    
    // This would typically query the database
    // For now, return a mock response
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "transactions": [],
            "total": 0,
            "offset": params.offset.unwrap_or(0),
            "limit": params.limit.unwrap_or(10)
        }
    })))
}

/// Get MEV analytics
async fn get_mev_analytics(
    State(_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<MEVAnalyticsResponse>, StatusCode> {
    info!("Getting MEV analytics with params: {:?}", params);
    
    let timeframe = params.get("timeframe").unwrap_or(&"24h".to_string()).clone();
    
    // Mock analytics data
    let mut mev_types = HashMap::new();
    mev_types.insert("sandwich".to_string(), "8000000000000000000".to_string());
    mev_types.insert("frontrun".to_string(), "4000000000000000000".to_string());
    mev_types.insert("arbitrage".to_string(), "3000000000000000000".to_string());
    
    Ok(Json(MEVAnalyticsResponse {
        success: true,
        data: MEVAnalyticsData {
            timeframe,
            total_mev_captured: "15000000000000000000".to_string(),
            total_distributed: "12000000000000000000".to_string(),
            protected_transactions: 12567,
            average_savings: "500000000000000".to_string(),
            mev_types,
        },
    }))
}

/// Get user analytics
async fn get_user_analytics(
    State(_state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Getting user analytics for address: {}", address);
    
    // Mock user analytics
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "address": address,
            "totalTransactions": 156,
            "totalSavings": "2500000000000000000",
            "avgSavingsPerTx": "16025641025641025",
            "protectionLevel": "Maximum",
            "mevDetected": 23,
            "mevPrevented": 23
        }
    })))
}

/// Get network analytics
async fn get_network_analytics(
    State(_state): State<AppState>,
    Path(chain_id): Path<u64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Getting network analytics for chain ID: {}", chain_id);
    
    // Mock network analytics
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "chainId": chain_id,
            "networkName": if chain_id == 1 { "Ethereum" } else { "Unknown" },
            "totalTransactions": 1567890,
            "totalMevSaved": "125000000000000000000",
            "avgBlockTime": "12.0",
            "mevDetectionRate": 0.15
        }
    })))
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Get system status
async fn system_status(
    State(_state): State<AppState>,
) -> Result<Json<SystemStatusResponse>, StatusCode> {
    let mut services = HashMap::new();
    
    services.insert(
        "encryption".to_string(),
        ServiceStatus {
            status: "healthy".to_string(),
            latency: Some("45ms".to_string()),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
    );
    
    services.insert(
        "detection".to_string(),
        ServiceStatus {
            status: "healthy".to_string(),
            latency: Some("12ms".to_string()),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
    );
    
    services.insert(
        "ordering".to_string(),
        ServiceStatus {
            status: "healthy".to_string(),
            latency: Some("89ms".to_string()),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
    );
    
    Ok(Json(SystemStatusResponse {
        success: true,
        data: SystemStatusData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: "72h 14m 32s".to_string(),
            status: "operational".to_string(),
            services,
        },
    }))
}

/// Get metrics (Prometheus format)
async fn get_metrics() -> Result<String, StatusCode> {
    // Return Prometheus-format metrics
    Ok(format!(
        r#"# HELP mev_shield_transactions_total Total number of protected transactions
# TYPE mev_shield_transactions_total counter
mev_shield_transactions_total 12567

# HELP mev_shield_mev_detected_total Total number of MEV attacks detected
# TYPE mev_shield_mev_detected_total counter
mev_shield_mev_detected_total 1876

# HELP mev_shield_mev_savings_total Total MEV savings in wei
# TYPE mev_shield_mev_savings_total counter
mev_shield_mev_savings_total 125000000000000000000

# HELP mev_shield_encryption_duration_seconds Time spent encrypting transactions
# TYPE mev_shield_encryption_duration_seconds histogram
mev_shield_encryption_duration_seconds_bucket{{le="0.01"}} 1000
mev_shield_encryption_duration_seconds_bucket{{le="0.05"}} 5000
mev_shield_encryption_duration_seconds_bucket{{le="0.1"}} 8000
mev_shield_encryption_duration_seconds_bucket{{le="+Inf"}} 10000
mev_shield_encryption_duration_seconds_sum 456.789
mev_shield_encryption_duration_seconds_count 10000

# HELP mev_shield_system_uptime_seconds System uptime in seconds
# TYPE mev_shield_system_uptime_seconds gauge
mev_shield_system_uptime_seconds 260672
"#
    ))
}

/// Get configuration (admin only)
async fn get_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !state.config.auth.admin_endpoints {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Return sanitized configuration
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "version": env!("CARGO_PKG_VERSION"),
            "api": {
                "port": state.config.port,
                "corsEnabled": state.config.cors_enabled,
                "rateLimiting": state.config.rate_limiting
            }
        }
    })))
}

/// Get validator information (admin only)
async fn get_validators(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !state.config.auth.admin_endpoints {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Mock validator data
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "totalValidators": 100,
            "activeValidators": 98,
            "threshold": 67,
            "validators": [
                {
                    "id": 0,
                    "status": "active",
                    "lastSeen": chrono::Utc::now().to_rfc3339(),
                    "reputation": 0.95
                }
            ]
        }
    })))
}

// Middleware

/// Authentication middleware
async fn auth_middleware(
    State(_state): State<AppState>,
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    // For now, allow all requests
    // In production, this would validate API keys or JWT tokens
    Ok(next.run(request).await)
}

/// Request timeout middleware
async fn timeout_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let timeout_duration = Duration::from_secs(30);
    
    match tokio::time::timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
    }
}

// Helper functions

/// Convert API transaction data to internal Transaction type
fn convert_transaction_data(
    data: TransactionData,
    chain_id: u64,
) -> Result<Transaction, Box<dyn std::error::Error>> {
    let from = Address::from_str(&data.from)?;
    let to = Address::from_str(&data.to)?;
    let value = data.value.parse::<num_bigint::BigUint>()?;
    let gas_price = data.gas_price.parse::<num_bigint::BigUint>()?;
    
    let tx_data = if data.data.starts_with("0x") {
        hex::decode(&data.data[2..])?
    } else {
        hex::decode(&data.data)?
    };
    
    Ok(Transaction {
        from,
        to,
        value,
        gas: data.gas,
        gas_price,
        gas_used: 0, // Will be set after execution
        nonce: data.nonce,
        data: tx_data,
        chain_id,
        submission_time: chrono::Utc::now(),
    })
}

/// Main function to start the API server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = MEVShieldConfig::default();
    
    // Create MEV Shield instance
    let mev_shield = Arc::new(MEVShield::new(config.clone()).await?);
    
    // Create and start API server
    let api_server = ApiServer::new(mev_shield, config.api).await?;
    api_server.start().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    
    async fn create_test_server() -> TestServer {
        let config = MEVShieldConfig::default();
        let mev_shield = Arc::new(MEVShield::new(config.clone()).await.unwrap());
        let api_server = ApiServer::new(mev_shield, config.api).await.unwrap();
        
        TestServer::new(api_server.create_router()).unwrap()
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let server = create_test_server().await;
        
        let response = server.get("/api/v1/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "healthy");
    }
    
    #[tokio::test]
    async fn test_system_status() {
        let server = create_test_server().await;
        
        let response = server.get("/api/v1/status").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: SystemStatusResponse = response.json();
        assert!(body.success);
        assert_eq!(body.data.status, "operational");
    }
    
    #[tokio::test]
    async fn test_mev_analytics() {
        let server = create_test_server().await;
        
        let response = server.get("/api/v1/analytics/mev?timeframe=24h").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: MEVAnalyticsResponse = response.json();
        assert!(body.success);
        assert_eq!(body.data.timeframe, "24h");
    }
}