use crate::dtos::mqtt_dto::DeleteMqttDTO;
use crate::dtos::response_dto::{ErrorResponseValidation, ResponseDTO};
use crate::handler::handler_error::AppError;
use crate::services::delete_mqtt_service::DeleteMqttService;
use crate::services::service_error::MqttServiceError;
use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;

pub struct AppState {
    pub delete_mqtt_service: Arc<DeleteMqttService>,
}

#[utoipa::path(
    delete,
    path = "/mqtt/{username}",
    tag = "MQTT",
    params(
        ("username" = String, Path, description = "Username of the client to delete")
    ),
    responses(
        (status = 200, description = "User mqtt deleted successfully"),
        (status = 400, description = "Validation Error", body = ErrorResponseValidation)
    ),
    security(
        ("api_key" = [])
    )
)]
/// Delete MQTT User
///
/// Hard-deletes an existing MQTT user by their username.
pub async fn delete_mqtt(
    data: web::Data<AppState>,
    params: web::Path<DeleteMqttDTO>,
) -> impl Responder {
    let username = &params.username;
    match data
        .delete_mqtt_service
        .delete_mqtt(username)
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
