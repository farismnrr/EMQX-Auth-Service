//! Get User Use Case

use crate::domain::MqttUserRepository;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetUserError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
}

/// Use case for getting a user
pub struct GetUserUseCase<R> {
    repository: R,
}

impl<R: MqttUserRepository> GetUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute_by_username(
        &self,
        username: &str,
    ) -> Result<crate::domain::MqttUser, GetUserError> {
        self.repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| GetUserError::UserNotFound(username.to_string()))
    }

    pub async fn execute_by_id(&self, id: i64) -> Result<crate::domain::MqttUser, GetUserError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| GetUserError::UserNotFound(id.to_string()))
    }
}
