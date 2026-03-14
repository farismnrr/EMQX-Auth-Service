use crate::application::commands::GetUserByUsernameCommand;
use crate::application::dtos::response_dto::UserDTO;
use crate::domain::repository_traits::mqtt_user_repository::{
    MqttUserRepositoryError, MqttUserRepositoryTrait,
};
use thiserror::Error;

/// Use case errors for get user by username operation
#[derive(Debug, Error)]
pub enum GetUserByUsernameError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Repository error: {0}")]
    Repository(#[from] MqttUserRepositoryError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Use case for getting a user by username
pub struct GetUserByUsernameUseCase<R: MqttUserRepositoryTrait> {
    repository: R,
}

impl<R: MqttUserRepositoryTrait> GetUserByUsernameUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: GetUserByUsernameCommand,
    ) -> Result<UserDTO, GetUserByUsernameError> {
        let user = match self.repository.find_by_username(&command.username).await? {
            Some(user) => user,
            None => return Err(GetUserByUsernameError::UserNotFound(command.username)),
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
