//! JWT port - interface for token operations

#![allow(dead_code)]

use thiserror::Error;

/// JWT errors
#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Token creation failed: {0}")]
    CreationFailed(String),
    #[error("Token validation failed: {0}")]
    ValidationFailed(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
}

/// Port for JWT operations
///
/// Follows Dependency Inversion Principle - infrastructure provides implementation
pub trait JwtPort: Send + Sync {
    /// Generate a JWT token
    fn generate_token(
        &self,
        username: &str,
        is_superuser: bool,
        expiration_hours: i64,
    ) -> Result<String, JwtError>;

    /// Validate a JWT token
    fn validate_token(&self, token: &str) -> Result<JwtClaims, JwtError>;
}

/// JWT claims
#[derive(Debug, Clone)]
pub struct JwtClaims {
    pub username: String,
    pub is_superuser: bool,
    pub exp: i64,
}
