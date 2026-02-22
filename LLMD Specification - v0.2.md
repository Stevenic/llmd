# LLMD v0.2 — Specification

---

# 1. Design Principles

1. **Scope is implicit** (never repeat full paths unless scope changes)
2. **Each line = one semantic unit**
3. **Minimal structural symbols**
4. **Lossy allowed**
5. **Deterministic compilation**
6. **Optimized for token count, not human reconstruction**

---

# 2. File Structure

LLMD is a UTF-8, line-oriented format.

Each non-empty line begins with exactly one of:

```
~   metadata
@   scope declaration
:   attribute line (scoped)
    plain text (no prefix — paragraphs, prose)
-   list item (scoped, both ordered and unordered)
→   forward relation (scoped)
←   reverse relation (scoped)
=   equivalence relation (scoped)
::  block start (scoped)
```

A line is plain text if it does not start with any of the above prefixes.

Horizontal rules (`---`, `***`, `___`) are stripped during parsing and never appear in output.

---

# 3. Scope Model (Core Mechanism)

## 3.1 Scope Declaration

```
@node
```

* Sets current scope.
* All following lines inherit this scope.
* Scope persists until next `@`.

Scope names:

* SHOULD be short
* SHOULD be normalized (see §7)
* SHOULD NOT contain `/` (hierarchy flattened by compiler)

Example:

```
@Auth
:methods=oauth2|apikey rate=1000/m
-oauth2 user-app
```

No repeated `Auth/...` prefixes.

---

## 4. Line Types

---

## 4.1 Metadata Line

Optional. SHOULD appear only once at top.

```
~k=v k=v
```

No pipes. Space-separated pairs.

Example:

```
~v=0.2 c=2 title=api_spec
```

Metadata SHOULD be omitted at high compression unless required.

---

## 4.2 Attribute Line

Structured facts for current scope.

```
:k=v k=v k=v
```

Rules:

* No leading scope prefix
* Keys lowercase
* No spaces around `=`
* Space separates pairs

Example:

```
:methods=oauth2|apikey rate=1000/m required=true
```

Preferred over prose whenever possible.

### Reserved Meta-Attributes

Keys prefixed with `_` are reserved for compiler-generated metadata:

* `:_col=<header>` — column header for a 2-column property table (emitted when the value column header is informative)
* `:_cols=col1|col2|col3` — column headers for a multi-column table
* `:_pfx=<prefix>` — common prefix extracted from subsequent keys; the reader should prepend this prefix to restore full key names

### Chunked Emission

At c1+, consecutive `:k=v` pairs are merged onto one line. When the number of pairs exceeds `max_kv_per_line` (default 4), they are split across multiple `:` lines for chunk-safe splitting:

```
:k1=v1 k2=v2 k3=v3 k4=v4
:k5=v5 k6=v6 k7=v7 k8=v8
:k9=v9
```

---

## 4.3 Text Line

Unstructured prose or paragraph content. Text lines have **no prefix** — they are plain text.

```
platform requires minimum three application nodes
```

Example:

```
@auth
API supports authentication via OAuth2 API keys
```

Compiler MAY:

* Remove stopwords (c2+)
* Apply phrase map replacements (c2)
* Strip trailing periods (c2)

---

## 4.4 List Item Line

Both ordered and unordered Markdown lists compile to `-` prefixed lines.

```
-item text
```

Nested list depth uses `.` prefixes:

* depth 0: `-item`
* depth 1: `-. child`
* depth 2: `-.. grandchild`

Example:

```
@compute
-Application nodes: 3 minimum
-. high availability recommended
-Worker nodes: 2 minimum
```

---

## 4.5 Relation Line

Declares relation from current scope.

```
→Node
←Node
=Node
```

Optional uncertainty:

```
→Node?
```

Example:

```
@API
→DB
→Cache?
```

This means:

API depends on DB
API optionally depends on Cache

No repeated prefixes.

---

## 4.6 Block Line

Used for code or preserved literals.

```
::type
<<<
raw content
>>>
```

Example:

```
::json
<<<
{"retry":3,"backoff":"exp"}
>>>
```

Rules:

* `<<<` and `>>>` must be alone on their lines
* Block content is raw and not parsed
* Compiler MAY minify JSON/YAML at c2+

---

# 5. Hierarchy Handling

Hierarchy is flattened at compile time.

Example Markdown:

```
# API
## Authentication
```

Compiler options:

### Option A (default): Merge path segments

```
@API
@Auth
```

### Option B (concatenate)

```
@API_Auth
```

LLMD itself does NOT encode explicit multi-level hierarchy.
Hierarchy is compiler concern, not runtime syntax.

This removes `/` token repetition.

---

# 6. Compression Levels (Normative)

`c ∈ {0,1,2}`

---

## c0 — Structural Normalize

* Convert Markdown structure
* Preserve most wording
* Preserve URLs
* Preserve punctuation
* Strip horizontal rules (`---`, `***`, `___`)

Goal: clean but not compressed.

---

## c1 — Compact Structure

* Convert lists → `-`
* Convert `Key: Value` → `:k=v`
* Collapse whitespace
* Remove extra blank lines
* No stopword removal yet

---

## c2 — Token Compaction

* Remove stopwords (configurable list)
* Remove filler phrases:

  * in order to → to
  * due to → because
  * is able to → can
* Normalize units:

  * "1000 requests per minute" → `1000/m`
* Strip trailing periods from text and list lines (but not `...`, `e.g.`, `i.e.`, `etc.`)
* Convert obvious sentences into attributes
* Drop URLs unless flagged

Must preserve:

* negation words
* modal strength (must/should/may)

---

# 7. Normalization Rules

## 7.1 Scope Names

At minimum:

* Trim whitespace
* Replace spaces with `_`

At c2+:

* Lowercase

---

## 7.2 Keys

Always:

* Lowercase
* Spaces → `_`
* Strip punctuation except `_` and `-`
* Trim leading/trailing `-`

Example:

```
Rate Limit → rate_limit
flm-text--secondary → flm-text--secondary
```

Preserving `-` is critical for CSS class names and other hyphenated identifiers.

---

## 7.3 Whitespace

* Single LF line endings
* No trailing spaces
* No multiple blank lines
* No extra spaces around operators

---

# 8. Valid File Rules

A valid LLMD file must:

* Begin with optional metadata
* Declare a scope before any scoped line
* Not mix block markers
* Not contain unknown line prefixes

---

# 9. Example (Token-Optimized)

Original Markdown (64 tokens approx):

```
## Authentication
The API supports authentication via OAuth2 and API keys.

- Use OAuth2 for user-facing apps.
- Use API keys for server-to-server.

Rate limit: 1000 requests per minute.
```

LLMD v0.2 (c2):

```
@auth
:rate_limit=1000/m
API supports authentication via OAuth2 API keys
-OAuth2 user-facing apps
-API keys server-to-server
```

~24 tokens

No repeated paths.
No verbose prose.
No unnecessary punctuation.

---

# 10. Why v0.2 Compresses Better

Compared to v0.1:

Removed:

* `>` prefix on every text line (saves 1 token per line)
* Trailing periods on text and list lines
* Horizontal rules (no semantic value)

Added:

* `-` prefix distinguishes list items from prose
* `→` / `←` Unicode arrows replace `->` / `<-`
* Expanded stopword and phrase map for more aggressive c2

---

# 11. Deterministic Compiler Model

Minimal pipeline:

1. Extract Markdown structure
2. Track current heading → scope
3. Emit `@scope` when heading changes
4. Convert lists → `-`
5. Convert simple pairs → `:k=v`
6. Apply compression rules by level
7. Emit file

No AST required (line-based parser is sufficient).

---

# 12. Optional Future Extensions (Not in v0.2 Core)

* Stable scope IDs for chunk-safe slicing
* Inline scope hash markers
* Global term alias table at file top

---

# Final Summary

LLMD v0.2 is:

* Scoped
* Minimal
* Deterministic
* Lossy by design
* Token-optimized
* Easy to implement
* Proven to reduce tokens substantially
