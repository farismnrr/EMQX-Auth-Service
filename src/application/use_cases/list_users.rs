//! List Users Use Case

use crate::domain::{MqttUser, MqttUserRepository};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListUsersError {
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
}

/// User list with pagination
pub struct UserList {
    pub users: Vec<MqttUser>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

/// Use case for listing users
pub struct ListUsersUseCase<R> {
    repository: R,
}

impl<R: MqttUserRepository> ListUsersUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, limit: u32, offset: u32) -> Result<UserList, ListUsersError> {
        let total = self.repository.count().await?;
        let users = self.repository.find_all(limit, offset).await?;

        Ok(UserList {
            users,
            total,
            limit,
            offset,
        })
    }
}
