use crate::repositories::repository_error::MqttRepositoryError;
use serde::Serialize;
use thiserror::Error;

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

    #[error("JWT error: {0}")]
    JwtError(String),
}
