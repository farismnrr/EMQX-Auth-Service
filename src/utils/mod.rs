//! Shared utilities

pub mod encryption_util;
pub mod error_util;

pub use encryption_util::{hex_to_key32, EncryptionError};
pub use error_util::*;
