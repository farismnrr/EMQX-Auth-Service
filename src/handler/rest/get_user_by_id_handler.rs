use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

use crate::dtos::response_dto::{ErrorResponseDTO, ResponseDTO};
use crate::handler::handler_error::AppError;
use crate::services::get_mqtt_list_service::GetMqttListService;
use crate::services::service_error::MqttServiceError;

pub struct AppState {
    pub get_mqtt_list_service: Arc<GetMqttListService>,
}

#[utoipa::path(
    get,
    path = "/mqtt/{id}",
    tag = "MQTT",
    params(
        ("id" = i32, Path, description = "ID of the MQTT user")
    ),
    responses(
        (status = 200, description = "User MQTT retrieved successfully", body = ResponseDTO),
        (status = 404, description = "User not found", body = ErrorResponseDTO)
    ),
    security(
        ("api_key" = [])
    )
)]
/// Get MQTT User by ID
///
/// Retrieves the details of a specific MQTT user by their ID.
pub async fn get_user_by_id_handler(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();

    match data.get_mqtt_list_service.get_mqtt_by_id(id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(ResponseDTO {
            success: true,
            message: "User MQTT retrieved successfully",
            data: Some(user),
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
