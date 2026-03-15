//! Verify Password Use Case

use crate::application::ports::EncryptionPort;
use crate::domain::MqttUserRepository;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifyPasswordError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
    #[error("Encryption error: {0}")]
    Encryption(String),
}

/// Use case for verifying a password
pub struct VerifyPasswordUseCase<R, E> {
    repository: R,
    encryption: E,
}

impl<R: MqttUserRepository, E: EncryptionPort> VerifyPasswordUseCase<R, E> {
    pub fn new(repository: R, encryption: E) -> Self {
        Self { repository, encryption }
    }

    pub async fn execute(&self, username: &str, password: &str) -> Result<bool, VerifyPasswordError> {
        let user = self.repository.find_by_username(username).await?
            .ok_or_else(|| VerifyPasswordError::UserNotFound(username.to_string()))?;

        let is_valid = self.encryption.verify_password(password, &user.password_ciphertext)
            .map_err(|e| VerifyPasswordError::Encryption(e.to_string()))?;

        if !is_valid {
            return Err(VerifyPasswordError::InvalidCredentials);
        }

        Ok(true)
    }
}
