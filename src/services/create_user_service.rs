use std::sync::Arc;
use uuid::Uuid;
use argon2::password_hash::rand_core::{OsRng, RngCore};

use crate::repositories::create_user_repository::CreateUserRepository;
use crate::services::service_error::UserServiceError;
use crate::utils::hash_password::hash_password;

pub struct CreateUserService {
    repo: Arc<CreateUserRepository>,
}

impl CreateUserService {
    pub fn new(repo: Arc<CreateUserRepository>) -> Self {
        Self { repo }
    }

    pub fn create_user(&self) -> Result<(String, String), UserServiceError> {
        let username = Self::create_username();
        let password = Self::create_password();
        let hashed = hash_password(&password)
            .map_err(|e| UserServiceError::Hashing(e))?;
        self.repo.create_user(&username, &hashed)?;
        Ok((username, password))
    }

    fn create_username() -> String {
        format!("user_{}", Uuid::new_v4())
    }
    
    fn create_password() -> String {
        let mut buf = [0u8; 32];
        let mut rng = OsRng;
        rng.try_fill_bytes(&mut buf).ok();
        hex::encode(buf)
    }
}
