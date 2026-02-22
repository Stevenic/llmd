pub mod blocks;
pub mod compress;
pub mod config;
pub mod emit;
pub mod inline;
pub mod ir;
pub mod normalize;
pub mod parse;
pub mod postprocess;
pub mod scope;

use config::Config;
use std::io;
use std::path::PathBuf;

pub fn compile(text: &str, config: &Config) -> String {
    let compression = config.compression;

    // Stage 0
    let lines = normalize::stage0(text);

    // Stage 1
    let ir::Stage1Result {
        lines: clean_lines,
        blocks,
    } = blocks::stage1(&lines);

    // Stage 2
    let ir = parse::stage2(&clean_lines);

    // Stages 3+4
    let mut output = emit::emit_llmd(&ir, &blocks, config);

    // Stage 5
    if compression >= 0 {
        output = compress::compress_c0(&output);
    }
    if compression >= 1 {
        output = compress::compress_c1(&output);
    }
    if compression >= 2 {
        output = compress::compress_c2(&output, config);
    }

    // Stage 6
    output = postprocess::stage6(&output, config);

    let mut result = output.join("\n");
    result.push('\n');
    result
}

pub fn list_files(inputs: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    let re = regex::Regex::new(r"(?i)\.(md|markdown|llmd)$").unwrap();
    let mut out: Vec<PathBuf> = Vec::new();

    for p in inputs {
        if p.is_dir() {
            for entry in std::fs::read_dir(p)? {
                let entry = entry?;
                let sub_path = entry.path();
                if sub_path.is_dir() {
                    let sub_files = list_files(&[sub_path])?;
                    out.extend(sub_files);
                } else if sub_path.is_file() {
                    if let Some(path_str) = sub_path.to_str() {
                        if re.is_match(path_str) {
                            out.push(sub_path);
                        }
                    }
                }
            }
        } else if p.is_file() {
            if let Some(path_str) = p.to_str() {
                if re.is_match(path_str) {
                    out.push(p.clone());
                }
            }
        }
    }

    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_compile() {
        let input = "# Title\n\nSome text.\n";
        let config = Config::default();
        let result = compile(input, &config);
        assert!(result.contains("@title"));
        assert!(result.contains(">Some text."));
    }

    #[test]
    fn test_determinism() {
        let input = "# Title\n\nSome text.\n- item\n";
        let config = Config::default();
        let r1 = compile(input, &config);
        let r2 = compile(input, &config);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_compile_c0() {
        let input = "# Title\n\nKey: value\n";
        let mut config = Config::default();
        config.compression = 0;
        let result = compile(input, &config);
        assert!(result.contains("@Title"));
        assert!(result.contains(":key=value"));
    }

    #[test]
    fn test_compile_c2() {
        let input = "# Title\n\nKey: value\n";
        let config = Config::default();
        let result = compile(input, &config);
        assert!(result.contains("@title"));
    }
}
