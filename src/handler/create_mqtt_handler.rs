use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

use crate::services::create_mqtt_service::CreateMqttService;
use crate::dtos::mqtt_dto::CreateMqttDTO;
use crate::dtos::response_dto::ResponseDTO;
use crate::handler::handler_error::AppError;

pub struct AppState {
    pub create_mqtt_service: Arc<CreateMqttService>,
}

pub async fn create_mqtt_handler(
    data: web::Data<AppState>,
) -> impl Responder {
    match data.create_mqtt_service.create_mqtt() {
        Ok((username, password)) => {
            let dto = CreateMqttDTO { username, password };
            HttpResponse::Ok().json(ResponseDTO {
                success: true,
                message: "User MQTT created successfully",
                data: Some(dto),
                result: None,
            })
        },
        Err(e) => e.to_http_response(),
    }
}