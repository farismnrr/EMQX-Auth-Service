use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;

use crate::dtos::response_dto::ResponseDTO;
use crate::handler::handler_error::AppError;
use crate::services::get_mqtt_credentials_service::GetMqttCredentialsService;

pub struct AppState {
    pub get_mqtt_credentials_service: Arc<GetMqttCredentialsService>,
}

pub async fn get_mqtt_credentials_handler(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let username = path.into_inner();
    match data
        .get_mqtt_credentials_service
        .get_credentials(&username)
        .await
    {
        Ok(creds) => HttpResponse::Ok().json(ResponseDTO {
            success: true,
            message: "Credentials retrieved successfully",
            data: Some(creds),
            result: None,
        }),
        Err(e) => e.to_http_response_with_details(None::<String>),
    }
}
