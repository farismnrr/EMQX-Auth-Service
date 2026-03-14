use crate::application::commands::VerifyPasswordCommand;
use crate::domain::repository_traits::mqtt_user_repository::{
    MqttUserRepositoryError, MqttUserRepositoryTrait,
};
use thiserror::Error;

/// Use case errors for verify password operation
#[derive(Debug, Error)]
pub enum VerifyPasswordError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Repository error: {0}")]
    Repository(#[from] MqttUserRepositoryError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Use case for verifying a user's password
pub struct VerifyPasswordUseCase<R: MqttUserRepositoryTrait> {
    repository: R,
}

impl<R: MqttUserRepositoryTrait> VerifyPasswordUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: VerifyPasswordCommand) -> Result<bool, VerifyPasswordError> {
        // Find user by username
        let user = match self.repository.find_by_username(&command.username).await? {
            Some(user) => user,
            None => return Err(VerifyPasswordError::UserNotFound(command.username)),
        };

        // Verify password
        let valid = user.verify_password(&command.password);

        if !valid {
            return Err(VerifyPasswordError::InvalidCredentials);
        }

        Ok(true)
    }
}
