use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

use crate::services::get_user_list_service::GetUserListService;
use crate::dtos::response_dto::ResponseDTO;
use crate::dtos::user_dto::GetUserListDTO;
use crate::utils::app_error::AppError;

pub struct AppState {
    pub get_user_list_service: Arc<GetUserListService>,
}

pub async fn get_user_list_handler(
    data: web::Data<AppState>,
) -> impl Responder {
    match data.get_user_list_service.get_user_list() {
        Ok(users) => HttpResponse::Ok().json(ResponseDTO {
            success: true,
            message: "User list retrieved successfully",
            data: Some(GetUserListDTO { users }),
        }),
        Err(e) => e.default_http_response(),
    }
}
