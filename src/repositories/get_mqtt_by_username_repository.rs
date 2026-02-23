use crate::entities::mqtt_entity::{Column, Entity as MqttUser, Model as MqttEntity};
use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct GetMqttByUsernameRepository {
    db: DatabaseConnection,
}

impl GetMqttByUsernameRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        GetMqttByUsernameRepository { db }
    }

    pub async fn get_mqtt_by_username(
        &self,
        username: &str,
    ) -> Result<MqttEntity, MqttRepositoryError> {
        debug!(
            "[Repository | GetByUsername] Fetching user MQTT for username: {}",
            username
        );

        let user = MqttUser::find()
            .filter(Column::Username.eq(username))
            .filter(Column::IsDeleted.eq(false))
            .one(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        match user {
            Some(m) => {
                debug!(
                    "[Repository | GetByUsername] Successfully fetched user MQTT for username: {}",
                    username
                );
                Ok(m)
            }
            None => {
                error!(
                    "[Repository | GetByUsername] User MQTT {} not found in MySQL",
                    username
                );
                Err(MqttRepositoryError::NotFound)
            }
        }
    }
}
