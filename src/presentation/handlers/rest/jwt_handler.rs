//! Generate JWT Token Handler

use actix_web::{web, HttpResponse};
use chrono::SecondsFormat;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::application::{ErrorResponse, SuccessResponse};
use crate::application::use_cases::GenerateJwtUseCase;
use crate::infrastructure::MqttUserRepositoryImpl;
use crate::utils::map_internal_error;

/// JWT generation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct JwtRequest {
    /// Username for which to generate the JWT token
    pub username: String,
}

/// JWT token response data
#[derive(Debug, Serialize, ToSchema)]
pub struct JwtResponseData {
    /// The JWT token string
    pub token: String,
    /// Token expiration time in RFC3339 format
    pub expires_at: String,
}

use serde::Serialize;

/// Application state for JWT handler
pub struct JwtAppState {
    pub use_case: GenerateJwtUseCase<MqttUserRepositoryImpl>,
}

/// Generate JWT token for EMQX authentication
/// 
/// This endpoint generates a JWT token that can be used as the password
/// when connecting to the EMQX MQTT broker.
/// 
/// The token is compatible with EMQX JWT authentication configuration:
/// - Algorithm: HS256 (HMAC-SHA256)
/// - Issuer: broker.i-ot.net
/// - Audience: mqtt
/// - Expiration: 24 hours
#[utoipa::path(
    post,
    path = "/mqtt/jwt",
    tag = "Users",
    request_body = JwtRequest,
    security(("api_key" = [])),
    responses(
        (status = 200, description = "JWT token generated successfully", body = SuccessResponse<JwtResponseData>,
            example = json!({
                "success": true,
                "message": "JWT token generated successfully",
                "data": {
                    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
                    "expires_at": "2026-03-20T12:00:00Z"
                }
            })),
        (status = 400, description = "Validation error", body = ErrorResponse,
            example = json!({
                "success": false,
                "message": "Validation error: username cannot be empty"
            })),
        (status = 404, description = "User not found", body = ErrorResponse,
            example = json!({
                "success": false,
                "message": "User not found: client_001"
            })),
    ),
)]
pub async fn jwt_handler(
    state: web::Data<JwtAppState>,
    body: web::Json<JwtRequest>,
) -> HttpResponse {
    match state.use_case.execute(&body.username).await {
        Ok(token_pair) => {
            let response_data = JwtResponseData {
                token: token_pair.token,
                expires_at: token_pair.expires_at.to_rfc3339_opts(SecondsFormat::Secs, true),
            };
            HttpResponse::Ok().json(SuccessResponse::new(
                "JWT token generated successfully",
                Some(response_data),
            ))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse::new(msg))
            } else if msg.contains("Validation") {
                HttpResponse::BadRequest().json(ErrorResponse::new(msg))
            } else if msg.contains("hex") {
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Invalid SECRET_KEY configuration".to_string()))
            } else {
                map_internal_error(e, "generate_jwt")
            }
        }
    }
}
