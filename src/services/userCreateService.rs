//! Service layer: user creation (no-op stub)
//!
//! This file currently provides a simple stub for the create-user service function.
//! The function accepts the expected parameters but performs no logic and returns
//! success immediately. Repositories and business logic will be added later.

/// Create a new user (service layer).
///
/// Currently this is a no-op stub that returns success immediately.
/// Parameters:
/// - `username`: username string
/// - `password`: password string
/// - `is_deleted`: whether the user is marked deleted
pub fn create_user_service(username: &str, password: &str, is_deleted: bool) -> Result<(), String> {
	// silence unused variable warnings while this is a stub
	let _ = (username, password, is_deleted);

	// no logic yet — return success
	Ok(())
}

