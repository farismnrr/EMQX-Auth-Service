//! List Users Use Case

use crate::domain::MqttUserRepository;
use crate::application::dtos::response::{UserDTO, UserListDTO};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListUsersError {
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
}

/// Use case for listing users
pub struct ListUsersUseCase<R> {
    repository: R,
}

impl<R: MqttUserRepository> ListUsersUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, limit: u32, offset: u32) -> Result<UserListDTO, ListUsersError> {
        let total = self.repository.count().await?;
        let users = self.repository.find_all(limit, offset).await?;

        let user_dtos: Vec<UserDTO> = users.into_iter()
            .map(|user| UserDTO::from_domain(user, None))
            .collect();

        Ok(UserListDTO {
            users: user_dtos,
            total,
            limit,
            offset,
        })
    }
}
