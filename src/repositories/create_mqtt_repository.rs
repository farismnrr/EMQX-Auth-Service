use crate::dtos::mqtt_dto::CreateMqttDTO;
use crate::entities::mqtt_entity::{ActiveModel, Column, Entity as MqttUser, Model as MqttEntity};
use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub struct CreateMqttRepository {
    db: DatabaseConnection,
}

impl CreateMqttRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        CreateMqttRepository { db }
    }

    pub async fn create(&self, dto: CreateMqttDTO) -> Result<(), MqttRepositoryError> {
        debug!(
            "[Repository | CreateMQTT] Starting user MQTT creation for username: {}",
            dto.username
        );

        let new_user = ActiveModel {
            username: Set(dto.username),
            password: Set(dto.password),
            is_deleted: Set(false),
            is_superuser: Set(dto.is_superuser),
            ..Default::default()
        };

        match MqttUser::insert(new_user).exec(&self.db).await {
            Ok(_) => {
                debug!(
                    "[Repository | CreateMQTT] User MQTT successfully written to MySQL",
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "[Repository | CreateMQTT] Failed to write user MQTT to MySQL: {e}",
                );
                Err(MqttRepositoryError::SeaOrm(e))
            }
        }
    }

    pub async fn get_by_username(
        &self,
        username: &str,
    ) -> Result<Option<MqttEntity>, MqttRepositoryError> {
        debug!(
            "[Repository | GetByUsername] Fetching user MQTT record with username: {}",
            username
        );

        let user = MqttUser::find()
            .filter(Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(MqttRepositoryError::SeaOrm)?;

        match &user {
            Some(_) => {
                debug!("[Repository | GetByUsername] User MQTT record found");
            }
            None => {
                debug!("[Repository | GetByUsername] User MQTT record not found");
            }
        }

        Ok(user)
    }
}
