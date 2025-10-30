//! RPC-style response helpers for user handler.
//!
//! Exposes a tiny helper to build RPC responses in this shape:
//! {
//!   isSuccess,
//!   message,
//!   data { username }
//! }
//!
//! The implementation deliberately avoids new dependencies and builds a small
//! JSON string manually. It's intended for use by gRPC handler wiring that will
//! translate service results into RPC responses.
// Reuse a shared utility for JSON escaping so other modules can use it too.
use crate::utils::escape_json_str;

/// Build the RPC response for a user-create operation.
///
/// - `username`: optional username to include in `data`; pass `None` for error
///   responses or when no data should be returned.
/// - `is_success`: whether the operation succeeded.
/// - `message`: human-readable message (will be escaped).
///
/// Returns a JSON string matching the requested RPC shape. The function keeps
/// the format stable so callers in `gRPC` can adopt it as the contract.
pub fn build_user_create_response(username: Option<&str>, is_success: bool, message: &str) -> String {
    let msg = escape_json_str(message);
    let data = if let Some(u) = username {
        format!(r#"{{"username":"{}"}}"#, escape_json_str(u))
    } else {
        "null".to_string()
    };

    format!(r#"{{"isSuccess":{},"message":"{}","data":{}}}"#, is_success, msg, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response_contains_username() {
        let json = build_user_create_response(Some("alice"), true, "created");
        // basic checks without pulling a JSON parser in
        assert!(json.contains("\"isSuccess\":true"));
        assert!(json.contains("\"message\":\"created\""));
        assert!(json.contains("\"data\":{"));
        assert!(json.contains("\"username\":\"alice\""));
    }

    #[test]
    fn error_response_has_null_data_and_escaped_message() {
        let json = build_user_create_response(None, false, "bad \"input\"");
        assert!(json.contains("\"isSuccess\":false"));
        // message should be escaped (contains \")
        assert!(json.contains("bad \\\"input\\\""));
        assert!(json.contains("\"data\":null"));
    }
}
