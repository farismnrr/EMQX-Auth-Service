//! Create User Handler

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::application::{CreateUserUseCase, ErrorResponse, SuccessResponse, SuccessResponseJson};
use crate::infrastructure::{EncryptionAdapter, MqttUserRepositoryImpl};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub is_superuser: bool,
}

pub struct CreateUserAppState {
    pub use_case: CreateUserUseCase<MqttUserRepositoryImpl, EncryptionAdapter>,
}

#[utoipa::path(
    post,
    path = "/mqtt/create",
    tag = "MQTT",
    request_body = CreateUserRequest,
    security(("api_key" = [])),
    responses(
        (status = 200, description = "User created successfully", body = SuccessResponseJson,
            example = json!({
                "success": true,
                "message": "User created successfully"
            })),
        (status = 400, description = "Validation error", body = ErrorResponse,
            example = json!({
                "success": false,
                "message": "Validation error: username is required"
            })),
        (status = 409, description = "User already exists", body = ErrorResponse,
            example = json!({
                "success": false,
                "message": "User already exists"
            })),
    ),
)]
pub async fn create_user_handler(
    state: web::Data<CreateUserAppState>,
    body: web::Json<CreateUserRequest>,
) -> HttpResponse {
    match state
        .use_case
        .execute(&body.username, &body.password, body.is_superuser)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(SuccessResponse::<()>::ok("User created successfully")),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") {
                HttpResponse::Conflict().json(ErrorResponse::new(msg))
            } else if msg.contains("Validation") {
                HttpResponse::BadRequest().json(ErrorResponse::new(msg))
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse::new(msg))
            }
        }
    }
}
