use crate::application::commands::ListUsersCommand;
use crate::application::dtos::response_dto::{PaginationInfo, UserDTO, UserListDTO};
use crate::domain::repository_traits::mqtt_user_repository::{
    MqttUserRepositoryError, MqttUserRepositoryTrait,
};
use thiserror::Error;

/// Use case errors for list users operation
#[derive(Debug, Error)]
pub enum ListUsersError {
    #[error("Repository error: {0}")]
    Repository(#[from] MqttUserRepositoryError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Use case for listing users with pagination
pub struct ListUsersUseCase<R: MqttUserRepositoryTrait> {
    repository: R,
}

impl<R: MqttUserRepositoryTrait> ListUsersUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: ListUsersCommand) -> Result<UserListDTO, ListUsersError> {
        // Get total count
        let total = self.repository.count().await?;

        // Get users with pagination
        let users = self
            .repository
            .find_all(command.limit, command.offset)
            .await?;

        // Convert to DTOs
        let user_dtos: Vec<UserDTO> = users
            .into_iter()
            .map(|user| UserDTO {
                id: user.id().unwrap_or(0),
                username: user.username().to_string(),
                is_superuser: user.is_superuser(),
                created_at: user.created_at,
                updated_at: user.updated_at,
            })
            .collect();

        let has_more = (command.offset + command.limit) < total as u32;

        Ok(UserListDTO {
            users: user_dtos,
            total,
            limit: command.limit,
            offset: command.offset,
        })
    }
}
