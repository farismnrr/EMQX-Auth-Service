//! Delete User Use Case

use crate::domain::MqttUserRepository;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeleteUserError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
}

/// Use case for deleting a user
pub struct DeleteUserUseCase<R> {
    repository: R,
}

impl<R: MqttUserRepository> DeleteUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, username: &str) -> Result<bool, DeleteUserError> {
        self.repository.delete_by_username(username).await?;
        Ok(true)
    }
}
