use crate::ir::IrNode;
use regex::Regex;
use std::sync::LazyLock;

static RE_THEMATIC_BREAK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[-*_]{3,}$").unwrap());

static RE_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6})\s+(.+)$").unwrap());
static RE_UL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)([-*+])\s+(.+)$").unwrap());
static RE_OL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)(\d+)\.\s+(.+)$").unwrap());
static RE_BLOCK_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\u{27E6}BLOCK:(\d+)\u{27E7}$").unwrap());
static RE_KV: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Za-z][A-Za-z0-9 _-]{0,63})\s*:\s+(.+)$").unwrap());
static RE_TABLE_DELIM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\|?[\s:-]+\|").unwrap());

fn is_structural(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return true;
    }
    if RE_HEADING.is_match(t) {
        return true;
    }
    if RE_UL.is_match(t) || RE_OL.is_match(t) {
        return true;
    }
    if RE_BLOCK_REF.is_match(t) {
        return true;
    }
    if t.contains('|') {
        return true;
    }
    if RE_KV.is_match(t) && !t.starts_with("http://") && !t.starts_with("https://") {
        return true;
    }
    false
}

fn parse_table_row(row: &str) -> Vec<String> {
    let mut cells: Vec<String> = row.split('|').map(|c| c.trim().to_string()).collect();
    if !cells.is_empty() && cells[0].is_empty() {
        cells.remove(0);
    }
    if !cells.is_empty() && cells.last().is_some_and(|c| c.is_empty()) {
        cells.pop();
    }
    cells
}

pub fn stage2(lines: &[String]) -> Vec<IrNode> {
    let mut ir: Vec<IrNode> = Vec::new();
    let mut i = 0;
    let n = lines.len();

    while i < n {
        let line = &lines[i];
        let t = line.trim();

        if t.is_empty() {
            ir.push(IrNode::Blank);
            i += 1;
            continue;
        }

        // Skip thematic breaks (---, ***, ___)
        if RE_THEMATIC_BREAK.is_match(t) {
            i += 1;
            continue;
        }

        if let Some(caps) = RE_BLOCK_REF.captures(t) {
            let index: usize = caps[1].parse().unwrap();
            ir.push(IrNode::BlockRef { index });
            i += 1;
            continue;
        }

        if let Some(caps) = RE_HEADING.captures(t) {
            let level = caps[1].len();
            let text = caps[2].trim().to_string();
            ir.push(IrNode::Heading { level, text });
            i += 1;
            continue;
        }

        // Table detection: line with |, next line is delimiter
        if t.contains('|') && i + 1 < n {
            let next = lines[i + 1].trim();
            if RE_TABLE_DELIM.is_match(next) && next.contains("---") {
                let mut rows = vec![parse_table_row(t)];
                i += 2; // skip header + delimiter
                while i < n && lines[i].contains('|') && !lines[i].trim().is_empty() {
                    rows.push(parse_table_row(lines[i].trim()));
                    i += 1;
                }
                ir.push(IrNode::Table { rows });
                continue;
            }
        }

        if let Some(caps) = RE_UL.captures(line) {
            let depth = caps[1].len() / 2;
            let text = caps[3].trim().to_string();
            ir.push(IrNode::ListItem {
                depth,
                text,
                ordered: false,
            });
            i += 1;
            continue;
        }

        if let Some(caps) = RE_OL.captures(line) {
            let depth = caps[1].len() / 2;
            let text = caps[3].trim().to_string();
            ir.push(IrNode::ListItem {
                depth,
                text,
                ordered: true,
            });
            i += 1;
            continue;
        }

        if let Some(caps) = RE_KV.captures(t) {
            if !t.starts_with("http://") && !t.starts_with("https://") {
                let key = caps[1].to_string();
                let value = caps[2].trim().to_string();
                ir.push(IrNode::Kv { key, value });
                i += 1;
                continue;
            }
        }

        // Paragraph: merge consecutive non-structural lines
        let mut para_lines = vec![t.to_string()];
        i += 1;
        while i < n {
            let nl = lines[i].trim();
            if nl.is_empty() || is_structural(&lines[i]) {
                break;
            }
            para_lines.push(nl.to_string());
            i += 1;
        }
        ir.push(IrNode::Paragraph {
            text: para_lines.join(" "),
        });
    }
    ir
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn test_heading() {
        let ir = stage2(&s(&["# Title"]));
        match &ir[0] {
            IrNode::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Title");
            }
            _ => panic!("expected heading"),
        }
    }

    #[test]
    fn test_paragraph_merging() {
        let ir = stage2(&s(&["line one", "line two", "", "line three"]));
        match &ir[0] {
            IrNode::Paragraph { text } => assert_eq!(text, "line one line two"),
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn test_unordered_list() {
        let ir = stage2(&s(&["- item one", "  - nested"]));
        match &ir[0] {
            IrNode::ListItem {
                depth,
                text,
                ordered,
            } => {
                assert_eq!(*depth, 0);
                assert_eq!(text, "item one");
                assert!(!ordered);
            }
            _ => panic!("expected list item"),
        }
        match &ir[1] {
            IrNode::ListItem { depth, .. } => assert_eq!(*depth, 1),
            _ => panic!("expected nested list item"),
        }
    }

    #[test]
    fn test_ordered_list() {
        let ir = stage2(&s(&["1. first", "2. second"]));
        match &ir[0] {
            IrNode::ListItem { ordered, .. } => assert!(ordered),
            _ => panic!("expected ordered list item"),
        }
    }

    #[test]
    fn test_kv_pair() {
        let ir = stage2(&s(&["Key: value"]));
        match &ir[0] {
            IrNode::Kv { key, value } => {
                assert_eq!(key, "Key");
                assert_eq!(value, "value");
            }
            _ => panic!("expected kv"),
        }
    }

    #[test]
    fn test_url_not_kv() {
        let ir = stage2(&s(&["https://example.com: not a kv"]));
        match &ir[0] {
            IrNode::Paragraph { .. } => {}
            _ => panic!("URL line should be paragraph, not KV"),
        }
    }

    #[test]
    fn test_table() {
        let ir = stage2(&s(&[
            "| Name | Value |",
            "| --- | --- |",
            "| a | 1 |",
            "| b | 2 |",
        ]));
        match &ir[0] {
            IrNode::Table { rows } => {
                assert_eq!(rows.len(), 3);
                assert_eq!(rows[0], vec!["Name", "Value"]);
                assert_eq!(rows[1], vec!["a", "1"]);
            }
            _ => panic!("expected table"),
        }
    }

    #[test]
    fn test_block_ref() {
        let ir = stage2(&s(&["\u{27E6}BLOCK:0\u{27E7}"]));
        match &ir[0] {
            IrNode::BlockRef { index } => assert_eq!(*index, 0),
            _ => panic!("expected block ref"),
        }
    }

    #[test]
    fn test_blank() {
        let ir = stage2(&s(&[""]));
        match &ir[0] {
            IrNode::Blank => {}
            _ => panic!("expected blank"),
        }
    }
}
