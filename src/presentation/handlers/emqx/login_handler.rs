//! EMQX Login Handler - Returns result and JWT token
//! 
//! Useful for clients to authenticate and obtain a token for MQTT connection.

use actix_web::{web, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;

use crate::presentation::handlers::emqx::auth_handler::{EmqxAuthRequest, EmqxAuthState};

/// EMQX login response
#[derive(Debug, Serialize, ToSchema)]
pub struct EmqxLoginResponse {
    pub result: String,  // "allow" | "deny"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_superuser: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/emqx/login",
    tag = "EMQX",
    request_body = EmqxAuthRequest,
    description = "EMQX HTTP login endpoint. Returns result and JWT token on success.",
    responses(
        (status = 200, description = "Login successful", body = EmqxLoginResponse,
            example = json!({ "result": "allow", "token": "...", "is_superuser": false })),
        (status = 401, description = "Login denied", body = EmqxLoginResponse,
            example = json!({ "result": "deny" })),
    ),
    security(("api_key" = []))
)]
pub async fn emqx_login_handler(
    state: web::Data<EmqxAuthState>,
    body: web::Json<EmqxAuthRequest>,
) -> HttpResponse {
    // Record auth request metrics
    state.metrics.auth().record_auth_request();

    // We use issue_token = true to get the JWT
    match state.use_case.execute(&body.username, &body.password, true).await {
        Ok((is_authenticated, token)) => {
            if is_authenticated {
                // Record success
                state.metrics.auth().record_auth_success();
                
                // Get superuser status separately or we can optimize the use case later
                // For now, we know it succeeded and returned a token
                let response = EmqxLoginResponse {
                    result: "allow".to_string(),
                    token,
                    is_superuser: None, // UseCase currently returns (bool, Option<String>)
                };
                HttpResponse::Ok().json(response)
            } else {
                // Record failure
                state.metrics.auth().record_auth_failure();
                
                let response = EmqxLoginResponse {
                    result: "deny".to_string(),
                    token: None,
                    is_superuser: None,
                };
                HttpResponse::Unauthorized().json(response)
            }
        }
        Err(_) => {
            // Record failure
            state.metrics.auth().record_auth_failure();
            
            let response = EmqxLoginResponse {
                result: "deny".to_string(),
                token: None,
                is_superuser: None,
            };
            HttpResponse::Unauthorized().json(response)
        }
    }
}
