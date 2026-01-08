//! Error types for vaya-auth

use std::fmt;

/// Result type for auth operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Errors that can occur during authentication
#[derive(Debug, Clone)]
pub enum AuthError {
    /// Invalid credentials
    InvalidCredentials,
    /// Token expired
    TokenExpired,
    /// Invalid token format
    InvalidToken(String),
    /// Token signature verification failed
    SignatureInvalid,
    /// User not found
    UserNotFound,
    /// User already exists
    UserExists,
    /// Account locked
    AccountLocked,
    /// Account suspended
    AccountSuspended,
    /// Session not found
    SessionNotFound,
    /// Session expired
    SessionExpired,
    /// Permission denied
    PermissionDenied,
    /// Missing required permission
    MissingPermission(String),
    /// Invalid password format
    InvalidPasswordFormat(String),
    /// Password too weak
    WeakPassword,
    /// MFA required
    MfaRequired,
    /// Invalid MFA code
    InvalidMfaCode,
    /// Rate limited
    RateLimited,
    /// Internal error
    Internal(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::TokenExpired => write!(f, "Token has expired"),
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            AuthError::SignatureInvalid => write!(f, "Token signature verification failed"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::UserExists => write!(f, "User already exists"),
            AuthError::AccountLocked => write!(f, "Account is locked"),
            AuthError::AccountSuspended => write!(f, "Account is suspended"),
            AuthError::SessionNotFound => write!(f, "Session not found"),
            AuthError::SessionExpired => write!(f, "Session has expired"),
            AuthError::PermissionDenied => write!(f, "Permission denied"),
            AuthError::MissingPermission(perm) => write!(f, "Missing permission: {}", perm),
            AuthError::InvalidPasswordFormat(msg) => write!(f, "Invalid password: {}", msg),
            AuthError::WeakPassword => write!(f, "Password is too weak"),
            AuthError::MfaRequired => write!(f, "MFA verification required"),
            AuthError::InvalidMfaCode => write!(f, "Invalid MFA code"),
            AuthError::RateLimited => write!(f, "Too many attempts, please try again later"),
            AuthError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}
