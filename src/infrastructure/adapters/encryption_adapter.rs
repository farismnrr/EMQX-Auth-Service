//! Encryption adapter - AES-256-GCM implementation

use crate::application::ports::encryption_port::{EncryptionError, EncryptionPort};
use crate::utils::encryption_util;

/// AES-256-GCM encryption adapter
#[derive(Clone)]
pub struct EncryptionAdapter {
    key: [u8; 32],
}

impl EncryptionAdapter {
    /// Create a new EncryptionAdapter with the provided 32-byte key
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }
}

impl EncryptionPort for EncryptionAdapter {
    fn encrypt_password(&self, password: &str) -> Result<String, EncryptionError> {
        encryption_util::aes256_gcm_encrypt(password, &self.key)
            .map_err(|e| EncryptionError::HashError(e.to_string()))
    }

    fn decrypt_password(&self, encrypted_password: &str) -> Result<String, EncryptionError> {
        encryption_util::aes256_gcm_decrypt(encrypted_password, &self.key)
            .map_err(|e| EncryptionError::HashError(e.to_string()))
    }

    fn verify_password(&self, password: &str, encrypted_password: &str) -> Result<bool, EncryptionError> {
        let decrypted = self.decrypt_password(encrypted_password)?;
        Ok(password == decrypted)
    }
}
