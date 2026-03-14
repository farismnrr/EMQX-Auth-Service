//! Check Login Handler

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::application::{AuthenticateUserUseCase, ErrorResponse, SuccessResponse, SuccessResponseJson};
use crate::infrastructure::{JwtAdapter, MqttUserRepositoryImpl};

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(default = "default_method")]
    pub method: String,
}

fn default_method() -> String {
    "credentials".to_string()
}

pub struct LoginAppState {
    pub use_case: AuthenticateUserUseCase<MqttUserRepositoryImpl, JwtAdapter>,
}

#[utoipa::path(
    post,
    path = "/mqtt/check",
    tag = "MQTT",
    request_body = LoginRequest,
    security(("api_key" = [])),
    responses(
        (status = 200, description = "Authentication successful", body = SuccessResponseJson,
            example = json!({
                "success": true,
                "message": "User authenticated successfully",
                "data": {
                    "result": "allow",
                    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
                }
            })),
        (status = 401, description = "Invalid credentials", body = ErrorResponse,
            example = json!({
                "success": false,
                "message": "Invalid credentials"
            })),
        (status = 404, description = "User not found", body = ErrorResponse,
            example = json!({
                "success": false,
                "message": "User not found"
            })),
    ),
)]
pub async fn check_login_handler(
    state: web::Data<LoginAppState>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let issue_token = body.method.to_lowercase() == "jwt";

    match state
        .use_case
        .execute(&body.username, &body.password, issue_token)
        .await
    {
        Ok((_, token)) => {
            let mut data = serde_json::json!({ "result": "allow" });
            if let Some(t) = token {
                data["token"] = serde_json::json!(t);
            }
            HttpResponse::Ok().json(SuccessResponse::new(
                "User authenticated successfully",
                Some(data),
            ))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse::new(msg))
            } else if msg.contains("Invalid") {
                HttpResponse::Unauthorized().json(ErrorResponse::new(msg))
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse::new(msg))
            }
        }
    }
}
