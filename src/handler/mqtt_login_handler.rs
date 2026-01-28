use crate::dtos::mqtt_dto::MqttLoginDTO;
use crate::handler::handler_error::AppError;
use crate::services::mqtt_login_service::MqttLoginService;
use actix_web::{HttpResponse, Responder, web};
use log::debug;

pub async fn login_with_credentials_handler(
    service: web::Data<MqttLoginService>,
    req_body: web::Json<MqttLoginDTO>,
) -> impl Responder {
    debug!("Checking credentials for: {}", req_body.username);

    match service.login_with_credentials(req_body.into_inner()).await {
        Ok((true, token)) => {
            if token.is_empty() {
                HttpResponse::Ok().json(serde_json::json!({ "result": "allow" }))
            } else {
                HttpResponse::Ok().json(serde_json::json!({ "result": "allow", "token": token }))
            }
        }
        Ok((false, _)) => HttpResponse::Ok().json(serde_json::json!({ "result": "deny" })),
        Err(e) => AppError::to_http_response(&e),
    }
}
