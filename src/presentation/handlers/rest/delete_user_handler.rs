//! Delete User Handler

use actix_web::{web, HttpResponse};

use crate::application::{DeleteUserUseCase, ErrorResponse, SuccessResponse, SuccessResponseJson};
use crate::infrastructure::MqttUserRepositoryImpl;
use crate::utils::map_internal_error;

pub struct DeleteUserAppState {
    pub use_case: DeleteUserUseCase<MqttUserRepositoryImpl>,
}

#[utoipa::path(
    delete,
    path = "/mqtt/{username}",
    tag = "MQTT",
    params(
        ("username" = String, Path, description = "Username of the user to delete")
    ),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "User deleted successfully", body = SuccessResponseJson,
            example = json!({
                "success": true,
                "message": "User deleted successfully"
            })),
        (status = 404, description = "User not found", body = ErrorResponse,
            example = json!({
                "success": false,
                "message": "User not found"
            })),
    ),
)]
pub async fn delete_user_handler(
    state: web::Data<DeleteUserAppState>,
    path: web::Path<String>,
) -> HttpResponse {
    match state.use_case.execute(&path).await {
        Ok(_) => HttpResponse::Ok().json(SuccessResponse::<()>::ok("User deleted successfully")),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse::new(msg))
            } else {
                map_internal_error(e, "delete_user")
            }
        }
    }
}
