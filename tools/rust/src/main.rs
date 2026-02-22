use clap::Parser;
use llmdc::config::Config;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "llmdc", about = "LLMD Compiler — compile Markdown to LLMD format")]
struct Cli {
    /// Input file(s) or directory
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Compression level (0-2, default: from config or 2)
    #[arg(short, long)]
    compression: Option<i32>,

    /// Scope mode: flat, concat, stacked (default: flat)
    #[arg(long)]
    scope_mode: Option<String>,

    /// Keep URLs at c2+
    #[arg(long)]
    keep_urls: bool,

    /// Split sentences into separate > lines at c2+
    #[arg(long)]
    sentence_split: bool,

    /// Re-emit @scope every N lines (default: 0 = off)
    #[arg(long)]
    anchor_every: Option<usize>,

    /// Config file path
    #[arg(long)]
    config: Option<PathBuf>,
}

fn die(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    process::exit(1);
}

fn load_config(path: &PathBuf) -> Config {
    let text = fs::read_to_string(path).unwrap_or_else(|e| die(&format!("cannot read config: {}", e)));
    serde_json::from_str(&text).unwrap_or_else(|e| die(&format!("invalid config JSON: {}", e)))
}

fn main() {
    let cli = Cli::parse();

    // Load config
    let mut config = if let Some(ref config_path) = cli.config {
        load_config(config_path)
    } else {
        let defaults = ["llmdc.config.json", "config/llmdc.config.json"];
        let mut loaded = None;
        for p in &defaults {
            let path = PathBuf::from(p);
            if path.is_file() {
                loaded = Some(load_config(&path));
                break;
            }
        }
        loaded.unwrap_or_default()
    };

    // CLI overrides
    if let Some(c) = cli.compression {
        config.compression = c;
    }
    if let Some(ref mode) = cli.scope_mode {
        config.scope_mode = match mode.as_str() {
            "flat" => llmdc::config::ScopeMode::Flat,
            "concat" => llmdc::config::ScopeMode::Concat,
            "stacked" => llmdc::config::ScopeMode::Stacked,
            _ => die(&format!("invalid scope mode: {}", mode)),
        };
    }
    if cli.keep_urls {
        config.keep_urls = true;
    }
    if cli.sentence_split {
        config.sentence_split = true;
    }
    if let Some(n) = cli.anchor_every {
        config.anchor_every = n;
    }

    // Collect input files
    let files = llmdc::list_files(&cli.inputs).unwrap_or_else(|e| die(&format!("{}", e)));
    if files.is_empty() {
        die("no input files found");
    }

    // Compile all files
    let mut all_text = String::new();
    for fp in &files {
        if !all_text.is_empty() {
            all_text.push('\n');
        }
        let content =
            fs::read_to_string(fp).unwrap_or_else(|e| die(&format!("cannot read {}: {}", fp.display(), e)));
        all_text.push_str(&content);
    }

    let result = llmdc::compile(&all_text, &config);

    if let Some(ref output_path) = cli.output {
        fs::write(output_path, &result)
            .unwrap_or_else(|e| die(&format!("cannot write {}: {}", output_path.display(), e)));
        let tokens: usize = result.split_whitespace().filter(|t| !t.is_empty()).count();
        eprintln!(
            "compiled {} file(s) -> {} (c{}, ~{} tokens)",
            files.len(),
            output_path.display(),
            config.compression,
            tokens
        );
    } else {
        print!("{}", result);
    }
}
