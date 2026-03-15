//! Authenticate User Use Case

use crate::application::ports::{JwtPort, EncryptionPort};
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
    #[error("Encryption error: {0}")]
    Encryption(String),
}

/// Use case for authenticating a user
pub struct AuthenticateUserUseCase<R, J, E> {
    repository: R,
    jwt_port: J,
    encryption: E,
}

impl<R: MqttUserRepository, J: JwtPort, E: EncryptionPort> AuthenticateUserUseCase<R, J, E> {
    pub fn new(repository: R, jwt_port: J, encryption: E) -> Self {
        Self {
            repository,
            jwt_port,
            encryption,
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

        let is_valid = self.encryption.verify_password(password, &user.password_ciphertext)
            .map_err(|e| AuthenticateUserError::Encryption(e.to_string()))?;

        if !is_valid {
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

        let is_valid = self.encryption.verify_password(password, &user.password_ciphertext)
            .map_err(|e| AuthenticateUserError::Encryption(e.to_string()))?;

        if !is_valid {
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
