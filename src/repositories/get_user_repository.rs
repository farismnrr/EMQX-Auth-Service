use rocksdb::{DB};
use std::sync::Arc;
use bincode::config::standard;
use bincode::decode_from_slice;
use log::error;
use crate::entities::user_entity::UserEntity;
use crate::repositories::repository_error::UserRepositoryError;

pub struct GetUserRepository {
    db: Arc<DB>,
}

impl GetUserRepository {
    pub fn new(db: Arc<DB>) -> Self {
        GetUserRepository { db }
    }

    pub fn get_active_users(&self, username: &str) -> Result<Option<UserEntity>, UserRepositoryError> {
        // Build RocksDB key
        let key: String = format!("users:{}", username);

        // Try to fetch record from DB
        let value = match self.db.get(key.as_bytes()) {
            Ok(v) => v,
            Err(e) => {
                error!("Database read error for user {username}: {e}");
                return Err(UserRepositoryError::Database(e));
            }
        };

        let Some(value) = value else {
            return Ok(None);
        };

        let (user, _) = match decode_from_slice::<UserEntity, _>(&value, standard()) {
            Ok(decoded) => decoded,
            Err(e) => {
                error!("Failed to decode user data for {username}: {e}");
                return Err(UserRepositoryError::Decode(e));
            }
        };

        if user.is_deleted {
            return Ok(None);
        }
        
        Ok(Some(user))
    }
}
