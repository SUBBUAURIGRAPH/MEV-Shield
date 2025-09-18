//! Authentication Middleware
//!
//! Provides JWT-based authentication middleware for Axum routes
//! with role-based access control and request rate limiting.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;
use tracing::{debug, warn, error};
use anyhow::Result;

use crate::auth::{
    jwt::{JwtService, JwtError, TokenBlacklist},
    models::{Claims, UserRole},
};

/// Authentication state shared across middleware
#[derive(Clone)]
pub struct AuthState {
    pub jwt_service: Arc<JwtService>,
    pub token_blacklist: Arc<tokio::sync::RwLock<TokenBlacklist>>,
}

impl AuthState {
    pub fn new(jwt_service: JwtService) -> Self {
        Self {
            jwt_service: Arc::new(jwt_service),
            token_blacklist: Arc::new(tokio::sync::RwLock::new(TokenBlacklist::new())),
        }
    }
}

/// Extension trait to add user claims to request extensions
pub trait RequestExt {
    fn user_claims(&self) -> Option<&Claims>;
    fn user_id(&self) -> Option<String>;
    fn user_role(&self) -> Option<&UserRole>;
    fn is_admin(&self) -> bool;
}

impl RequestExt for Request {
    fn user_claims(&self) -> Option<&Claims> {
        self.extensions().get::<Claims>()
    }

    fn user_id(&self) -> Option<String> {
        self.user_claims().map(|claims| claims.sub.clone())
    }

    fn user_role(&self) -> Option<&UserRole> {
        self.user_claims().map(|claims| &claims.role)
    }

    fn is_admin(&self) -> bool {
        self.user_role().map_or(false, |role| role.is_admin())
    }
}

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    debug!("Processing authentication middleware");

    // Extract token from Authorization header or cookies
    let token = match extract_token(&request.headers(), request.extensions().get::<CookieJar>()) {
        Some(token) => token,
        None => {
            warn!("No authentication token found");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Check if token is blacklisted
    {
        let blacklist = auth_state.token_blacklist.read().await;
        if blacklist.is_blacklisted(&token) {
            warn!("Blacklisted token attempted access");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // Validate the token
    let claims = match auth_state.jwt_service.validate_access_token(&token) {
        Ok(claims) => claims,
        Err(JwtError::TokenExpired) => {
            debug!("Token expired");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(e) => {
            error!("Token validation error: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    debug!("Authentication successful for user: {}", claims.email);

    // Add claims to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Optional authentication middleware (doesn't fail if no token)
pub async fn optional_auth_middleware(
    State(auth_state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try to extract and validate token, but don't fail if missing
    if let Some(token) = extract_token(&request.headers(), request.extensions().get::<CookieJar>()) {
        // Check blacklist
        let blacklist = auth_state.token_blacklist.read().await;
        if !blacklist.is_blacklisted(&token) {
            // Validate token
            if let Ok(claims) = auth_state.jwt_service.validate_access_token(&token) {
                request.extensions_mut().insert(claims);
            }
        }
    }

    Ok(next.run(request).await)
}

/// Admin-only middleware
pub async fn admin_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request.extensions().get::<Claims>();
    
    match claims {
        Some(claims) if claims.role.is_admin() => {
            debug!("Admin access granted for user: {}", claims.email);
            Ok(next.run(request).await)
        }
        Some(claims) => {
            warn!("Non-admin user {} attempted admin access", claims.email);
            Err(StatusCode::FORBIDDEN)
        }
        None => {
            warn!("Unauthenticated admin access attempt");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Role-based access control middleware
pub async fn require_role_middleware(
    required_role: UserRole,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> {
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            let claims = request.extensions().get::<Claims>();
            
            match claims {
                Some(claims) => {
                    if can_access_with_role(&claims.role, &required_role) {
                        debug!("Role-based access granted for user: {} with role: {:?}", 
                               claims.email, claims.role);
                        Ok(next.run(request).await)
                    } else {
                        warn!("User {} with role {:?} attempted access requiring {:?}", 
                              claims.email, claims.role, required_role);
                        Err(StatusCode::FORBIDDEN)
                    }
                }
                None => {
                    warn!("Unauthenticated access attempt to role-protected endpoint");
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        })
    }
}

/// Rate limiting middleware (basic implementation)
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // In production, implement proper rate limiting with Redis or similar
    // This is a basic placeholder implementation
    
    let user_id = request.user_id();
    debug!("Rate limiting check for user: {:?}", user_id);
    
    // For now, just pass through
    // TODO: Implement proper rate limiting
    Ok(next.run(request).await)
}

/// Extract token from Authorization header or cookies
fn extract_token(headers: &HeaderMap, cookies: Option<&CookieJar>) -> Option<String> {
    // First, try Authorization header
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Ok(token) = JwtService::default().extract_token_from_header(auth_str) {
                return Some(token);
            }
        }
    }

    // Fallback to cookies
    if let Some(cookies) = cookies {
        if let Some(cookie) = cookies.get("access_token") {
            return Some(cookie.value().to_string());
        }
    }

    None
}

/// Check if a user role can access an endpoint requiring a specific role
fn can_access_with_role(user_role: &UserRole, required_role: &UserRole) -> bool {
    match required_role {
        UserRole::Admin => user_role.is_admin(),
        UserRole::User => user_role.can_access_protected(),
        UserRole::Validator => matches!(user_role, UserRole::Admin | UserRole::Validator),
        UserRole::ReadOnly => user_role.can_view_analytics(),
    }
}

/// Utility function to blacklist a token
pub async fn blacklist_token(
    auth_state: &AuthState,
    token: &str,
) -> Result<()> {
    let expiration = auth_state.jwt_service.get_token_expiration(token)?;
    let mut blacklist = auth_state.token_blacklist.write().await;
    blacklist.add_token(token, expiration);
    Ok(())
}

/// Create middleware stack for protected routes
pub fn create_auth_middleware_stack() -> tower::util::Stack<
    axum::middleware::FromFn<AuthState, fn(State<AuthState>, Request, Next) -> _>,
    tower::util::Stack<
        axum::middleware::FromFn<(), fn(Request, Next) -> _>,
        tower::ServiceBuilder<tower::layer::util::Identity>
    >
> {
    tower::ServiceBuilder::new()
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::middleware::from_fn_with_state(
            AuthState::new(JwtService::default()),
            auth_middleware,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{jwt::JwtConfig, models::User};
    use axum::{
        body::Body,
        http::{Request, Method},
    };

    async fn create_test_request_with_token(token: &str) -> Request<Body> {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        request
    }

    #[tokio::test]
    async fn test_token_extraction() {
        let headers = HeaderMap::new();
        let cookies = None;
        
        // No token should return None
        assert!(extract_token(&headers, cookies).is_none());
        
        // Test with valid Authorization header
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer test_token".parse().unwrap());
        
        // Note: This will fail validation but should extract the token
        // In a real test, we'd need a valid JWT
    }

    #[test]
    fn test_role_access_control() {
        assert!(can_access_with_role(&UserRole::Admin, &UserRole::Admin));
        assert!(can_access_with_role(&UserRole::Admin, &UserRole::User));
        assert!(can_access_with_role(&UserRole::Admin, &UserRole::Validator));
        assert!(can_access_with_role(&UserRole::Admin, &UserRole::ReadOnly));
        
        assert!(!can_access_with_role(&UserRole::User, &UserRole::Admin));
        assert!(can_access_with_role(&UserRole::User, &UserRole::User));
        assert!(!can_access_with_role(&UserRole::User, &UserRole::Validator));
        assert!(can_access_with_role(&UserRole::User, &UserRole::ReadOnly));
        
        assert!(!can_access_with_role(&UserRole::ReadOnly, &UserRole::Admin));
        assert!(!can_access_with_role(&UserRole::ReadOnly, &UserRole::User));
        assert!(!can_access_with_role(&UserRole::ReadOnly, &UserRole::Validator));
        assert!(can_access_with_role(&UserRole::ReadOnly, &UserRole::ReadOnly));
    }

    #[tokio::test]
    async fn test_blacklist_functionality() {
        let auth_state = AuthState::new(JwtService::default());
        let test_token = "test_token";
        
        // Token should not be blacklisted initially
        {
            let blacklist = auth_state.token_blacklist.read().await;
            assert!(!blacklist.is_blacklisted(test_token));
        }
        
        // Add token to blacklist (this would fail without a valid JWT, but tests the mechanism)
        // In a real test, we'd create a valid JWT token first
    }
}