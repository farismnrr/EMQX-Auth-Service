//! Issue Token Use Case

use crate::domain::MqttUserRepository;
use crate::application::ports::JwtPort;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IssueTokenError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
    #[error("JWT error: {0}")]
    Jwt(String),
}

/// Use case for issuing a JWT token
pub struct IssueTokenUseCase<R, J> {
    repository: R,
    jwt_port: J,
}

impl<R: MqttUserRepository, J: JwtPort> IssueTokenUseCase<R, J> {
    pub fn new(repository: R, jwt_port: J) -> Self {
        Self { repository, jwt_port }
    }

    pub async fn execute(&self, username: &str) -> Result<String, IssueTokenError> {
        let user = self.repository.find_by_username(username).await?
            .ok_or_else(|| IssueTokenError::UserNotFound(username.to_string()))?;

        let token = self.jwt_port.generate_token(&user.username, user.is_superuser, 24)
            .map_err(|e| IssueTokenError::Jwt(e.to_string()))?;

        Ok(token)
    }
}
