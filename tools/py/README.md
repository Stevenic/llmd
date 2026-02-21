# LLMD Tools — Python

## Setup

```bash
cd tools/py
pip install -r requirements.txt
```

Requires Python 3.10+.

## Tools

### llmdc.py — LLMD Compiler

Compiles Markdown files into LLMD format with configurable compression levels (c0–c3).

```bash
# Basic usage (defaults to c2 compression)
python llmdc.py input.md

# Specify compression level and output file
python llmdc.py input.md -c 2 -o output.llmd

# Compile a directory of markdown files
python llmdc.py docs/ -c 2 -o combined.llmd

# Full compression with dictionary
python llmdc.py input.md -c 3 --dict ../../dict/llmd-core.dict.json -o output.llmd

# Stack multiple dictionaries (later overrides earlier)
python llmdc.py input.md -c 3 \
  --dict ../../dict/llmd-core.dict.json \
  --dict ../../dict/llmd-auto.dict.json \
  -o output.llmd

# Use concat scope mode (API > Auth becomes @API_Auth)
python llmdc.py input.md -c 2 --scope-mode concat

# Split sentences into separate > lines
python llmdc.py input.md -c 2 --sentence-split

# Keep URLs in output at c2+
python llmdc.py input.md -c 2 --keep-urls

# Re-emit @scope every 30 lines for chunk safety
python llmdc.py input.md -c 2 --anchor-every 30

# Use a custom config file
python llmdc.py input.md --config my-config.json

# Show all options
python llmdc.py --help
```

**Compression levels:**

| Level | Description |
|-------|-------------|
| c0 | Structural normalize — clean whitespace, preserve wording |
| c1 | Compact structure — merge attributes, collapse blanks |
| c2 | Token compaction — stopwords, phrase map, unit normalization |
| c3 | Symbolic compression — apply DCS dictionary mappings |

### validate_dict.py — Dictionary Validator

Validates a DCS dictionary file against the JSON schema.

```bash
python validate_dict.py ../../dict/llmd-core.dict.json
```

### dcs_auto.py — Automatic Dictionary Generator

Generates a DCS dictionary from a corpus of documents using frequency analysis. No LLM calls required.

```bash
# Generate from a single file
python dcs_auto.py ../../config/auto_config.json output.dict.json input.md

# Generate from a directory
python dcs_auto.py ../../config/auto_config.json output.dict.json docs/

# Generate with a base dictionary to avoid alias collisions
python dcs_auto.py ../../config/auto_config.json output.dict.json docs/ \
  --base ../../dict/llmd-core.dict.json
```

### bench.py — Benchmark Harness

Measures token reduction from dictionary application.

```bash
python bench.py ../../config/auto_config.json ../../dict/llmd-core.dict.json input.md

# Benchmark a directory
python bench.py ../../config/auto_config.json ../../dict/llmd-core.dict.json docs/
```

Example output:

```
Files: 3
Est tokens BEFORE: 1240
Est tokens AFTER : 520
Saved: 720 (58.1% reduction, final size 41.9%)
```

## Virtual Environment (recommended)

```bash
cd tools/py
python -m venv .venv
source .venv/bin/activate   # Linux/macOS
.venv\Scripts\activate      # Windows
pip install -r requirements.txt
```
