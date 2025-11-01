use rocksdb::DB;
use std::sync::Arc;
use bincode::{decode_from_slice, config::standard};
use log::{error, warn};
use crate::entities::user_entity::UserEntity;
use crate::repositories::repository_error::UserRepositoryError;

pub struct GetUserListRepository {
    db: Arc<DB>,
}

impl GetUserListRepository {
    pub fn new(db: Arc<DB>) -> Self {
        GetUserListRepository { db }
    }

    pub fn get_user_list(&self) -> Result<Vec<UserEntity>, UserRepositoryError> {
        let mut users = Vec::new();

        for (key, value) in self.db.iterator(rocksdb::IteratorMode::Start).flatten() {
            // Try to convert key to UTF-8 string
            let key_str = match String::from_utf8(key.to_vec()) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to convert key to string: {e}");
                    return Err(UserRepositoryError::Utf8(e))
                }
            };

            // Skip non-user keys
            if !key_str.starts_with("users:") {
                warn!("Skipping non-user key: {}", key_str);
                continue;
            }

            // Decode user data from bincode
            let user = match decode_from_slice::<UserEntity, _>(&value, standard()) {
                Ok((user, _)) => user,
                Err(e) => {
                    error!("Failed to decode user for key {}: {}", key_str, e);
                    return Err(UserRepositoryError::Decode(e))
                }
            };

            // Skip deleted users
            if user.is_deleted {
                warn!("Skipping deleted user: {}", user.username);
                continue;
            }

            users.push(user);
        }

        Ok(users)
    }
}