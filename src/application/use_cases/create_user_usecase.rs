use crate::application::commands::CreateUserCommand;
use crate::application::ports::EncryptionPort;
use crate::domain::entities::mqtt_user::MqttUser;
use crate::domain::repository_traits::mqtt_user_repository::{
    MqttUserRepositoryError, MqttUserRepositoryTrait,
};
use crate::domain::value_objects::password::{Password, PasswordError};
use thiserror::Error;

/// Use case errors for create user operation
#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Password error: {0}")]
    PasswordError(#[from] PasswordError),

    #[error("Repository error: {0}")]
    Repository(#[from] MqttUserRepositoryError),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Use case for creating a new MQTT user
/// 
/// This use case encapsulates the business logic for user creation,
/// following the Single Responsibility Principle.
pub struct CreateUserUseCase<R: MqttUserRepositoryTrait, E: EncryptionPort> {
    repository: R,
    encryption: E,
}

impl<R: MqttUserRepositoryTrait, E: EncryptionPort> CreateUserUseCase<R, E> {
    pub fn new(repository: R, encryption: E) -> Self {
        Self { repository, encryption }
    }

    pub async fn execute(&self, command: CreateUserCommand) -> Result<bool, CreateUserError> {
        // Validate input
        self.validate(&command)?;

        // Check if user already exists
        if self.repository.exists_by_username(&command.username).await? {
            return Err(CreateUserError::UserAlreadyExists(command.username));
        }

        // Create password value object (includes hashing)
        let password = Password::new(command.password)?;

        // Create domain entity
        let user = MqttUser::new(command.username, password, command.is_superuser);

        // Persist user
        self.repository.insert(user).await?;

        Ok(true)
    }

    fn validate(&self, command: &CreateUserCommand) -> Result<(), CreateUserError> {
        let mut errors = Vec::new();

        if command.username.trim().is_empty() {
            errors.push("username cannot be empty".to_string());
        }

        if command.username.len() > 64 {
            errors.push("username cannot exceed 64 characters".to_string());
        }

        if command.password.is_empty() {
            errors.push("password cannot be empty".to_string());
        }

        if !errors.is_empty() {
            return Err(CreateUserError::ValidationError(errors.join(", ")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::mqtt_user::MqttUser;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    // Mock repository for testing
    struct MockRepository {
        users: Arc<Mutex<Vec<MqttUser>>>,
    }

    impl MockRepository {
        fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl MqttUserRepositoryTrait for MockRepository {
        async fn find_by_id(&self, id: i64) -> Result<Option<MqttUser>, MqttUserRepositoryError> {
            Ok(self.users.lock().unwrap().iter().find(|u| u.id() == Some(id)).cloned())
        }

        async fn find_by_username(&self, username: &str) -> Result<Option<MqttUser>, MqttUserRepositoryError> {
            Ok(self.users.lock().unwrap().iter().find(|u| u.username() == username).cloned())
        }

        async fn find_all(&self, limit: u32, offset: u32) -> Result<Vec<MqttUser>, MqttUserRepositoryError> {
            Ok(self.users.lock().unwrap().iter().skip(offset as usize).take(limit as usize).cloned().collect())
        }

        async fn count(&self) -> Result<u64, MqttUserRepositoryError> {
            Ok(self.users.lock().unwrap().len() as u64)
        }

        async fn insert(&self, user: MqttUser) -> Result<MqttUser, MqttUserRepositoryError> {
            self.users.lock().unwrap().push(user.clone());
            Ok(user)
        }

        async fn update(&self, user: MqttUser) -> Result<MqttUser, MqttUserRepositoryError> {
            Ok(user)
        }

        async fn delete_by_username(&self, username: &str) -> Result<(), MqttUserRepositoryError> {
            self.users.lock().unwrap().retain(|u| u.username() != username);
            Ok(())
        }

        async fn exists_by_username(&self, username: &str) -> Result<bool, MqttUserRepositoryError> {
            Ok(self.users.lock().unwrap().iter().any(|u| u.username() == username))
        }
    }

    // Mock encryption for testing
    struct MockEncryption;

    impl EncryptionPort for MockEncryption {
        fn encrypt(&self, password: &str) -> Result<String, EncryptionError> {
            Ok(format!("encrypted_{}", password))
        }

        fn decrypt(&self, encrypted_password: &str) -> Result<String, EncryptionError> {
            Ok(encrypted_password.trim_start_matches("encrypted_").to_string())
        }

        fn generate_key() -> String {
            "test-key".to_string()
        }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let repo = MockRepository::new();
        let encryption = MockEncryption;
        let use_case = CreateUserUseCase::new(repo, encryption);

        let command = CreateUserCommand {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            is_superuser: false,
        };

        let result = use_case.execute(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_user_duplicate() {
        let repo = MockRepository::new();
        let encryption = MockEncryption;
        let use_case = CreateUserUseCase::new(repo.clone(), encryption);

        let command = CreateUserCommand {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            is_superuser: false,
        };

        // Create first user
        let result = use_case.execute(command.clone()).await;
        assert!(result.is_ok());

        // Try to create duplicate
        let result = use_case.execute(command).await;
        assert!(matches!(result, Err(CreateUserError::UserAlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_create_user_validation_empty_username() {
        let repo = MockRepository::new();
        let encryption = MockEncryption;
        let use_case = CreateUserUseCase::new(repo, encryption);

        let command = CreateUserCommand {
            username: "".to_string(),
            password: "password123".to_string(),
            is_superuser: false,
        };

        let result = use_case.execute(command).await;
        assert!(matches!(result, Err(CreateUserError::ValidationError(_))));
    }
}
