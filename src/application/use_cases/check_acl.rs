//! Check ACL Use Case

use crate::domain::MqttUserRepository;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckAclError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
}

/// Use case for checking ACL permissions
pub struct CheckAclUseCase<R> {
    repository: R,
}

impl<R: MqttUserRepository> CheckAclUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, username: &str, topic: &str) -> Result<bool, CheckAclError> {
        let user = self
            .repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| CheckAclError::UserNotFound(username.to_string()))?;

        // ACL logic: superusers have access to everything
        // Regular users can access topics containing their username
        let allowed = if user.is_superuser() {
            true
        } else {
            topic.contains(&user.username)
        };

        Ok(allowed)
    }
}
