use crate::application::commands::DeleteUserCommand;
use crate::domain::repository_traits::mqtt_user_repository::{
    MqttUserRepositoryError, MqttUserRepositoryTrait,
};
use thiserror::Error;

/// Use case errors for delete user operation
#[derive(Debug, Error)]
pub enum DeleteUserError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Repository error: {0}")]
    Repository(#[from] MqttUserRepositoryError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Use case for deleting a user
pub struct DeleteUserUseCase<R: MqttUserRepositoryTrait> {
    repository: R,
}

impl<R: MqttUserRepositoryTrait> DeleteUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: DeleteUserCommand) -> Result<bool, DeleteUserError> {
        self.repository.delete_by_username(&command.username).await?;
        Ok(true)
    }
}
