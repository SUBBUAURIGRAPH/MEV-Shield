//! Authentication Routes
//!
//! HTTP endpoints for user authentication including login, logout,
//! token refresh, password management, and user registration.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::Json,
    routing::{post, get},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::auth::{
    jwt::{JwtService, JwtConfig},
    middleware::{AuthState, blacklist_token, RequestExt},
    models::*,
    password::PasswordService,
};

/// Authentication service state
#[derive(Clone)]
pub struct AuthServiceState {
    pub auth_state: AuthState,
    pub password_service: Arc<PasswordService>,
    pub users: Arc<RwLock<HashMap<String, User>>>, // In production, use database
    pub password_reset_tokens: Arc<RwLock<HashMap<String, (Uuid, chrono::DateTime<chrono::Utc>)>>>,
}

impl AuthServiceState {
    pub fn new() -> Self {
        let jwt_service = JwtService::new(JwtConfig::default());
        let auth_state = AuthState::new(jwt_service);
        let password_service = Arc::new(PasswordService::new());
        
        // Create default admin user for testing
        let mut users = HashMap::new();
        let admin_password_hash = password_service
            .hash_password("AdminPassword123!")
            .expect("Failed to hash admin password");
        
        let admin_user = User::new(
            "admin@mevshield.com".to_string(),
            admin_password_hash,
            UserRole::Admin,
        );
        
        users.insert(admin_user.email.clone(), admin_user);
        
        Self {
            auth_state,
            password_service,
            users: Arc::new(RwLock::new(users)),
            password_reset_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Find user by email
    pub async fn find_user_by_email(&self, email: &str) -> Option<User> {
        let users = self.users.read().await;
        users.get(email).cloned()
    }

    /// Update user
    pub async fn update_user(&self, user: &User) {
        let mut users = self.users.write().await;
        users.insert(user.email.clone(), user.clone());
    }

    /// Create new user
    pub async fn create_user(&self, email: String, password: String, role: UserRole) -> Result<User, String> {
        let users = self.users.read().await;
        if users.contains_key(&email) {
            return Err("User already exists".to_string());
        }
        drop(users);

        let password_hash = self.password_service
            .hash_password(&password)
            .map_err(|e| e.to_string())?;
        
        let user = User::new(email, password_hash, role);
        
        let mut users = self.users.write().await;
        users.insert(user.email.clone(), user.clone());
        
        Ok(user)
    }
}

/// Login endpoint
pub async fn login(
    State(state): State<AuthServiceState>,
    mut cookies: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, Json<serde_json::Value>), StatusCode> {
    debug!("Login attempt for email: {}", request.email);

    // Find user by email
    let mut user = match state.find_user_by_email(&request.email).await {
        Some(user) => user,
        None => {
            warn!("Login attempt for non-existent user: {}", request.email);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Check if account is locked
    if user.is_locked() {
        warn!("Login attempt for locked account: {}", request.email);
        return Ok((cookies, Json(json!({
            "success": false,
            "error": "Account is temporarily locked due to multiple failed login attempts"
        }))));
    }

    // Verify password
    let password_valid = match state.password_service.verify_password(&request.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            error!("Password verification error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if !password_valid {
        warn!("Failed login attempt for user: {}", request.email);
        user.record_failed_login();
        state.update_user(&user).await;
        
        return Ok((cookies, Json(json!({
            "success": false,
            "error": "Invalid credentials"
        }))));
    }

    // Check if user is active
    if !user.is_active {
        warn!("Login attempt for inactive user: {}", request.email);
        return Ok((cookies, Json(json!({
            "success": false,
            "error": "Account is deactivated"
        }))));
    }

    // Record successful login
    user.record_successful_login();
    state.update_user(&user).await;

    // Generate tokens
    let tokens = match state.auth_state.jwt_service.generate_tokens(&user) {
        Ok(tokens) => tokens,
        Err(e) => {
            error!("Token generation error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    info!("Successful login for user: {}", request.email);

    // Set secure HTTP-only cookies
    let access_cookie = Cookie::build(("access_token", &tokens.access_token))
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .path("/")
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", &tokens.refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .path("/auth")
        .build();

    cookies = cookies.add(access_cookie);
    cookies = cookies.add(refresh_cookie);

    let response = LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type,
        expires_in: tokens.expires_in,
        user: user.to_public_info(),
    };

    Ok((cookies, Json(json!({
        "success": true,
        "data": response
    }))))
}

/// Logout endpoint
pub async fn logout(
    State(state): State<AuthServiceState>,
    mut cookies: CookieJar,
    request: Request,
) -> Result<(CookieJar, Json<serde_json::Value>), StatusCode> {
    debug!("Logout request");

    // Extract token from request
    if let Some(token) = cookies.get("access_token").map(|c| c.value().to_string())
        .or_else(|| {
            request.headers().get("authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| state.auth_state.jwt_service.extract_token_from_header(s).ok())
        })
    {
        // Blacklist the token
        if let Err(e) = blacklist_token(&state.auth_state, &token).await {
            error!("Failed to blacklist token: {}", e);
        }
    }

    // Remove cookies
    let access_cookie = Cookie::build(("access_token", ""))
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .path("/")
        .max_age(axum_extra::extract::cookie::time::Duration::ZERO)
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .path("/auth")
        .max_age(axum_extra::extract::cookie::time::Duration::ZERO)
        .build();

    cookies = cookies.add(access_cookie);
    cookies = cookies.add(refresh_cookie);

    info!("User logged out successfully");

    Ok((cookies, Json(json!({
        "success": true,
        "message": "Logged out successfully"
    }))))
}

/// Token refresh endpoint
pub async fn refresh_token(
    State(state): State<AuthServiceState>,
    mut cookies: CookieJar,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<(CookieJar, Json<serde_json::Value>), StatusCode> {
    debug!("Token refresh request");

    // Validate refresh token
    let refresh_claims = match state.auth_state.jwt_service.validate_refresh_token(&request.refresh_token) {
        Ok(claims) => claims,
        Err(e) => {
            warn!("Invalid refresh token: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Find user
    let user = match state.find_user_by_email(&refresh_claims.email).await {
        Some(user) => user,
        None => {
            warn!("Refresh token for non-existent user: {}", refresh_claims.email);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Check if user is still active
    if !user.is_active {
        warn!("Refresh token for inactive user: {}", refresh_claims.email);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate new access token
    let new_access_token = match state.auth_state.jwt_service.refresh_access_token(&request.refresh_token, &user) {
        Ok(token) => token,
        Err(e) => {
            error!("Token refresh error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    info!("Token refreshed for user: {}", user.email);

    // Update access token cookie
    let access_cookie = Cookie::build(("access_token", &new_access_token))
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .path("/")
        .build();

    cookies = cookies.add(access_cookie);

    Ok((cookies, Json(json!({
        "success": true,
        "data": {
            "access_token": new_access_token,
            "token_type": "Bearer",
            "expires_in": state.auth_state.jwt_service.config.access_token_expiry.num_seconds()
        }
    }))))
}

/// Get current user info
pub async fn me(request: Request) -> Result<Json<serde_json::Value>, StatusCode> {
    let claims = request.extensions().get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    debug!("User info request for: {}", claims.email);

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": claims.sub,
            "email": claims.email,
            "role": claims.role
        }
    })))
}

/// Change password endpoint
pub async fn change_password(
    State(state): State<AuthServiceState>,
    request: Request,
    Json(change_request): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let claims = request.extensions().get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    debug!("Password change request for user: {}", claims.email);

    // Find user
    let mut user = match state.find_user_by_email(&claims.email).await {
        Some(user) => user,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Verify current password
    let current_password_valid = match state.password_service
        .verify_password(&change_request.current_password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            error!("Password verification error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if !current_password_valid {
        warn!("Invalid current password for user: {}", claims.email);
        return Ok(Json(json!({
            "success": false,
            "error": "Current password is incorrect"
        })));
    }

    // Hash new password
    let new_password_hash = match state.password_service.hash_password(&change_request.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            return Ok(Json(json!({
                "success": false,
                "error": e.to_string()
            })));
        }
    };

    // Update user password
    user.password_hash = new_password_hash;
    state.update_user(&user).await;

    info!("Password changed for user: {}", claims.email);

    Ok(Json(json!({
        "success": true,
        "message": "Password changed successfully"
    })))
}

/// Password reset request
pub async fn request_password_reset(
    State(state): State<AuthServiceState>,
    Json(reset_request): Json<PasswordResetRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Password reset request for email: {}", reset_request.email);

    // Check if user exists (but don't reveal if they don't)
    if let Some(user) = state.find_user_by_email(&reset_request.email).await {
        // Generate reset token
        let reset_token = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1); // 1 hour expiry

        // Store reset token
        let mut reset_tokens = state.password_reset_tokens.write().await;
        reset_tokens.insert(reset_token.clone(), (user.id, expires_at));

        info!("Password reset token generated for user: {}", user.email);
        
        // In production, send reset token via email
        // For now, just log it (NEVER do this in production!)
        debug!("Password reset token (FOR TESTING ONLY): {}", reset_token);
    }

    // Always return success to prevent user enumeration
    Ok(Json(json!({
        "success": true,
        "message": "If an account with that email exists, a password reset link has been sent."
    })))
}

/// Password reset confirmation
pub async fn confirm_password_reset(
    State(state): State<AuthServiceState>,
    Json(confirm_request): Json<PasswordResetConfirm>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Password reset confirmation");

    // Find and validate reset token
    let (user_id, expires_at) = {
        let mut reset_tokens = state.password_reset_tokens.write().await;
        match reset_tokens.remove(&confirm_request.token) {
            Some((user_id, expires_at)) => (user_id, expires_at),
            None => {
                warn!("Invalid password reset token");
                return Ok(Json(json!({
                    "success": false,
                    "error": "Invalid or expired reset token"
                })));
            }
        }
    };

    // Check if token is expired
    if chrono::Utc::now() > expires_at {
        warn!("Expired password reset token");
        return Ok(Json(json!({
            "success": false,
            "error": "Reset token has expired"
        })));
    }

    // Find user by ID
    let users = state.users.read().await;
    let mut user = users.values()
        .find(|u| u.id == user_id)
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    drop(users);

    // Hash new password
    let new_password_hash = match state.password_service.hash_password(&confirm_request.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            return Ok(Json(json!({
                "success": false,
                "error": e.to_string()
            })));
        }
    };

    // Update password and unlock account if needed
    user.password_hash = new_password_hash;
    user.unlock_account();
    state.update_user(&user).await;

    info!("Password reset completed for user: {}", user.email);

    Ok(Json(json!({
        "success": true,
        "message": "Password has been reset successfully"
    })))
}

/// Create authentication router
pub fn create_auth_router() -> Router<AuthServiceState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/me", get(me))
        .route("/change-password", post(change_password))
        .route("/reset-password", post(request_password_reset))
        .route("/reset-password/confirm", post(confirm_password_reset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, Method, StatusCode},
    };
    use axum_test::TestServer;

    async fn create_test_server() -> TestServer {
        let state = AuthServiceState::new();
        let app = create_auth_router().with_state(state);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_login_success() {
        let server = create_test_server().await;
        
        let login_request = LoginRequest {
            email: "admin@mevshield.com".to_string(),
            password: "AdminPassword123!".to_string(),
        };
        
        let response = server.post("/login")
            .json(&login_request)
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: serde_json::Value = response.json();
        assert_eq!(body["success"], true);
        assert!(body["data"]["access_token"].is_string());
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let server = create_test_server().await;
        
        let login_request = LoginRequest {
            email: "admin@mevshield.com".to_string(),
            password: "wrong_password".to_string(),
        };
        
        let response = server.post("/login")
            .json(&login_request)
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: serde_json::Value = response.json();
        assert_eq!(body["success"], false);
    }

    #[tokio::test]
    async fn test_login_nonexistent_user() {
        let server = create_test_server().await;
        
        let login_request = LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "password".to_string(),
        };
        
        let response = server.post("/login")
            .json(&login_request)
            .await;
        
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }
}