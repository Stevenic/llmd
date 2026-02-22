use regex::Regex;
use std::sync::LazyLock;

static RE_WHITESPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static RE_NON_SCOPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^A-Za-z0-9_-]").unwrap());
static RE_NON_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^a-z0-9_-]").unwrap());
static RE_LEADING_TRAILING_DASH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-+|-+$").unwrap());

pub fn norm_scope_name(text: &str, compression: i32) -> String {
    let s = RE_WHITESPACE.replace_all(text.trim(), "_").to_string();
    let s = RE_NON_SCOPE.replace_all(&s, "").to_string();
    if compression >= 2 {
        s.to_lowercase()
    } else {
        s
    }
}

pub fn norm_key(text: &str) -> String {
    let s = text.trim().to_lowercase();
    let s = RE_WHITESPACE.replace_all(&s, "_").to_string();
    let s = RE_NON_KEY.replace_all(&s, "").to_string();
    RE_LEADING_TRAILING_DASH.replace_all(&s, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_norm_scope_name_c0() {
        assert_eq!(norm_scope_name("Hello World", 0), "Hello_World");
    }

    #[test]
    fn test_norm_scope_name_c2() {
        assert_eq!(norm_scope_name("Hello World", 2), "hello_world");
    }

    #[test]
    fn test_norm_scope_name_special_chars() {
        assert_eq!(norm_scope_name("API Reference!", 2), "api_reference");
    }

    #[test]
    fn test_norm_key() {
        assert_eq!(norm_key("Max Connections"), "max_connections");
    }

    #[test]
    fn test_norm_key_preserves_hyphens() {
        assert_eq!(norm_key("my-key"), "my-key");
    }

    #[test]
    fn test_norm_key_strips_leading_trailing_dashes() {
        assert_eq!(norm_key("-key-"), "key");
    }

    #[test]
    fn test_norm_key_special_chars() {
        assert_eq!(norm_key("Key (special)"), "key_special");
    }
}
