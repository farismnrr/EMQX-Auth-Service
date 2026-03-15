//! Error utility for handling internal server errors

use actix_web::HttpResponse;
use tracing::error;
use crate::application::ErrorResponse;

/// Standard message for internal server errors to prevent information leakage
pub const GENERIC_INTERNAL_ERROR: &str = "Internal server error";

/// Maps any internal error to a 500 response while logging the actual error.
/// 
/// This ensures that sensitive information is not leaked to the client while still providing a way to trace errors.
pub fn map_internal_error(error: impl std::fmt::Display, operation: &str) -> HttpResponse {
    error!(operation = operation, error = %error, "Internal server error occurred");
    
    HttpResponse::InternalServerError().json(ErrorResponse::new(GENERIC_INTERNAL_ERROR))
}
