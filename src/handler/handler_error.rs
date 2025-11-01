use actix_web::http::StatusCode;
use crate::utils::app_error::AppError;
use crate::repositories::repository_error::UserRepositoryError;
use crate::services::service_error::UserServiceError;

impl AppError for UserRepositoryError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Encode(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Decode(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Utf8(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        self.to_string()
    }
}

impl AppError for UserServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Repository(e) => e.status_code(),
            Self::Hashing(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        self.to_string()
    }
}
