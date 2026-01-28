use crate::dtos::mqtt_dto::CreateMqttDTO;
use crate::handler::handler_error::AppError;
use crate::services::create_mqtt_service::CreateMqttService;
use actix_web::{HttpResponse, Responder, web};

pub async fn create_mqtt_handler(
    service: web::Data<CreateMqttService>,
    req_body: web::Json<CreateMqttDTO>,
) -> impl Responder {
    match service.create_mqtt(req_body.into_inner()).await {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({ "message": "User created" })),
        Err(e) => AppError::to_http_response(&e),
    }
}
