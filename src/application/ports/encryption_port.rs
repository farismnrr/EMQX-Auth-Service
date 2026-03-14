//! Encryption port - interface for password hashing

/// Encryption errors
#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Hash error: {0}")]
    HashError(String),
}

/// Port for encryption operations
///
/// Follows Dependency Inversion Principle - infrastructure provides implementation
pub trait EncryptionPort: Send + Sync {
    /// Hash a password
    fn hash_password(&self, password: &str) -> Result<String, EncryptionError>;

    /// Verify a password against a hash
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, EncryptionError>;
}
