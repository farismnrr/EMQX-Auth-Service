use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;

use crate::dtos::mqtt_dto::GetMqttListDTO;
use crate::dtos::response_dto::ResponseDTO;
use crate::handler::handler_error::AppError;
use crate::services::get_mqtt_list_service::GetMqttListService;

pub struct AppState {
    pub get_mqtt_list_service: Arc<GetMqttListService>,
}

#[utoipa::path(
    get,
    path = "/mqtt",
    tag = "MQTT",
    responses(
        (status = 200, description = "User MQTT list retrieved successfully")
    ),
    security(
        ("api_key" = [])
    )
)]
/// Get MQTT List
///
/// Retrieves a list of all registered MQTT users.
pub async fn get_mqtt_list_handler(data: web::Data<AppState>) -> impl Responder {
    match data.get_mqtt_list_service.get_mqtt_list().await {
        Ok(users) => HttpResponse::Ok().json(ResponseDTO {
            success: true,
            message: "User MQTT list retrieved successfully",
            data: Some(GetMqttListDTO { users }),
            result: None,
        }),
        Err(e) => e.to_http_response(),
    }
}
