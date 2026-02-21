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
| c3    | Symbolic compression  | Apply DCS dictionary mappings                        |

---

## System Components

```
┌─────────────────────────────────────────────────────────┐
│                    LLMD System                          │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Compiler     │  │  DCS         │  │  DCS-AUTO    │  │
│  │  (6 stages)   │  │  (dictionary │  │  (dict       │  │
│  │  Markdown →   │  │  compression)│  │  generator)  │  │
│  │  LLMD         │  │              │  │              │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                 │                  │          │
│         ▼                 ▼                  ▼          │
│  ┌────────────────────────────────────────────────┐     │
│  │  Config / Schema / Dictionaries                │     │
│  └────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────┘
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
Stage 5: Compress ────────── Apply c0→c1→c2→c3 passes
    │
    ▼
Stage 6: Post-process ───── Anchors, validation
    │
    ▼
LLMD Output
```

---

## Dictionary Compression System (DCS)

DCS provides deterministic token reduction through whole-token replacements across five namespaces:

| Namespace | Applies To            | Example                    |
|-----------|-----------------------|----------------------------|
| `scope`   | `@Scope` labels       | `authentication` → `auth`  |
| `key`     | `:k=v` attribute keys | `methods` → `m`            |
| `value`   | Enum-like values      | `OAuth2` → `O2`            |
| `text`    | `>` item text         | `service` → `svc`          |
| `type`    | Block type labels     | `javascript` → `js`        |

### Safety Mechanisms

- **Token-mode matching**: Whole tokens only, never substrings
- **Longest-match wins**: Prevents ambiguous overlapping replacements
- **Protected words**: Negations (`no`, `not`, `never`), modals (`must`, `should`, `may`), numbers
- **Value eligibility**: Only enum-like values are replaced (no URLs, dates, free text)
- **Max passes**: Default 1 pass to prevent infinite replacement loops

---

## DCS-AUTO (Automatic Dictionary Generation)

Generates dictionaries from source corpora without LLM calls, using frequency analysis:

```
Source Documents
    │
    ▼
Canonicalize to LLMD c1
    │
    ▼
Extract Tokens by Namespace (@scope, :key, :value, >text)
    │
    ▼
Filter (min_len=6, min_freq=3, exclude stopwords/protected/numeric/URLs)
    │
    ▼
Score (gain_per_use × frequency - overhead)
    │
    ▼
Rank & Cap (max_entries=256, priority: key > scope > value > text)
    │
    ▼
Assign Aliases (deterministic base36: s0, s1, k0, k1, v0, t0, ...)
    │
    ▼
Output Dictionary (JSON)
```

---

## Directory Structure

```
llmd/
├── LLMD Specification - v0.1.md       # Format spec (line types, scoping rules)
├── LLMD Compiler Design v0.1.md       # 6-stage compiler pipeline spec
├── LLMD Dictionary Compression System (DCS) v1.0.md  # DCS spec
├── DCS-AUTO v0.1.md                   # Auto dictionary generation spec
├── README.md
├── LICENSE
│
├── config/
│   ├── llmdc.config.json              # Compiler configuration
│   └── auto_config.json               # DCS-AUTO generator settings
│
├── schema/
│   └── llmd-dcs-dictionary.schema.json  # JSON Schema (Draft 2020-12) for dicts
│
├── dict/
│   └── llmd-core.dict.json            # Hand-curated core dictionary
│
├── tools/
│   ├── js/                            # Node.js implementations
│   │   ├── validate-dict.js           # Dict validation (AJV)
│   │   ├── dcs_auto.js                # AUTO dictionary generator
│   │   └── bench.js                   # Token reduction benchmarks
│   └── py/                            # Python implementations
│       ├── validate_dict.py           # Dict validation (jsonschema)
│       ├── dcs_auto.py                # AUTO dictionary generator
│       └── bench.py                   # Token reduction benchmarks
│
└── corpora/
    └── samples/                       # Sample documents for testing
```

---

## File Relationships

```
                    ┌─────────────────────┐
                    │  Specification Docs  │
                    │  (Format, Compiler,  │
                    │   DCS, DCS-AUTO)     │
                    └─────────┬───────────┘
                              │ defines
                    ┌─────────▼───────────┐
                    │  schema/            │
                    │  dictionary.schema  │◄──── validates ────┐
                    └─────────────────────┘                    │
                                                               │
  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
  │  config/     │      │  dict/       │      │  corpora/    │
  │  auto_config │─────►│  core.dict   │◄─────│  samples/    │
  │  llmdc.config│      │              │      │              │
  └──────┬───────┘      └──────┬───────┘      └──────┬───────┘
         │                     │                      │
         │    ┌────────────────┼──────────────────────┘
         │    │                │
         ▼    ▼                ▼
  ┌────────────────────────────────────────┐
  │  tools/ (js/ and py/)                  │
  │  validate-dict  ◄── schema + dict      │
  │  dcs_auto       ◄── config + corpora   │
  │  bench          ◄── dict + corpora     │
  └────────────────────────────────────────┘
```

---

## Determinism Guarantees

The entire system is designed to be 100% deterministic:

1. **File ordering**: Always sorted lexicographically
2. **Token matching**: Longest-match rule with stable tie-breakers
3. **Alias generation**: Deterministic base36 with namespace prefixes
4. **Config-driven**: Stopwords, phrase maps, dictionaries are fixed JSON
5. **No randomness**: Output is purely a function of input + config

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

**Output (LLMD c3 with DCS, ~27 tokens):**
```
@auth
:m=O2|K rate=1000/m
>O2 usr apps
>K svc-svc
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

## Dual Implementation Strategy

All tooling is implemented in both JavaScript (Node.js) and Python for portability:

| Tool           | JS Dependency | Python Dependency |
|----------------|--------------|-------------------|
| validate-dict  | AJV          | jsonschema        |
| dcs_auto       | Node.js      | Python 3          |
| bench          | Node.js      | Python 3          |
