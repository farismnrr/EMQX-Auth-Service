use crate::application::commands::AuthenticateUserCommand;
use crate::application::ports::JwtPort;
use crate::domain::repository_traits::mqtt_user_repository::{
    MqttUserRepositoryError, MqttUserRepositoryTrait,
};
use thiserror::Error;

/// Use case errors for authenticate user operation
#[derive(Debug, Error)]
pub enum AuthenticateUserError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Repository error: {0}")]
    Repository(#[from] MqttUserRepositoryError),

    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Use case for authenticating a user
pub struct AuthenticateUserUseCase<R: MqttUserRepositoryTrait, J: JwtPort> {
    repository: R,
    jwt_port: J,
    secret_key: String,
}

impl<R: MqttUserRepositoryTrait, J: JwtPort> AuthenticateUserUseCase<R, J> {
    pub fn new(repository: R, jwt_port: J, secret_key: impl Into<String>) -> Self {
        Self {
            repository,
            jwt_port,
            secret_key: secret_key.into(),
        }
    }

    /// Authenticate user and return (is_authenticated, is_superuser, token)
    pub async fn execute_for_emqx(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(bool, bool), AuthenticateUserError> {
        // Find user by username
        let user = match self.repository.find_by_username(username).await? {
            Some(user) => user,
            None => return Err(AuthenticateUserError::UserNotFound(username.to_string())),
        };

        // Verify password
        if !user.verify_password(password) {
            return Err(AuthenticateUserError::InvalidCredentials);
        }

        Ok((true, user.is_superuser()))
    }

    pub async fn execute(
        &self,
        command: AuthenticateUserCommand,
        issue_token: bool,
    ) -> Result<(bool, Option<String>), AuthenticateUserError> {
        // Find user by username
        let user = match self.repository.find_by_username(&command.username).await? {
            Some(user) => user,
            None => return Err(AuthenticateUserError::UserNotFound(command.username)),
        };

        // Verify password
        if !user.verify_password(&command.password) {
            return Err(AuthenticateUserError::InvalidCredentials);
        }

        // Issue token if requested
        let token = if issue_token {
            Some(
                self.jwt_port
                    .generate_token(&user.username(), user.is_superuser(), 24)
                    .map_err(|e| AuthenticateUserError::JwtError(e.to_string()))?,
            )
        } else {
            None
        };

        Ok((true, token))
    }
}
