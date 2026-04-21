//! Create User Use Case

use crate::domain::{MqttUser, MqttUserRepository};
use bcrypt::{hash, DEFAULT_COST};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
    #[error("Failed to hash password: {0}")]
    HashError(String),
}

/// Use case for creating a new MQTT user
pub struct CreateUserUseCase<R> {
    repository: R,
}

impl<R: MqttUserRepository> CreateUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
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

        // Hash password with bcrypt
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| CreateUserError::HashError(e.to_string()))?;

        // Create user with hashed password
        let now = chrono::Utc::now();
        let naive_now = now.naive_utc();
        let user = MqttUser {
            id: 0, // Will be set by database
            username: username.to_string(),
            password: password_hash,
            is_superuser,
            created_at: Some(naive_now),
            updated_at: Some(naive_now),
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
