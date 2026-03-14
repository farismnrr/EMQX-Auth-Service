//! REST API handlers

pub mod create_user_handler;
pub mod delete_user_handler;
pub mod get_user_by_id_handler;
pub mod get_user_by_username_handler;
pub mod list_users_handler;

pub use create_user_handler::create_user_handler;
pub use delete_user_handler::delete_user_handler;
pub use get_user_by_id_handler::get_user_by_id_handler;
pub use get_user_by_username_handler::get_user_by_username_handler;
pub use list_users_handler::list_users_handler;
