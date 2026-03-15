//! MQTT User entity

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// MQTT User entity
///
/// Represents a user in the MQTT authentication system.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MqttUser {
    pub id: i64,
    pub username: String,
    pub password_ciphertext: String,
    pub is_superuser: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MqttUser {
    /// Check if user is superuser
    pub fn is_superuser(&self) -> bool {
        self.is_superuser
    }
}
