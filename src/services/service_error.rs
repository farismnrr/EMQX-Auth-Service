use thiserror::Error;
use serde::Serialize;
use crate::repositories::repository_error::MqttRepositoryError;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum MqttServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] MqttRepositoryError),

    #[error("{0}")]
    MqttNotFound(String),

    #[error("{0}")]
    InvalidCredentials(String),

    #[error("{0}")]
    Conflict(String),

    #[error("Bad request")]
    BadRequest(Vec<ValidationError>),

    #[error("Mqtt is not active or deleted: {0}")]
    MqttNotActive(String),

    #[error("JWT error: {0}")]
    JwtError(String),
}