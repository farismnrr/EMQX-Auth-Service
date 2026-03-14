//! Verify Password Use Case

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
}

/// Use case for verifying a password
pub struct VerifyPasswordUseCase<R> {
    repository: R,
}

impl<R: MqttUserRepository> VerifyPasswordUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, username: &str, password: &str) -> Result<bool, VerifyPasswordError> {
        let user = self.repository.find_by_username(username).await?
            .ok_or_else(|| VerifyPasswordError::UserNotFound(username.to_string()))?;

        if !user.verify_password(password) {
            return Err(VerifyPasswordError::InvalidCredentials);
        }

        Ok(true)
    }
}
