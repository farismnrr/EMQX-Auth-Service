use crate::dtos::mqtt_dto::DeleteMqttDTO;
use crate::dtos::response_dto::ResponseDTO;
use crate::handler::handler_error::AppError;
use crate::services::service_error::MqttServiceError;
use crate::services::soft_delete_mqtt_service::SoftDeleteMqttService;
use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;

pub struct AppState {
    pub soft_delete_mqtt_service: Arc<SoftDeleteMqttService>,
}

pub async fn soft_delete_mqtt(
    data: web::Data<AppState>,
    params: web::Path<DeleteMqttDTO>,
) -> impl Responder {
    let username = &params.username;
    match data
        .soft_delete_mqtt_service
        .soft_delete_mqtt(username)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(ResponseDTO::<()> {
            success: true,
            message: "User mqtt deleted successfully",
            data: None,
            result: None,
        }),
        Err(e) => match &e {
            MqttServiceError::BadRequest(validation_errors) => {
                e.to_http_response_with_details(Some(validation_errors))
            }
            _ => e.to_http_response_with_details(None::<String>),
        },
    }
}
