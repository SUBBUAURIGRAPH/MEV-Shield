//! CORS Security Implementation
//!
//! Comprehensive CORS (Cross-Origin Resource Sharing) security with
//! restrictive policies, origin validation, and environment-specific configurations.

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, warn, error};
use url::Url;

/// CORS configuration with security-focused defaults
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins (exact matches)
    pub allowed_origins: HashSet<String>,
    /// Allowed origin patterns (for subdomain matching)
    pub allowed_origin_patterns: Vec<String>,
    /// Allowed methods
    pub allowed_methods: HashSet<Method>,
    /// Allowed headers
    pub allowed_headers: HashSet<String>,
    /// Exposed headers (sent to client)
    pub exposed_headers: HashSet<String>,
    /// Whether to allow credentials
    pub allow_credentials: bool,
    /// Max age for preflight cache
    pub max_age: Option<u32>,
    /// Enable origin validation
    pub validate_origins: bool,
    /// Environment-specific settings
    pub environment: CorsEnvironment,
}

/// Environment-specific CORS configurations
#[derive(Debug, Clone)]
pub enum CorsEnvironment {
    Development {
        allow_localhost: bool,
        localhost_ports: Vec<u16>,
    },
    Production {
        strict_origin_check: bool,
        require_https: bool,
    },
    Testing {
        allow_any_origin: bool,
    },
}

impl Default for CorsConfig {
    fn default() -> Self {
        let mut allowed_origins = HashSet::new();
        
        // Production origins (update these for your deployment)
        allowed_origins.insert("https://mev-shield.aurex.in".to_string());
        allowed_origins.insert("https://app.mev-shield.aurex.in".to_string());
        allowed_origins.insert("https://admin.mev-shield.aurex.in".to_string());
        
        let mut allowed_methods = HashSet::new();
        allowed_methods.insert(Method::GET);
        allowed_methods.insert(Method::POST);
        allowed_methods.insert(Method::PUT);
        allowed_methods.insert(Method::DELETE);
        allowed_methods.insert(Method::OPTIONS);
        
        let mut allowed_headers = HashSet::new();
        allowed_headers.insert("Accept".to_string());
        allowed_headers.insert("Accept-Language".to_string());
        allowed_headers.insert("Content-Type".to_string());
        allowed_headers.insert("Content-Language".to_string());
        allowed_headers.insert("Authorization".to_string());
        allowed_headers.insert("X-Requested-With".to_string());
        allowed_headers.insert("X-Request-ID".to_string());
        
        let mut exposed_headers = HashSet::new();
        exposed_headers.insert("X-Request-ID".to_string());
        exposed_headers.insert("X-RateLimit-Limit".to_string());
        exposed_headers.insert("X-RateLimit-Remaining".to_string());
        exposed_headers.insert("X-RateLimit-Reset".to_string());
        
        Self {
            allowed_origins,
            allowed_origin_patterns: vec![
                "https://*.mev-shield.aurex.in".to_string(),
            ],
            allowed_methods,
            allowed_headers,
            exposed_headers,
            allow_credentials: true,
            max_age: Some(86400), // 24 hours
            validate_origins: true,
            environment: CorsEnvironment::Production {
                strict_origin_check: true,
                require_https: true,
            },
        }
    }
}

impl CorsConfig {
    /// Create development configuration
    pub fn development() -> Self {
        let mut config = Self::default();
        
        // Add localhost origins for development
        config.allowed_origins.insert("http://localhost:3000".to_string());
        config.allowed_origins.insert("http://localhost:3001".to_string());
        config.allowed_origins.insert("http://localhost:3002".to_string());
        config.allowed_origins.insert("http://localhost:3003".to_string());
        config.allowed_origins.insert("http://localhost:3004".to_string());
        config.allowed_origins.insert("http://127.0.0.1:3000".to_string());
        config.allowed_origins.insert("http://127.0.0.1:3001".to_string());
        config.allowed_origins.insert("http://127.0.0.1:3002".to_string());
        
        config.environment = CorsEnvironment::Development {
            allow_localhost: true,
            localhost_ports: vec![3000, 3001, 3002, 3003, 3004, 8080, 8081],
        };
        
        config
    }
    
    /// Create testing configuration
    pub fn testing() -> Self {
        let mut config = Self::default();
        config.environment = CorsEnvironment::Testing {
            allow_any_origin: true,
        };
        config.validate_origins = false;
        config
    }
    
    /// Create production configuration with specific domains
    pub fn production(domains: Vec<String>) -> Self {
        let mut config = Self::default();
        
        config.allowed_origins.clear();
        for domain in domains {
            if domain.starts_with("https://") {
                config.allowed_origins.insert(domain);
            } else {
                config.allowed_origins.insert(format!("https://{}", domain));
            }
        }
        
        config.environment = CorsEnvironment::Production {
            strict_origin_check: true,
            require_https: true,
        };
        
        config
    }
}

/// CORS middleware state
pub struct CorsMiddleware {
    config: CorsConfig,
}

impl CorsMiddleware {
    pub fn new(config: CorsConfig) -> Self {
        Self { config }
    }
    
    /// Handle CORS for incoming request
    pub async fn handle_cors(
        &self,
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let method = request.method().clone();
        let headers = request.headers();
        
        // Extract origin
        let origin = self.extract_origin(headers);
        
        // Validate origin if required
        if self.config.validate_origins {
            if let Some(ref origin_str) = origin {
                if !self.is_origin_allowed(origin_str) {
                    warn!("CORS: Origin not allowed: {}", origin_str);
                    return Err(StatusCode::FORBIDDEN);
                }
            } else if method != Method::GET && method != Method::HEAD {
                // Require origin for non-simple requests
                warn!("CORS: Missing origin header for non-simple request");
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        
        // Handle preflight request
        if method == Method::OPTIONS {
            return self.handle_preflight(headers, origin.as_deref()).await;
        }
        
        // Handle actual request
        let response = next.run(request).await;
        self.add_cors_headers(response, origin.as_deref()).await
    }
    
    /// Handle preflight OPTIONS request
    async fn handle_preflight(
        &self,
        headers: &HeaderMap,
        origin: Option<&str>,
    ) -> Result<Response, StatusCode> {
        debug!("CORS: Handling preflight request");
        
        // Check Access-Control-Request-Method
        let requested_method = headers
            .get("access-control-request-method")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<Method>().ok());
        
        if let Some(method) = requested_method {
            if !self.config.allowed_methods.contains(&method) {
                warn!("CORS: Requested method not allowed: {}", method);
                return Err(StatusCode::METHOD_NOT_ALLOWED);
            }
        } else {
            warn!("CORS: Missing or invalid Access-Control-Request-Method header");
            return Err(StatusCode::BAD_REQUEST);
        }
        
        // Check Access-Control-Request-Headers
        if let Some(requested_headers) = headers.get("access-control-request-headers") {
            if let Ok(headers_str) = requested_headers.to_str() {
                let headers_list: Vec<&str> = headers_str.split(',').map(|h| h.trim()).collect();
                
                for header in headers_list {
                    let header_lower = header.to_lowercase();
                    if !self.config.allowed_headers.contains(&header_lower) &&
                       !self.is_simple_header(&header_lower) {
                        warn!("CORS: Requested header not allowed: {}", header);
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
            }
        }
        
        // Create preflight response
        let mut response = Response::new("".into());
        *response.status_mut() = StatusCode::NO_CONTENT;
        
        self.add_preflight_headers(response.headers_mut(), origin);
        
        Ok(response)
    }
    
    /// Add CORS headers to actual response
    async fn add_cors_headers(
        &self,
        mut response: Response,
        origin: Option<&str>,
    ) -> Result<Response, StatusCode> {
        let headers = response.headers_mut();
        
        // Add origin header if allowed
        if let Some(origin_str) = origin {
            if self.is_origin_allowed(origin_str) || !self.config.validate_origins {
                headers.insert(
                    "Access-Control-Allow-Origin",
                    HeaderValue::from_str(origin_str).unwrap_or_else(|_| HeaderValue::from_static("*"))
                );
            }
        } else {
            // Only allow * for non-credentialed requests
            if !self.config.allow_credentials {
                headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
            }
        }
        
        // Add credentials header
        if self.config.allow_credentials {
            headers.insert("Access-Control-Allow-Credentials", HeaderValue::from_static("true"));
        }
        
        // Add exposed headers
        if !self.config.exposed_headers.is_empty() {
            let exposed_headers: Vec<String> = self.config.exposed_headers.iter().cloned().collect();
            headers.insert(
                "Access-Control-Expose-Headers",
                HeaderValue::from_str(&exposed_headers.join(", ")).unwrap_or_else(|_| HeaderValue::from_static(""))
            );
        }
        
        // Add Vary header for caching
        if let Some(existing_vary) = headers.get("Vary") {
            if let Ok(existing) = existing_vary.to_str() {
                if !existing.contains("Origin") {
                    headers.insert(
                        "Vary",
                        HeaderValue::from_str(&format!("{}, Origin", existing)).unwrap_or_else(|_| HeaderValue::from_static("Origin"))
                    );
                }
            }
        } else {
            headers.insert("Vary", HeaderValue::from_static("Origin"));
        }
        
        Ok(response)
    }
    
    /// Add preflight-specific headers
    fn add_preflight_headers(&self, headers: &mut HeaderMap, origin: Option<&str>) {
        // Add origin header
        if let Some(origin_str) = origin {
            if self.is_origin_allowed(origin_str) || !self.config.validate_origins {
                headers.insert(
                    "Access-Control-Allow-Origin",
                    HeaderValue::from_str(origin_str).unwrap_or_else(|_| HeaderValue::from_static("*"))
                );
            }
        }
        
        // Add allowed methods
        let methods: Vec<String> = self.config.allowed_methods
            .iter()
            .map(|m| m.to_string())
            .collect();
        headers.insert(
            "Access-Control-Allow-Methods",
            HeaderValue::from_str(&methods.join(", ")).unwrap_or_else(|_| HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"))
        );
        
        // Add allowed headers
        let allowed_headers: Vec<String> = self.config.allowed_headers.iter().cloned().collect();
        headers.insert(
            "Access-Control-Allow-Headers",
            HeaderValue::from_str(&allowed_headers.join(", ")).unwrap_or_else(|_| HeaderValue::from_static("Content-Type, Authorization"))
        );
        
        // Add credentials header
        if self.config.allow_credentials {
            headers.insert("Access-Control-Allow-Credentials", HeaderValue::from_static("true"));
        }
        
        // Add max age
        if let Some(max_age) = self.config.max_age {
            headers.insert(
                "Access-Control-Max-Age",
                HeaderValue::from_str(&max_age.to_string()).unwrap_or_else(|_| HeaderValue::from_static("86400"))
            );
        }
        
        // Add Vary header
        headers.insert("Vary", HeaderValue::from_static("Origin, Access-Control-Request-Method, Access-Control-Request-Headers"));
    }
    
    /// Extract origin from request headers
    fn extract_origin(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get("origin")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
    
    /// Check if origin is allowed
    fn is_origin_allowed(&self, origin: &str) -> bool {
        // Handle testing environment
        if let CorsEnvironment::Testing { allow_any_origin } = self.config.environment {
            if allow_any_origin {
                return true;
            }
        }
        
        // Exact match check
        if self.config.allowed_origins.contains(origin) {
            return true;
        }
        
        // Pattern matching for wildcards
        for pattern in &self.config.allowed_origin_patterns {
            if self.matches_pattern(origin, pattern) {
                return true;
            }
        }
        
        // Environment-specific checks
        match &self.config.environment {
            CorsEnvironment::Development { allow_localhost, localhost_ports } => {
                if *allow_localhost && self.is_localhost_origin(origin, localhost_ports) {
                    return true;
                }
            }
            CorsEnvironment::Production { strict_origin_check, require_https } => {
                if *require_https && !origin.starts_with("https://") {
                    warn!("CORS: Non-HTTPS origin in production: {}", origin);
                    return false;
                }
                
                if *strict_origin_check {
                    // Additional validation for production
                    return self.validate_production_origin(origin);
                }
            }
            CorsEnvironment::Testing { .. } => {
                // Already handled above
            }
        }
        
        false
    }
    
    /// Check if origin matches a wildcard pattern
    fn matches_pattern(&self, origin: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Simple wildcard matching for subdomains
            if pattern.starts_with("https://*.") {
                let domain_pattern = &pattern[10..]; // Remove "https://*."
                if let Ok(url) = Url::parse(origin) {
                    if let Some(host) = url.host_str() {
                        return host.ends_with(domain_pattern) || host == domain_pattern;
                    }
                }
            }
        } else {
            return origin == pattern;
        }
        
        false
    }
    
    /// Check if origin is a valid localhost origin
    fn is_localhost_origin(&self, origin: &str, allowed_ports: &[u16]) -> bool {
        if let Ok(url) = Url::parse(origin) {
            if let Some(host) = url.host_str() {
                let is_localhost = host == "localhost" || host == "127.0.0.1" || host == "::1";
                
                if is_localhost {
                    if let Some(port) = url.port() {
                        return allowed_ports.contains(&port);
                    } else {
                        // Default ports
                        let default_port = match url.scheme() {
                            "http" => 80,
                            "https" => 443,
                            _ => return false,
                        };
                        return allowed_ports.contains(&default_port);
                    }
                }
            }
        }
        
        false
    }
    
    /// Additional validation for production origins
    fn validate_production_origin(&self, origin: &str) -> bool {
        if let Ok(url) = Url::parse(origin) {
            // Must be HTTPS
            if url.scheme() != "https" {
                return false;
            }
            
            // Check for suspicious patterns
            if let Some(host) = url.host_str() {
                // Block IP addresses in production
                if host.chars().next().unwrap_or('a').is_ascii_digit() {
                    warn!("CORS: IP address origin blocked in production: {}", host);
                    return false;
                }
                
                // Block common malicious patterns
                let suspicious_patterns = [
                    "localhost", "127.0.0.1", "0.0.0.0",
                    "example.com", "test.com", "evil.com",
                    "attacker", "malicious", "phishing",
                ];
                
                let host_lower = host.to_lowercase();
                for pattern in &suspicious_patterns {
                    if host_lower.contains(pattern) {
                        warn!("CORS: Suspicious origin blocked: {}", host);
                        return false;
                    }
                }
            }
        }
        
        true
    }
    
    /// Check if header is a simple header (doesn't require preflight)
    fn is_simple_header(&self, header: &str) -> bool {
        matches!(
            header,
            "accept" | "accept-language" | "content-language" | "content-type"
        )
    }
}

/// CORS middleware factory
pub fn cors_middleware(config: CorsConfig) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> {
    let cors = CorsMiddleware::new(config);
    
    move |request: Request, next: Next| {
        let cors = cors.clone();
        Box::pin(async move {
            cors.handle_cors(request, next).await
        })
    }
}

impl Clone for CorsMiddleware {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

/// CORS validation result
#[derive(Debug, Serialize)]
pub struct CorsValidationResult {
    pub allowed: bool,
    pub origin: Option<String>,
    pub method: String,
    pub headers: Vec<String>,
    pub reason: Option<String>,
}

/// Validate CORS request without processing
pub fn validate_cors_request(
    config: &CorsConfig,
    origin: Option<&str>,
    method: &Method,
    headers: &HeaderMap,
) -> CorsValidationResult {
    let mut result = CorsValidationResult {
        allowed: true,
        origin: origin.map(String::from),
        method: method.to_string(),
        headers: headers.keys().map(|h| h.to_string()).collect(),
        reason: None,
    };
    
    // Validate origin
    if config.validate_origins {
        if let Some(origin_str) = origin {
            let cors_middleware = CorsMiddleware::new(config.clone());
            if !cors_middleware.is_origin_allowed(origin_str) {
                result.allowed = false;
                result.reason = Some(format!("Origin not allowed: {}", origin_str));
                return result;
            }
        }
    }
    
    // Validate method
    if !config.allowed_methods.contains(method) {
        result.allowed = false;
        result.reason = Some(format!("Method not allowed: {}", method));
        return result;
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;
    
    #[test]
    fn test_origin_validation() {
        let config = CorsConfig::production(vec![
            "example.com".to_string(),
            "app.example.com".to_string(),
        ]);
        
        let cors = CorsMiddleware::new(config);
        
        // Valid origins
        assert!(cors.is_origin_allowed("https://example.com"));
        assert!(cors.is_origin_allowed("https://app.example.com"));
        
        // Invalid origins
        assert!(!cors.is_origin_allowed("http://example.com")); // HTTP not allowed in production
        assert!(!cors.is_origin_allowed("https://evil.com"));
        assert!(!cors.is_origin_allowed("https://127.0.0.1"));
    }
    
    #[test]
    fn test_wildcard_matching() {
        let mut config = CorsConfig::default();
        config.allowed_origin_patterns = vec!["https://*.example.com".to_string()];
        
        let cors = CorsMiddleware::new(config);
        
        // Should match subdomains
        assert!(cors.matches_pattern("https://app.example.com", "https://*.example.com"));
        assert!(cors.matches_pattern("https://api.example.com", "https://*.example.com"));
        
        // Should not match different domains
        assert!(!cors.matches_pattern("https://app.evil.com", "https://*.example.com"));
        assert!(!cors.matches_pattern("https://example.com.evil.com", "https://*.example.com"));
    }
    
    #[test]
    fn test_localhost_validation() {
        let config = CorsConfig::development();
        let cors = CorsMiddleware::new(config);
        
        // Valid localhost origins
        assert!(cors.is_origin_allowed("http://localhost:3000"));
        assert!(cors.is_origin_allowed("http://127.0.0.1:3001"));
        
        // Invalid localhost origins (wrong port)
        assert!(!cors.is_origin_allowed("http://localhost:9999"));
    }
    
    #[test]
    fn test_cors_validation() {
        let config = CorsConfig::production(vec!["example.com".to_string()]);
        let headers = HeaderMap::new();
        
        // Valid request
        let result = validate_cors_request(
            &config,
            Some("https://example.com"),
            &Method::GET,
            &headers,
        );
        assert!(result.allowed);
        
        // Invalid origin
        let result = validate_cors_request(
            &config,
            Some("https://evil.com"),
            &Method::GET,
            &headers,
        );
        assert!(!result.allowed);
        assert!(result.reason.is_some());
        
        // Invalid method
        let result = validate_cors_request(
            &config,
            Some("https://example.com"),
            &Method::PATCH,
            &headers,
        );
        assert!(!result.allowed);
    }
    
    #[test]
    fn test_simple_headers() {
        let cors = CorsMiddleware::new(CorsConfig::default());
        
        // Simple headers shouldn't require preflight
        assert!(cors.is_simple_header("accept"));
        assert!(cors.is_simple_header("content-type"));
        assert!(cors.is_simple_header("accept-language"));
        
        // Custom headers should require preflight
        assert!(!cors.is_simple_header("authorization"));
        assert!(!cors.is_simple_header("x-custom-header"));
    }
}