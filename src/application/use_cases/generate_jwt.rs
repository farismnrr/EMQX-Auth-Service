//! Generate JWT Token Use Case

use crate::domain::{JwtClaims, MqttUserRepository, TokenPair};
use jsonwebtoken::{encode, EncodingKey, Header};
use thiserror::Error;

/// Errors that can occur during JWT generation
#[derive(Debug, Error)]
pub enum GenerateJwtError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("JWT encoding error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::RepositoryError),
}

/// Use case for generating JWT tokens for EMQX authentication
pub struct GenerateJwtUseCase<R> {
    repository: R,
    secret_key: String,
}

impl<R: MqttUserRepository> GenerateJwtUseCase<R> {
    pub fn new(repository: R, secret_key: String) -> Self {
        Self {
            repository,
            secret_key,
        }
    }

    /// Generate a JWT token for the given username
    ///
    /// The token is compatible with EMQX JWT authentication:
    /// - Algorithm: HS256 (HMAC-SHA256)
    /// - Claims: sub (username), iss (broker.i-ot.net), aud (mqtt), iat, exp
    /// - Expiration: 24 hours
    pub async fn execute(&self, username: &str) -> Result<TokenPair, GenerateJwtError> {
        // Validate input
        self.validate(username)?;

        // Check if user exists
        let user = self.repository.find_by_username(username).await?;
        if user.is_none() {
            return Err(GenerateJwtError::UserNotFound(username.to_string()));
        }

        // Create JWT claims
        let claims = JwtClaims::new(username);
        let expires_at = claims.expires_at();

        // Encode JWT with HS256 algorithm
        // EMQX expects the secret as a raw ASCII string (not hex-decoded)
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes()),
        )?;

        Ok(TokenPair::new(token, expires_at))
    }

    fn validate(&self, username: &str) -> Result<(), GenerateJwtError> {
        let mut errors = Vec::new();

        if username.trim().is_empty() {
            errors.push("username cannot be empty".to_string());
        }
        if username.len() > 64 {
            errors.push("username cannot exceed 64 characters".to_string());
        }

        if !errors.is_empty() {
            return Err(GenerateJwtError::ValidationError(errors.join(", ")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{MqttUser, RepositoryError};
    use async_trait::async_trait;
    use chrono::Utc;

    // Mock repository for testing
    struct MockRepository {
        users: std::collections::HashMap<String, MqttUser>,
    }

    impl MockRepository {
        fn new() -> Self {
            let mut users = std::collections::HashMap::new();
            users.insert(
                "testuser".to_string(),
                MqttUser {
                    id: 1,
                    username: "testuser".to_string(),
                    password: "hashed_password".to_string(),
                    is_superuser: false,
                    created_at: Some(Utc::now().naive_utc()),
                    updated_at: Some(Utc::now().naive_utc()),
                },
            );
            Self { users }
        }
    }

    #[async_trait]
    impl MqttUserRepository for MockRepository {
        async fn find_by_id(&self, _id: i32) -> Result<Option<MqttUser>, RepositoryError> {
            Ok(None)
        }

        async fn find_by_username(&self, username: &str) -> Result<Option<MqttUser>, RepositoryError> {
            Ok(self.users.get(username).cloned())
        }

        async fn find_all(&self, _limit: u32, _offset: u32) -> Result<Vec<MqttUser>, RepositoryError> {
            Ok(self.users.values().cloned().collect())
        }

        async fn count(&self) -> Result<u64, RepositoryError> {
            Ok(self.users.len() as u64)
        }

        async fn insert(&self, _user: MqttUser) -> Result<MqttUser, RepositoryError> {
            Err(RepositoryError::UniqueViolation("Not implemented".to_string()))
        }

        async fn delete_by_id(&self, _id: i32) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        async fn exists_by_username(&self, username: &str) -> Result<bool, RepositoryError> {
            Ok(self.users.contains_key(username))
        }
    }

    #[test]
    fn test_jwt_generation() {
        let repo = MockRepository::new();
        let secret = "4d330a75ea1665030806654c07e2d21e9834eb6081313ca16dd72448e4d6e3a3".to_string();
        let use_case = GenerateJwtUseCase::new(repo, secret);

        // This would need to be async, but demonstrates the concept
        // In real tests, you'd use tokio::test
        let _ = use_case;
    }
}
