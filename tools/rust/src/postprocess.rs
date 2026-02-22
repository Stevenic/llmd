use crate::config::Config;

fn is_text_line(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    !line.starts_with('@')
        && !line.starts_with(':')
        && !line.starts_with('-')
        && !line.starts_with('~')
        && !line.starts_with("::")
        && !line.starts_with("<<<")
        && !line.starts_with(">>>")
        && !line.starts_with('\u{2192}')
        && !line.starts_with('\u{2190}')
        && !line.starts_with('=')
}

pub fn stage6(lines: &[String], config: &Config) -> Vec<String> {
    let anchor_every = config.anchor_every;

    // Validation
    let mut first_scope = false;
    let mut in_block = false;

    for (i, line) in lines.iter().enumerate() {
        if line == "<<<" {
            in_block = true;
            continue;
        }
        if line == ">>>" {
            in_block = false;
            continue;
        }
        if in_block {
            continue;
        }
        if line.starts_with('@') {
            first_scope = true;
            continue;
        }
        if line.starts_with('~') {
            continue;
        }
        if !first_scope
            && (line.starts_with(':')
                || line.starts_with('-')
                || line.starts_with('\u{2192}')
                || line.starts_with('\u{2190}')
                || line.starts_with('=')
                || is_text_line(line))
        {
            eprintln!(
                "validation warning: line {}: scoped line before first @scope",
                i + 1
            );
        }
    }

    // Anchors
    if anchor_every > 0 {
        let mut current_scope: Option<String> = None;
        let mut lines_since_anchor: usize = 0;
        let mut out: Vec<String> = Vec::new();

        for line in lines {
            if line.starts_with('@') {
                current_scope = Some(line.clone());
                lines_since_anchor = 0;
                out.push(line.clone());
                continue;
            }
            lines_since_anchor += 1;
            if lines_since_anchor >= anchor_every {
                if let Some(ref scope) = current_scope {
                    out.push(scope.clone());
                    lines_since_anchor = 0;
                }
            }
            out.push(line.clone());
        }
        return out;
    }

    lines.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_insertion() {
        let mut config = Config::default();
        config.anchor_every = 2;
        let lines = vec![
            "@scope".to_string(),
            "-line1".to_string(),
            "-line2".to_string(),
            "-line3".to_string(),
        ];
        let result = stage6(&lines, &config);
        assert_eq!(
            result,
            vec!["@scope", "-line1", "@scope", "-line2", "-line3"]
        );
    }

    #[test]
    fn test_no_anchors() {
        let mut config = Config::default();
        config.anchor_every = 0;
        let lines = vec!["@scope".to_string(), "-line1".to_string()];
        let result = stage6(&lines, &config);
        assert_eq!(result, vec!["@scope", "-line1"]);
    }
}
