# LLMD Tools — Python

## Setup

No dependencies required. Requires Python 3.10+.

## llmdc.py — LLMD Compiler

Compiles Markdown files into LLMD format with configurable compression levels (c0–c2).

```bash
# Basic usage (defaults to c2 compression)
python llmdc.py input.md

# Specify compression level and output file
python llmdc.py input.md -c 2 -o output.llmd

# Compile a directory of markdown files
python llmdc.py docs/ -c 2 -o combined.llmd

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
| c2 | Token compaction — stopwords, phrase map, unit normalization, boolean compression |
