use actix_web::{web, HttpResponse, Responder};
use log::{debug, error, info};
use serde::Deserialize;
use std::sync::Arc;

use crate::dtos::mqtt_admin_dto::RpcDeleteUserData;
use crate::dtos::response_dto::{ErrorResponseDTO, ResponseDTO};
use crate::services::mqtt_admin_service::MqttAdminService;

#[derive(Deserialize)]
pub struct DeletePath {
    pub username: String,
}

#[utoipa::path(
    delete,
    path = "/mqtt/{username}",
    responses(
        (status = 200, description = "User deleted successfully", body = ResponseDTO),
        (status = 401, description = "Unauthorized", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
    params(
        ("username" = String, Path, description = "Username to delete")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn delete_user_handler(
    path: web::Path<DeletePath>,
    admin_service: web::Data<Arc<MqttAdminService>>,
) -> impl Responder {
    let username = path.username.clone();
    debug!("🗑️ REST Request: Delete user {}", username);

    let request_id = uuid::Uuid::new_v4().to_string();
    let data = RpcDeleteUserData {
        username: username.clone(),
    };

    match admin_service.rpc_delete_user(request_id, data).await {
        response if response.success => {
            info!("✅ REST: User {} deleted successfully", username);
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": response.message
            }))
        }
        response => {
            error!(
                "❌ REST: Failed to delete user {}: {}",
                username, response.message
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": response.message,
                "code": response.code
            }))
        }
    }
}
