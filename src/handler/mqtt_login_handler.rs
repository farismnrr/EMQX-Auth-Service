use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

use crate::services::mqtt_login_service::MqttLoginService;
use crate::dtos::mqtt_dto::{CheckMqttActiveDTO, MqttJwtDTO};
use crate::dtos::response_dto::ResponseDTO;
use crate::handler::handler_error::handle_mqtt_login;

pub struct AppState {
    pub login_with_credentials_service: Arc<MqttLoginService>,
}

pub async fn login_with_credentials_handler(
    data: web::Data<AppState>,
    body: web::Json<CheckMqttActiveDTO>,
) -> impl Responder {
    match data.login_with_credentials_service.login_with_credentials(body.into_inner()) {
        Ok((_, token)) => {
            if token.is_empty() {
                // Credentials login - return None data
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
        },
        Err(e) => handle_mqtt_login(&e),
    }
}
