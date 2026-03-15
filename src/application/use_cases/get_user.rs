//! Get User Use Case

use crate::application::ports::EncryptionPort;
use crate::domain::MqttUserRepository;
use crate::application::dtos::response::UserDTO;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetUserError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
    #[error("Encryption error: {0}")]
    Encryption(String),
}

/// Use case for getting a user
pub struct GetUserUseCase<R, E> {
    repository: R,
    encryption: E,
}

impl<R: MqttUserRepository, E: EncryptionPort> GetUserUseCase<R, E> {
    pub fn new(repository: R, encryption: E) -> Self {
        Self { repository, encryption }
    }

    pub async fn execute_by_username(
        &self,
        username: &str,
    ) -> Result<UserDTO, GetUserError> {
        let user = self.repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| GetUserError::UserNotFound(username.to_string()))?;
        
        let decrypted = self.encryption.decrypt_password(&user.password_ciphertext)
            .map_err(|e| GetUserError::Encryption(e.to_string()))?;

        Ok(UserDTO::from_domain(user, Some(decrypted)))
    }

    pub async fn execute_by_id(&self, id: i64) -> Result<UserDTO, GetUserError> {
        let user = self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| GetUserError::UserNotFound(id.to_string()))?;

        let decrypted = self.encryption.decrypt_password(&user.password_ciphertext)
            .map_err(|e| GetUserError::Encryption(e.to_string()))?;

        Ok(UserDTO::from_domain(user, Some(decrypted)))
    }
}
