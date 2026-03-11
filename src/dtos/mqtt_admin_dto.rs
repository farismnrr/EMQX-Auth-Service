use serde::{Deserialize, Serialize};

// =============================================================================
// Request DTOs
// =============================================================================

#[derive(Debug, Deserialize, Serialize)]
pub struct AdminCreateUserRequest {
    pub request_id: String,
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub is_superuser: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdminDeleteUserRequest {
    pub request_id: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdminListUsersRequest {
    pub request_id: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdminGetUserByUsernameRequest {
    pub request_id: String,
    pub username: String,
}

// =============================================================================
// Response DTOs
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUserResponse {
    pub id: i32,
    pub username: String,
    pub is_superuser: bool,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminPaginationInfo {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminListUsersData {
    pub users: Vec<AdminUserResponse>,
    pub pagination: AdminPaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AdminResponseData {
    CreateUser(AdminUserResponse),
    ListUsers(AdminListUsersData),
    GetUser(AdminUserResponse),
    Empty,
}

impl Default for AdminResponseData {
    fn default() -> Self {
        AdminResponseData::Empty
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminResponse {
    pub request_id: String,
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<AdminResponseData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

impl AdminResponse {
    pub fn success(request_id: String, message: String, data: Option<AdminResponseData>) -> Self {
        Self {
            request_id,
            success: true,
            message,
            data,
            errors: None,
        }
    }

    pub fn error(
        request_id: String,
        message: String,
        errors: Option<Vec<String>>,
    ) -> Self {
        Self {
            request_id,
            success: false,
            message,
            data: None,
            errors,
        }
    }
}

// =============================================================================
// MQTT Topic Constants
// =============================================================================

pub mod topics {
    pub const ADMIN_USERS_CREATE: &str = "admins/users/create";
    pub const ADMIN_USERS_DELETE: &str = "admins/users/delete";
    pub const ADMIN_USERS_LIST: &str = "admins/users";
    pub const ADMIN_USERS_GET_BY_USERNAME: &str = "admins/users/";
    pub const ADMIN_USERS_GET_BY_USERNAME_WILDCARD: &str = "admins/users/+";
    
    // Shared subscription group for multi-instance support
    // Format: $share/{group_id}/{topic}
    // Only one subscriber in the group receives each message
    pub const SHARED_SUBSCRIPTION_GROUP: &str = "emqx_auth_service";
    pub const ADMIN_USERS_CREATE_SHARED: &str = "$share/emqx_auth_service/admins/users/create";
    pub const ADMIN_USERS_DELETE_SHARED: &str = "$share/emqx_auth_service/admins/users/delete";
    pub const ADMIN_USERS_LIST_SHARED: &str = "$share/emqx_auth_service/admins/users";
    pub const ADMIN_USERS_GET_BY_USERNAME_SHARED: &str = "$share/emqx_auth_service/admins/users/+";

    pub const RESPONSE_CREATE: &str = "admins/users/create/response";
    pub const RESPONSE_DELETE: &str = "admins/users/delete/response";
    pub const RESPONSE_LIST: &str = "admins/users/list/response";
    pub const RESPONSE_DETAIL: &str = "admins/users/detail/response";
}
