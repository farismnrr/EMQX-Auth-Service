//! MQTT User SeaORM model

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "mqtt_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
#[allow(dead_code)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Convert to domain entity
impl From<Model> for crate::domain::MqttUser {
    fn from(model: Model) -> Self {
        crate::domain::MqttUser {
            id: model.id,
            username: model.username,
            password_hash: model.password,
            is_superuser: model.is_superuser,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
