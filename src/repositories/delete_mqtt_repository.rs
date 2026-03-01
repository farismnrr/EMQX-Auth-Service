use crate::entities::mqtt_entity::{Column, Entity as MqttUser};
use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct DeleteMqttRepository {
    db: DatabaseConnection,
}

impl DeleteMqttRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        DeleteMqttRepository { db }
    }

    pub async fn delete_mqtt(&self, username: &str) -> Result<(), MqttRepositoryError> {
        debug!(
            "[Repository | Delete] Deleting user MQTT {} from MySQL",
            username
        );

        let user = MqttUser::find()
            .filter(Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        match user {
            Some(m) => {
                MqttUser::delete_by_id(m.id)
                    .exec(&self.db)
                    .await
                    .map_err(MqttRepositoryError::SeaOrm)?;
                debug!(
                    "[Repository | Delete] Successfully deleted user MQTT {}",
                    username
                );
                Ok(())
            }
            None => {
                error!(
                    "[Repository | Delete] User MQTT {} not found in MySQL",
                    username
                );
                Err(MqttRepositoryError::NotFound)
            }
        }
    }
}
