use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;

use crate::dtos::response_dto::ErrorResponseDTO;
use crate::repositories::repository_error::MqttRepositoryError;
use crate::services::service_error::{MqttServiceError, ValidationError};

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

    fn to_http_response_with_result<T: Serialize>(
        &self,
        result: Option<&str>,
        details: Option<T>,
    ) -> HttpResponse {
        let response = ErrorResponseDTO {
            success: false,
            message: &self.message(),
            result,
            details,
        };
        HttpResponse::build(self.status_code()).json(response)
    }
}

impl AppError for MqttRepositoryError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn message(&self) -> String {
        self.to_string()
    }
}

impl AppError for MqttServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Repository(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MqttNotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidCredentials(_) => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::JwtError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::BadRequest(_) => "Validation error".to_string(),
            _ => self.to_string(),
        }
    }

    fn to_http_response(&self) -> HttpResponse {
        match self {
            MqttServiceError::BadRequest(errors) => self
                .to_http_response_with_result::<&Vec<ValidationError>>(Some("deny"), Some(errors)),
            _ => self.to_http_response_with_details::<()>(None),
        }
    }
}
