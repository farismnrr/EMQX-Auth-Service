//! Token and authentication MQTT RPC handlers

use serde::{Deserialize, Serialize};
use crate::application::dtos::mqtt_rpc::MqttRpcResponse;
use crate::application::use_cases::{
    IssueTokenUseCase, AuthenticateUserUseCase,
    IssueTokenError, AuthenticateUserError
};
use crate::infrastructure::{JwtAdapter, MqttUserRepositoryImpl, EncryptionAdapter};

#[derive(Debug, Deserialize)]
pub struct IssueTokenPayload {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct IssueTokenResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyPasswordPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyPasswordResponse {
    pub valid: bool,
}

pub struct TokenHandlers {
    pub issue_token_use_case: IssueTokenUseCase<MqttUserRepositoryImpl, JwtAdapter>,
    pub auth_use_case: AuthenticateUserUseCase<MqttUserRepositoryImpl, JwtAdapter, EncryptionAdapter>,
}

impl TokenHandlers {
    pub async fn handle_issue(&self, request_id: String, payload: IssueTokenPayload) -> MqttRpcResponse<IssueTokenResponse> {
        match self.issue_token_use_case.execute(&payload.username).await {
            Ok(token) => MqttRpcResponse::success(request_id, "Token issued successfully", Some(IssueTokenResponse { token })),
            Err(e) => {
                let (msg, code) = match &e {
                    IssueTokenError::UserNotFound(m) => (m.clone(), "USER_NOT_FOUND"),
                    _ => (e.to_string(), "JWT_ISSUE_FAILED"),
                };
                MqttRpcResponse::error(request_id, msg, code)
            }
        }
    }

    pub async fn handle_verify(&self, request_id: String, payload: VerifyPasswordPayload) -> MqttRpcResponse<VerifyPasswordResponse> {
        match self.auth_use_case.execute_for_emqx(&payload.username, &payload.password).await {
            Ok((valid, _)) => MqttRpcResponse::success(request_id, "Verification complete", Some(VerifyPasswordResponse { valid })),
            Err(e) => {
                let (msg, code) = match &e {
                    AuthenticateUserError::UserNotFound(m) => (m.clone(), "USER_NOT_FOUND"),
                    AuthenticateUserError::InvalidCredentials => ("Invalid credentials".to_string(), "INVALID_CREDENTIALS"),
                    AuthenticateUserError::Encryption(m) => (m.clone(), "ENCRYPTION_ERROR"),
                    _ => (e.to_string(), "INTERNAL_ERROR"),
                };
                MqttRpcResponse::error(request_id, msg, code)
            }
        }
    }
}
