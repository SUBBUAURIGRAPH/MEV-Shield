//! Authentication Models
//!
//! Data structures for user authentication and authorization.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// User model for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
}

/// User roles for role-based access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    User,
    Validator,
    ReadOnly,
}

impl UserRole {
    /// Check if role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Check if role can access protected endpoints
    pub fn can_access_protected(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::User | UserRole::Validator)
    }

    /// Check if role can view analytics
    pub fn can_view_analytics(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::User | UserRole::ReadOnly)
    }
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub email: String,      // User email
    pub role: UserRole,     // User role
    pub exp: i64,          // Expiration time
    pub iat: i64,          // Issued at
    pub nbf: i64,          // Not before
    pub iss: String,       // Issuer
    pub aud: String,       // Audience
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

/// Public user information (no sensitive data)
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub last_login: Option<DateTime<Utc>>,
}

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Password change request
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Password reset request
#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

/// Password reset confirmation
#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

impl User {
    /// Create a new user
    pub fn new(email: String, password_hash: String, role: UserRole) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            role,
            is_active: true,
            created_at: Utc::now(),
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
        }
    }

    /// Check if account is locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    /// Lock account for specified duration (in minutes)
    pub fn lock_account(&mut self, duration_minutes: i64) {
        self.locked_until = Some(Utc::now() + chrono::Duration::minutes(duration_minutes));
    }

    /// Unlock account
    pub fn unlock_account(&mut self) {
        self.locked_until = None;
        self.failed_login_attempts = 0;
    }

    /// Record failed login attempt
    pub fn record_failed_login(&mut self) {
        self.failed_login_attempts += 1;
        
        // Lock account after 5 failed attempts
        if self.failed_login_attempts >= 5 {
            self.lock_account(15); // Lock for 15 minutes
        }
    }

    /// Record successful login
    pub fn record_successful_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.failed_login_attempts = 0;
        self.locked_until = None;
    }

    /// Get public user info
    pub fn to_public_info(&self) -> UserInfo {
        UserInfo {
            id: self.id,
            email: self.email.clone(),
            role: self.role.clone(),
            last_login: self.last_login,
        }
    }
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        user.to_public_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            UserRole::User,
        );
        
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::User);
        assert!(user.is_active);
        assert_eq!(user.failed_login_attempts, 0);
        assert!(user.locked_until.is_none());
    }

    #[test]
    fn test_account_locking() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            UserRole::User,
        );
        
        assert!(!user.is_locked());
        
        user.lock_account(15);
        assert!(user.is_locked());
        
        user.unlock_account();
        assert!(!user.is_locked());
        assert_eq!(user.failed_login_attempts, 0);
    }

    #[test]
    fn test_failed_login_attempts() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            UserRole::User,
        );
        
        for i in 1..5 {
            user.record_failed_login();
            assert_eq!(user.failed_login_attempts, i);
            assert!(!user.is_locked());
        }
        
        // 5th attempt should lock the account
        user.record_failed_login();
        assert_eq!(user.failed_login_attempts, 5);
        assert!(user.is_locked());
    }

    #[test]
    fn test_user_roles() {
        assert!(UserRole::Admin.is_admin());
        assert!(!UserRole::User.is_admin());
        
        assert!(UserRole::Admin.can_access_protected());
        assert!(UserRole::User.can_access_protected());
        assert!(UserRole::Validator.can_access_protected());
        assert!(!UserRole::ReadOnly.can_access_protected());
        
        assert!(UserRole::Admin.can_view_analytics());
        assert!(UserRole::User.can_view_analytics());
        assert!(UserRole::ReadOnly.can_view_analytics());
    }
}