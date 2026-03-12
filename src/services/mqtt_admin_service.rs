use crate::dtos::mqtt_admin_dto::{
    AdminCreateUserRequest, AdminDeleteUserRequest, AdminGetUserByUsernameRequest, AdminListUsersRequest,
    AdminResponse, AdminResponseData, AdminUserResponse, AdminListUsersData,
    AdminPaginationInfo,
};
use crate::dtos::mqtt_dto::CreateMqttDTO;
use crate::repositories::create_mqtt_repository::CreateMqttRepository;
use crate::repositories::get_mqtt_list_repository::GetMqttListRepository;
use log::{debug, error, info};
use std::sync::Arc;

pub struct MqttAdminService {
    create_repo: Arc<CreateMqttRepository>,
    list_repo: Arc<GetMqttListRepository>,
}

impl MqttAdminService {
    pub fn new(
        create_repo: Arc<CreateMqttRepository>,
        list_repo: Arc<GetMqttListRepository>,
    ) -> Self {
        Self {
            create_repo,
            list_repo,
        }
    }

    pub async fn create_user(
        &self,
        request: AdminCreateUserRequest,
    ) -> AdminResponse {
        debug!("[MQTT Admin Service] Creating user: {}", request.username);

        // Check if username already exists
        match self.create_repo.get_by_username(&request.username).await {
            Ok(Some(_)) => {
                return AdminResponse::error(
                    request.request_id,
                    "Username already exists".to_string(),
                    Some(vec!["Username is already taken".to_string()]),
                );
            }
            Ok(None) => {}
            Err(e) => {
                error!("[MQTT Admin Service] Error checking username: {}", e);
                return AdminResponse::error(
                    request.request_id,
                    "Internal error".to_string(),
                    None,
                );
            }
        }

        // Create the user
        let dto = CreateMqttDTO {
            username: request.username.clone(),
            password: request.password,
            is_superuser: request.is_superuser,
        };

        match self.create_repo.create(dto).await {
            Ok(_) => {
                info!("[MQTT Admin Service] User created successfully: {}", request.username);
                
                // Get the created user to return ID
                match self.create_repo.get_by_username(&request.username).await {
                    Ok(Some(user)) => {
                        let user_response = AdminUserResponse {
                            id: user.id,
                            username: user.username,
                            is_superuser: user.is_superuser,
                        };
                        AdminResponse::success(
                            request.request_id,
                            "User created successfully".to_string(),
                            Some(AdminResponseData::CreateUser(user_response)),
                        )
                    }
                    _ => AdminResponse::success(
                        request.request_id,
                        "User created successfully".to_string(),
                        None,
                    ),
                }
            }
            Err(e) => {
                error!("[MQTT Admin Service] Error creating user: {}", e);
                AdminResponse::error(
                    request.request_id,
                    "Failed to create user".to_string(),
                    Some(vec![e.to_string()]),
                )
            }
        }
    }

    pub async fn delete_user(
        &self,
        request: AdminDeleteUserRequest,
    ) -> AdminResponse {
        debug!("[MQTT Admin Service] Deleting user: {}", request.username);

        match self.list_repo.delete_mqtt_by_username(&request.username).await {
            Ok(_) => {
                info!("[MQTT Admin Service] User deleted successfully: {}", request.username);
                AdminResponse::success(
                    request.request_id,
                    "User deleted successfully".to_string(),
                    None,
                )
            }
            Err(e) => {
                error!("[MQTT Admin Service] Error deleting user: {}", e);
                AdminResponse::error(
                    request.request_id,
                    "Failed to delete user".to_string(),
                    Some(vec![e.to_string()]),
                )
            }
        }
    }

    pub async fn list_users(
        &self,
        request: AdminListUsersRequest,
    ) -> AdminResponse {
        debug!("[MQTT Admin Service] Listing users");

        let page = request.page.unwrap_or(1);
        let page_size = request.page_size.unwrap_or(10).min(100);

        if page < 1 {
            return AdminResponse::error(
                request.request_id,
                "Invalid page number".to_string(),
                Some(vec!["Page must be >= 1".to_string()]),
            );
        }

        if page_size < 1 {
            return AdminResponse::error(
                request.request_id,
                "Invalid page size".to_string(),
                Some(vec!["Page size must be >= 1".to_string()]),
            );
        }

        match self.list_repo.get_mqtt_list_paginated(page, page_size).await {
            Ok((users, total)) => {
                let total_pages = (total as f64 / page_size as f64).ceil() as i64;
                
                let user_responses: Vec<AdminUserResponse> = users
                    .into_iter()
                    .map(|u| AdminUserResponse {
                        id: u.id,
                        username: u.username,
                        is_superuser: u.is_superuser,
                    })
                    .collect();

                let list_data = AdminListUsersData {
                    users: user_responses,
                    pagination: AdminPaginationInfo {
                        total,
                        page,
                        page_size,
                        total_pages,
                    },
                };

                info!(
                    "[MQTT Admin Service] Listed {} users (total: {}, page: {})",
                    list_data.users.len(),
                    total,
                    page
                );

                AdminResponse::success(
                    request.request_id,
                    "Users retrieved successfully".to_string(),
                    Some(AdminResponseData::ListUsers(list_data)),
                )
            }
            Err(e) => {
                error!("[MQTT Admin Service] Error listing users: {}", e);
                AdminResponse::error(
                    request.request_id,
                    "Failed to list users".to_string(),
                    Some(vec![e.to_string()]),
                )
            }
        }
    }

    pub async fn get_user_by_username(
        &self,
        request: AdminGetUserByUsernameRequest,
    ) -> AdminResponse {
        debug!("[MQTT Admin Service] Getting user by username: {}", request.username);

        match self.list_repo.get_mqtt_by_username(&request.username).await {
            Ok(Some(user)) => {
                let username = user.username.clone();
                let user_response = AdminUserResponse {
                    id: user.id,
                    username: user.username,
                    is_superuser: user.is_superuser,
                };
                info!("[MQTT Admin Service] User retrieved successfully: {}", username);
                AdminResponse::success(
                    request.request_id,
                    "User retrieved successfully".to_string(),
                    Some(AdminResponseData::GetUser(user_response)),
                )
            }
            Ok(None) => {
                debug!("[MQTT Admin Service] User not found: {}", request.username);
                AdminResponse::error(
                    request.request_id,
                    "User not found".to_string(),
                    None,
                )
            }
            Err(e) => {
                error!("[MQTT Admin Service] Error getting user: {}", e);
                AdminResponse::error(
                    request.request_id,
                    "Failed to get user".to_string(),
                    Some(vec![e.to_string()]),
                )
            }
        }
    }
}
