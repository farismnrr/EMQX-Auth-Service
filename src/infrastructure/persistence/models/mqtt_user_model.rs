//! MQTT User SeaORM model

#![allow(dead_code)]

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "mqtt_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub username: String,
    #[sea_orm(column_name = "password_hash")]
    pub password_hash: String,
    pub is_superuser: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
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
            password: model.password_hash,
            is_superuser: model.is_superuser.unwrap_or(false),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
