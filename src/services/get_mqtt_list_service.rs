use crate::dtos::mqtt_dto::MqttDTO;
use crate::repositories::get_mqtt_list_repository::GetMqttListRepository;
use crate::services::service_error::MqttServiceError;
use log::debug;
use std::sync::Arc;

pub struct GetMqttListService {
    repo: Arc<GetMqttListRepository>,
}

impl GetMqttListService {
    pub fn new(repo: Arc<GetMqttListRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_mqtt_list(&self) -> Result<Vec<MqttDTO>, MqttServiceError> {
        let mqtts = self.repo.get_mqtt_list().await?;
        let dto_mqtts: Vec<MqttDTO> = mqtts
            .into_iter()
            .map(|mqtt| MqttDTO {
                username: mqtt.username,
                password: mqtt.password,
                is_superuser: mqtt.is_superuser,
                is_deleted: mqtt.is_deleted,
            })
            .collect();
        debug!("[Service | GetMQTTList] User MQTT list retrieved successfully.");
        Ok(dto_mqtts)
    }
}
