//! Check ACL Handler

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::application::{CheckAclUseCase, ErrorResponse, SuccessResponse, SuccessResponseJson};
use crate::infrastructure::MqttUserRepositoryImpl;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AclRequest {
    pub username: String,
    pub topic: String,
}

pub struct AclAppState {
    pub use_case: CheckAclUseCase<MqttUserRepositoryImpl>,
}

#[utoipa::path(
    post,
    path = "/mqtt/acl",
    tag = "MQTT",
    request_body = AclRequest,
    security(("api_key" = [])),
    responses(
        (status = 200, description = "ACL check successful", body = SuccessResponseJson,
            example = json!({
                "success": true,
                "message": "ACL check completed",
                "data": {
                    "result": "allow"
                }
            })),
        (status = 404, description = "User not found", body = ErrorResponse,
            example = json!({
                "success": false,
                "message": "User not found"
            })),
    ),
)]
pub async fn check_acl_handler(
    state: web::Data<AclAppState>,
    body: web::Json<AclRequest>,
) -> HttpResponse {
    match state.use_case.execute(&body.username, &body.topic).await {
        Ok(allowed) => {
            let result = if allowed { "allow" } else { "deny" };
            let data = serde_json::json!({ "result": result });
            HttpResponse::Ok().json(SuccessResponse::new("ACL check completed", Some(data)))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse::new(msg))
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse::new(msg))
            }
        }
    }
}
