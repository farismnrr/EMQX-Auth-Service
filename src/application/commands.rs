//! Application commands (input DTOs)

use serde::Deserialize;

/// Create user command
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateUserCommand {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub is_superuser: bool,
}

/// Login command
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LoginCommand {
    pub username: String,
    pub password: String,
    #[serde(default = "default_method")]
    pub method: String,
}

fn default_method() -> String {
    "credentials".to_string()
}

/// ACL check command
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AclCommand {
    pub username: String,
    pub topic: String,
}
