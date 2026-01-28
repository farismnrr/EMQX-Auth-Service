use crate::handler::handler_error::AppError;
use crate::services::soft_delete_mqtt_service::SoftDeleteMqttService;
use actix_web::{HttpResponse, Responder, web};
use log::info;

pub async fn soft_delete_mqtt(
    service: web::Data<SoftDeleteMqttService>,
    path: web::Path<String>,
) -> impl Responder {
    let username = path.into_inner();
    info!("Soft deleting MQTT user: {}", username);

    match service.soft_delete_mqtt(&username).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "message": "User soft deleted" })),
        Err(e) => AppError::to_http_response(&e),
    }
}
