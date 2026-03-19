//! Utility functions

use actix_web::HttpResponse;

/// Map internal errors to HTTP 500 response
pub fn map_internal_error<E: std::fmt::Display>(e: E, _operation: &str) -> HttpResponse {
    let msg = e.to_string();
    HttpResponse::InternalServerError().json(serde_json::json!({
        "success": false,
        "message": msg
    }))
}
