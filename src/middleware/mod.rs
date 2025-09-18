//! MEV Shield Middleware Module
//!
//! Comprehensive middleware collection for security, rate limiting,
//! CORS, and request/response processing.

pub mod rate_limit;
pub mod cors;

pub use rate_limit::*;
pub use cors::*;

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Request ID middleware for tracing
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let request_id = Uuid::new_v4().to_string();
    
    // Add request ID to extensions for use in other middleware/handlers
    request.extensions_mut().insert(request_id.clone());
    
    let response = next.run(request).await;
    
    // Add request ID to response headers
    let mut response = response;
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.parse().unwrap()
    );
    
    Ok(response)
}

/// Request timing middleware
pub async fn timing_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    if duration.as_millis() > 1000 {
        warn!("Slow request: {} {} - {}ms - {}", method, path, duration.as_millis(), status);
    } else {
        debug!("Request: {} {} - {}ms - {}", method, path, duration.as_millis(), status);
    }
    
    // Add timing header
    let mut response = response;
    response.headers_mut().insert(
        "X-Response-Time",
        format!("{}ms", duration.as_millis()).parse().unwrap()
    );
    
    Ok(response)
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let response = next.run(request).await;
    
    let mut response = response;
    let headers = response.headers_mut();
    
    // Core security headers
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    
    // Remove server information
    headers.remove("Server");
    headers.remove("X-Powered-By");
    
    // Add custom headers
    headers.insert("X-Security-Policy", "MEV-Shield-v1".parse().unwrap());
    headers.insert("X-Content-Security-Policy", 
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:".parse().unwrap());
    
    Ok(response)
}

/// Request validation middleware
pub async fn request_validation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Validate content type for POST/PUT requests
    if matches!(request.method().as_str(), "POST" | "PUT" | "PATCH") {
        if let Some(content_type) = request.headers().get("content-type") {
            let content_type_str = content_type.to_str().unwrap_or("");
            
            // Only allow specific content types
            let allowed_types = [
                "application/json",
                "application/x-www-form-urlencoded",
                "multipart/form-data",
                "text/plain",
            ];
            
            let is_allowed = allowed_types.iter().any(|&allowed| 
                content_type_str.starts_with(allowed)
            );
            
            if !is_allowed {
                warn!("Unsupported content type: {}", content_type_str);
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }
        } else {
            warn!("Missing content type for {} request", request.method());
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Validate content length
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                // 10MB limit
                if length > 10 * 1024 * 1024 {
                    warn!("Request body too large: {} bytes", length);
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// Health check bypass middleware
pub async fn health_check_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    // Bypass other middleware for health checks
    if path == "/health" || path == "/api/v1/health" {
        let mut response = Response::new("OK".into());
        *response.status_mut() = StatusCode::OK;
        response.headers_mut().insert(
            "Content-Type",
            "text/plain".parse().unwrap()
        );
        response.headers_mut().insert(
            "Cache-Control",
            "no-cache".parse().unwrap()
        );
        return Ok(response);
    }
    
    Ok(next.run(request).await)
}

/// Request size limiting middleware
pub async fn request_size_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check if content-length header exists and validate size
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                const MAX_SIZE: usize = 50 * 1024 * 1024; // 50MB
                
                if length > MAX_SIZE {
                    warn!("Request body too large: {} bytes (max: {})", length, MAX_SIZE);
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// Malicious request detection middleware
pub async fn malicious_request_detection_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let uri = request.uri();
    let path = uri.path();
    let query = uri.query().unwrap_or("");
    
    // Check for common attack patterns in URL
    let suspicious_patterns = [
        "../", "..\\", "..%2f", "..%5c",
        "<script", "%3cscript", "javascript:",
        "union select", "drop table", "insert into",
        "exec(", "eval(", "system(",
        "/etc/passwd", "/proc/", "cmd.exe",
        "powershell", "curl ", "wget ",
        "base64_decode", "file_get_contents",
    ];
    
    let full_url = format!("{}{}", path, if query.is_empty() { "" } else { "?" });
    let full_url_lower = format!("{}{}", full_url, query).to_lowercase();
    
    for pattern in &suspicious_patterns {
        if full_url_lower.contains(pattern) {
            warn!("Suspicious request pattern detected: {} in URL: {}", pattern, full_url);
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Check headers for suspicious content
    for (name, value) in request.headers() {
        if let Ok(value_str) = value.to_str() {
            let value_lower = value_str.to_lowercase();
            for pattern in &suspicious_patterns {
                if value_lower.contains(pattern) {
                    warn!("Suspicious pattern in header {}: {}", name, pattern);
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// User agent validation middleware
pub async fn user_agent_validation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(user_agent) = request.headers().get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            let ua_lower = ua_str.to_lowercase();
            
            // Block known bad user agents
            let blocked_agents = [
                "sqlmap", "nikto", "nessus", "openvas", "w3af",
                "dirbuster", "nmap", "masscan", "gobuster",
                "dirb", "wpscan", "burpsuite", "owasp zap",
                "python-requests", "curl/", "wget/",
                "scanner", "bot", "crawler", "spider",
            ];
            
            for blocked in &blocked_agents {
                if ua_lower.contains(blocked) {
                    warn!("Blocked user agent detected: {}", ua_str);
                    return Err(StatusCode::FORBIDDEN);
                }
            }
            
            // Check for empty or suspicious user agents
            if ua_str.is_empty() || ua_str.len() < 10 || ua_str.len() > 500 {
                warn!("Suspicious user agent length: {}", ua_str.len());
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        // Require user agent header
        warn!("Missing User-Agent header");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    Ok(next.run(request).await)
}

/// IP whitelist/blacklist middleware
pub async fn ip_filtering_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract IP address from headers
    let client_ip = extract_client_ip(request.headers());
    
    if let Some(ip) = client_ip {
        // Example blacklist (in production, load from database/config)
        let blacklisted_ips = [
            // Add known malicious IPs
        ];
        
        // Example blacklisted IP ranges (in production, use proper CIDR matching)
        let blacklisted_ranges = [
            // Add blacklisted ranges
        ];
        
        let ip_str = ip.to_string();
        
        // Check blacklist
        if blacklisted_ips.contains(&ip_str.as_str()) {
            warn!("Blacklisted IP detected: {}", ip);
            return Err(StatusCode::FORBIDDEN);
        }
        
        // Check against ranges (simplified check)
        for range in &blacklisted_ranges {
            if ip_str.starts_with(range) {
                warn!("IP in blacklisted range: {} (range: {})", ip, range);
                return Err(StatusCode::FORBIDDEN);
            }
        }
        
        debug!("Request from IP: {}", ip);
    }
    
    Ok(next.run(request).await)
}

/// Extract client IP from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<std::net::IpAddr> {
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

/// Error handling middleware
pub async fn error_handling_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match next.run(request).await {
        Ok(response) => {
            // Log successful responses if needed
            if response.status().is_server_error() {
                warn!("Server error response: {}", response.status());
            }
            Ok(response)
        }
        Err(status) => {
            // Convert status codes to proper error responses
            let error_message = match status {
                StatusCode::BAD_REQUEST => "Bad Request",
                StatusCode::UNAUTHORIZED => "Unauthorized",
                StatusCode::FORBIDDEN => "Forbidden",
                StatusCode::NOT_FOUND => "Not Found",
                StatusCode::METHOD_NOT_ALLOWED => "Method Not Allowed",
                StatusCode::PAYLOAD_TOO_LARGE => "Payload Too Large",
                StatusCode::TOO_MANY_REQUESTS => "Too Many Requests",
                StatusCode::INTERNAL_SERVER_ERROR => "Internal Server Error",
                _ => "Unknown Error",
            };
            
            warn!("Request failed with status: {} - {}", status, error_message);
            
            let mut response = Response::new(
                format!(r#"{{"error": "{}", "code": {}}}"#, error_message, status.as_u16()).into()
            );
            *response.status_mut() = status;
            response.headers_mut().insert(
                "Content-Type",
                "application/json".parse().unwrap()
            );
            
            Ok(response)
        }
    }
}