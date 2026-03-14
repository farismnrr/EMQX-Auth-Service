use crate::application::commands::IssueTokenCommand;
use crate::application::ports::JwtPort;
use crate::domain::repository_traits::mqtt_user_repository::{
    MqttUserRepositoryError, MqttUserRepositoryTrait,
};
use thiserror::Error;

/// Use case errors for issue token operation
#[derive(Debug, Error)]
pub enum IssueTokenError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Repository error: {0}")]
    Repository(#[from] MqttUserRepositoryError),

    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Use case for issuing a JWT token
pub struct IssueTokenUseCase<R: MqttUserRepositoryTrait, J: JwtPort> {
    repository: R,
    jwt_port: J,
}

impl<R: MqttUserRepositoryTrait, J: JwtPort> IssueTokenUseCase<R, J> {
    pub fn new(repository: R, jwt_port: J) -> Self {
        Self { repository, jwt_port }
    }

    pub async fn execute(&self, command: IssueTokenCommand) -> Result<String, IssueTokenError> {
        // Find user by username
        let user = match self.repository.find_by_username(&command.username).await? {
            Some(user) => user,
            None => return Err(IssueTokenError::UserNotFound(command.username)),
        };

        // Generate JWT token
        let token = self
            .jwt_port
            .generate_token(user.username(), user.is_superuser(), 24)
            .map_err(|e| IssueTokenError::JwtError(e.to_string()))?;

        Ok(token)
    }
}
