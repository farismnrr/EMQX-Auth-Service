use crate::application::commands::CheckAclCommand;
use crate::domain::repository_traits::mqtt_user_repository::{
    MqttUserRepositoryError, MqttUserRepositoryTrait,
};
use crate::domain::services::auth_domain_service::AuthDomainService;
use thiserror::Error;

/// Use case errors for ACL check operation
#[derive(Debug, Error)]
pub enum CheckAclError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Repository error: {0}")]
    Repository(#[from] MqttUserRepositoryError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Use case for checking ACL permissions
pub struct CheckAclUseCase<R: MqttUserRepositoryTrait> {
    repository: R,
    domain_service: AuthDomainService,
}

impl<R: MqttUserRepositoryTrait> CheckAclUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            domain_service: AuthDomainService::new(),
        }
    }

    pub async fn execute(&self, command: CheckAclCommand) -> Result<bool, CheckAclError> {
        // Find user by username
        let user = match self.repository.find_by_username(&command.username).await? {
            Some(user) => user,
            None => return Err(CheckAclError::UserNotFound(command.username)),
        };

        // Check ACL using domain service
        let allowed = self.domain_service.check_acl(&user, &command.topic);

        Ok(allowed)
    }
}
