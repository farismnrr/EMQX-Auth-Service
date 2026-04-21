//! MQTT User entity

#![allow(dead_code)]

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// MQTT User entity
///
/// Represents a user in the MQTT authentication system.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MqttUser {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl MqttUser {
    /// Check if user is superuser
    pub fn is_superuser(&self) -> bool {
        self.is_superuser
    }
}
