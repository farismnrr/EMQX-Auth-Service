use std::sync::Arc;
use crate::repositories::check_user_active_repository::CheckUserActiveRepository;
use crate::services::service_error::UserServiceError;
use crate::dtos::user_dto::ValidateUserDTO;

pub struct CheckUserActiveService {
    repo: Arc<CheckUserActiveRepository>,
}

impl CheckUserActiveService {
    pub fn new(repo: Arc<CheckUserActiveRepository>) -> Self {
        Self { repo }
    }

    pub fn validate_user(&self, dto: ValidateUserDTO) -> Result<bool, UserServiceError> {
        let user = self.repo.check_user_active(&dto.username)?;
        if user.is_none() {
            return Err(UserServiceError::UserNotFound("User not found".to_string()));
        }

        if user.as_ref().unwrap().is_deleted {
            return Err(UserServiceError::UserNotActive("User is not active or deleted".to_string()));
        }

        if user.as_ref().unwrap().password != dto.password {
            return Err(UserServiceError::InvalidCredentials("Invalid credentials".to_string()));
        }

        Ok(user.is_some())
    }
}