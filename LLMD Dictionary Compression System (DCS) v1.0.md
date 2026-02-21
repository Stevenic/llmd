# LLMD Dictionary Compression System (DCS) v1.0

## 1. Purpose

The Dictionary Compression System (DCS) provides deterministic token reduction by replacing longer strings with shorter aliases.

It targets:

* **Scope names** (`@Auth`)
* **Attribute keys** (`:rate_limit=...`)
* **Attribute values** (selected domains like units, enums)
* **Item text** (`>oauth2 user-app`)
* **Block headers** (`::json`)

The DCS MUST NOT require LLM calls.

---

## 2. Design Constraints

### 2.1 Determinism

Given:

* input text `T`
* compression level `c`
* dictionary `D`
* config `CFG`

Output MUST be identical across runs.

### 2.2 Non-reversible is OK

DCS is allowed to be lossy.

### 2.3 Safety-first substitutions

DCS MUST NOT apply substitutions that can plausibly change meaning due to substring collisions (e.g., `at` inside `rate`).

Therefore, DCS is **token/word-boundary aware** by default.

---

## 3. Dictionary Model

### 3.1 Dictionary File Format (JSON)

A DCS dictionary is a JSON object with these top-level fields:

```json
{
  "version": "1.0",
  "policy": { ... },
  "maps": { ... },
  "units": { ... },
  "stop": { ... }
}
```

Only `maps` is required.

---

## 4. Maps

### 4.1 Map Namespaces

`maps` is divided into namespaces which define where replacements are allowed:

* `scope`: applies to `@Scope` labels
* `key`: applies to attribute keys on `:k=v`
* `value`: applies to attribute values (restricted)
* `text`: applies to `>` item text (restricted)
* `type`: applies to block types (`::json`, `::yaml`)
* `global`: applies anywhere allowed by policy (optional, discouraged)

Example skeleton:

```json
{
  "maps": {
    "scope": { "authentication": "auth", "authorization": "authz" },
    "key":   { "configuration": "cfg", "database": "db" },
    "value": { "oauth2": "O2", "apikey": "K" },
    "text":  { "server-to-server": "svc-svc" },
    "type":  { "javascript": "js" }
  }
}
```

### 4.2 Replacement Rules

Each namespace map is `string -> string`:

* keys MUST be unique within a namespace
* replacements SHOULD be ASCII `[A-Za-z0-9._-]` for maximal token efficiency
* replacements MUST NOT be empty

### 4.3 Precedence

Precedence order when multiple namespaces could apply:

1. `type` (only in `::`)
2. `scope` (only in `@`)
3. `key` (only left side of `=`)
4. `value` (only right side of `=`, if eligible)
5. `text` (only in `>` lines)
6. `global` (only if explicitly enabled)

Within a namespace, **longest match wins** (see §6).

---

## 5. Policy

A `policy` object configures safety and match behavior.

```json
{
  "policy": {
    "case": "smart",
    "match": "word",
    "longest_match": true,
    "normalize_unicode": "NFKC",
    "max_passes": 2,
    "protect": {
      "negations": true,
      "modals": true,
      "numbers": true
    }
  }
}
```

### 5.1 `case`

* `lower`: lowercase input before matching
* `preserve`: match exact case only
* `smart` (recommended):

  * match case-insensitively
  * output replacement exactly as specified

### 5.2 `match`

* `word` (recommended): replace only at word boundaries
* `token`: replace only when the whole token equals the key (strongest safety)
* `substring`: replace anywhere (NOT recommended except for known-safe patterns)

### 5.3 `max_passes`

Number of times to run replacements over the text (to allow chained normalization).

* MUST default to 1 or 2
* MUST prevent infinite loops

---

## 6. Matching & Tokenization (Normative)

### 6.1 Normalization

Before matching, DCS MUST:

* Unicode normalize per `policy.normalize_unicode` (default NFKC)
* Normalize whitespace (single spaces) **outside blocks**
* For `scope` and `key`, apply LLMD normalization rules first (lowercase, `_`, etc.)

### 6.2 Token boundaries

When `match=word`, the system MUST treat word boundaries as:

* boundary exists between:

  * alphanumeric and non-alphanumeric
  * `_` and non-`_`? (configurable; recommended: `_` counts as word char)
* hyphenated tokens (`server-to-server`) are treated as a single token for matching unless split is enabled.

Implementation guidance:

* For `word` mode, use regex with boundary-like behavior:

  * “token chars” = `[A-Za-z0-9_./-]`
  * boundary = transitions into/out of token chars
* For `token` mode, split on whitespace and replace full tokens only.

### 6.3 Longest-match wins

If multiple dictionary keys can match at a position, the system MUST choose the **longest key**.

Example:

* keys: `api`, `apikey`
* input: `apikey`
* result: replace `apikey`, not `api`

---

## 7. Application by LLMD Line Type

### 7.1 Scope Lines (`@`)

Input:

```
@Authentication Methods
```

Process:

1. Normalize scope name: `authentication_methods`
2. Apply `maps.scope` (and optionally `maps.global` if enabled)
3. Output:

```
@auth_m
```

(depending on your dictionary)

Scope compression is often the biggest win.

### 7.2 Attribute Lines (`:k=v`)

Example:

```
:rate_limit=1000/m auth_methods=oauth2|apikey
```

Rules:

* Apply `maps.key` to keys only
* Apply `maps.value` to values only if value eligibility allows (see §8)
* If values contain `|` enumeration, apply mapping to each element separately

Example output:

```
:rate=1000/m methods=O2|K
```

### 7.3 Item Lines (`>text`)

Apply `maps.text` word-wise with protections (negations/modals).
Example:

```
>use api keys for server-to-server
```

→

```
>use K svc-svc
```

(if your dictionary defines it)

### 7.4 Relation Lines (`->Node`)

Relation target nodes are treated as scope-like identifiers:

* normalize then apply `maps.scope`

### 7.5 Blocks

* DCS MUST NOT modify block content between `<<<` and `>>>`
* It MAY compress the `::type` via `maps.type`

---

## 8. Value Eligibility Rules (Prevent Semantic Damage)

Attribute values are dangerous to rewrite blindly. DCS uses allowlists.

A value is eligible for dictionary substitution only if:

1. It is an **enum token**:

   * matches `^[A-Za-z][A-Za-z0-9._-]*$`
   * OR appears in an enumeration separated by `|` or `,`
2. It is not:

   * a number
   * a date/time
   * a URL/email
   * a path-like string (unless configured)
3. It is not quoted `"like this"` (quoted values are preserved)

Config example:

```json
{
  "policy": {
    "value_eligibility": {
      "allow_enums": true,
      "allow_paths": false,
      "allow_free_text": false
    }
  }
}
```

This keeps `rate=1000/m` safe and prevents rewriting arbitrary prose.

---

## 9. Stopword / Glue Integration (Optional but Formal)

You can embed stopword sets inside the dictionary file so “c2/c3” is self-contained:

```json
{
  "stop": {
    "c2": ["the","a","an","very","really","just","that"],
    "protect": ["no","not","never","must","should","may"]
  }
}
```

Rules:

* Stopword removal applies only in `>` lines and only at c2+
* Protected words MUST never be removed

---

## 10. Collision & Ambiguity Handling

### 10.1 Alias Uniqueness (Recommended)

Within each namespace, replacements SHOULD be unique to avoid ambiguity, but not required.

### 10.2 Reserved Token Avoidance

Replacements MUST NOT begin with LLMD control prefixes:

* `~ @ : > :: -> <- = <<< >>>`

If a replacement would violate this, compiler MUST:

* prefix with `_` or
* choose a different alias

### 10.3 Forbidden loops

If a replacement produces a string that matches another key, you can get cascades.

DCS MUST:

* enforce `max_passes`
* OR explicitly prohibit keys that map into other keys (recommended validation)

---

## 11. Validation Rules (What a compiler should check)

A `--validate-dict` mode SHOULD verify:

* JSON schema integrity
* no empty keys/values
* no replacements that start with reserved prefixes
* no cycles if `max_passes > 1`
* longest-match conflicts are acceptable but should be reported (informative)

---

## 12. Reference Dictionary Strategy (Practical Guidance)

### 12.1 Core dictionary (small, high value)

Start with stable, common terms:

* authentication → auth
* authorization → authz
* configuration → cfg
* database → db
* service → svc
* request → req
* response → resp
* error → err
* performance → perf
* oauth2 → O2
* apikey → K

### 12.2 Domain dictionaries

Allow `--dict myteam.json` so teams can add their own.

### 12.3 Frequency-based optional mode (still deterministic)

You *can* auto-generate a dictionary without LLM by:

* scanning the document(s)
* counting token frequencies
* mapping top N long terms to short aliases

But: this is only safe if you enforce **token mode** + **stable ordering** + **no collisions**.

If you want this, define it as `DCS-AUTO` and keep it separate from the static dictionary.

---

## 13. Example (End-to-end)

Dictionary:

```json
{
  "version": "1.0",
  "policy": { "case": "smart", "match": "token", "longest_match": true, "max_passes": 1,
    "protect": { "negations": true, "modals": true, "numbers": true }
  },
  "maps": {
    "scope": { "authentication": "auth" },
    "key":   { "methods": "m", "rate_limit": "rate" },
    "value": { "oauth2": "O2", "apikey": "K" },
    "text":  { "user-facing": "usr", "server-to-server": "svc-svc" }
  }
}
```

Input LLMD (c2 before dict):

```
@authentication
:methods=oauth2|apikey rate_limit=1000/m
>oauth2 user-facing apps
>apikey server-to-server
```

After DCS (c3):

```
@auth
:m=O2|K rate=1000/m
>O2 usr apps
>K svc-svc
```

---

## 14. Deterministic Implementation Sketch (Algorithm)

For each non-block line:

1. Identify line type
2. Normalize per LLMD rules
3. Tokenize according to `policy.match`
4. Apply namespace map with longest-match rule
5. Reassemble preserving separators (`=`, `|`, spaces)
6. Emit line

Block contents are passed through unchanged.
