use crate::dtos::mqtt_dto::MqttAclDTO;
use crate::handler::handler_error::AppError;
use crate::services::mqtt_acl_service::MqttAclService;
use actix_web::{HttpResponse, Responder, web};
use log::debug;

pub async fn mqtt_acl_handler(
    service: web::Data<MqttAclService>,
    req_body: web::Json<MqttAclDTO>,
) -> impl Responder {
    debug!(
        "Checking ACL for: {}, topic: {}",
        req_body.username, req_body.topic
    );

    match service.check_acl_permission(req_body.into_inner()).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({ "result": "allow" })),
        Ok(false) => HttpResponse::Ok().json(serde_json::json!({ "result": "deny" })),
        Err(e) => AppError::to_http_response(&e),
    }
}
