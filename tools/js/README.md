# LLMD Tools — JavaScript

## Setup

```bash
cd tools/js
npm install
```

Requires Node.js v18+.

## Tools

### llmdc — LLMD Compiler

Compiles Markdown files into LLMD format with configurable compression levels (c0–c3).

```bash
# Basic usage (defaults to c2 compression)
node llmdc.js input.md

# Specify compression level and output file
node llmdc.js input.md -c 2 -o output.llmd

# Compile a directory of markdown files
node llmdc.js docs/ -c 2 -o combined.llmd

# Full compression with dictionary
node llmdc.js input.md -c 3 --dict ../../dict/llmd-core.dict.json -o output.llmd

# Stack multiple dictionaries (later overrides earlier)
node llmdc.js input.md -c 3 \
  --dict ../../dict/llmd-core.dict.json \
  --dict ../../dict/llmd-auto.dict.json \
  -o output.llmd

# Use concat scope mode (API > Auth becomes @API_Auth)
node llmdc.js input.md -c 2 --scope-mode concat

# Split sentences into separate > lines
node llmdc.js input.md -c 2 --sentence-split

# Keep URLs in output at c2+
node llmdc.js input.md -c 2 --keep-urls

# Re-emit @scope every 30 lines for chunk safety
node llmdc.js input.md -c 2 --anchor-every 30

# Use a custom config file
node llmdc.js input.md --config my-config.json

# Show all options
node llmdc.js --help
```

**Compression levels:**

| Level | Description |
|-------|-------------|
| c0 | Structural normalize — clean whitespace, preserve wording |
| c1 | Compact structure — merge attributes, collapse blanks |
| c2 | Token compaction — stopwords, phrase map, unit normalization |
| c3 | Symbolic compression — apply DCS dictionary mappings |

### validate-dict — Dictionary Validator

Validates a DCS dictionary file against the JSON schema.

```bash
node validate-dict.js ../../dict/llmd-core.dict.json
```

### dcs_auto — Automatic Dictionary Generator

Generates a DCS dictionary from a corpus of documents using frequency analysis. No LLM calls required.

```bash
# Generate from a single file
node dcs_auto.js ../../config/auto_config.json output.dict.json input.md

# Generate from a directory
node dcs_auto.js ../../config/auto_config.json output.dict.json docs/

# Generate with a base dictionary to avoid alias collisions
node dcs_auto.js ../../config/auto_config.json output.dict.json docs/ \
  --base ../../dict/llmd-core.dict.json
```

### bench — Benchmark Harness

Measures token reduction from dictionary application.

```bash
node bench.js ../../config/auto_config.json ../../dict/llmd-core.dict.json input.md

# Benchmark a directory
node bench.js ../../config/auto_config.json ../../dict/llmd-core.dict.json docs/
```

Example output:

```
Files: 3
Est tokens BEFORE: 1240
Est tokens AFTER : 520
Saved: 720 (58.1% reduction, final size 41.9%)
```

## npm Scripts

Alternatively, use the shorthand scripts defined in `package.json`:

```bash
npm run llmdc -- input.md -c 2 -o output.llmd
npm run validate-dict -- ../../dict/llmd-core.dict.json
npm run dcs-auto -- ../../config/auto_config.json output.dict.json docs/
npm run bench -- ../../config/auto_config.json ../../dict/llmd-core.dict.json docs/
```
