//! Encryption adapter - bcrypt implementation

use crate::application::ports::encryption_port::{EncryptionError, EncryptionPort};

/// Bcrypt encryption adapter
#[derive(Clone)]
pub struct EncryptionAdapter;

impl EncryptionAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EncryptionAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptionPort for EncryptionAdapter {
    fn hash_password(&self, password: &str) -> Result<String, EncryptionError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| EncryptionError::HashError(e.to_string()))
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, EncryptionError> {
        Ok(bcrypt::verify(password, hash).unwrap_or(false))
    }
}
