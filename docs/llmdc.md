# llmdc — LLMD Compiler

Compiles Markdown files into the LLMD token-optimized format through a 6-stage pipeline.

Available as `node tools/js/llmdc.js`, `python tools/py/llmdc.py`, and `cargo run --bin llmdc`.

---

## Usage

```bash
# Basic compilation (defaults to c2)
llmdc input.md

# Compile to file at compression level 2
llmdc input.md -o output.llmd -c 2

# Compile a directory
llmdc docs/ -c 2 -o out.llmd

```

---

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `-o, --output <path>` | Output file (stdout if omitted) | stdout |
| `-c, --compression <0-2>` | Compression level | from config or `2` |
| `--scope-mode <mode>` | `flat`, `concat`, or `stacked` | `flat` |
| `--keep-urls` | Preserve URLs at c2+ | `false` |
| `--sentence-split` | Split sentences into separate text lines at c2+ | `false` |
| `--anchor-every <n>` | Re-emit `@scope` every N lines | `0` (off) |
| `--config <path>` | Config file path | auto-detect |
| `-h, --help` | Show help | |

---

## Config File

Auto-detected from `llmdc.config.json` or `config/llmdc.config.json`. CLI flags override config values.

See [`config/llmdc.config.json`](../config/llmdc.config.json) for the full default configuration.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `compression` | int | `2` | Compression level (0-2) |
| `scope_mode` | string | `"flat"` | Scope resolution mode |
| `keep_urls` | bool | `false` | Preserve URLs at c2+ |
| `sentence_split` | bool | `false` | Split sentences at c2+ |
| `anchor_every` | int | `0` | Scope anchor interval (0 = off) |
| `max_kv_per_line` | int | `4` | Max key-value pairs per `:` line |
| `prefix_extraction` | bool | `true` | Enable common prefix extraction |
| `min_prefix_len` | int | `6` | Minimum prefix length to extract |
| `min_prefix_pct` | float | `0.6` | Minimum % of keys sharing prefix |
| `bool_compress` | bool | `true` | Compress boolean values at c2+ |
| `stopwords` | string[] | see config | Words removed from text/list lines at c2+ |
| `protect_words` | string[] | see config | Words never removed |
| `phrase_map` | object | see config | Phrase replacements at c2+ |
| `units` | object | see config | Unit normalizations at c2+ |

---

## Pipeline

### Stage 0: Normalize
UTF-8 decode, NFKC unicode normalization, line ending normalization, trailing whitespace trim.

### Stage 1: Extract Blocks
Fenced code blocks replaced with `⟦BLOCK:n⟧` placeholders. Block content is preserved verbatim.

### Stage 2: Parse to IR
Lightweight state machine producing IR nodes: `Heading`, `Paragraph`, `ListItem`, `Table`, `KVLine`, `Blank`, `BlockRef`.

### Stage 3: Scope Resolution
Headings map to `@scope` declarations via `normScopeName()` (trim, spaces→`_`, lowercase at c2+, strip punctuation except `_` and `-`).

### Stage 4: Emit LLMD
Walk the IR and generate LLMD lines:

- **Headings** → `@scope`
- **Paragraphs** → plain text (no prefix; optionally sentence-split at c2+)
- **Lists** → `-item` with `.` depth prefixes
- **KV lines** → `:key=value` (buffered, chunked by `max_kv_per_line`)
- **Tables** → classified via `classifyTable()`:
  - **`property`** (2-col, unique identifier-like keys) → `:k=v` pairs, with optional `:_col=<header>`
  - **`keyed_multi`** (3+ col, unique identifier-like keys) → `:_cols=h1¦h2¦h3` then `:key=v1¦v2`
  - **`raw`** (everything else) → `:_cols=h1¦h2¦h3` then `c1¦c2¦c3` per row
- **Code blocks** → `::lang` + `<<<` content `>>>`

#### Key Normalization (`normKey`)
Lowercase, spaces→`_`, strip punctuation except `_` and `-`, trim leading/trailing `-`.

#### Common Prefix Extraction
When ≥60% of keys in a KV buffer share a prefix of ≥6 chars, emits `:_pfx=<prefix>` and strips the prefix from matching keys.

#### Boolean Compression (c2+)
Table columns where all values are boolean-like (`Yes/No`, `true/false`, `enabled/disabled`) are mapped to compact forms (`Y/N`, `T/F`).

### Stage 5: Compression Passes
Applied progressively, skipping block content:

| Level | Name | Transformations |
|-------|------|-----------------|
| c0 | Normalize | Whitespace normalize, blank line collapse |
| c1 | Compact | Merge consecutive `:k=v`, prefix extraction |
| c2 | Token compact | Stopword removal, phrase map, unit normalization, boolean compression |
### Stage 6: Post-process
Validation (no scoped lines before first `@`), optional scope anchors.

---

## Input Formats

- `.md`, `.markdown` — Markdown files
- `.llmd` — passthrough/normalize mode
- Directories — recursively scanned for matching files (sorted lexicographically)

## Output Format

`.llmd` text file. See the [LLMD Specification](../LLMD%20Specification%20-%20v0.2.md) for line type reference.

---

## Examples

### Input
```markdown
## Text Styles
| Class | Effect |
|-------|--------|
| flm-text--secondary | Color: --bodySubtext |
| flm-text--disabled | Color: --disabledText |
| flm-text--error | Color: --errorText |
```

### Output (c2)
```
@text_styles
:_col=effect
:_pfx=flm-text--
:secondary=Color: --bodySubtext disabled=Color: --disabledText error=Color: --errorText
```
