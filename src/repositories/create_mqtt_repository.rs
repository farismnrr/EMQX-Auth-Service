use crate::entities::mqtt_entity::{ActiveModel, Entity as MqttUser};
use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error};
use sea_orm::{DatabaseConnection, EntityTrait, Set};

pub struct CreateMqttRepository {
    db: DatabaseConnection,
}

impl CreateMqttRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        CreateMqttRepository { db }
    }

    pub async fn create_mqtt(
        &self,
        username: &str,
        password_hash: &str,
        is_superuser: bool,
    ) -> Result<(), MqttRepositoryError> {
        debug!(
            "[Repository | CreateMQTT] Starting user MQTT creation for username: {}",
            username
        );

        let new_user = ActiveModel {
            username: Set(username.to_owned()),
            password: Set(password_hash.to_owned()),
            is_deleted: Set(false),
            is_superuser: Set(is_superuser),
            ..Default::default()
        };

        match MqttUser::insert(new_user).exec(&self.db).await {
            Ok(_) => {
                debug!(
                    "[Repository | CreateMQTT] User MQTT {} successfully written to MySQL",
                    username
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "[Repository | CreateMQTT] Failed to write user MQTT {} to MySQL: {e}",
                    username
                );
                Err(MqttRepositoryError::SeaOrm(e))
            }
        }
    }
}
