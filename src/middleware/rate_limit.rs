//! Comprehensive Rate Limiting Middleware
//!
//! Implements multi-tier rate limiting with per-user, per-IP, and per-endpoint
//! limits to prevent abuse and DDoS attacks.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, warn, error};
use crate::auth::models::Claims;

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Per-IP limits
    pub ip_limits: IpLimits,
    /// Per-user limits
    pub user_limits: UserLimits,
    /// Per-endpoint limits
    pub endpoint_limits: HashMap<String, EndpointLimit>,
    /// Global limits
    pub global_limits: GlobalLimits,
    /// Window size for rate limiting
    pub window_size: Duration,
    /// Enable burst allowance
    pub allow_burst: bool,
    /// Burst multiplier
    pub burst_multiplier: f64,
}

#[derive(Debug, Clone)]
pub struct IpLimits {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub concurrent_connections: u32,
}

#[derive(Debug, Clone)]
pub struct UserLimits {
    pub authenticated_requests_per_minute: u32,
    pub authenticated_requests_per_hour: u32,
    pub admin_requests_per_minute: u32,
    pub admin_requests_per_hour: u32,
}

#[derive(Debug, Clone)]
pub struct EndpointLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub concurrent_requests: u32,
    pub require_auth: bool,
    pub admin_only: bool,
}

#[derive(Debug, Clone)]
pub struct GlobalLimits {
    pub total_requests_per_second: u32,
    pub total_requests_per_minute: u32,
    pub max_concurrent_requests: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut endpoint_limits = HashMap::new();
        
        // Authentication endpoints
        endpoint_limits.insert("/auth/login".to_string(), EndpointLimit {
            requests_per_minute: 5,
            requests_per_hour: 20,
            concurrent_requests: 2,
            require_auth: false,
            admin_only: false,
        });
        
        endpoint_limits.insert("/auth/register".to_string(), EndpointLimit {
            requests_per_minute: 3,
            requests_per_hour: 10,
            concurrent_requests: 1,
            require_auth: false,
            admin_only: false,
        });
        
        // Transaction endpoints
        endpoint_limits.insert("/api/v1/transactions".to_string(), EndpointLimit {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            concurrent_requests: 10,
            require_auth: true,
            admin_only: false,
        });
        
        // Admin endpoints
        endpoint_limits.insert("/api/v1/admin/".to_string(), EndpointLimit {
            requests_per_minute: 30,
            requests_per_hour: 500,
            concurrent_requests: 5,
            require_auth: true,
            admin_only: true,
        });
        
        // Public endpoints
        endpoint_limits.insert("/api/v1/health".to_string(), EndpointLimit {
            requests_per_minute: 120,
            requests_per_hour: 2000,
            concurrent_requests: 20,
            require_auth: false,
            admin_only: false,
        });
        
        Self {
            ip_limits: IpLimits {
                requests_per_minute: 120,
                requests_per_hour: 2000,
                concurrent_connections: 20,
            },
            user_limits: UserLimits {
                authenticated_requests_per_minute: 300,
                authenticated_requests_per_hour: 5000,
                admin_requests_per_minute: 600,
                admin_requests_per_hour: 10000,
            },
            endpoint_limits,
            global_limits: GlobalLimits {
                total_requests_per_second: 1000,
                total_requests_per_minute: 30000,
                max_concurrent_requests: 500,
            },
            window_size: Duration::from_secs(60),
            allow_burst: true,
            burst_multiplier: 1.5,
        }
    }
}

/// Rate limit entry tracking request counts and timing
#[derive(Debug, Clone)]
struct RateLimitEntry {
    requests: Vec<Instant>,
    concurrent_requests: u32,
    last_request: Instant,
    burst_allowance: f64,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            concurrent_requests: 0,
            last_request: Instant::now(),
            burst_allowance: 0.0,
        }
    }
    
    fn add_request(&mut self, now: Instant) {
        self.requests.push(now);
        self.last_request = now;
        self.concurrent_requests += 1;
    }
    
    fn finish_request(&mut self) {
        if self.concurrent_requests > 0 {
            self.concurrent_requests -= 1;
        }
    }
    
    fn cleanup_old_requests(&mut self, window: Duration) {
        let cutoff = Instant::now() - window;
        self.requests.retain(|&request_time| request_time > cutoff);
    }
    
    fn request_count(&self, window: Duration) -> usize {
        let cutoff = Instant::now() - window;
        self.requests.iter().filter(|&&time| time > cutoff).count()
    }
    
    fn update_burst_allowance(&mut self, rate: f64, burst_multiplier: f64) {
        let now = Instant::now();
        let time_passed = now.duration_since(self.last_request).as_secs_f64();
        
        // Add tokens based on time passed
        self.burst_allowance += time_passed * rate;
        
        // Cap at maximum burst
        let max_burst = rate * burst_multiplier;
        if self.burst_allowance > max_burst {
            self.burst_allowance = max_burst;
        }
    }
    
    fn can_consume_burst(&mut self, tokens: f64) -> bool {
        if self.burst_allowance >= tokens {
            self.burst_allowance -= tokens;
            true
        } else {
            false
        }
    }
}

/// Rate limiter state
pub struct RateLimiterState {
    config: RateLimitConfig,
    ip_entries: Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>,
    user_entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    endpoint_entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    global_entry: Arc<RwLock<RateLimitEntry>>,
}

impl RateLimiterState {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            ip_entries: Arc::new(RwLock::new(HashMap::new())),
            user_entries: Arc::new(RwLock::new(HashMap::new())),
            endpoint_entries: Arc::new(RwLock::new(HashMap::new())),
            global_entry: Arc::new(RwLock::new(RateLimitEntry::new())),
        }
    }
    
    /// Check if request should be rate limited
    pub async fn check_rate_limit(
        &self,
        ip: IpAddr,
        user_id: Option<&str>,
        endpoint: &str,
        is_admin: bool,
    ) -> Result<RateLimitResult, RateLimitError> {
        let now = Instant::now();
        
        // Check global limits first
        if let Err(e) = self.check_global_limits().await {
            return Err(e);
        }
        
        // Check IP-based limits
        if let Err(e) = self.check_ip_limits(ip).await {
            return Err(e);
        }
        
        // Check user-based limits if authenticated
        if let Some(user_id) = user_id {
            if let Err(e) = self.check_user_limits(user_id, is_admin).await {
                return Err(e);
            }
        }
        
        // Check endpoint-specific limits
        if let Err(e) = self.check_endpoint_limits(endpoint, user_id.is_some(), is_admin).await {
            return Err(e);
        }
        
        // All checks passed, record the request
        self.record_request(ip, user_id, endpoint).await;
        
        Ok(RateLimitResult {
            allowed: true,
            remaining: self.calculate_remaining(ip, user_id, endpoint).await,
            reset_time: now + self.config.window_size,
            retry_after: None,
        })
    }
    
    async fn check_global_limits(&self) -> Result<(), RateLimitError> {
        let mut global_entry = self.global_entry.write().await;
        
        global_entry.cleanup_old_requests(Duration::from_secs(60));
        
        // Check per-minute global limit
        let requests_per_minute = global_entry.request_count(Duration::from_secs(60));
        if requests_per_minute >= self.config.global_limits.total_requests_per_minute as usize {
            return Err(RateLimitError::GlobalLimitExceeded {
                limit: self.config.global_limits.total_requests_per_minute,
                window: "minute".to_string(),
            });
        }
        
        // Check per-second global limit
        let requests_per_second = global_entry.request_count(Duration::from_secs(1));
        if requests_per_second >= self.config.global_limits.total_requests_per_second as usize {
            return Err(RateLimitError::GlobalLimitExceeded {
                limit: self.config.global_limits.total_requests_per_second,
                window: "second".to_string(),
            });
        }
        
        // Check concurrent requests
        if global_entry.concurrent_requests >= self.config.global_limits.max_concurrent_requests {
            return Err(RateLimitError::ConcurrentLimitExceeded {
                limit: self.config.global_limits.max_concurrent_requests,
            });
        }
        
        Ok(())
    }
    
    async fn check_ip_limits(&self, ip: IpAddr) -> Result<(), RateLimitError> {
        let mut ip_entries = self.ip_entries.write().await;
        let entry = ip_entries.entry(ip).or_insert_with(RateLimitEntry::new);
        
        entry.cleanup_old_requests(Duration::from_secs(3600)); // 1 hour
        
        // Check per-minute IP limit
        let requests_per_minute = entry.request_count(Duration::from_secs(60));
        if requests_per_minute >= self.config.ip_limits.requests_per_minute as usize {
            // Try burst allowance if enabled
            if self.config.allow_burst {
                let rate = self.config.ip_limits.requests_per_minute as f64 / 60.0;
                entry.update_burst_allowance(rate, self.config.burst_multiplier);
                
                if !entry.can_consume_burst(1.0) {
                    return Err(RateLimitError::IpLimitExceeded {
                        ip,
                        limit: self.config.ip_limits.requests_per_minute,
                        window: "minute".to_string(),
                    });
                }
            } else {
                return Err(RateLimitError::IpLimitExceeded {
                    ip,
                    limit: self.config.ip_limits.requests_per_minute,
                    window: "minute".to_string(),
                });
            }
        }
        
        // Check per-hour IP limit
        let requests_per_hour = entry.request_count(Duration::from_secs(3600));
        if requests_per_hour >= self.config.ip_limits.requests_per_hour as usize {
            return Err(RateLimitError::IpLimitExceeded {
                ip,
                limit: self.config.ip_limits.requests_per_hour,
                window: "hour".to_string(),
            });
        }
        
        // Check concurrent connections
        if entry.concurrent_requests >= self.config.ip_limits.concurrent_connections {
            return Err(RateLimitError::ConcurrentLimitExceeded {
                limit: self.config.ip_limits.concurrent_connections,
            });
        }
        
        Ok(())
    }
    
    async fn check_user_limits(&self, user_id: &str, is_admin: bool) -> Result<(), RateLimitError> {
        let mut user_entries = self.user_entries.write().await;
        let entry = user_entries.entry(user_id.to_string()).or_insert_with(RateLimitEntry::new);
        
        entry.cleanup_old_requests(Duration::from_secs(3600)); // 1 hour
        
        let (limit_per_minute, limit_per_hour) = if is_admin {
            (
                self.config.user_limits.admin_requests_per_minute,
                self.config.user_limits.admin_requests_per_hour,
            )
        } else {
            (
                self.config.user_limits.authenticated_requests_per_minute,
                self.config.user_limits.authenticated_requests_per_hour,
            )
        };
        
        // Check per-minute user limit
        let requests_per_minute = entry.request_count(Duration::from_secs(60));
        if requests_per_minute >= limit_per_minute as usize {
            return Err(RateLimitError::UserLimitExceeded {
                user_id: user_id.to_string(),
                limit: limit_per_minute,
                window: "minute".to_string(),
            });
        }
        
        // Check per-hour user limit
        let requests_per_hour = entry.request_count(Duration::from_secs(3600));
        if requests_per_hour >= limit_per_hour as usize {
            return Err(RateLimitError::UserLimitExceeded {
                user_id: user_id.to_string(),
                limit: limit_per_hour,
                window: "hour".to_string(),
            });
        }
        
        Ok(())
    }
    
    async fn check_endpoint_limits(
        &self,
        endpoint: &str,
        is_authenticated: bool,
        is_admin: bool,
    ) -> Result<(), RateLimitError> {
        // Find matching endpoint configuration
        let endpoint_config = self.find_endpoint_config(endpoint);
        
        if let Some(config) = endpoint_config {
            // Check authentication requirements
            if config.require_auth && !is_authenticated {
                return Err(RateLimitError::AuthenticationRequired);
            }
            
            if config.admin_only && !is_admin {
                return Err(RateLimitError::AdminRequired);
            }
            
            let mut endpoint_entries = self.endpoint_entries.write().await;
            let entry = endpoint_entries.entry(endpoint.to_string()).or_insert_with(RateLimitEntry::new);
            
            entry.cleanup_old_requests(Duration::from_secs(3600)); // 1 hour
            
            // Check per-minute endpoint limit
            let requests_per_minute = entry.request_count(Duration::from_secs(60));
            if requests_per_minute >= config.requests_per_minute as usize {
                return Err(RateLimitError::EndpointLimitExceeded {
                    endpoint: endpoint.to_string(),
                    limit: config.requests_per_minute,
                    window: "minute".to_string(),
                });
            }
            
            // Check per-hour endpoint limit
            let requests_per_hour = entry.request_count(Duration::from_secs(3600));
            if requests_per_hour >= config.requests_per_hour as usize {
                return Err(RateLimitError::EndpointLimitExceeded {
                    endpoint: endpoint.to_string(),
                    limit: config.requests_per_hour,
                    window: "hour".to_string(),
                });
            }
            
            // Check concurrent requests
            if entry.concurrent_requests >= config.concurrent_requests {
                return Err(RateLimitError::ConcurrentLimitExceeded {
                    limit: config.concurrent_requests,
                });
            }
        }
        
        Ok(())
    }
    
    fn find_endpoint_config(&self, endpoint: &str) -> Option<&EndpointLimit> {
        // Exact match first
        if let Some(config) = self.config.endpoint_limits.get(endpoint) {
            return Some(config);
        }
        
        // Prefix match for grouped endpoints
        for (pattern, config) in &self.config.endpoint_limits {
            if pattern.ends_with('/') && endpoint.starts_with(pattern) {
                return Some(config);
            }
        }
        
        None
    }
    
    async fn record_request(&self, ip: IpAddr, user_id: Option<&str>, endpoint: &str) {
        let now = Instant::now();
        
        // Record global request
        {
            let mut global_entry = self.global_entry.write().await;
            global_entry.add_request(now);
        }
        
        // Record IP request
        {
            let mut ip_entries = self.ip_entries.write().await;
            let entry = ip_entries.entry(ip).or_insert_with(RateLimitEntry::new);
            entry.add_request(now);
        }
        
        // Record user request if authenticated
        if let Some(user_id) = user_id {
            let mut user_entries = self.user_entries.write().await;
            let entry = user_entries.entry(user_id.to_string()).or_insert_with(RateLimitEntry::new);
            entry.add_request(now);
        }
        
        // Record endpoint request
        {
            let mut endpoint_entries = self.endpoint_entries.write().await;
            let entry = endpoint_entries.entry(endpoint.to_string()).or_insert_with(RateLimitEntry::new);
            entry.add_request(now);
        }
    }
    
    async fn calculate_remaining(&self, ip: IpAddr, user_id: Option<&str>, endpoint: &str) -> u32 {
        // Calculate the most restrictive remaining count
        let mut remaining = u32::MAX;
        
        // Check IP limits
        if let Some(entry) = self.ip_entries.read().await.get(&ip) {
            let ip_remaining = self.config.ip_limits.requests_per_minute
                .saturating_sub(entry.request_count(Duration::from_secs(60)) as u32);
            remaining = remaining.min(ip_remaining);
        }
        
        // Check user limits if authenticated
        if let Some(user_id) = user_id {
            if let Some(entry) = self.user_entries.read().await.get(user_id) {
                let user_remaining = self.config.user_limits.authenticated_requests_per_minute
                    .saturating_sub(entry.request_count(Duration::from_secs(60)) as u32);
                remaining = remaining.min(user_remaining);
            }
        }
        
        // Check endpoint limits
        if let Some(config) = self.find_endpoint_config(endpoint) {
            if let Some(entry) = self.endpoint_entries.read().await.get(endpoint) {
                let endpoint_remaining = config.requests_per_minute
                    .saturating_sub(entry.request_count(Duration::from_secs(60)) as u32);
                remaining = remaining.min(endpoint_remaining);
            }
        }
        
        if remaining == u32::MAX {
            self.config.ip_limits.requests_per_minute
        } else {
            remaining
        }
    }
    
    /// Mark request as finished (for concurrent request tracking)
    pub async fn finish_request(&self, ip: IpAddr, user_id: Option<&str>, endpoint: &str) {
        // Finish global request
        {
            let mut global_entry = self.global_entry.write().await;
            global_entry.finish_request();
        }
        
        // Finish IP request
        {
            let mut ip_entries = self.ip_entries.write().await;
            if let Some(entry) = ip_entries.get_mut(&ip) {
                entry.finish_request();
            }
        }
        
        // Finish user request if authenticated
        if let Some(user_id) = user_id {
            let mut user_entries = self.user_entries.write().await;
            if let Some(entry) = user_entries.get_mut(user_id) {
                entry.finish_request();
            }
        }
        
        // Finish endpoint request
        {
            let mut endpoint_entries = self.endpoint_entries.write().await;
            if let Some(entry) = endpoint_entries.get_mut(endpoint) {
                entry.finish_request();
            }
        }
    }
    
    /// Cleanup old entries to prevent memory leaks
    pub async fn cleanup_old_entries(&self) {
        let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
        
        // Cleanup IP entries
        {
            let mut ip_entries = self.ip_entries.write().await;
            ip_entries.retain(|_, entry| entry.last_request > cutoff);
        }
        
        // Cleanup user entries
        {
            let mut user_entries = self.user_entries.write().await;
            user_entries.retain(|_, entry| entry.last_request > cutoff);
        }
        
        // Cleanup endpoint entries
        {
            let mut endpoint_entries = self.endpoint_entries.write().await;
            endpoint_entries.retain(|_, entry| entry.last_request > cutoff);
        }
    }
}

/// Rate limit check result
#[derive(Debug, Serialize)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_time: Instant,
    pub retry_after: Option<Duration>,
}

/// Rate limit errors
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Global rate limit exceeded: {limit} requests per {window}")]
    GlobalLimitExceeded { limit: u32, window: String },
    
    #[error("IP rate limit exceeded for {ip}: {limit} requests per {window}")]
    IpLimitExceeded { ip: IpAddr, limit: u32, window: String },
    
    #[error("User rate limit exceeded for {user_id}: {limit} requests per {window}")]
    UserLimitExceeded { user_id: String, limit: u32, window: String },
    
    #[error("Endpoint rate limit exceeded for {endpoint}: {limit} requests per {window}")]
    EndpointLimitExceeded { endpoint: String, limit: u32, window: String },
    
    #[error("Concurrent request limit exceeded: {limit} concurrent requests")]
    ConcurrentLimitExceeded { limit: u32 },
    
    #[error("Authentication required for this endpoint")]
    AuthenticationRequired,
    
    #[error("Admin privileges required for this endpoint")]
    AdminRequired,
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimiterState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    
    // Extract IP address
    let ip = extract_client_ip(&request.headers())
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap());
    
    // Extract user information if available
    let user_claims = request.extensions().get::<Claims>();
    let user_id = user_claims.map(|claims| claims.sub.as_str());
    let is_admin = user_claims.map_or(false, |claims| claims.role.is_admin());
    
    // Check rate limits
    match rate_limiter.check_rate_limit(ip, user_id, &path, is_admin).await {
        Ok(result) => {
            // Add rate limit headers to request for downstream middleware
            request.extensions_mut().insert(result);
            
            let response = next.run(request).await;
            
            // Mark request as finished
            rate_limiter.finish_request(ip, user_id, &path).await;
            
            Ok(response)
        }
        Err(e) => {
            warn!("Rate limit exceeded: {}", e);
            
            // Return appropriate error response with headers
            let mut response = Response::new(format!("Rate limit exceeded: {}", e).into());
            *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
            
            // Add rate limit headers
            response.headers_mut().insert(
                "X-RateLimit-Limit",
                "0".parse().unwrap()
            );
            response.headers_mut().insert(
                "X-RateLimit-Remaining",
                "0".parse().unwrap()
            );
            response.headers_mut().insert(
                "Retry-After",
                "60".parse().unwrap()
            );
            
            Ok(response)
        }
    }
}

/// Extract client IP from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<IpAddr> {
    // Try X-Forwarded-For first
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            if let Some(first_ip) = xff_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }
    
    // Try X-Real-IP
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return Some(ip);
            }
        }
    }
    
    // Try CF-Connecting-IP (Cloudflare)
    if let Some(cf_ip) = headers.get("cf-connecting-ip") {
        if let Ok(ip_str) = cf_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return Some(ip);
            }
        }
    }
    
    None
}

/// Middleware to add rate limit headers to responses
pub async fn add_rate_limit_headers(
    mut response: Response,
) -> Response {
    // Add standard rate limit headers
    response.headers_mut().insert(
        "X-RateLimit-Policy",
        "MEV-Shield-v1".parse().unwrap()
    );
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_ip_rate_limiting() {
        let config = RateLimitConfig {
            ip_limits: IpLimits {
                requests_per_minute: 3,
                requests_per_hour: 10,
                concurrent_connections: 2,
            },
            ..Default::default()
        };
        
        let rate_limiter = RateLimiterState::new(config);
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        
        // First 3 requests should succeed
        for _ in 0..3 {
            let result = rate_limiter.check_rate_limit(ip, None, "/test", false).await;
            assert!(result.is_ok());
            rate_limiter.finish_request(ip, None, "/test").await;
        }
        
        // 4th request should fail
        let result = rate_limiter.check_rate_limit(ip, None, "/test", false).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_user_rate_limiting() {
        let config = RateLimitConfig {
            user_limits: UserLimits {
                authenticated_requests_per_minute: 2,
                authenticated_requests_per_hour: 5,
                admin_requests_per_minute: 5,
                admin_requests_per_hour: 10,
            },
            ..Default::default()
        };
        
        let rate_limiter = RateLimiterState::new(config);
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let user_id = "test_user";
        
        // First 2 requests should succeed
        for _ in 0..2 {
            let result = rate_limiter.check_rate_limit(ip, Some(user_id), "/test", false).await;
            assert!(result.is_ok());
            rate_limiter.finish_request(ip, Some(user_id), "/test").await;
        }
        
        // 3rd request should fail
        let result = rate_limiter.check_rate_limit(ip, Some(user_id), "/test", false).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_endpoint_rate_limiting() {
        let mut endpoint_limits = HashMap::new();
        endpoint_limits.insert("/test".to_string(), EndpointLimit {
            requests_per_minute: 1,
            requests_per_hour: 5,
            concurrent_requests: 1,
            require_auth: false,
            admin_only: false,
        });
        
        let config = RateLimitConfig {
            endpoint_limits,
            ..Default::default()
        };
        
        let rate_limiter = RateLimiterState::new(config);
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        
        // First request should succeed
        let result = rate_limiter.check_rate_limit(ip, None, "/test", false).await;
        assert!(result.is_ok());
        rate_limiter.finish_request(ip, None, "/test").await;
        
        // Second request should fail
        let result = rate_limiter.check_rate_limit(ip, None, "/test", false).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_burst_allowance() {
        let config = RateLimitConfig {
            ip_limits: IpLimits {
                requests_per_minute: 2,
                requests_per_hour: 10,
                concurrent_connections: 5,
            },
            allow_burst: true,
            burst_multiplier: 2.0,
            ..Default::default()
        };
        
        let rate_limiter = RateLimiterState::new(config);
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        
        // Consume normal limit
        for _ in 0..2 {
            let result = rate_limiter.check_rate_limit(ip, None, "/test", false).await;
            assert!(result.is_ok());
            rate_limiter.finish_request(ip, None, "/test").await;
        }
        
        // Wait a bit to accumulate burst tokens
        sleep(Duration::from_secs(1)).await;
        
        // Should be able to make one more request using burst
        let result = rate_limiter.check_rate_limit(ip, None, "/test", false).await;
        // This might succeed due to burst allowance
    }
    
    #[tokio::test]
    async fn test_cleanup() {
        let config = RateLimitConfig::default();
        let rate_limiter = RateLimiterState::new(config);
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        
        // Make some requests
        for _ in 0..5 {
            let _ = rate_limiter.check_rate_limit(ip, None, "/test", false).await;
            rate_limiter.finish_request(ip, None, "/test").await;
        }
        
        // Verify entries exist
        assert!(rate_limiter.ip_entries.read().await.contains_key(&ip));
        
        // Run cleanup
        rate_limiter.cleanup_old_entries().await;
        
        // Recent entries should still exist
        assert!(rate_limiter.ip_entries.read().await.contains_key(&ip));
    }
}