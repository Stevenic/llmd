use crate::ir::{CodeBlock, Stage1Result};
use regex::Regex;
use std::sync::LazyLock;

static RE_FENCE_OPEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(`{3,})([a-zA-Z0-9_]*)\s*$").unwrap());

pub fn stage1(lines: &[String]) -> Stage1Result {
    let mut blocks: Vec<CodeBlock> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    let mut in_block = false;
    let mut lang = String::new();
    let mut buf: Vec<String> = Vec::new();
    let mut fence = String::new();

    for line in lines {
        if !in_block {
            if let Some(caps) = RE_FENCE_OPEN.captures(line) {
                in_block = true;
                fence = caps[1].to_string();
                lang = caps.get(2).map_or("", |m| m.as_str()).to_string();
                buf.clear();
                continue;
            }
            out.push(line.clone());
        } else if line.trim_end() == fence {
            let idx = blocks.len();
            blocks.push(CodeBlock {
                index: idx,
                lang: lang.clone(),
                content: buf.join("\n"),
            });
            out.push(format!("\u{27E6}BLOCK:{}\u{27E7}", idx));
            in_block = false;
            fence.clear();
            lang.clear();
            buf.clear();
        } else {
            buf.push(line.clone());
        }
    }

    // Handle unclosed block
    if in_block && !buf.is_empty() {
        let idx = blocks.len();
        blocks.push(CodeBlock {
            index: idx,
            lang: lang.clone(),
            content: buf.join("\n"),
        });
        out.push(format!("\u{27E6}BLOCK:{}\u{27E7}", idx));
    }

    Stage1Result { lines: out, blocks }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn test_simple_block() {
        let lines = s(&["before", "```js", "code here", "```", "after"]);
        let result = stage1(&lines);
        assert_eq!(result.lines, vec!["before", "\u{27E6}BLOCK:0\u{27E7}", "after"]);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.blocks[0].lang, "js");
        assert_eq!(result.blocks[0].content, "code here");
    }

    #[test]
    fn test_multiple_blocks() {
        let lines = s(&["```py", "x=1", "```", "text", "```", "y=2", "```"]);
        let result = stage1(&lines);
        assert_eq!(result.blocks.len(), 2);
        assert_eq!(result.blocks[0].lang, "py");
        assert_eq!(result.blocks[0].content, "x=1");
        assert_eq!(result.blocks[1].lang, "");
        assert_eq!(result.blocks[1].content, "y=2");
    }

    #[test]
    fn test_unclosed_block() {
        let lines = s(&["```js", "code", "more code"]);
        let result = stage1(&lines);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.blocks[0].content, "code\nmore code");
    }

    #[test]
    fn test_fence_length_matching() {
        let lines = s(&["````", "```", "inner", "```", "````"]);
        let result = stage1(&lines);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.blocks[0].content, "```\ninner\n```");
    }

    #[test]
    fn test_block_with_language() {
        let lines = s(&["```json", r#"{"key": "value"}"#, "```"]);
        let result = stage1(&lines);
        assert_eq!(result.blocks[0].lang, "json");
    }
}
