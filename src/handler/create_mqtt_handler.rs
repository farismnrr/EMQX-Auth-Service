use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;

use crate::dtos::mqtt_dto::CreateMqttDTO;
use crate::dtos::response_dto::{ErrorResponseValidation, ResponseDTO};
use crate::handler::handler_error::AppError;
use crate::services::create_mqtt_service::CreateMqttService;
use crate::services::service_error::MqttServiceError;

pub struct AppState {
    pub create_mqtt_service: Arc<CreateMqttService>,
}

#[utoipa::path(
    post,
    path = "/mqtt/create",
    tag = "MQTT",
    request_body = CreateMqttDTO,
    responses(
        (status = 200, description = "User mqtt created successfully"),
        (status = 400, description = "Validation Error", body = ErrorResponseValidation)
    ),
    security(
        ("api_key" = [])
    )
)]
/// Create MQTT User
///
/// Creates a new MQTT user or superuser with a username and password.
pub async fn create_mqtt_handler(
    data: web::Data<AppState>,
    body: web::Json<CreateMqttDTO>,
) -> impl Responder {
    match data
        .create_mqtt_service
        .create_mqtt(body.into_inner())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(ResponseDTO::<()> {
            success: true,
            message: "User mqtt created successfully",
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
