use llmdc::config::Config;
use std::fs;
use std::path::Path;

fn load_config() -> Config {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("config")
        .join("llmdc.config.json");
    let text = fs::read_to_string(&config_path)
        .unwrap_or_else(|e| panic!("cannot read config at {}: {}", config_path.display(), e));
    serde_json::from_str(&text).unwrap()
}

fn read_sample(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("corpora")
        .join("samples")
        .join(name);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {}", path.display(), e));
    // Normalize line endings for cross-platform comparison
    text.replace("\r\n", "\n")
}

#[test]
fn test_api_spec_parity() {
    let config = load_config();
    let input = read_sample("api-spec.md");
    let expected = read_sample("api-spec.llmd");
    let result = llmdc::compile(&input, &config);
    assert_eq!(result, expected);
}

#[test]
fn test_fluentlm_components_parity() {
    let config = load_config();
    let input = read_sample("fluentlm-components.md");
    let expected = read_sample("fluentlm-components.llmd");
    let result = llmdc::compile(&input, &config);
    assert_eq!(result, expected);
}
