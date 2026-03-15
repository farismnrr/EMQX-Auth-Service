//! EMQX Authentication Handler - Returns format compatible with EMQX HTTP authenticator
//! 
//! EMQX expects: { "result": "allow" | "deny" | "ignore", "is_superuser": false }
//! Ref: https://docs.emqx.com/en/emqx/latest/access-control/authn/http.html

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::application::AuthenticateUserUseCase;
use crate::infrastructure::{JwtAdapter, MqttUserRepositoryImpl, AppMetrics, EncryptionAdapter};

/// EMQX authentication request
#[derive(Debug, Deserialize, ToSchema)]
pub struct EmqxAuthRequest {
    pub username: String,
    pub password: String,
}

/// EMQX authentication response - matches EMQX expected format
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct EmqxAuthResponse {
    pub result: String,  // "allow" | "deny" | "ignore"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_superuser: Option<bool>,
}

pub struct EmqxAuthState {
    pub use_case: AuthenticateUserUseCase<MqttUserRepositoryImpl, JwtAdapter, EncryptionAdapter>,
    pub metrics: AppMetrics,
}

#[utoipa::path(
    post,
    path = "/emqx/auth",
    tag = "EMQX",
    request_body = EmqxAuthRequest,
    description = "EMQX HTTP authentication endpoint. Returns result in EMQX native format.",
    responses(
        (status = 200, description = "Authentication successful", body = EmqxAuthResponse,
            example = json!({ "result": "allow", "is_superuser": false })),
        (status = 200, description = "Authentication denied (user not found or invalid password)", body = EmqxAuthResponse,
            example = json!({ "result": "deny" })),
    ),
)]
pub async fn emqx_auth_handler(
    state: web::Data<EmqxAuthState>,
    body: web::Json<EmqxAuthRequest>,
) -> HttpResponse {
    // Record auth request
    state.metrics.auth().record_auth_request();

    match state.use_case.execute_for_emqx(&body.username, &body.password).await {
        Ok((is_authenticated, is_superuser)) => {
            if is_authenticated {
                // Record success
                state.metrics.auth().record_auth_success();
                
                let response = EmqxAuthResponse {
                    result: "allow".to_string(),
                    is_superuser: Some(is_superuser),
                };
                HttpResponse::Ok().json(response)
            } else {
                // Record failure
                state.metrics.auth().record_auth_failure();
                
                let response = EmqxAuthResponse {
                    result: "deny".to_string(),
                    is_superuser: None,
                };
                HttpResponse::Ok().json(response)
            }
        }
        Err(_) => {
            // Record failure
            state.metrics.auth().record_auth_failure();
            
            // Authentication failed (user not found or invalid password)
            let response = EmqxAuthResponse {
                result: "deny".to_string(),
                is_superuser: None,
            };
            HttpResponse::Ok().json(response)
        }
    }
}
