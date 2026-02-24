//! schema2llmd — JSON Schema to LLMD converter (Rust)
//!
//! Converts JSON Schema definitions into compressed LLMD format
//! with llmdc config compression (stopwords, phrase_map, units).

use clap::Parser;
use llmdc::config::Config;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "schema2llmd", about = "JSON Schema to LLMD converter")]
struct Cli {
    /// Input JSON Schema file
    schema: PathBuf,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Config file path (auto-detect llmdc.config.json)
    #[arg(long)]
    config: Option<PathBuf>,
}

fn die(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    process::exit(1);
}

fn load_config(config_path: Option<&PathBuf>) -> Config {
    if let Some(p) = config_path {
        let text = fs::read_to_string(p)
            .unwrap_or_else(|e| die(&format!("cannot read config: {}", e)));
        return serde_json::from_str(&text)
            .unwrap_or_else(|e| die(&format!("invalid config JSON: {}", e)));
    }
    for p in &["llmdc.config.json", "config/llmdc.config.json"] {
        if let Ok(text) = fs::read_to_string(p) {
            if let Ok(config) = serde_json::from_str(&text) {
                return config;
            }
        }
    }
    Config::default()
}

// ---------------------------------------------------------------------------
// Schema context
// ---------------------------------------------------------------------------

struct SchemaCtx {
    root: Value,
}

impl SchemaCtx {
    fn new(root: Value) -> Self {
        Self { root }
    }

    fn definitions(&self) -> &serde_json::Map<String, Value> {
        static EMPTY: std::sync::LazyLock<serde_json::Map<String, Value>> =
            std::sync::LazyLock::new(serde_json::Map::new);
        self.root
            .get("definitions")
            .and_then(|v| v.as_object())
            .unwrap_or(&EMPTY)
    }

    fn resolve_ref<'a>(&'a self, ref_str: &str) -> Option<&'a Value> {
        let path = ref_str.trim_start_matches("#/");
        let mut obj = &self.root;
        for part in path.split('/') {
            obj = obj.get(part)?;
        }
        Some(obj)
    }

    fn collect_properties<'a>(
        &'a self,
        node: &'a Value,
        visited: &mut HashSet<String>,
    ) -> Vec<(String, &'a Value)> {
        if node.is_null() {
            return vec![];
        }
        if let Some(r) = node.get("$ref").and_then(|v| v.as_str()) {
            if visited.contains(r) {
                return vec![];
            }
            visited.insert(r.to_string());
            if let Some(resolved) = self.resolve_ref(r) {
                return self.collect_properties(resolved, visited);
            }
            return vec![];
        }
        let mut props: Vec<(String, &'a Value)> = vec![];
        if let Some(arr) = node.get("allOf").and_then(|v| v.as_array()) {
            for sub in arr {
                let mut v = visited.clone();
                props.extend(self.collect_properties(sub, &mut v));
            }
        }
        for key in &["anyOf", "oneOf"] {
            if let Some(arr) = node.get(*key).and_then(|v| v.as_array()) {
                for sub in arr {
                    let mut v = visited.clone();
                    props.extend(self.collect_properties(sub, &mut v));
                }
            }
        }
        if let Some(obj) = node.get("properties").and_then(|v| v.as_object()) {
            for (key, val) in obj {
                props.push((key.clone(), val));
            }
        }
        let mut seen: indexmap::IndexMap<String, &'a Value> = indexmap::IndexMap::new();
        for (k, v) in props {
            seen.insert(k, v);
        }
        seen.into_iter().collect()
    }

    fn get_required(&self, node: &Value) -> HashSet<String> {
        if node.is_null() {
            return HashSet::new();
        }
        let mut req = HashSet::new();
        if let Some(arr) = node.get("required").and_then(|v| v.as_array()) {
            for r in arr {
                if let Some(s) = r.as_str() {
                    req.insert(s.to_string());
                }
            }
        }
        if let Some(arr) = node.get("allOf").and_then(|v| v.as_array()) {
            for sub in arr {
                req.extend(self.get_required(sub));
            }
        }
        if let Some(r) = node.get("$ref").and_then(|v| v.as_str()) {
            if let Some(resolved) = self.resolve_ref(r) {
                req.extend(self.get_required(resolved));
            }
        }
        req
    }

    fn get_type(&self, prop: &Value) -> String {
        if prop.is_null() {
            return "any".into();
        }
        if prop.get("const").is_some() {
            return "string".into();
        }
        if prop.get("type").and_then(|v| v.as_str()) == Some("array") {
            if let Some(items) = prop.get("items") {
                let item_type = self.get_item_type_summary(items);
                return format!("array of {}", item_type);
            }
            return "array".into();
        }
        if let Some(t) = prop.get("type").and_then(|v| v.as_str()) {
            return t.to_string();
        }
        if let Some(r) = prop.get("$ref").and_then(|v| v.as_str()) {
            if let Some(resolved) = self.resolve_ref(r) {
                if let Some(t) = resolved.get("type").and_then(|v| v.as_str()) {
                    match t {
                        "string" | "number" | "boolean" | "object" => return t.to_string(),
                        _ => {}
                    }
                }
            }
            return "string".into();
        }
        let options = prop
            .get("oneOf")
            .or_else(|| prop.get("anyOf"))
            .and_then(|v| v.as_array());
        if let Some(opts) = options {
            let mut types = HashSet::new();
            for opt in opts {
                if let Some(t) = opt.get("type").and_then(|v| v.as_str()) {
                    types.insert(t.to_string());
                } else if let Some(r) = opt.get("$ref").and_then(|v| v.as_str()) {
                    if let Some(resolved) = self.resolve_ref(r) {
                        if let Some(t) = resolved.get("type").and_then(|v| v.as_str()) {
                            types.insert(t.to_string());
                        } else {
                            types.insert("object".to_string());
                        }
                    }
                }
            }
            if types.len() == 1 {
                return types.into_iter().next().unwrap();
            }
            let mut v: Vec<_> = types.into_iter().collect();
            v.sort();
            return v.join("¦");
        }
        "any".into()
    }

    fn get_item_type_summary(&self, items: &Value) -> String {
        if items.is_null() {
            return "any".into();
        }
        match items.get("type").and_then(|v| v.as_str()) {
            Some("string") => return "string".into(),
            Some("number") => return "number".into(),
            _ => {}
        }
        if let Some(r) = items.get("$ref").and_then(|v| v.as_str()) {
            let name = r.split('/').last().unwrap_or("any");
            return clean_def_name(name);
        }
        "any".into()
    }

    fn describe_property(&self, prop: &Value) -> String {
        if prop.is_null() {
            return String::new();
        }
        let mut desc = prop
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .replace('\n', " ");
        desc = collapse_whitespace(&desc);
        {
            use std::sync::LazyLock;
            static RE_LINK: LazyLock<regex::Regex> =
                LazyLock::new(|| regex::Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap());
            desc = RE_LINK.replace_all(&desc, "$1").to_string();
        }
        if desc.len() > 200 {
            desc = format!("{}...", &desc[..197]);
        }
        let mut parts = vec![];
        if !desc.is_empty() {
            parts.push(desc);
        }
        if let Some(def) = prop.get("default") {
            parts.push(format!("Default: {}.", serde_json::to_string(def).unwrap()));
        }
        let values = self.extract_allowed_values(prop);
        if !values.is_empty() {
            parts.push(format!("[{}]", values.join(", ")));
        }
        parts.join(" ")
    }

    fn extract_allowed_values(&self, prop: &Value) -> Vec<String> {
        if prop.is_null() {
            return vec![];
        }
        if let Some(c) = prop.get("const").and_then(|v| v.as_str()) {
            return vec![c.to_string()];
        }
        if let Some(p) = prop.get("pattern").and_then(|v| v.as_str()) {
            return extract_values_from_pattern(p);
        }
        if let Some(r) = prop.get("$ref").and_then(|v| v.as_str()) {
            if let Some(resolved) = self.resolve_ref(r) {
                if let Some(p) = resolved.get("pattern").and_then(|v| v.as_str()) {
                    return extract_values_from_pattern(p);
                }
                if resolved.get("oneOf").is_some() || resolved.get("anyOf").is_some() {
                    return self.extract_allowed_values(resolved);
                }
            }
        }
        let options = prop
            .get("oneOf")
            .or_else(|| prop.get("anyOf"))
            .and_then(|v| v.as_array());
        if let Some(opts) = options {
            let mut vals = vec![];
            for opt in opts {
                if let Some(c) = opt.get("const").and_then(|v| v.as_str()) {
                    vals.push(c.to_string());
                } else if let Some(r) = opt.get("$ref").and_then(|v| v.as_str()) {
                    if let Some(resolved) = self.resolve_ref(r) {
                        if let Some(c) = resolved.get("const").and_then(|v| v.as_str()) {
                            vals.push(c.to_string());
                        }
                    }
                }
                if let Some(p) = opt.get("pattern").and_then(|v| v.as_str()) {
                    vals.extend(extract_values_from_pattern(p));
                }
            }
            return vals;
        }
        vec![]
    }

    fn is_object_definition(&self, _def_name: &str, def_schema: &Value) -> bool {
        if def_schema.is_null() {
            return false;
        }
        let typ = def_schema.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let has_props = def_schema.get("properties").is_some();
        if (typ == "string" || typ == "number" || typ == "boolean") && !has_props {
            return false;
        }
        if !has_props && def_schema.get("allOf").is_none() && typ != "object" {
            let branches = def_schema
                .get("anyOf")
                .or_else(|| def_schema.get("oneOf"))
                .and_then(|v| v.as_array());
            if let Some(branches) = branches {
                let has_obj = branches.iter().any(|b| {
                    if let Some(r) = b.get("$ref").and_then(|v| v.as_str()) {
                        self.resolve_ref(r)
                            .and_then(|r| r.get("properties"))
                            .is_some()
                    } else {
                        b.get("properties").is_some()
                    }
                });
                if !has_obj {
                    return false;
                }
            } else {
                return false;
            }
        }
        has_props || typ == "object" || def_schema.get("allOf").is_some()
    }
}

// ---------------------------------------------------------------------------
// Standalone helpers
// ---------------------------------------------------------------------------

fn clean_def_name(name: &str) -> String {
    use std::sync::LazyLock;
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^\d+\.").unwrap());
    RE.replace(name, "").to_string()
}

fn collapse_whitespace(s: &str) -> String {
    use std::sync::LazyLock;
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\s+").unwrap());
    RE.replace_all(s, " ").trim().to_string()
}

fn extract_values_from_pattern(pattern: &str) -> Vec<String> {
    if pattern.is_empty() {
        return vec![];
    }
    let mut values = vec![];
    let mut p = pattern.to_string();
    if p.starts_with('^') {
        p = p[1..].to_string();
    }
    static RE_TRAIL: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"\$\s*$").unwrap());
    p = RE_TRAIL.replace(&p, "").to_string();
    if p.starts_with('(') && p.ends_with(')') {
        p = p[1..p.len() - 1].to_string();
    }
    let alternatives = split_top_level(&p, '|');
    use std::sync::LazyLock;
    static CHAR_PATTERN_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^(\[[A-Za-z0-9]\|[A-Za-z0-9]\])+(\d*)$").unwrap());
    static CHAR_GROUP_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"\[([A-Za-z0-9])\|[A-Za-z0-9]\]").unwrap());
    static TRAILING_DIGITS_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(\d+)$").unwrap());
    static SIMPLE_LITERAL_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^[A-Za-z0-9_:.*]+$").unwrap());
    let char_pattern_re = &*CHAR_PATTERN_RE;
    let char_group_re = &*CHAR_GROUP_RE;
    let trailing_digits_re = &*TRAILING_DIGITS_RE;
    let simple_literal_re = &*SIMPLE_LITERAL_RE;

    for alt in &alternatives {
        let mut cleaned = alt.trim().to_string();
        while cleaned.starts_with('(')
            && cleaned.ends_with(')')
            && is_balanced(&cleaned[1..cleaned.len() - 1])
        {
            cleaned = cleaned[1..cleaned.len() - 1].trim().to_string();
        }
        if char_pattern_re.is_match(&cleaned) {
            let mut val = String::new();
            for m in char_group_re.captures_iter(&cleaned) {
                val.push_str(&m[1]);
            }
            if let Some(caps) = trailing_digits_re.captures(&cleaned) {
                let brackets_end: usize = char_group_re
                    .find_iter(&cleaned)
                    .last()
                    .map(|m| m.end())
                    .unwrap_or(0);
                if caps.get(0).unwrap().start() >= brackets_end {
                    val.push_str(&caps[1]);
                }
            }
            if !val.is_empty() {
                values.push(val);
            }
        } else if simple_literal_re.is_match(&cleaned) {
            values.push(cleaned);
        }
    }
    values
}

fn split_top_level(s: &str, sep: char) -> Vec<String> {
    let mut parts = vec![];
    let mut depth: i32 = 0;
    let mut current = String::new();
    let mut in_bracket = false;
    for c in s.chars() {
        if c == '[' {
            in_bracket = true;
        }
        if c == ']' {
            in_bracket = false;
        }
        if c == '(' && !in_bracket {
            depth += 1;
        }
        if c == ')' && !in_bracket {
            depth -= 1;
        }
        if c == sep && depth == 0 && !in_bracket {
            parts.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn is_balanced(s: &str) -> bool {
    let mut depth: i32 = 0;
    for c in s.chars() {
        if c == '(' {
            depth += 1;
        }
        if c == ')' {
            depth -= 1;
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}

// ---------------------------------------------------------------------------
// Compression (pre-compiled regexes for performance)
// ---------------------------------------------------------------------------

struct Compressor {
    phrase_regexes: Vec<(regex::Regex, String)>,
    unit_num_regexes: Vec<(regex::Regex, String)>,
    unit_regexes: Vec<(regex::Regex, String)>,
    bool_compress: bool,
    stopwords: HashSet<String>,
    protect: HashSet<String>,
    re_alpha: regex::Regex,
    re_ws: regex::Regex,
}

impl Compressor {
    fn new(config: &Config) -> Self {
        let stopwords: HashSet<String> = config.stopwords.iter().map(|s| s.to_lowercase()).collect();
        let protect: HashSet<String> = config.protect_words.iter().map(|s| s.to_lowercase()).collect();

        // Pre-compile phrase map regexes (longest-first)
        let mut phrases: Vec<(&String, &String)> = config.phrase_map.iter().collect();
        phrases.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        let phrase_regexes: Vec<(regex::Regex, String)> = phrases
            .iter()
            .map(|(phrase, replacement)| {
                let re = regex::Regex::new(&format!("(?i){}", regex::escape(phrase))).unwrap();
                (re, replacement.to_string())
            })
            .collect();

        // Pre-compile unit regexes (longest-first)
        let mut unit_keys: Vec<(&String, &String)> = config.units.iter().collect();
        unit_keys.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        let unit_num_regexes: Vec<(regex::Regex, String)> = unit_keys
            .iter()
            .map(|(unit, abbrev)| {
                let re = regex::Regex::new(&format!(r"(?i)(\d+)\s+{}", regex::escape(unit))).unwrap();
                let replacement = format!("${{1}}{}", abbrev);
                (re, replacement)
            })
            .collect();
        let unit_regexes: Vec<(regex::Regex, String)> = unit_keys
            .iter()
            .map(|(unit, abbrev)| {
                let re = regex::Regex::new(&format!("(?i){}", regex::escape(unit))).unwrap();
                (re, abbrev.to_string())
            })
            .collect();

        Compressor {
            phrase_regexes,
            unit_num_regexes,
            unit_regexes,
            bool_compress: config.bool_compress,
            stopwords,
            protect,
            re_alpha: regex::Regex::new(r"[^a-z]").unwrap(),
            re_ws: regex::Regex::new(r"\s+").unwrap(),
        }
    }

    fn compress(&self, text: &str) -> String {
        let mut body = text.to_string();

        // Apply phrase map
        for (re, replacement) in &self.phrase_regexes {
            body = re.replace_all(&body, replacement.as_str()).to_string();
        }

        // Apply unit normalization
        for (re, replacement) in &self.unit_num_regexes {
            body = re.replace_all(&body, replacement.as_str()).to_string();
        }
        for (re, replacement) in &self.unit_regexes {
            body = re.replace_all(&body, replacement.as_str()).to_string();
        }

        // Boolean compression
        if self.bool_compress {
            let bool_map: HashMap<&str, &str> = [
                ("yes", "Y"), ("no", "N"),
                ("true", "T"), ("false", "F"),
                ("enabled", "Y"), ("disabled", "N"),
            ].into();
            let tokens: Vec<&str> = body.split_whitespace().collect();
            body = tokens
                .iter()
                .map(|t| {
                    let low = t.to_lowercase();
                    match bool_map.get(low.as_str()) {
                        Some(v) => v.to_string(),
                        None => t.to_string(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        }

        // Stopword removal (skip protected words)
        let tokens: Vec<&str> = body.split_whitespace().collect();
        let filtered: Vec<&str> = tokens
            .iter()
            .filter(|t| {
                let low = self.re_alpha.replace_all(&t.to_lowercase(), "").to_string();
                if low.is_empty() {
                    return true;
                }
                if self.protect.contains(&low) {
                    return true;
                }
                !self.stopwords.contains(&low)
            })
            .copied()
            .collect();
        body = filtered.join(" ");

        // Trailing period stripping
        if body.ends_with('.')
            && !body.ends_with("...")
            && !body.ends_with("e.g.")
            && !body.ends_with("i.e.")
            && !body.ends_with("etc.")
        {
            body = body[..body.len() - 1].to_string();
        }

        // Clean up extra whitespace
        body = self.re_ws.replace_all(&body, " ").trim().to_string();

        body
    }
}

// ---------------------------------------------------------------------------
// LLMD Emission
// ---------------------------------------------------------------------------

fn generate_llmd(ctx: &SchemaCtx, config: &Config) -> String {
    let compressor = Compressor::new(config);

    let mut object_defs: Vec<(String, &Value)> = vec![];
    for (name, def_schema) in ctx.definitions() {
        if ctx.is_object_definition(name, def_schema) {
            object_defs.push((name.clone(), def_schema));
        }
    }

    // Collect all unique properties across all objects
    let mut all_properties: HashMap<String, Vec<&Value>> = HashMap::new();
    let mut prop_order: Vec<String> = vec![];

    for (_, def_schema) in &object_defs {
        let mut visited = HashSet::new();
        let props = ctx.collect_properties(def_schema, &mut visited);
        for (prop_name, prop_schema) in &props {
            if !all_properties.contains_key(prop_name) {
                prop_order.push(prop_name.clone());
                all_properties.insert(prop_name.clone(), vec![]);
            }
            all_properties.get_mut(prop_name).unwrap().push(prop_schema);
        }
    }

    let mut lines: Vec<String> = vec![];

    // --- Objects section ---
    lines.push("@Objects.Properties".to_string());
    lines.push("Required properties marked with `!`.".to_string());

    for (def_name, def_schema) in &object_defs {
        let mut visited = HashSet::new();
        let props = ctx.collect_properties(def_schema, &mut visited);
        let required = ctx.get_required(def_schema);

        let prop_list: Vec<String> = props
            .iter()
            .map(|(p, _)| {
                if required.contains(p) {
                    format!("{}!", p)
                } else {
                    p.clone()
                }
            })
            .collect();

        lines.push(format!(
            ":{}.properties={}",
            clean_def_name(def_name),
            prop_list.join(", ")
        ));
    }

    // --- Properties section ---
    lines.push("@Properties".to_string());

    let mut documented: HashSet<String> = HashSet::new();
    for prop_name in &prop_order {
        if documented.contains(prop_name) {
            continue;
        }
        documented.insert(prop_name.clone());

        let schemas = &all_properties[prop_name];

        // Pick the richest schema (longest description)
        let mut best_schema = schemas[0];
        let mut best_desc = String::new();
        for s in schemas {
            let d = ctx.describe_property(s);
            if d.len() > best_desc.len() {
                best_desc = d;
                best_schema = s;
            }
        }

        let type_str = ctx.get_type(best_schema);
        let description = if best_desc.is_empty() {
            ctx.describe_property(best_schema)
        } else {
            best_desc
        };

        let description = compressor.compress(&description);

        lines.push(format!("-{} ({}): {}", prop_name, type_str, description));
    }

    lines.join("\n") + "\n"
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();

    let input_path = cli.schema.canonicalize().unwrap_or_else(|_| cli.schema.clone());
    let content = fs::read_to_string(&input_path)
        .unwrap_or_else(|e| die(&format!("cannot read schema: {}", e)));
    let root: Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| die(&format!("invalid JSON: {}", e)));

    let ctx = SchemaCtx::new(root);
    let config = load_config(cli.config.as_ref());
    let result = generate_llmd(&ctx, &config);

    if let Some(output_path) = &cli.output {
        fs::write(output_path, &result)
            .unwrap_or_else(|e| die(&format!("cannot write output: {}", e)));
        let tokens: usize = result.split_whitespace().count();
        eprintln!(
            "schema2llmd: {} -> {} (~{} tokens)",
            input_path.display(),
            output_path.display(),
            tokens
        );
    } else {
        print!("{}", result);
    }
}
