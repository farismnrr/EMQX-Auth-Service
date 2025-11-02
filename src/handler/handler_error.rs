//! Error handling and HTTP response conversion for the application.

use actix_web::{http::StatusCode, HttpResponse};
use serde::Serialize;

use crate::dtos::response_dto::ErrorResponseDTO;
use crate::repositories::repository_error::UserRepositoryError;
use crate::services::service_error::{UserServiceError, ValidationError};

pub trait AppError: Sized {
    fn status_code(&self) -> StatusCode;
    fn message(&self) -> String;

    fn to_http_response(&self) -> HttpResponse {
        self.to_http_response_with_details::<()>(None)
    }

    fn to_http_response_with_details<T: Serialize>(&self, details: Option<T>) -> HttpResponse {
        let response = ErrorResponseDTO {
            success: false,
            message: &self.message(),
            result: None,
            details,
        };
        HttpResponse::build(self.status_code()).json(response)
    }
}

impl AppError for UserRepositoryError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn message(&self) -> String {
        self.to_string()
    }
}

impl AppError for UserServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Repository(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UserNotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidCredentials(_) => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::UserNotActive(_) => StatusCode::FORBIDDEN,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::BadRequest(_) => "Validation error".to_string(),
            _ => self.to_string(),
        }
    }
}

pub fn handle_check_user_active_error(error: &UserServiceError) -> HttpResponse {
    match error {
        UserServiceError::BadRequest(errors) => {
            let response: ErrorResponseDTO<&Vec<ValidationError>> = ErrorResponseDTO {
                success: false,
                message: "Validation error",
                result: Some("deny"),
                details: Some(errors),
            };
            HttpResponse::BadRequest().json(response)
        }
        _ => error.to_http_response(),
    }
}

