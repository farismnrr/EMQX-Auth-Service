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
            })
            .collect();
        debug!("[Service | GetMQTTList] User MQTT list retrieved successfully.");
        Ok(dto_mqtts)
    }

    pub async fn get_mqtt_list_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<MqttDTO>, i64), MqttServiceError> {
        let (mqtts, total) = self.repo.get_mqtt_list_paginated(page, page_size).await?;
        let dto_mqtts: Vec<MqttDTO> = mqtts
            .into_iter()
            .map(|mqtt| MqttDTO {
                username: mqtt.username,
                password: mqtt.password,
                is_superuser: mqtt.is_superuser,
            })
            .collect();
        debug!(
            "[Service | GetMQTTListPaginated] User MQTT list retrieved successfully (total: {}).",
            total
        );
        Ok((dto_mqtts, total))
    }

    pub async fn get_mqtt_by_id(&self, id: i32) -> Result<Option<MqttDTO>, MqttServiceError> {
        let mqtt = self.repo.get_mqtt_by_id(id).await?;
        match mqtt {
            Some(m) => {
                let dto = MqttDTO {
                    username: m.username,
                    password: m.password,
                    is_superuser: m.is_superuser,
                };
                debug!("[Service | GetMQTTById] User MQTT retrieved successfully.");
                Ok(Some(dto))
            }
            None => {
                debug!("[Service | GetMQTTById] User MQTT not found.");
                Ok(None)
            }
        }
    }
}
