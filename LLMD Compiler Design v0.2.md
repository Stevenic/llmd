# LLMD Compiler Design v0.2

## Goals

* Deterministic compilation: same input + config ⇒ same output
* Token-first output (scoped `@`, no repeated prefixes)
* Supports c0–c2 (structure → compaction)
* Fast, streaming-friendly, simple to implement in JS/Python/Rust
* Robust enough for "real markdown" (lists, headings, tables, code fences, links)

---

# CLI + Files

## CLI

`llmdc` (LLMD compiler)

Examples:

* `llmdc in.md -o out.llmd -c 2`
* `llmdc docs/ -c 2 -o out.llmd --concat`
## Inputs

* `.md`, `.markdown` (primary)
* `.llmd` passthrough/normalize mode (optional)

## Outputs

* `.llmd` text file

---

# Output format reminders (v0.2)

* `@Scope` sets scope
* `:k=v k=v` attributes under current scope
* `plain text` paragraphs under current scope (no prefix)
* `-item` list items under current scope
* `→Scope` / `←Scope` relations from current scope (optional)
* `::type ... <<< >>>` blocks (optional)

---

# Compiler Architecture (Pipeline)

### Stage 0: Read + Normalize

* UTF-8 decode
* Normalize line endings to `\n`
* Trim trailing spaces
* Normalize Unicode (NFKC recommended)
* Deterministic file ordering if compiling a directory

### Stage 1: Protect / Extract Blocks

Identify and extract:

* fenced code blocks `lang … `
* optionally HTML blocks or indented code blocks

Replace each extracted block with a placeholder line:

* `⟦BLOCK:n⟧`

Store block metadata: `{index, lang, content}`.

> Why: prevents stopword removal/compression passes from touching code.

### Stage 2: Markdown Structure Pass (light parse)

A lightweight state machine (no full AST required) that emits an **intermediate IR**:

#### IR node types

* `Heading(level, text)`
* `Paragraph(text)` (one or more lines merged)
* `ListItem(depth, text, ordered?)`
* `Table(rows)` (optional best-effort)
* `KVLine(key, value)` (extracted from `Key: Value`)
* `Blank`
* `BlockRef(index)` (placeholder)

Thematic breaks (`---`, `***`, `___`) are detected and skipped — they produce no IR node.

This pass should be conservative and deterministic.

**Key heuristics**

* Headings: `^(#{1,6})\s+(.+)$`
* List items:

  * unordered: `^(\s*)([-*+])\s+(.+)$`
  * ordered: `^(\s*)(\d+\.)\s+(.+)$`
* Paragraphs: consecutive non-structural lines merged until blank or next structure token
* Tables: detect pipe tables only if:

  * header row contains `|`
  * next row is delimiter with `---` segments

### Stage 3: Scope Resolution

Maintain a scope stack based on headings.

#### Scope naming

* Normalize heading text → scope token:

  * trim
  * spaces → `_`
  * lowercase at c2+
  * strip punctuation except `_` and `-`

#### Key naming (`normKey`)

* trim, lowercase, spaces → `_`
* strip punctuation except `_` and `-` (preserves hyphenated identifiers like CSS class names)
* trim leading/trailing `-`

#### Scope model

* v0.2 is *flat* (hierarchy optional)
* But compiler needs a deterministic way to include hierarchy if desired.

Provide `--scope-mode`:

1. `flat` (default): each heading becomes a scope by itself
2. `concat`: `H1_H2_H3` concatenated
3. `stacked`: keep a short parent prefix only when needed (rare)

**Recommendation:** `flat` + (optional) prefixing in large corpora to reduce collisions.

### Stage 4: Emit LLMD Lines (pre-compression)

Walk the IR, emitting LLMD lines:

#### Headings

Emit `@scope` when scope changes.

#### Paragraphs

Emit plain text (no prefix). Sentence-split at c2+ if enabled.

#### Lists

Emit `-item` under current scope.
Nested list depth handling:

* **default:** prefix depth with dots after `-`: `-. child`, `-.. grandchild`

  * depth 0: `-item`
  * depth 1: `-. child`
  * depth 2: `-.. grandchild`

This keeps v0.2 scoped semantics without introducing hierarchy tokens.

#### Key: Value lines

Prefer attributes:

* `:key=value`
  At c1+ consecutive pairs merge into one attribute line, chunked by `max_kv_per_line` (default 4).

#### Common Prefix Extraction

When flushing the KV buffer at c1+, the compiler checks for a shared prefix among keys:

* If ≥60% of keys share a prefix of ≥6 characters (configurable via `min_prefix_pct` and `min_prefix_len`), emit `:_pfx=<prefix>` and strip the prefix from matching keys.
* Keys not sharing the prefix keep their full name.

Example: `flm-text--secondary`, `flm-text--disabled`, `flm-text--error` → `:_pfx=flm-text--` + `:secondary=... disabled=... error=...`

#### Tables

The compiler classifies each table using `classifyTable()`, which returns one of three types:

* **`property`** — 2-column table with unique, identifier-like first column. Emit `:k=v` for each row. If the second column header is informative (not generic like "Value"/"Description"), emit `:_col=<header>` first.
* **`keyed_multi`** — 3+ column table with unique, identifier-like first column. Emit `:_cols=col1¦col2¦col3` header, then `:key=val1¦val2` for each row.
* **`raw`** — anything else (non-unique keys, prose-like first column, inconsistent column counts). Emit `:_cols=col1¦col2¦col3` header, then `c1¦c2¦c3` per row (plain text, no prefix).

A column is "identifier-like" when its values are unique across all data rows, start with a letter/dot/hyphen, and contain no more than 4 whitespace-delimited words.

At c2+, boolean/enum value compression applies to columns where every value is boolean-like (`Yes/No`, `true/false`, `enabled/disabled`), mapping them to compact forms (`Y/N`, `T/F`).

#### Links

* c0–c1: keep `text<url>` (or `text (url)`)
* c2+: keep `text` only unless `--keep-urls`

#### Block placeholders

Emit block header + raw content:

* `::code` or `::json` (type derived from fence lang if present)
* `<<<` + content + `>>>`

### Stage 5: Compression Passes (c0–c2)

Apply progressively, skipping block content.

#### c0: normalize only

* whitespace normalize
* strip residual horizontal rules
* no rewriting besides structure conversion

#### c1: structural compaction

* merge consecutive attribute pairs: `:k=v k=v` (chunked by `max_kv_per_line`, default 4)
* extract common key prefixes when threshold is met (`:_pfx=<prefix>`)
* collapse blank lines (ideally none except between scopes if desired)
* normalize list depth prefixes

#### c2: token compaction

Affects text lines (no prefix), `-` list lines, and sometimes `:` attribute values:

* Stopword removal in text and `-` lines (protected words preserved)
* Phrase map replacements
* Unit normalization (`requests per minute` → `/m`, `1000 requests per minute` → `1000/m` when pattern matches)
* Trailing period stripping: remove final `.` from text and `-` lines (but not `...`, `e.g.`, `i.e.`, `etc.`)
* Boolean/enum value compression: columns where all values are boolean-like (`Yes/No`, `true/false`, `enabled/disabled`) map to compact forms (`Y/N`, `T/F`)
* Sentence splitting (optional): 1 sentence → 1 text line if it decreases tokens

**Identifying text lines:** A line is a text line if it does NOT start with any known prefix (`@`, `:`, `-`, `~`, `::`, `<<<`, `>>>`, `→`, `←`, `=`).

**Conservative rule:** never remove `no/not/never/must/should/may` and never remove punctuation that flips meaning (like `?` for uncertainty if you use it).

### Stage 6: Post-processing

* Optionally insert **anchors** for chunk safety:

  * every N lines: re-emit current scope: `@scope`
  * `--anchor-every 30` default off
* Validate:

  * no scoped line before first `@`
  * blocks closed properly
  * lines start with valid prefixes

---

# Determinism Rules

The compiler MUST:

* sort directory inputs lexicographically
* never depend on hash iteration order (always sort)
* use stable tie-breakers in table/key inference
* use fixed configs for stopwords/phrase map

---

# Compiler Config (one JSON file)

`llmdc.config.json` sketch:

```json
{
  "compression": 2,
  "scope_mode": "flat",
  "keep_urls": false,
  "sentence_split": true,
  "anchor_every": 0,
  "max_kv_per_line": 4,
  "prefix_extraction": true,
  "min_prefix_len": 6,
  "min_prefix_pct": 0.6,
  "bool_compress": true,
  "stopwords": ["the","a","an","really","just","that","is","are","was","were","of","in","on","at","for","with","by","from","to"],
  "protect_words": ["no","not","never","must","should","may"],
  "phrase_map": {
    "in order to": "to",
    "as well as": "¦",
    "due to": "because",
    "is able to": "can",
    "is used to": "",
    "is responsible for": "handles",
    "refers to": "="
  },
  "units": {
    "requests per minute": "/m",
    "milliseconds": "ms",
    "seconds": "s"
  }
}
```

CLI flags override config.

---

---

# Edge Cases + Policies

## Headings missing

If no heading appears before content:

* compiler injects `@root` at first output line

## Long paragraphs

At c2+:

* optionally split sentences if it reduces tokens
* otherwise keep as a single text line

## Tables that don't parse

Fallback to plain text line per row.

## HTML + weird markdown

Treat as paragraph text unless recognized structure.

## Code fences

Always preserved exactly (or minified only if explicitly enabled for json/yaml at c2+)

---

# Minimal IR → LLMD example (how the compiler "thinks")

Markdown:

```
## Authentication
The API supports OAuth2 and API keys.
- Use OAuth2 for user apps
- Use API keys for server-to-server
Rate limit: 1000 requests per minute
```

IR:

* Heading(2,"Authentication")
* Paragraph("The API supports OAuth2 and API keys.")
* ListItem(0,"Use OAuth2 for user apps")
* ListItem(0,"Use API keys for server-to-server")
* KVLine("Rate limit","1000 requests per minute")

LLMD (c2):

```
@authentication
API supports OAuth2 and API keys
-Use OAuth2 user apps
-Use API keys server-to-server
:rate_limit=1000/m
```
