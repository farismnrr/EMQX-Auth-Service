use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

use crate::dtos::mqtt_admin_dto::AdminUserResponse;
use crate::dtos::response_dto::{ErrorResponseDTO, ResponseDTO};
use crate::handler::handler_error::AppError;
use crate::services::get_mqtt_credentials_service::GetMqttCredentialsService;
use crate::services::service_error::MqttServiceError;

pub struct AppState {
    pub get_mqtt_credentials_service: Arc<GetMqttCredentialsService>,
}

#[utoipa::path(
    get,
    path = "/mqtt/users/{username}",
    tag = "MQTT",
    params(
        ("username" = String, Path, description = "Username of the MQTT user")
    ),
    responses(
        (status = 200, description = "User MQTT retrieved successfully", body = ResponseDTO<AdminUserResponse>),
        (status = 404, description = "User not found", body = ErrorResponseDTO)
    ),
    security(
        ("api_key" = [])
    )
)]
/// Get MQTT User by Username
///
/// Retrieves the details of a specific MQTT user by their username.
pub async fn get_user_by_username_handler(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let username = path.into_inner();

    match data
        .get_mqtt_credentials_service
        .get_mqtt_by_username(&username)
        .await
    {
        Ok(Some(user)) => HttpResponse::Ok().json(ResponseDTO {
            success: true,
            message: "User MQTT retrieved successfully",
            data: Some(AdminUserResponse {
                id: user.id,
                username: user.username,
                is_superuser: user.is_superuser,
            }),
            result: None,
            is_superuser: None,
        }),
        Ok(None) => {
            let error = MqttServiceError::MqttNotFound("User not found".to_string());
            error.to_http_response()
        }
        Err(e) => e.to_http_response(),
    }
}
