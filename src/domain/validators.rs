//! Domain validators - Reusable validation logic
//!
//! This module provides centralized validation for user credentials
//! to avoid duplication across use cases and handlers.

/// Validation result type
pub type ValidationResult<T> = Result<T, Vec<String>>;

/// User credential validator
pub struct UserValidator;

impl UserValidator {
    /// Validate username
    pub fn validate_username(username: &str) -> Result<(), String> {
        if username.trim().is_empty() {
            return Err("username cannot be empty".to_string());
        }
        if username.len() > 64 {
            return Err("username cannot exceed 64 characters".to_string());
        }
        if username.len() < 3 {
            return Err("username must be at least 3 characters".to_string());
        }
        Ok(())
    }

    /// Validate password
    pub fn validate_password(password: &str) -> Result<(), String> {
        if password.is_empty() {
            return Err("password cannot be empty".to_string());
        }
        if password.len() < 6 {
            return Err("password must be at least 6 characters".to_string());
        }
        if password.len() > 128 {
            return Err("password cannot exceed 128 characters".to_string());
        }
        Ok(())
    }

    /// Validate both username and password
    pub fn validate_credentials(username: &str, password: &str) -> ValidationResult<()> {
        let mut errors = Vec::new();

        if let Err(e) = Self::validate_username(username) {
            errors.push(e);
        }
        if let Err(e) = Self::validate_password(password) {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        assert!(UserValidator::validate_username("admin").is_ok());
        assert!(UserValidator::validate_username("user123").is_ok());
    }

    #[test]
    fn test_invalid_username() {
        assert!(UserValidator::validate_username("").is_err());
        assert!(UserValidator::validate_username("   ").is_err());
        assert!(UserValidator::validate_username(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_valid_password() {
        assert!(UserValidator::validate_password("password123").is_ok());
        assert!(UserValidator::validate_password("123456").is_ok());
    }

    #[test]
    fn test_invalid_password() {
        assert!(UserValidator::validate_password("").is_err());
        assert!(UserValidator::validate_password("12345").is_err());
    }
}
