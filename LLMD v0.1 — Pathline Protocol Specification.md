# LLMD v0.1 — Pathline Protocol Specification

## 1. Overview

**LLMD** is a compact, line-oriented text format designed to preserve **document structure + key information** with **minimal token overhead** for LLM consumption.

Key properties:

* **Line-oriented**: each line is independently meaningful.
* **Path-scoped**: every line is attached to a **node path**.
* **Lossy allowed**: compilation from Markdown may drop some data depending on compression level.
* **Deterministic**: given the same input + config, output must be identical.

LLMD is not required to be reversible back into the original Markdown.

---

## 2. File Format

### 2.1 Encoding

* MUST be UTF-8.
* MUST use `\n` (LF) line endings in output (normalize on compile).

### 2.2 Structure

An LLMD file is a sequence of lines:

* Optional **metadata lines**
* Content lines of these types:

  * Node
  * Attribute
  * Item
  * Relation
  * Block (for code or preserved literals)

Blank lines are allowed but SHOULD be minimized by compilers (see compression levels).

---

## 3. Core Concepts

### 3.1 Node Path

A **node path** is a hierarchical identifier that scopes all content.

**Syntax:**

* Path segments separated by `/`
* Example: `System/Auth/OAuth2`
* Segment characters: recommended `[A-Za-z0-9._-]`
* Compilers SHOULD normalize segments per rules (see §6)

**Semantics:**

* `A/B/C` implies hierarchy: `A` contains `B` contains `C`.
* Paths are **labels**, not filesystem locations.

### 3.2 Line = Fact Unit

Each LLMD line is intended to be a single “fact unit” to help:

* chunking
* retrieval
* model parsing

---

## 4. Line Types (Normative)

Each non-empty line MUST match one of the following.

### 4.1 Metadata Line (`~`)

Used for file-level hints or document descriptors.

**Syntax:**

```
~ key:value | key:value | ...
```

* `~` + space + one or more `key:value` pairs
* `|` (pipe) separates pairs
* Keys SHOULD be lowercase
* Values are raw strings (trimmed)

**Example:**

```
~ title:Auth Spec | v:0.1 | c:2 | source:README.md
```

**Recommended keys (non-exhaustive):**

* `title`, `v` (version), `c` (compression level), `source`, `lang`, `date`

### 4.2 Node Line (`@`)

Declares or labels a node path. Node lines are optional but recommended for readability and section boundaries.

**Syntax:**

```
@ path
```

**Example:**

```
@ System/Auth
```

**Semantics:**

* Declares that the node exists.
* May be emitted whenever a new section begins.
* Duplicate node lines are allowed; compilers SHOULD avoid unnecessary repetition at higher compression.

### 4.3 Attribute Line (`:`)

Attaches key/value attributes to a node.

**Syntax:**

```
path : k=v | k=v | k=v
```

* `path`, space, `:`, space, one or more `k=v` pairs
* `|` separates pairs
* Keys SHOULD be normalized (see §6)
* Values SHOULD be compacted per compression rules

**Example:**

```
System/Auth : methods=oauth2,apikey | rate=1000/m | required=true
```

**Notes:**

* `k=v` pairs are order-preserving as emitted by the compiler.
* Commas inside values are allowed; `|` is the pair separator.

### 4.4 Item Line (`>`)

Attaches an unstructured list item, note, or statement to a node.

**Syntax:**

```
path > text...
```

**Example:**

```
System/Auth > oauth2 for user-facing apps
System/Auth > apikey for svc-to-svc
```

**Semantics:**

* `text` is treated as an atomic statement.
* Compilers MAY compress text at higher levels (stopwords, phrase maps, dictionary).

### 4.5 Relation Line (`->`, `<-`, `=`, `!`, `?`)

Declares a relationship between two paths or between a path and a literal.

**Syntax (path-to-path):**

```
path1 -> path2
path1 <- path2
path1 =  path2
```

**Syntax (path-to-literal):**

```
path1 -> "literal text"
```

**Operators:**

* `->` means “depends on / leads to / uses” (compiler chooses based on source cues)
* `<-` means inverse dependency
* `=` means equivalence / alias / same-as
* `!` unary prefix means negation / forbidden
* `?` unary suffix means optional / uncertain

**Examples:**

```
System/API -> System/DB
System/API -> System/Cache?
System/Auth ! -> System/Auth/Password   (password auth forbidden)
System/Auth/OAuth2 = System/Auth/O2     (alias)
```

**Normative guidance:**

* Compilers SHOULD use `?` to mark uncertain inference rather than asserting as fact.
* If semantics are ambiguous, prefer `>` item lines over relations.

### 4.6 Block Line (`::`)

Used to embed a literal multi-line block such as code, configuration, or preserved text.

**Syntax:**

```
path :: type [| opt=value ...]
<<<
...raw block content...
>>>
```

* Block header is a single line.
* `type` is a short label: `code`, `json`, `yaml`, `text`, `sql`, etc.
* Optional options use `|` separator.

**Example:**

```
System/API :: json | minified=true
<<<
{"a":1,"b":[2,3]}
>>>
```

**Semantics:**

* Content between `<<<` and `>>>` is raw and MUST NOT be parsed as LLMD.
* Compilers MAY minify certain block types at higher compression levels (configurable).
* `<<<` and `>>>` MUST appear on their own lines.

---

## 5. Grammar (EBNF-ish, Informative)

```
file        := { line "\n" }
line        := meta | node | attr | item | rel | block | blank
meta        := "~" SP kvpairs
node        := "@" SP path
attr        := path SP ":" SP kvpairs_eq
item        := path SP ">" SP text
rel         := path SP op SP (path [ "?" ] | quoted_text)
block       := path SP "::" SP type { SP "|" SP opt } "\n"
              "<<<" "\n" raw "\n" ">>>" 
blank       := "" | SP*

path        := segment { "/" segment }
segment     := 1*( ALNUM | "_" | "-" | "." )

kvpairs     := kv { SP "|" SP kv }
kv          := key ":" value
kvpairs_eq  := kveq { SP "|" SP kveq }
kveq        := key "=" value
```

---

## 6. Normalization Rules (Deterministic)

Compilers MUST apply normalization consistently.

### 6.1 Whitespace

* Trim leading/trailing whitespace on all non-block lines.
* Convert internal runs of whitespace in `text` to a single space (unless inside quoted literal or block content).
* Output MUST use single spaces around separators exactly as shown:

  * `path : ...`
  * `path > ...`
  * `path -> path`

### 6.2 Key normalization

For attribute keys (`k`):

* Lowercase
* Replace spaces with `_`
* Remove punctuation except `_` and `-`
* Example: `Rate Limit` → `rate_limit`

### 6.3 Path normalization

For paths:

* Each segment trimmed
* Replace spaces with `_` (or `-`, but choose one)
* Optionally lowercase segments at compression ≥2 (configurable)
* Avoid empty segments
* Example: `Authentication Methods` → `authentication_methods`

### 6.4 Ordering

* Preserve document order as much as possible.
* Within an attribute line, preserve the compiler’s extraction order.
* Compilers MAY reorder in higher compression only if explicitly configured.

---

## 7. Lossiness and Compression Levels (Normative)

LLMD compilation defines compression level `c ∈ {0,1,2,3}` (extendable).

### 7.1 c0 — Normalize

Goal: minimal transformation.

* Keep most original wording.
* Convert headings/lists into LLMD lines but do not rewrite prose.
* Preserve URLs and punctuation.
* Preserve code blocks unchanged.

### 7.2 c1 — Structural Compaction

Goal: remove Markdown scaffolding.

* Emit paths for hierarchy.
* Convert bullets/numbered items to `path > ...`.
* Convert `Key: Value` to attributes when unambiguous.
* Collapse excess blank lines.

### 7.3 c2 — Text Compaction (Lossy)

Goal: reduce tokens from prose.

* Stopword removal (configurable list), excluding:

  * negations (`no`, `not`, `never`) MUST be preserved
  * modals (`must`, `should`, `may`) SHOULD be preserved
* Phrase map replacement (deterministic):

  * `in order to` → `to`
  * `as well as` → `|`
  * `due to` → `because`
  * `is able to` → `can`
* Prefer attributes over prose:

  * e.g. `Rate limit: 1000 requests/min` → `rate=1000/m`
* URLs MAY be dropped unless `--keep-urls` is enabled.

### 7.4 c3 — Symbolization + Dictionary

Goal: maximal compression while keeping readability for LLM.

* Apply deterministic abbreviation dictionary to:

  * common domain terms (auth, cfg, db, api, req, resp, err, perf, lat, etc.)
* Replace relationship phrases with operators where safely inferred:

  * “depends on” → `->`
  * “optional” → `?`
* Aggressively strip punctuation outside blocks.
* Inline short enumerations:

  * `methods=oauth2|apikey` or `methods{oauth2,apikey}` (configurable style)

**Important constraint:**

* Any rule that could flip meaning MUST be conservative. If unsure, keep original text as `>` item.

---

## 8. Markdown Compilation Mapping (Recommended)

This section defines deterministic conversion rules from Markdown constructs into LLMD.

### 8.1 Headings → Paths

Markdown:

```
## Authentication
### OAuth2
```

LLMD:

```
@ Authentication
@ Authentication/OAuth2
```

Heading depth MAY be used to build path hierarchy:

* H1 creates/sets root
* H2 child of H1
* H3 child of H2, etc.

### 8.2 Paragraphs → Item lines

Paragraph text under current heading becomes:

```
current/path > paragraph text...
```

At c2+, compiler MAY split sentences into multiple `>` lines.

### 8.3 Lists → Item lines

Bullets and numbered lists:

```
current/path > item text...
```

Nested list items append to path:

* Option A (recommended): keep same path, preserve nesting in text prefix `.`:

  * `path > . child item`
* Option B: extend path with generated segments:

  * `path/Item_1 > child...` (riskier)

Choose one approach; be consistent.

### 8.4 Tables → Attributes/Items

For Markdown tables:

* First column becomes attribute keys if it looks like `Property | Value`
* Otherwise represent each row as:

  * `path : col1=value1 | col2=value2 ...` (if keys are stable)
  * or `path > row: v1 | v2 | ...` (fallback)

### 8.5 Links

Markdown:
`[text](url)`
LLMD:

* c0–c1: `text<url>`
* c2+: `text` (drop URL) unless `--keep-urls`

### 8.6 Code blocks

Markdown fenced code becomes LLMD block:

```
path :: code | lang=js
<<<
...code...
>>>
```

At c2+, optional minification may apply for `json/yaml` etc.

---

## 9. Reserved Characters & Escaping

LLMD is intentionally minimal about escaping.

### 9.1 Reserved tokens

* Line-type markers: `~ @ : > :: <<< >>>`
* Relation operators: `-> <- = ! ?`

### 9.2 Escaping strategy (recommended)

Instead of complex escaping:

* If a line’s text would begin with a reserved marker unintentionally, compiler SHOULD:

  * prefix with a harmless character like `_` or
  * wrap as a block `:: text`

For quoted literals in relations:

* Use double quotes `"..."`.
* Inside, escape `"` as `\"` if needed.

---

## 10. Validity Requirements

An LLMD file is **valid** if:

* All block delimiters match (`<<<` then `>>>`).
* Every non-empty, non-block-content line matches a known line type.
* Paths use the chosen normalization rules.

Compilers SHOULD offer a `--validate` mode that checks these invariants.

---

## 11. Implementation Notes (Non-normative but Practical)

### 11.1 Streaming friendliness

Because it’s line-oriented:

* You can compile and emit output incrementally.
* You can chunk by line ranges while preserving context through paths.

### 11.2 Configurable knobs

A reference compiler SHOULD accept:

* `compression level`
* `stopword list`
* `phrase map`
* `dictionary`
* `keep/drop urls`
* `minify blocks by type`

---

## 12. Example (End-to-end)

Input Markdown:

````md
# API Spec

## Authentication
The API supports authentication via OAuth2 and API keys.

- Use OAuth2 for user-facing apps.
- Use API keys for server-to-server.

Rate limit: 1000 requests per minute.

```json
{ "retry": 3, "backoff": "exp" }
````

```

LLMD (c2):
```

~ title:api_spec | v:0.1 | c:2
@ API_Spec
@ API_Spec/Authentication
API_Spec/Authentication : methods=oauth2,apikey | rate=1000/m
API_Spec/Authentication > oauth2 for user-facing apps
API_Spec/Authentication > apikey for server-to-server
API_Spec/Authentication :: json | minified=true
<<<
{"retry":3,"backoff":"exp"}

> > >

```
