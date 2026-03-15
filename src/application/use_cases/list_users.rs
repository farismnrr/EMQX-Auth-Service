//! List Users Use Case

use crate::domain::MqttUserRepository;
use crate::application::ports::EncryptionPort;
use crate::application::dtos::response::{UserDTO, UserListDTO};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListUsersError {
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
    #[error("Encryption error: {0}")]
    Encryption(String),
}

/// Use case for listing users
pub struct ListUsersUseCase<R, E> {
    repository: R,
    encryption: E,
}

impl<R: MqttUserRepository, E: EncryptionPort> ListUsersUseCase<R, E> {
    pub fn new(repository: R, encryption: E) -> Self {
        Self { repository, encryption }
    }

    pub async fn execute(&self, limit: u32, offset: u32) -> Result<UserListDTO, ListUsersError> {
        let total = self.repository.count().await?;
        let users = self.repository.find_all(limit, offset).await?;

        let mut user_dtos = Vec::new();
        for user in users {
            let decrypted = self.encryption.decrypt_password(&user.password_ciphertext)
                .map_err(|e| ListUsersError::Encryption(e.to_string()))?;
            
            user_dtos.push(UserDTO::from_domain(user, Some(decrypted)));
        }

        Ok(UserListDTO {
            users: user_dtos,
            total,
            limit,
            offset,
        })
    }
}
