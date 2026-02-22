use crate::config::{Config, ScopeMode};
use crate::inline::process_inline;
use crate::ir::{CodeBlock, IrNode};
use crate::scope::{norm_key, norm_scope_name};
use fancy_regex::Regex as FancyRegex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static RE_SENTENCE_SPLIT: LazyLock<FancyRegex> =
    LazyLock::new(|| FancyRegex::new(r"(?<=[.!?])\s+(?=[A-Z])").unwrap());

const GENERIC_HEADERS: &[&str] = &[
    "value",
    "description",
    "details",
    "info",
    "notes",
    "default",
    "type",
];

fn is_informative_header(header: &str) -> bool {
    if header.is_empty() {
        return false;
    }
    let low = header.trim().to_lowercase();
    !GENERIC_HEADERS.contains(&low.as_str())
}

fn classify_table(rows: &[Vec<String>]) -> &'static str {
    if rows.len() < 2 {
        return "raw";
    }
    let num_cols = rows[0].len();
    for r in &rows[1..] {
        if r.len() != num_cols {
            return "raw";
        }
    }
    if num_cols < 2 {
        return "raw";
    }
    // Check if first column values are unique and identifier-like
    let mut first_col_vals: HashSet<String> = HashSet::new();
    let re_ident = regex::Regex::new(r"^[A-Za-z._-]").unwrap();
    for r in &rows[1..] {
        let val = r[0].trim().to_string();
        if first_col_vals.contains(&val) {
            return "raw";
        }
        first_col_vals.insert(val.clone());
        if !re_ident.is_match(&val) || val.split_whitespace().count() > 4 {
            return "raw";
        }
    }
    if num_cols == 2 {
        return "property";
    }
    "keyed_multi"
}

fn bool_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("yes", "Y");
    m.insert("no", "N");
    m.insert("true", "T");
    m.insert("false", "F");
    m.insert("enabled", "Y");
    m.insert("disabled", "N");
    m
}

fn compress_bool_value(val: &str, enabled: bool) -> String {
    if !enabled {
        return val.to_string();
    }
    let bm = bool_map();
    let low = val.trim().to_lowercase();
    bm.get(low.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| val.to_string())
}

fn split_sentences(text: &str, sentence_split: bool, compression: i32) -> Vec<String> {
    if !sentence_split || compression < 2 {
        return vec![text.to_string()];
    }
    // Use fancy_regex for lookbehind
    let parts: Vec<String> = RE_SENTENCE_SPLIT
        .split(text)
        .filter_map(|r| r.ok())
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
        .collect();
    if parts.is_empty() {
        vec![text.to_string()]
    } else {
        parts
    }
}

fn find_common_prefix(keys: &[String]) -> String {
    if keys.len() < 2 {
        return String::new();
    }
    let mut prefix = keys[0].clone();
    for k in &keys[1..] {
        while !k.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return String::new();
            }
        }
    }
    // Trim to last separator boundary (-, _, .)
    let last_sep = [
        prefix.rfind('-'),
        prefix.rfind('_'),
        prefix.rfind('.'),
    ]
    .iter()
    .filter_map(|x| *x)
    .max();

    if let Some(pos) = last_sep {
        if pos > 0 {
            prefix.truncate(pos + 1);
        } else {
            return String::new();
        }
    } else {
        return String::new();
    }
    prefix
}

struct KvPair {
    key: String,
    value: String,
}

pub fn emit_llmd(ir: &[IrNode], blocks: &[CodeBlock], config: &Config) -> Vec<String> {
    let compression = config.compression;
    let keep_urls = config.keep_urls;
    let sentence_split = config.sentence_split;
    let bool_compress_enabled = config.bool_compress && compression >= 2;
    let max_kv_per_line = config.max_kv_per_line;
    let prefix_extraction = config.prefix_extraction;
    let min_prefix_len = config.min_prefix_len;
    let min_prefix_pct = config.min_prefix_pct;

    let bm = bool_map();

    let mut out: Vec<String> = Vec::new();
    let mut current_scope: Option<String> = None;
    let mut heading_stack: Vec<(usize, String)> = Vec::new();
    let mut kv_buffer: Vec<KvPair> = Vec::new();

    let resolve_scope = |level: usize, text: &str, stack: &mut Vec<(usize, String)>| -> String {
        let name = norm_scope_name(text, compression);
        while !stack.is_empty() && stack.last().unwrap().0 >= level {
            stack.pop();
        }
        stack.push((level, name.clone()));
        match config.scope_mode {
            ScopeMode::Flat => name,
            ScopeMode::Concat | ScopeMode::Stacked => {
                stack.iter().map(|h| h.1.as_str()).collect::<Vec<_>>().join("_")
            }
        }
    };

    let emit_scope =
        |scope: &str, current: &mut Option<String>, out: &mut Vec<String>| {
            if !scope.is_empty() && current.as_deref() != Some(scope) {
                out.push(format!("@{}", scope));
                *current = Some(scope.to_string());
            }
        };

    let ensure_scope = |current: &mut Option<String>, out: &mut Vec<String>| {
        if current.is_none() {
            out.push("@root".to_string());
            *current = Some("root".to_string());
        }
    };

    let process_text = |text: &str| -> String { process_inline(text, compression, keep_urls) };

    let process_cell = |cell: &str, col_idx: usize, bool_cols: &HashSet<usize>| -> String {
        let text = process_text(cell);
        if bool_cols.contains(&col_idx) {
            compress_bool_value(&text, bool_compress_enabled)
        } else {
            text
        }
    };

    let flush_kv = |kv_buffer: &mut Vec<KvPair>, out: &mut Vec<String>| {
        if kv_buffer.is_empty() {
            return;
        }

        // Try prefix extraction at c1+
        if compression >= 1 && prefix_extraction && kv_buffer.len() >= 3 {
            let keys: Vec<String> = kv_buffer.iter().map(|kv| kv.key.clone()).collect();
            let prefix = find_common_prefix(&keys);
            if prefix.len() >= min_prefix_len {
                let match_count = keys.iter().filter(|k| k.starts_with(&prefix)).count();
                if match_count as f64 / keys.len() as f64 >= min_prefix_pct {
                    out.push(format!(":_pfx={}", prefix));
                    let adjusted: Vec<KvPair> = kv_buffer
                        .drain(..)
                        .map(|kv| {
                            let key = if kv.key.starts_with(&prefix) {
                                kv.key[prefix.len()..].to_string()
                            } else {
                                kv.key
                            };
                            KvPair {
                                key,
                                value: kv.value,
                            }
                        })
                        .collect();
                    for chunk in adjusted.chunks(max_kv_per_line) {
                        let pairs: Vec<String> = chunk
                            .iter()
                            .map(|kv| format!("{}={}", kv.key, kv.value))
                            .collect();
                        out.push(format!(":{}", pairs.join(" ")));
                    }
                    return;
                }
            }
        }

        if compression >= 1 {
            for chunk in kv_buffer.chunks(max_kv_per_line) {
                let pairs: Vec<String> = chunk
                    .iter()
                    .map(|kv| format!("{}={}", kv.key, kv.value))
                    .collect();
                out.push(format!(":{}", pairs.join(" ")));
            }
        } else {
            for kv in kv_buffer.iter() {
                out.push(format!(":{}={}", kv.key, kv.value));
            }
        }
        kv_buffer.clear();
    };

    for node in ir {
        if !matches!(node, IrNode::Kv { .. }) {
            flush_kv(&mut kv_buffer, &mut out);
        }

        match node {
            IrNode::Heading { level, text } => {
                let scope = resolve_scope(*level, text, &mut heading_stack);
                emit_scope(&scope, &mut current_scope, &mut out);
            }
            IrNode::Paragraph { text } => {
                ensure_scope(&mut current_scope, &mut out);
                let text = process_text(text);
                let sentences = split_sentences(&text, sentence_split, compression);
                for s in sentences {
                    let s = s.trim();
                    if !s.is_empty() {
                        out.push(format!(">{}", s));
                    }
                }
            }
            IrNode::ListItem { depth, text, .. } => {
                ensure_scope(&mut current_scope, &mut out);
                let text = process_text(text);
                let prefix = ".".repeat(*depth);
                if prefix.is_empty() {
                    out.push(format!(">{}", text));
                } else {
                    out.push(format!(">{} {}", prefix, text));
                }
            }
            IrNode::Kv { key, value } => {
                ensure_scope(&mut current_scope, &mut out);
                let k = norm_key(key);
                let v = process_text(value);
                if !k.is_empty() {
                    kv_buffer.push(KvPair { key: k, value: v });
                } else {
                    out.push(format!(
                        ">{}",
                        process_text(&format!("{}: {}", key, value))
                    ));
                }
            }
            IrNode::Table { rows } => {
                ensure_scope(&mut current_scope, &mut out);
                let table_type = classify_table(rows);

                // Detect boolean columns for compression
                let mut bool_cols: HashSet<usize> = HashSet::new();
                if bool_compress_enabled && rows.len() > 1 {
                    for c in 1..rows[0].len() {
                        let all_bool = rows[1..].iter().all(|r| {
                            let val = r.get(c).map_or("", |s| s.as_str()).trim().to_lowercase();
                            bm.contains_key(val.as_str())
                        });
                        if all_bool {
                            bool_cols.insert(c);
                        }
                    }
                }

                match table_type {
                    "property" => {
                        // Emit column header if informative
                        if rows[0].len() >= 2 && is_informative_header(&rows[0][1]) {
                            let col_header = norm_key(&rows[0][1]);
                            if !col_header.is_empty() {
                                out.push(format!(":_col={}", col_header));
                            }
                        }
                        for r in &rows[1..] {
                            let k = norm_key(&r[0]);
                            let v = process_cell(&r[1], 1, &bool_cols);
                            if !k.is_empty() {
                                kv_buffer.push(KvPair { key: k, value: v });
                            } else {
                                out.push(format!(
                                    ">{}",
                                    process_text(&format!("{}|{}", r[0], r[1]))
                                ));
                            }
                        }
                    }
                    "keyed_multi" => {
                        let col_headers: Vec<String> =
                            rows[0].iter().map(|h| norm_key(h)).collect();
                        out.push(format!(":_cols={}", col_headers.join("|")));
                        for r in &rows[1..] {
                            let k = norm_key(&r[0]);
                            let vals: Vec<String> = r[1..]
                                .iter()
                                .enumerate()
                                .map(|(ci, c)| process_cell(c, ci + 1, &bool_cols))
                                .collect();
                            if !k.is_empty() {
                                kv_buffer.push(KvPair {
                                    key: k,
                                    value: vals.join("|"),
                                });
                            } else {
                                let cells: Vec<String> = r
                                    .iter()
                                    .enumerate()
                                    .map(|(ci, c)| process_cell(c, ci, &bool_cols))
                                    .collect();
                                out.push(format!(">{}", cells.join("|")));
                            }
                        }
                    }
                    _ => {
                        // raw
                        if rows[0].len() >= 2 {
                            let col_headers: Vec<String> =
                                rows[0].iter().map(|h| norm_key(h)).collect();
                            out.push(format!(":_cols={}", col_headers.join("|")));
                        }
                        for r in &rows[1..] {
                            let cells: Vec<String> = r
                                .iter()
                                .enumerate()
                                .map(|(ci, c)| process_cell(c, ci, &bool_cols))
                                .collect();
                            out.push(format!(">{}", cells.join("|")));
                        }
                    }
                }
            }
            IrNode::BlockRef { index } => {
                ensure_scope(&mut current_scope, &mut out);
                let block = &blocks[*index];
                let lang = if block.lang.is_empty() {
                    "code"
                } else {
                    &block.lang
                };
                out.push(format!("::{}", lang));
                out.push("<<<".to_string());
                out.push(block.content.clone());
                out.push(">>>".to_string());
            }
            IrNode::Blank => {}
        }
    }
    flush_kv(&mut kv_buffer, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_emission() {
        let ir = vec![
            IrNode::Heading {
                level: 1,
                text: "Title".to_string(),
            },
            IrNode::Paragraph {
                text: "content".to_string(),
            },
        ];
        let config = Config::default();
        let result = emit_llmd(&ir, &[], &config);
        assert_eq!(result[0], "@title");
        assert_eq!(result[1], ">content");
    }

    #[test]
    fn test_root_scope_injection() {
        let ir = vec![IrNode::Paragraph {
            text: "orphan text".to_string(),
        }];
        let config = Config::default();
        let result = emit_llmd(&ir, &[], &config);
        assert_eq!(result[0], "@root");
        assert_eq!(result[1], ">orphan text");
    }

    #[test]
    fn test_kv_c0_separate() {
        let ir = vec![
            IrNode::Heading {
                level: 1,
                text: "S".to_string(),
            },
            IrNode::Kv {
                key: "Key A".to_string(),
                value: "1".to_string(),
            },
            IrNode::Kv {
                key: "Key B".to_string(),
                value: "2".to_string(),
            },
        ];
        let mut config = Config::default();
        config.compression = 0;
        let result = emit_llmd(&ir, &[], &config);
        assert!(result.contains(&":key_a=1".to_string()));
        assert!(result.contains(&":key_b=2".to_string()));
    }

    #[test]
    fn test_kv_c1_merged() {
        let ir = vec![
            IrNode::Heading {
                level: 1,
                text: "S".to_string(),
            },
            IrNode::Kv {
                key: "A".to_string(),
                value: "1".to_string(),
            },
            IrNode::Kv {
                key: "B".to_string(),
                value: "2".to_string(),
            },
        ];
        let mut config = Config::default();
        config.compression = 1;
        let result = emit_llmd(&ir, &[], &config);
        assert!(result.contains(&":a=1 b=2".to_string()));
    }

    #[test]
    fn test_property_table() {
        let ir = vec![
            IrNode::Heading {
                level: 1,
                text: "S".to_string(),
            },
            IrNode::Table {
                rows: vec![
                    vec!["Name".to_string(), "Value".to_string()],
                    vec!["key1".to_string(), "val1".to_string()],
                    vec!["key2".to_string(), "val2".to_string()],
                ],
            },
        ];
        let config = Config::default();
        let result = emit_llmd(&ir, &[], &config);
        // "Value" is a generic header, should not emit :_col
        assert!(result.contains(&":key1=val1 key2=val2".to_string()));
    }

    #[test]
    fn test_block_ref_emission() {
        let ir = vec![
            IrNode::Heading {
                level: 1,
                text: "S".to_string(),
            },
            IrNode::BlockRef { index: 0 },
        ];
        let blocks = vec![CodeBlock {
            index: 0,
            lang: "json".to_string(),
            content: r#"{"key": "value"}"#.to_string(),
        }];
        let config = Config::default();
        let result = emit_llmd(&ir, &blocks, &config);
        assert!(result.contains(&"::json".to_string()));
        assert!(result.contains(&"<<<".to_string()));
        assert!(result.contains(&">>>".to_string()));
    }

    #[test]
    fn test_concat_scope_mode() {
        let ir = vec![
            IrNode::Heading {
                level: 1,
                text: "A".to_string(),
            },
            IrNode::Heading {
                level: 2,
                text: "B".to_string(),
            },
            IrNode::Paragraph {
                text: "text".to_string(),
            },
        ];
        let mut config = Config::default();
        config.scope_mode = ScopeMode::Concat;
        let result = emit_llmd(&ir, &[], &config);
        assert!(result.contains(&"@a_b".to_string()));
    }

    #[test]
    fn test_list_depth_prefixes() {
        let ir = vec![
            IrNode::Heading {
                level: 1,
                text: "S".to_string(),
            },
            IrNode::ListItem {
                depth: 0,
                text: "top".to_string(),
                ordered: false,
            },
            IrNode::ListItem {
                depth: 1,
                text: "nested".to_string(),
                ordered: false,
            },
        ];
        let config = Config::default();
        let result = emit_llmd(&ir, &[], &config);
        assert!(result.contains(&">top".to_string()));
        assert!(result.contains(&">. nested".to_string()));
    }

    #[test]
    fn test_find_common_prefix() {
        let keys = vec![
            "rate_limit-a".to_string(),
            "rate_limit-b".to_string(),
            "rate_limit-c".to_string(),
        ];
        assert_eq!(find_common_prefix(&keys), "rate_limit-");
    }

    #[test]
    fn test_classify_table_property() {
        let rows = vec![
            vec!["Name".into(), "Value".into()],
            vec!["key1".into(), "val1".into()],
        ];
        assert_eq!(classify_table(&rows), "property");
    }

    #[test]
    fn test_classify_table_keyed_multi() {
        let rows = vec![
            vec!["Name".into(), "Type".into(), "Desc".into()],
            vec!["key1".into(), "str".into(), "a desc".into()],
        ];
        assert_eq!(classify_table(&rows), "keyed_multi");
    }

    #[test]
    fn test_classify_table_raw() {
        let rows = vec![vec!["Only col".into()]];
        assert_eq!(classify_table(&rows), "raw");
    }
}
