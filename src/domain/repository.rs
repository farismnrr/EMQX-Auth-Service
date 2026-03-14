//! Repository traits for data access abstraction

#![allow(dead_code)]

use crate::domain::MqttUser;
use async_trait::async_trait;

/// Repository errors
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[allow(dead_code)]
    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),
}

/// Repository trait for MQTT user operations
///
/// Follows Dependency Inversion Principle (DIP):
/// - High-level modules (use cases) depend on this abstraction
/// - Low-level modules (database) implement this interface
#[async_trait]
pub trait MqttUserRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<MqttUser>, RepositoryError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<MqttUser>, RepositoryError>;
    async fn find_all(&self, limit: u32, offset: u32) -> Result<Vec<MqttUser>, RepositoryError>;
    async fn count(&self) -> Result<u64, RepositoryError>;
    async fn insert(&self, user: MqttUser) -> Result<MqttUser, RepositoryError>;
    async fn update(&self, user: MqttUser) -> Result<MqttUser, RepositoryError>;
    async fn delete_by_username(&self, username: &str) -> Result<(), RepositoryError>;
    async fn exists_by_username(&self, username: &str) -> Result<bool, RepositoryError>;
}
