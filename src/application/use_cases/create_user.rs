//! Create User Use Case

use crate::application::ports::EncryptionPort;
use crate::domain::{MqttUser, MqttUserRepository};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
    #[error("Encryption error: {0}")]
    Encryption(String),
}

/// Use case for creating a new MQTT user
pub struct CreateUserUseCase<R, E> {
    repository: R,
    encryption: E,
}

impl<R: MqttUserRepository, E: EncryptionPort> CreateUserUseCase<R, E> {
    pub fn new(repository: R, encryption: E) -> Self {
        Self {
            repository,
            encryption,
        }
    }

    pub async fn execute(
        &self,
        username: &str,
        password: &str,
        is_superuser: bool,
    ) -> Result<bool, CreateUserError> {
        // Validate input (SRP - validation is part of this use case)
        self.validate(username, password)?;

        // Check if user exists
        if self.repository.exists_by_username(username).await? {
            return Err(CreateUserError::UserAlreadyExists(username.to_string()));
        }

        // Hash password
        let password_hash = self
            .encryption
            .hash_password(password)
            .map_err(|e| CreateUserError::Encryption(e.to_string()))?;

        // Create user
        let now = chrono::Utc::now();
        let user = MqttUser {
            id: 0, // Will be set by database
            username: username.to_string(),
            password_hash,
            is_superuser,
            created_at: now,
            updated_at: now,
        };

        self.repository.insert(user).await?;
        Ok(true)
    }

    fn validate(&self, username: &str, password: &str) -> Result<(), CreateUserError> {
        let mut errors = Vec::new();

        if username.trim().is_empty() {
            errors.push("username cannot be empty".to_string());
        }
        if username.len() > 64 {
            errors.push("username cannot exceed 64 characters".to_string());
        }
        if password.is_empty() {
            errors.push("password cannot be empty".to_string());
        }
        if password.len() < 6 {
            errors.push("password must be at least 6 characters".to_string());
        }

        if !errors.is_empty() {
            return Err(CreateUserError::ValidationError(errors.join(", ")));
        }

        Ok(())
    }
}
