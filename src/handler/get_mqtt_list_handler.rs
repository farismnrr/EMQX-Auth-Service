use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

use crate::services::get_mqtt_list_service::GetMqttListService;
use crate::dtos::response_dto::ResponseDTO;
use crate::dtos::mqtt_dto::GetMqttListDTO;
use crate::handler::handler_error::AppError;

pub struct AppState {
    pub get_mqtt_list_service: Arc<GetMqttListService>,
}

pub async fn get_mqtt_list_handler(
    data: web::Data<AppState>,
) -> impl Responder {
    match data.get_mqtt_list_service.get_mqtt_list() {
        Ok(users) => HttpResponse::Ok().json(ResponseDTO {
            success: true,
            message: "User MQTT list retrieved successfully",
            data: Some(GetMqttListDTO { users }),
            result: None,
        }),
        Err(e) => e.to_http_response(),
    }
}

