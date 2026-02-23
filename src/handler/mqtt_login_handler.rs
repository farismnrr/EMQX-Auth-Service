use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;

use crate::dtos::mqtt_dto::{MqttJwtDTO, MqttLoginDTO};
use crate::dtos::response_dto::ResponseDTO;
use crate::handler::handler_error::AppError;
use crate::services::mqtt_login_service::MqttLoginService;
use crate::services::service_error::MqttServiceError;

pub struct AppState {
    pub mqtt_login_service: Arc<MqttLoginService>,
}

pub async fn login_with_credentials_handler(
    data: web::Data<AppState>,
    body: web::Json<MqttLoginDTO>,
) -> impl Responder {
    match data
        .mqtt_login_service
        .login_with_credentials(body.into_inner())
        .await
    {
        Ok((_, token)) => {
            if token.is_empty() {
                HttpResponse::Ok().json(ResponseDTO::<()> {
                    success: true,
                    message: "User MQTT is active",
                    data: None,
                    result: Some("allow"),
                })
            } else {
                HttpResponse::Ok().json(ResponseDTO::<MqttJwtDTO> {
                    success: true,
                    message: "User MQTT is active",
                    data: Some(MqttJwtDTO { token }),
                    result: Some("allow"),
                })
            }
        }
        Err(e) => match &e {
            MqttServiceError::BadRequest(validation_errors) => {
                e.to_http_response_with_result(Some("deny"), Some(validation_errors))
            }
            _ => e.to_http_response_with_result(Some("deny"), None::<String>),
        },
    }
}
