use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;

use crate::dtos::mqtt_dto::MqttAclDTO;
use crate::dtos::response_dto::{ErrorResponseValidation, ResponseDTO};
use crate::handler::handler_error::AppError;
use crate::services::mqtt_acl_service::MqttAclService;
use crate::services::service_error::MqttServiceError;

pub struct AppState {
    pub mqtt_acl_service: Arc<MqttAclService>,
}

#[utoipa::path(
    post,
    path = "/mqtt/acl",
    tag = "MQTT",
    request_body = MqttAclDTO,
    responses(
        (status = 200, description = "ACL permission checked"),
        (status = 400, description = "Validation Error", body = ErrorResponseValidation)
    ),
    security(
        ("api_key" = [])
    )
)]
/// Check MQTT ACL Authorization
///
/// Verifies whether the specified MQTT user has permissions to access the given topic.
pub async fn mqtt_acl_handler(
    data: web::Data<AppState>,
    body: web::Json<MqttAclDTO>,
) -> impl Responder {
    match data
        .mqtt_acl_service
        .check_acl_permission(body.into_inner())
        .await
    {
        Ok(true) => HttpResponse::Ok().json(ResponseDTO::<()> {
            success: true,
            message: "User has access",
            data: None,
            result: Some("allow"),
        }),
        Ok(false) => HttpResponse::Ok().json(ResponseDTO::<()> {
            success: true,
            message: "User does not have access",
            data: None,
            result: Some("deny"),
        }),
        Err(e) => match &e {
            MqttServiceError::BadRequest(validation_errors) => {
                e.to_http_response_with_result(Some("deny"), Some(validation_errors))
            }
            _ => e.to_http_response_with_result(Some("deny"), None::<String>),
        },
    }
}
