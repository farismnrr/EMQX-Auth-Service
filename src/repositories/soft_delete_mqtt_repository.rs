use crate::entities::mqtt_entity::{ActiveModel, Column, Entity as MqttUser};
use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub struct SoftDeleteMqttRepository {
    db: DatabaseConnection,
}

impl SoftDeleteMqttRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        SoftDeleteMqttRepository { db }
    }

    pub async fn soft_delete_mqtt(&self, username: &str) -> Result<(), MqttRepositoryError> {
        debug!(
            "[Repository | SoftDelete] Marking user MQTT {} as deleted in MySQL",
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
                let mut active_user: ActiveModel = m.into();
                active_user.is_deleted = Set(true);

                active_user
                    .update(&self.db)
                    .await
                    .map_err(MqttRepositoryError::SeaOrm)?;
                debug!(
                    "[Repository | SoftDelete] Successfully marked user MQTT {} as deleted",
                    username
                );
                Ok(())
            }
            None => {
                error!(
                    "[Repository | SoftDelete] User MQTT {} not found or already deleted in MySQL",
                    username
                );
                Err(MqttRepositoryError::NotFound)
            }
        }
    }
}
