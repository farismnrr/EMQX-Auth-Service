use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;

use crate::dtos::mqtt_dto::{GetMqttListDTO, GetMqttListPaginatedDTO, PaginationInfo, PaginationQuery};
use crate::dtos::response_dto::ResponseDTO;
use crate::handler::handler_error::AppError;
use crate::services::get_mqtt_list_service::GetMqttListService;
use crate::services::service_error::MqttServiceError;

pub struct AppState {
    pub get_mqtt_list_service: Arc<GetMqttListService>,
}

pub async fn get_mqtt_list_handler(
    data: web::Data<AppState>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    // Check if pagination is requested
    if query.page.is_some() || query.page_size.is_some() {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10).min(100); // Max 100 per page

        if page < 1 {
            let error = MqttServiceError::BadRequest(vec![]);
            return error.to_http_response();
        }
        if page_size < 1 {
            let error = MqttServiceError::BadRequest(vec![]);
            return error.to_http_response();
        }

        match data
            .get_mqtt_list_service
            .get_mqtt_list_paginated(page, page_size)
            .await
        {
            Ok((users, total)) => {
                let total_pages = (total as f64 / page_size as f64).ceil() as i64;
                HttpResponse::Ok().json(ResponseDTO {
                    success: true,
                    message: "User MQTT list retrieved successfully",
                    data: Some(GetMqttListPaginatedDTO {
                        users,
                        pagination: PaginationInfo {
                            total,
                            page,
                            page_size,
                            total_pages,
                        },
                    }),
                    result: None,
                    is_superuser: None,
                })
            }
            Err(e) => e.to_http_response(),
        }
    } else {
        // Legacy behavior - return all users without pagination
        match data.get_mqtt_list_service.get_mqtt_list().await {
            Ok(users) => HttpResponse::Ok().json(ResponseDTO {
                success: true,
                message: "User MQTT list retrieved successfully",
                data: Some(GetMqttListDTO { users }),
                result: None,
                is_superuser: None,
            }),
            Err(e) => e.to_http_response(),
        }
    }
}

pub async fn get_mqtt_by_id_handler(
    data: web::Data<AppState>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();

    match data.get_mqtt_list_service.get_mqtt_by_id(id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(ResponseDTO {
            success: true,
            message: "User MQTT retrieved successfully",
            data: Some(user),
            result: None,
            is_superuser: None,
        }),
        Ok(None) => {
            let error = MqttServiceError::MqttNotFound("User not found".to_string());
            error.to_http_response()
        }
        Err(e) => e.to_http_response(),
    }
}
