//! EMQX Authorization (ACL) Handler - Returns format compatible with EMQX HTTP authorizer
//! 
//! EMQX expects: { "result": "allow" | "deny" | "ignore" }
//! Ref: https://docs.emqx.com/en/emqx/latest/access-control/authz/http.html

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::application::CheckAclUseCase;
use crate::infrastructure::MqttUserRepositoryImpl;

/// EMQX ACL request - includes action field that EMQX sends
#[derive(Debug, Deserialize, ToSchema)]
pub struct EmqxAclRequest {
    pub username: String,
    pub topic: String,
    #[serde(default = "default_action")]
    pub action: String,  // "publish" | "subscribe"
}

fn default_action() -> String {
    "publish".to_string()
}

/// EMQX ACL response - matches EMQX expected format
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct EmqxAclResponse {
    pub result: String,  // "allow" | "deny" | "ignore"
}

pub struct EmqxAclState {
    pub use_case: CheckAclUseCase<MqttUserRepositoryImpl>,
}

#[utoipa::path(
    post,
    path = "/emqx/acl",
    tag = "EMQX",
    request_body = EmqxAclRequest,
    description = "EMQX HTTP authorization (ACL) endpoint. Returns result in EMQX native format.",
    responses(
        (status = 200, description = "Access allowed", body = EmqxAclResponse,
            example = json!({ "result": "allow" })),
        (status = 200, description = "Access denied (user not found or topic not permitted)", body = EmqxAclResponse,
            example = json!({ "result": "deny" })),
    ),
)]
pub async fn emqx_acl_handler(
    state: web::Data<EmqxAclState>,
    body: web::Json<EmqxAclRequest>,
) -> HttpResponse {
    match state.use_case.execute(&body.username, &body.topic).await {
        Ok(allowed) => {
            let result = if allowed { "allow" } else { "deny" };
            let response = EmqxAclResponse {
                result: result.to_string(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(_) => {
            // User not found or error - deny by default
            let response = EmqxAclResponse {
                result: "deny".to_string(),
            };
            HttpResponse::Ok().json(response)
        }
    }
}
