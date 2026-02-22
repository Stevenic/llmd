use crate::config::Config;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static RE_MULTI_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

pub fn compress_c0(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        let t = RE_MULTI_SPACE.replace_all(line, " ").trim().to_string();
        if !t.is_empty() {
            out.push(t);
        }
    }
    out
}

pub fn compress_c1(lines: &[String]) -> Vec<String> {
    // Merging already handled during emission
    compress_c0(lines)
}

pub fn compress_c2(lines: &[String], config: &Config) -> Vec<String> {
    let stopwords: HashSet<String> = config
        .stopwords
        .iter()
        .map(|s| s.to_lowercase())
        .collect();
    let protect: HashSet<String> = config
        .protect_words
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    // Pre-compile phrase map regexes, sorted by length desc for longest match
    let mut phrase_entries: Vec<(&String, &String)> = config.phrase_map.iter().collect();
    phrase_entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let phrase_regexes: Vec<(Regex, &str)> = phrase_entries
        .iter()
        .map(|(phrase, replacement)| {
            let re = Regex::new(&format!("(?i){}", regex::escape(phrase))).unwrap();
            (re, replacement.as_str())
        })
        .collect();

    // Pre-compile unit regexes, sorted by length desc for longest match
    let mut unit_entries: Vec<(&String, &String)> = config.units.iter().collect();
    unit_entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let unit_regexes: Vec<(Regex, Regex, &str)> = unit_entries
        .iter()
        .map(|(unit, val)| {
            let re_num =
                Regex::new(&format!(r"(?i)(\d+)\s+{}", regex::escape(unit))).unwrap();
            let re_standalone =
                Regex::new(&format!("(?i){}", regex::escape(unit))).unwrap();
            (re_num, re_standalone, val.as_str())
        })
        .collect();

    let mut in_block = false;

    lines
        .iter()
        .map(|line| {
            if line == "<<<" {
                in_block = true;
                return line.clone();
            }
            if line == ">>>" {
                in_block = false;
                return line.clone();
            }
            if in_block {
                return line.clone();
            }
            if line.starts_with("::") || line.starts_with('@') {
                return line.clone();
            }

            let mut text = line.clone();

            // Apply phrase map and units on > and : lines
            if text.starts_with('>') || text.starts_with(':') {
                let prefix = &text[..1];
                let mut body = text[1..].to_string();

                for (re, replacement) in &phrase_regexes {
                    body = re.replace_all(&body, *replacement).to_string();
                }

                for (re_num, re_standalone, unit_val) in &unit_regexes {
                    // "N unit" pattern
                    let replacement = format!("${{1}}{}", unit_val);
                    body = re_num.replace_all(&body, replacement.as_str()).to_string();
                    // Standalone
                    body = re_standalone
                        .replace_all(&body, *unit_val)
                        .to_string();
                }

                text = format!("{}{}", prefix, body);
            }

            // Stopword removal on > lines only
            if text.starts_with('>') {
                let body = &text[1..];
                let tokens: Vec<&str> = body.split_whitespace().collect();
                let filtered: Vec<&str> = tokens
                    .into_iter()
                    .filter(|t| {
                        let low: String = t
                            .to_lowercase()
                            .chars()
                            .filter(|c| c.is_ascii_lowercase())
                            .collect();
                        if low.is_empty() {
                            return true;
                        }
                        if protect.contains(&low) {
                            return true;
                        }
                        !stopwords.contains(&low)
                    })
                    .collect();
                text = format!(">{}", filtered.join(" "));
            }

            text
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c0_whitespace_normalization() {
        let lines = vec!["  hello   world  ".to_string(), "  ".to_string()];
        let result = compress_c0(&lines);
        assert_eq!(result, vec!["hello world"]);
    }

    #[test]
    fn test_stopword_removal() {
        let mut config = Config::default();
        config.stopwords = vec!["the".to_string(), "a".to_string()];
        let lines = vec![">the big a dog".to_string()];
        let result = compress_c2(&lines, &config);
        assert_eq!(result, vec![">big dog"]);
    }

    #[test]
    fn test_protected_words() {
        let mut config = Config::default();
        config.stopwords = vec!["not".to_string()];
        config.protect_words = vec!["not".to_string()];
        let lines = vec![">do not delete".to_string()];
        let result = compress_c2(&lines, &config);
        assert_eq!(result, vec![">do not delete"]);
    }

    #[test]
    fn test_phrase_map() {
        let mut config = Config::default();
        config
            .phrase_map
            .insert("in order to".to_string(), "to".to_string());
        let lines = vec![">do this in order to achieve".to_string()];
        let result = compress_c2(&lines, &config);
        assert_eq!(result, vec![">do this to achieve"]);
    }

    #[test]
    fn test_unit_normalization() {
        let mut config = Config::default();
        config
            .units
            .insert("milliseconds".to_string(), "ms".to_string());
        let lines = vec![":timeout=500 milliseconds".to_string()];
        let result = compress_c2(&lines, &config);
        assert_eq!(result, vec![":timeout=500ms"]);
    }

    #[test]
    fn test_code_block_protection() {
        let config = Config::default();
        let lines = vec![
            "<<<".to_string(),
            "the code with stopwords".to_string(),
            ">>>".to_string(),
        ];
        let result = compress_c2(&lines, &config);
        assert_eq!(result[1], "the code with stopwords");
    }

    #[test]
    fn test_scope_lines_not_compressed() {
        let mut config = Config::default();
        config.stopwords = vec!["the".to_string()];
        let lines = vec!["@the_scope".to_string()];
        let result = compress_c2(&lines, &config);
        assert_eq!(result, vec!["@the_scope"]);
    }

    #[test]
    fn test_block_start_lines_not_compressed() {
        let mut config = Config::default();
        config.stopwords = vec!["the".to_string()];
        let lines = vec!["::the_lang".to_string()];
        let result = compress_c2(&lines, &config);
        assert_eq!(result, vec!["::the_lang"]);
    }
}
