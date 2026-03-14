//! List Users Handler

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::application::{ErrorResponse, ListUsersUseCase, SuccessResponse, SuccessResponseJson, UserListDTO};
use crate::infrastructure::MqttUserRepositoryImpl;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListUsersQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    100
}

impl Default for ListUsersQuery {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
        }
    }
}

pub struct ListUsersAppState {
    pub use_case: ListUsersUseCase<MqttUserRepositoryImpl>,
}

#[utoipa::path(
    get,
    path = "/mqtt",
    tag = "MQTT",
    params(ListUsersQuery),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "User list retrieved successfully", body = SuccessResponseJson,
            example = json!({
                "success": true,
                "message": "User list retrieved successfully",
                "data": {
                    "users": [
                        {
                            "id": 1,
                            "username": "device_001",
                            "is_superuser": false
                        }
                    ],
                    "total": 1,
                    "limit": 100,
                    "offset": 0
                }
            })),
    ),
)]
pub async fn list_users_handler(
    state: web::Data<ListUsersAppState>,
    query: Option<web::Query<ListUsersQuery>>,
) -> HttpResponse {
    let q = query.map(|q| q.into_inner()).unwrap_or_default();
    let limit = q.limit;
    let offset = q.offset;

    match state.use_case.execute(limit, offset).await {
        Ok(list) => {
            let user_dtos: Vec<_> = list.users.into_iter().map(|u| u.into()).collect();
            let response_data = UserListDTO {
                users: user_dtos,
                total: list.total,
                limit: list.limit,
                offset: list.offset,
            };
            HttpResponse::Ok().json(SuccessResponse::new(
                "User list retrieved successfully",
                Some(response_data),
            ))
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse::new(e.to_string())),
    }
}
