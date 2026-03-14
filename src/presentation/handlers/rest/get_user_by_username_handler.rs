//! Get User By Username Handler

use actix_web::{web, HttpResponse};

use crate::application::{ErrorResponse, GetUserUseCase, SuccessResponse, SuccessResponseJson, UserDTO};
use crate::infrastructure::MqttUserRepositoryImpl;

pub struct GetUserByUsernameAppState {
    pub use_case: GetUserUseCase<MqttUserRepositoryImpl>,
}

#[utoipa::path(
    get,
    path = "/mqtt/users/{username}",
    tag = "MQTT",
    params(
        ("username" = String, Path, description = "Username to look up")
    ),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "User retrieved successfully", body = SuccessResponseJson, 
            example = json!({
                "success": true,
                "message": "User retrieved successfully",
                "data": {
                    "id": 1,
                    "username": "device_001",
                    "is_superuser": false
                }
            })),
        (status = 404, description = "User not found", body = ErrorResponse,
            example = json!({
                "success": false,
                "message": "User not found"
            })),
    ),
)]
pub async fn get_user_by_username_handler(
    state: web::Data<GetUserByUsernameAppState>,
    path: web::Path<String>,
) -> HttpResponse {
    match state.use_case.execute_by_username(&path).await {
        Ok(user) => HttpResponse::Ok().json(SuccessResponse::new(
            "User retrieved successfully",
            Some(UserDTO::from(user)),
        )),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse::new(msg))
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse::new(msg))
            }
        }
    }
}
