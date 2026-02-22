use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScopeMode {
    #[default]
    Flat,
    Concat,
    Stacked,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_compression")]
    pub compression: i32,

    #[serde(default)]
    pub scope_mode: ScopeMode,

    #[serde(default)]
    pub keep_urls: bool,

    #[serde(default)]
    pub sentence_split: bool,

    #[serde(default)]
    pub anchor_every: usize,

    #[serde(default = "default_max_kv_per_line")]
    pub max_kv_per_line: usize,

    #[serde(default = "default_bool_compress")]
    pub bool_compress: bool,

    #[serde(default = "default_prefix_extraction")]
    pub prefix_extraction: bool,

    #[serde(default = "default_min_prefix_len")]
    pub min_prefix_len: usize,

    #[serde(default = "default_min_prefix_pct")]
    pub min_prefix_pct: f64,

    #[serde(default)]
    pub stopwords: Vec<String>,

    #[serde(default)]
    pub protect_words: Vec<String>,

    #[serde(default)]
    pub phrase_map: HashMap<String, String>,

    #[serde(default)]
    pub units: HashMap<String, String>,
}

fn default_compression() -> i32 {
    2
}
fn default_max_kv_per_line() -> usize {
    4
}
fn default_bool_compress() -> bool {
    true
}
fn default_prefix_extraction() -> bool {
    true
}
fn default_min_prefix_len() -> usize {
    6
}
fn default_min_prefix_pct() -> f64 {
    0.6
}

impl Default for Config {
    fn default() -> Self {
        Config {
            compression: 2,
            scope_mode: ScopeMode::Flat,
            keep_urls: false,
            sentence_split: false,
            anchor_every: 0,
            max_kv_per_line: 4,
            bool_compress: true,
            prefix_extraction: true,
            min_prefix_len: 6,
            min_prefix_pct: 0.6,
            stopwords: Vec::new(),
            protect_words: Vec::new(),
            phrase_map: HashMap::new(),
            units: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.compression, 2);
        assert_eq!(config.scope_mode, ScopeMode::Flat);
        assert!(!config.keep_urls);
        assert!(!config.sentence_split);
        assert_eq!(config.anchor_every, 0);
        assert_eq!(config.max_kv_per_line, 4);
        assert!(config.bool_compress);
        assert!(config.prefix_extraction);
        assert_eq!(config.min_prefix_len, 6);
        assert!((config.min_prefix_pct - 0.6).abs() < f64::EPSILON);
        assert!(config.stopwords.is_empty());
        assert!(config.protect_words.is_empty());
        assert!(config.phrase_map.is_empty());
        assert!(config.units.is_empty());
    }

    #[test]
    fn test_deserialize_full_config() {
        let json = r#"{
            "compression": 2,
            "scope_mode": "flat",
            "keep_urls": false,
            "sentence_split": false,
            "anchor_every": 0,
            "max_kv_per_line": 4,
            "bool_compress": true,
            "prefix_extraction": true,
            "min_prefix_len": 6,
            "min_prefix_pct": 0.6,
            "stopwords": ["the", "a"],
            "protect_words": ["no", "not"],
            "phrase_map": {"in order to": "to"},
            "units": {"milliseconds": "ms"}
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.compression, 2);
        assert_eq!(config.scope_mode, ScopeMode::Flat);
        assert_eq!(config.stopwords, vec!["the", "a"]);
        assert_eq!(config.protect_words, vec!["no", "not"]);
        assert_eq!(config.phrase_map.get("in order to"), Some(&"to".to_string()));
        assert_eq!(config.units.get("milliseconds"), Some(&"ms".to_string()));
    }

    #[test]
    fn test_deserialize_partial_config() {
        let json = r#"{"compression": 1}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.compression, 1);
        assert_eq!(config.scope_mode, ScopeMode::Flat);
        assert!(config.stopwords.is_empty());
    }

    #[test]
    fn test_scope_mode_variants() {
        let flat: Config = serde_json::from_str(r#"{"scope_mode": "flat"}"#).unwrap();
        assert_eq!(flat.scope_mode, ScopeMode::Flat);

        let concat: Config = serde_json::from_str(r#"{"scope_mode": "concat"}"#).unwrap();
        assert_eq!(concat.scope_mode, ScopeMode::Concat);

        let stacked: Config = serde_json::from_str(r#"{"scope_mode": "stacked"}"#).unwrap();
        assert_eq!(stacked.scope_mode, ScopeMode::Stacked);
    }
}
