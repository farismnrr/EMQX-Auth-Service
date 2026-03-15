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
pub trait EncryptionPort: Send + Sync {
    /// Encrypt a password
    fn encrypt_password(&self, password: &str) -> Result<String, EncryptionError>;

    /// Decrypt a password
    fn decrypt_password(&self, encrypted_password: &str) -> Result<String, EncryptionError>;

    /// Verify a password against an encrypted password
    fn verify_password(&self, password: &str, encrypted_password: &str) -> Result<bool, EncryptionError>;
}
