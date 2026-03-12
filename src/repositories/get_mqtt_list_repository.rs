use crate::entities::mqtt_entity::{Column, Entity as MqttUser, Model as MqttEntity};
use crate::repositories::repository_error::MqttRepositoryError;
use log::debug;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};

pub struct GetMqttListRepository {
    db: DatabaseConnection,
}

impl GetMqttListRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        GetMqttListRepository { db }
    }

    pub async fn get_mqtt_list(&self) -> Result<Vec<MqttEntity>, MqttRepositoryError> {
        debug!("[Repository | GetMQTTList] Fetching all user MQTT records from MySQL");

        let users = MqttUser::find()
            .order_by_asc(Column::Id)
            .all(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        debug!(
            "[Repository | GetMQTTList] Successfully fetched {} user MQTT records",
            users.len()
        );
        Ok(users)
    }

    pub async fn get_mqtt_list_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<MqttEntity>, i64), MqttRepositoryError> {
        debug!(
            "[Repository | GetMQTTListPaginated] Fetching page {} with page_size {}",
            page, page_size
        );

        let offset = (page - 1) * page_size;

        // Get total count
        let total = MqttUser::find()
            .count(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        // Get paginated results
        let users = MqttUser::find()
            .order_by_asc(Column::Id)
            .offset(offset as u64)
            .limit(page_size as u64)
            .all(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        debug!(
            "[Repository | GetMQTTListPaginated] Successfully fetched {} user MQTT records (total: {})",
            users.len(),
            total
        );
        Ok((users, total as i64))
    }

    pub async fn get_mqtt_by_username(&self, username: &str) -> Result<Option<MqttEntity>, MqttRepositoryError> {
        debug!("[Repository | GetMQTTByUsername] Fetching user MQTT record with username {}", username);

        let user = MqttUser::find()
            .filter(Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        match user {
            Some(u) => {
                debug!("[Repository | GetMQTTByUsername] Successfully fetched user MQTT record");
                Ok(Some(u))
            }
            None => {
                debug!("[Repository | GetMQTTByUsername] User MQTT record not found");
                Ok(None)
            }
        }
    }

    pub async fn get_mqtt_by_id(&self, id: i32) -> Result<Option<MqttEntity>, MqttRepositoryError> {
        debug!("[Repository | GetMQTTById] Fetching user MQTT record with id {}", id);

        let user = MqttUser::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        match user {
            Some(u) => {
                debug!("[Repository | GetMQTTById] Successfully fetched user MQTT record");
                Ok(Some(u))
            }
            None => {
                debug!("[Repository | GetMQTTById] User MQTT record not found");
                Ok(None)
            }
        }
    }

    pub async fn delete_mqtt_by_username(&self, username: &str) -> Result<(), MqttRepositoryError> {
        debug!("[Repository | DeleteMQTT] Deleting user MQTT record with username {}", username);

        let user = MqttUser::find()
            .filter(Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        match user {
            Some(m) => {
                let active_model = m.into_active_model();
                active_model
                    .delete(&self.db)
                    .await
                    .map_err(MqttRepositoryError::SeaOrm)?;
                debug!("[Repository | DeleteMQTT] Successfully deleted user MQTT record");
                Ok(())
            }
            None => {
                debug!("[Repository | DeleteMQTT] User MQTT record not found");
                Err(MqttRepositoryError::NotFound)
            }
        }
    }
}
