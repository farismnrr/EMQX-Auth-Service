use crate::entities::mqtt_entity::{Entity as MqttUser, Model as MqttEntity};
use crate::repositories::repository_error::MqttRepositoryError;
use log::debug;
use sea_orm::{DatabaseConnection, EntityTrait};

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
            .all(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        debug!(
            "[Repository | GetMQTTList] Successfully fetched {} user MQTT records",
            users.len()
        );
        Ok(users)
    }
}
