use crate::application::commands::GetUserByIdCommand;
use crate::application::dtos::response_dto::UserDTO;
use crate::domain::repository_traits::mqtt_user_repository::{
    MqttUserRepositoryError, MqttUserRepositoryTrait,
};
use thiserror::Error;

/// Use case errors for get user by ID operation
#[derive(Debug, Error)]
pub enum GetUserByIdError {
    #[error("User not found: {0}")]
    UserNotFound(i64),

    #[error("Repository error: {0}")]
    Repository(#[from] MqttUserRepositoryError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Use case for getting a user by ID
pub struct GetUserByIdUseCase<R: MqttUserRepositoryTrait> {
    repository: R,
}

impl<R: MqttUserRepositoryTrait> GetUserByIdUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: GetUserByIdCommand) -> Result<UserDTO, GetUserByIdError> {
        let user = match self.repository.find_by_id(command.id).await? {
            Some(user) => user,
            None => return Err(GetUserByIdError::UserNotFound(command.id)),
        };

        Ok(UserDTO {
            id: user.id().unwrap_or(0),
            username: user.username().to_string(),
            is_superuser: user.is_superuser(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }
}
