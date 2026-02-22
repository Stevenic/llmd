# LLMD Architecture

LLMD (LLM-optimized Deterministic format) is a deterministic compiler system that converts Markdown into a compact, token-efficient format optimized for LLM consumption while preserving semantic meaning.

**Author:** Steven Ickman | **License:** MIT

---

## Core Concepts

LLMD replaces verbose hierarchical Markdown with implicit scoping. Instead of repeating full paths, a single `@scope` declaration sets context for all subsequent lines until the next scope change.

### Line Types

| Prefix | Name      | Purpose                                      |
|--------|-----------|----------------------------------------------|
| `~`    | Metadata  | Optional file-level metadata (one per file)  |
| `@`    | Scope     | Sets implicit context for following lines    |
| `:`    | Attribute | Scoped key-value pairs (`:k=v`)              |
| `>`    | Item      | Unstructured content under current scope     |
| `->`   | Relation  | Declares dependencies from current scope     |
| `::`   | Block     | Preserves raw code/JSON between `<<<`/`>>>`  |

### Compression Levels

| Level | Name                  | Description                                          |
|-------|-----------------------|------------------------------------------------------|
| c0    | Structural normalize  | Clean whitespace, normalize structure                |
| c1    | Compact structure     | Lists to `>`, `Key: Value` to `:k=v`, collapse space |
| c2    | Token compaction      | Stopword removal, phrase normalization, unit simplify |
---

## System Components

```
┌─────────────────────────────────────────┐
│              LLMD System                │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │  Compiler (6 stages)             │   │
│  │  Markdown → LLMD                 │   │
│  └──────────────┬───────────────────┘   │
│                 │                        │
│                 ▼                        │
│  ┌──────────────────────────────────┐   │
│  │  Config                          │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

---

## Compiler Pipeline

The compiler transforms Markdown to LLMD through six sequential stages:

```
Markdown Input
    │
    ▼
Stage 0: Normalize ──────── UTF-8, NFKC unicode, line endings
    │
    ▼
Stage 1: Extract Blocks ─── Protect code blocks with placeholders
    │
    ▼
Stage 2: Parse to IR ────── Headings, paragraphs, lists, tables, KV pairs
    │
    ▼
Stage 3: Resolve Scopes ─── Hierarchy handling (flat/concat/stacked modes)
    │
    ▼
Stage 4: Emit LLMD ──────── Generate @ : > -> :: lines (pre-compression)
                             ├── classifyTable(): property / keyed_multi / raw
                             ├── Column header emission (:_col, :_cols)
                             ├── Chunked KV emission (max_kv_per_line)
                             ├── Common prefix extraction (:_pfx)
                             └── Boolean/enum compression (c2+)
    │
    ▼
Stage 5: Compress ────────── Apply c0→c1→c2 passes
    │
    ▼
Stage 6: Post-process ───── Anchors, validation
    │
    ▼
LLMD Output
```

---

## Directory Structure

```
llmd/
├── LLMD Specification - v0.1.md       # Format spec (line types, scoping rules)
├── LLMD Compiler Design v0.1.md       # 6-stage compiler pipeline spec
├── README.md
├── LICENSE
│
├── config/
│   └── llmdc.config.json              # Compiler configuration
│
├── tools/
│   ├── js/                            # Node.js implementations
│   │   └── llmdc.js                   # Compiler
│   ├── py/                            # Python implementations
│   │   └── llmdc.py                   # Compiler
│   └── rust/                          # Rust implementation
│       └── src/                       # Compiler (single binary)
│
└── corpora/
    └── samples/                       # Sample documents for testing
```

---

## File Relationships

```
  ┌─────────────────────┐
  │  Specification Docs  │
  │  (Format, Compiler)  │
  └─────────┬───────────┘
            │ defines
            ▼
  ┌──────────────┐      ┌──────────────┐
  │  config/     │      │  corpora/    │
  │  llmdc.config│      │  samples/    │
  └──────┬───────┘      └──────┬───────┘
         │                      │
         ▼                      ▼
  ┌────────────────────────────────────────┐
  │  tools/ (js/ and py/)                  │
  │  llmdc  ◄── config + source docs       │
  └────────────────────────────────────────┘
```

---

## Determinism Guarantees

The entire system is designed to be 100% deterministic:

1. **File ordering**: Always sorted lexicographically
2. **Config-driven**: Stopwords, phrase maps are fixed JSON
3. **No randomness**: Output is purely a function of input + config

---

## Compression Example

**Input (Markdown, ~64 tokens):**
```markdown
## Authentication
The API supports authentication via OAuth2 and API keys.
- Use OAuth2 for user-facing apps.
- Use API keys for server-to-server.
Rate limit: 1000 requests per minute.
```

**Output (LLMD c2, ~35 tokens):**
```
@auth
:methods=oauth2|apikey rate=1000/m
>oauth2 user-app
>apikey svc-svc
```

### Table Compression Example

**Input (3-column CSS component table):**
```markdown
| Class | Child Class | Where |
|-------|-------------|-------|
| flm-button-label | Primary text | compound button |
| flm-button-description | Secondary text | compound button |
```

**Output (LLMD c2, keyed_multi classification):**
```
:_cols=class|child_class|where
:flm-button-label=Primary text|compound button
:flm-button-description=Secondary text|compound button
```

### Prefix Extraction Example

**Input (property table with shared prefix):**
```markdown
| Class | Effect |
|-------|--------|
| flm-text--secondary | Color: --bodySubtext |
| flm-text--disabled | Color: --disabledText |
| flm-text--error | Color: --errorText |
```

**Output (LLMD c2, with prefix extraction):**
```
:_pfx=flm-text--
:secondary=Color: --bodySubtext disabled=Color: --disabledText
:error=Color: --errorText
```

---

## Triple Implementation Strategy

The compiler is implemented in JavaScript (Node.js 18+), Python (3.10+), and Rust (1.80+). All three produce byte-identical output for the same input and config. The JS and Python implementations require no external dependencies. The Rust implementation compiles to a single static binary with no runtime dependencies.
