//! Authenticate User Use Case

use crate::application::ports::JwtPort;
use crate::domain::MqttUserRepository;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthenticateUserError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
    #[error("JWT error: {0}")]
    Jwt(String),
}

/// Use case for authenticating a user
pub struct AuthenticateUserUseCase<R, J> {
    repository: R,
    jwt_port: J,
}

impl<R: MqttUserRepository, J: JwtPort> AuthenticateUserUseCase<R, J> {
    pub fn new(repository: R, jwt_port: J) -> Self {
        Self {
            repository,
            jwt_port,
        }
    }

    /// Authenticate user for EMQX - returns (is_authenticated, is_superuser)
    pub async fn execute_for_emqx(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(bool, bool), AuthenticateUserError> {
        let user = self
            .repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| AuthenticateUserError::UserNotFound(username.to_string()))?;

        if !user.verify_password(password) {
            return Err(AuthenticateUserError::InvalidCredentials);
        }

        Ok((true, user.is_superuser))
    }

    pub async fn execute(
        &self,
        username: &str,
        password: &str,
        issue_token: bool,
    ) -> Result<(bool, Option<String>), AuthenticateUserError> {
        let user = self
            .repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| AuthenticateUserError::UserNotFound(username.to_string()))?;

        if !user.verify_password(password) {
            return Err(AuthenticateUserError::InvalidCredentials);
        }

        let token = if issue_token {
            Some(
                self.jwt_port
                    .generate_token(&user.username, user.is_superuser, 24)
                    .map_err(|e| AuthenticateUserError::Jwt(e.to_string()))?,
            )
        } else {
            None
        };

        Ok((true, token))
    }
}
