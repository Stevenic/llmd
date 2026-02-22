# LLMD Tools — Rust

## Setup

Requires Rust 1.80+ (for `LazyLock`). No external runtime dependencies — compiles to a single static binary.

```bash
cd tools/rust
cargo build --release
```

The binary will be at `target/release/llmdc` (or `llmdc.exe` on Windows).

## llmdc — LLMD Compiler

Compiles Markdown files into LLMD format with configurable compression levels (c0–c2).

```bash
# Basic usage (defaults to c2 compression)
cargo run -- input.md

# Specify compression level and output file
cargo run -- input.md -c 2 -o output.llmd

# Compile a directory of markdown files
cargo run -- docs/ -c 2 -o combined.llmd

# Use concat scope mode (API > Auth becomes @API_Auth)
cargo run -- input.md -c 2 --scope-mode concat

# Split sentences into separate > lines
cargo run -- input.md -c 2 --sentence-split

# Keep URLs in output at c2+
cargo run -- input.md -c 2 --keep-urls

# Re-emit @scope every 30 lines for chunk safety
cargo run -- input.md -c 2 --anchor-every 30

# Use a custom config file
cargo run -- input.md --config my-config.json

# Show all options
cargo run -- --help
```

**Compression levels:**

| Level | Description |
|-------|-------------|
| c0 | Structural normalize — clean whitespace, preserve wording |
| c1 | Compact structure — merge attributes, collapse blanks |
| c2 | Token compaction — stopwords, phrase map, unit normalization, boolean compression |

## Testing

```bash
cargo test          # Run all unit + integration tests
cargo clippy        # Lint check
```

## Parity

This implementation produces byte-identical output to the JS and Python implementations for all inputs and config combinations.
