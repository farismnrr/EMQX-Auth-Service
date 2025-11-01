use rocksdb::DB;
use std::sync::Arc;
use bincode::{encode_to_vec, config::standard};
use log::error;
use crate::entities::user_entity::UserEntity;
use crate::repositories::repository_error::UserRepositoryError;

pub struct CreateUserRepository {
    db: Arc<DB>,
}

impl CreateUserRepository {
    pub fn new(db: Arc<DB>) -> Self {
        CreateUserRepository { db }
    }

    pub fn create_user(&self, username: &str, password_hash: &str) -> Result<(), UserRepositoryError> {
        // Build user entity
        let user = UserEntity::create(username, password_hash);
        let key = format!("users:{}", user.username);

        // Encode user to binary
        let value = match encode_to_vec(&user, standard()) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to serialize user {}: {e}", username);
                return Err(UserRepositoryError::Encode(e));
            }
        };

        // Write to RocksDB
        match self.db.put(key.as_bytes(), value) {
            Ok(_) => Ok(()),
            Err(e) => Err(UserRepositoryError::Database(e)),
        }
    }
}