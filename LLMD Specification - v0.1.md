# LLMD v0.1 — Specification

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
>   item line (scoped)
->  relation (scoped)
::  block start (scoped)
```

Anything else is invalid outside a block.

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
>oauth2 user-app
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

---

## 4.3 Item Line

Unstructured or semi-structured statement.

```
>text...
```

No space required after `>` (saves tokens).

Example:

```
>oauth2 user-app
>apikey svc-svc
```

Compiler MAY:

* Remove stopwords (c2+)
* Apply dictionary replacements (c3)

---

## 4.4 Relation Line

Declares relation from current scope.

```
->Node
<-Node
=Node
```

Optional uncertainty:

```
->Node?
```

Example:

```
@API
->DB
->Cache?
```

This means:

API depends on DB
API optionally depends on Cache

No repeated prefixes.

---

## 4.5 Block Line

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

### Option C (dictionary compressed)

```
@A
```

LLMD itself does NOT encode explicit multi-level hierarchy.
Hierarchy is compiler concern, not runtime syntax.

This removes `/` token repetition.

---

# 6. Compression Levels (Normative)

`c ∈ {0,1,2,3}`

---

## c0 — Structural Normalize

* Convert Markdown structure
* Preserve most wording
* Preserve URLs
* Preserve punctuation

Goal: clean but not compressed.

---

## c1 — Compact Structure

* Convert lists → `>`
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
* Convert obvious sentences into attributes
* Drop URLs unless flagged

Must preserve:

* negation words
* modal strength (must/should/may)

---

## c3 — Symbolic Compression

* Apply deterministic abbreviation dictionary
* Shorten scope names
* Replace phrases with operators:

  * depends on → `->`
  * optional → `?`
* Remove most punctuation
* Aggressively inline enumerations

Example:

```
oauth2 → O2
apikey → K
authentication → auth
database → db
configuration → cfg
```

Dictionary MUST be static and deterministic.

---

# 7. Normalization Rules

## 7.1 Scope Names

At minimum:

* Trim whitespace
* Replace spaces with `_`

At c2+:

* Lowercase

At c3:

* Apply dictionary shortening

---

## 7.2 Keys

Always:

* Lowercase
* Spaces → `_`
* Strip punctuation except `_`

Example:

```
Rate Limit → rate_limit
```

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

LLMD v0.1 (c2):

```
@Auth
:methods=oauth2|apikey rate=1000/m
>oauth2 user-app
>apikey svc-svc
```

~27 tokens
~58% reduction
~42% final size

No repeated paths.
No verbose prose.
No unnecessary punctuation.

---

# 10. Why v0.1 Compresses Better

Compared to v0.1:

Removed:

* Path repetition
* Pipes in metadata
* Repeated scope prefixes
* Hierarchical separators
* Verbose block headers

Everything now leans toward:

* Short scope
* Short keys
* Minimal structure

---

# 11. Deterministic Compiler Model

Minimal pipeline:

1. Extract Markdown structure
2. Track current heading → scope
3. Emit `@scope` when heading changes
4. Convert lists → `>`
5. Convert simple pairs → `:k=v`
6. Apply compression rules by level
7. Apply dictionary if c3
8. Emit file

No AST required (line-based parser is sufficient).

---

# 12. Optional Future Extensions (Not in v0.1 Core)

* Stable scope IDs for chunk-safe slicing
* Inline scope hash markers
* Frequency-based auto-shortening dictionary
* Global term alias table at file top

---

# Final Summary

LLMD v0.1 is:

* Scoped
* Minimal
* Deterministic
* Lossy by design
* Token-optimized
* Easy to implement
* Proven to reduce tokens substantially
