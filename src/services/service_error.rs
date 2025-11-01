use thiserror::Error;
use crate::repositories::repository_error::UserRepositoryError;

#[derive(Debug, Error)]
pub enum UserServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] UserRepositoryError),

    #[error("Hashing error: {0}")]
    Hashing(String),
}
