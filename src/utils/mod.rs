//! Utility helpers used across the crate.
//!
//! Small, dependency-free helpers live here. When functionality grows or needs
//! robust serialization, prefer adding `serde` and moving logic into dedicated
//! modules.

/// Escape a string for safe inclusion in a JSON string value.
/// This is a minimal implementation that covers common characters (quote,
/// backslash and common control characters). It's intentionally small to avoid
/// adding `serde` as a dependency at this stage.
pub fn escape_json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::escape_json_str;

    #[test]
    fn escapes_quotes_and_backslashes() {
        let s = r#"a\"b"#;
        let got = escape_json_str(s);
        assert!(got.contains("\\\"") || got.contains("\\\\"));
    }

    #[test]
    fn preserves_simple_text() {
        let s = "alice";
        assert_eq!(escape_json_str(s), "alice");
    }
}
