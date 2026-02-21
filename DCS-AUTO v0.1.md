# DCS-AUTO v0.1 — Frequency-Based Dictionary Generator

## 1. Goal

Automatically build a dictionary `D_auto` that maps frequent long terms → short aliases, without any LLM calls, while staying:

* **Deterministic** (same inputs/config → same dict)
* **Safe** (no substring collisions, no reserved-prefix issues)
* **Token-efficient** (aliases chosen to reduce token count)
* **Stable across files** (optionally corpus-based)

This dictionary is intended primarily for **c3**.

---

## 2. Inputs

DCS-AUTO takes:

### Required

* One or more **source documents** (Markdown and/or LLMD and/or plain text)
* A config object `CFG_auto`

### Optional

* A base dictionary `D_base` (static hand-crafted) to “reserve” abbreviations and prevent conflicts.

---

## 3. Output

A JSON dictionary file that conforms to your schema:

* version: `"1.0"`
* policy: copied or omitted
* maps: generated (typically only `scope`, `key`, `value`, `text`)
* optional: stop/units (not required)

File example name:

* `llmd-auto.dict.json`

---

## 4. Safety Model (Critical)

AUTO generation MUST obey these safety constraints:

### 4.1 Token-only substitutions

All AUTO mappings MUST be applied in **token mode** (exact token equality), not substring.

* This prevents `auth` replacing inside `authorization`.
* It also makes chunk-level behavior predictable.

### 4.2 Namespace restrictions

AUTO should generate mappings only for safe namespaces by default:

* ✅ `scope` (safe if scope names are normalized tokens)
* ✅ `key` (safe because keys are normalized tokens)
* ✅ `value` (safe only for enum-like values)
* ✅ `text` (safe only for “vocabulary tokens” in item lines, not free text substrings)
* ❌ `global` (disabled)

### 4.3 Protected tokens

Never map:

* negations: `no not never`
* modals: `must should may`
* ultra-common function words (even if frequent) — they don’t compress well and can confuse
* numbers and number-like tokens
* URLs/emails
* tokens shorter than a threshold (e.g., < 5 chars)

### 4.4 No alias collisions

Generated aliases must not:

* collide with existing aliases in `D_base` or earlier picks
* equal any existing **source token** (to avoid ambiguity when reading)
* start with reserved LLMD prefixes (`@`, `:`, `>`, `~`, `::`, `->`, `<<<`, `>>>`)
* include whitespace

---

## 5. Deterministic Pipeline

### Step A — Canonicalize sources

For each input file:

1. If Markdown: compile to **LLMD c1** *without dictionary* (structure normalized, minimal loss)
2. If already LLMD: normalize whitespace
3. Ignore block contents (`<<< ... >>>`) entirely or treat as separate mode (default: ignore)

This produces a canonical stream of LLMD lines.

### Step B — Extract candidate tokens by namespace

From canonical LLMD:

* `@scope` lines → candidate tokens for `maps.scope`
* `:k=v` lines:

  * keys → `maps.key`
  * values:

    * split enums by `|` and `,`
    * include only tokens matching enum regex: `^[A-Za-z][A-Za-z0-9._-]*$`
* `>` item lines:

  * tokenize by whitespace
  * further split on punctuation except `_` and `-` (configurable)
  * collect tokens → `maps.text` candidates

Store:

* `freq[token] += 1`
* `contexts[token].add(namespace)` (some tokens appear in multiple namespaces)

### Step C — Candidate filtering

A token becomes a candidate only if it passes:

* length ≥ `min_len` (default 6)
* freq ≥ `min_freq` (default 3)
* not in `stoplist`
* not protected
* not numeric-like
* not URL/email-like
* not already mapped in `D_base` keys (optional)
* not already used as an alias in `D_base` values (reserved)

### Step D — Score candidates

We need a deterministic scoring function that approximates token savings.

#### Savings model (simple but effective)

For a candidate token `t` and alias `a`:

* `gain_per_use = estTokens(t) - estTokens(a)`
* `total_gain = freq[t] * gain_per_use - overhead(t,a)`

Where:

* `estTokens(x)` is a deterministic heuristic:

  * simplest: `ceil(len(x)/4)` as an approximation
  * better: optional model-specific tokenizer later, but not required
* `overhead` can be 0 unless you also emit an alias legend (LLMD v0.2 doesn’t require it)

Rank by:

1. `total_gain` descending
2. `len(t)` descending
3. `freq[t)` descending
4. `t` lexicographically ascending (tie-breaker)

This ensures stable ordering.

### Step E — Choose mapping set

Select top `N` candidates or all with `total_gain >= min_gain`.

Default:

* `max_entries`: 256
* `min_gain`: 10 (across corpus)

### Step F — Generate aliases deterministically

This is the hardest part.

We want aliases that are:

* short
* unique
* stable
* not confusing

We use a staged aliasing strategy.

---

## 6. Alias Generation Algorithm (Deterministic + Collision-Free)

### 6.1 Reserved alias set

Initialize `RESERVED` with:

* all aliases in `D_base` (all namespaces)
* all protected tokens
* all existing source tokens (optional strict mode)
* all LLMD reserved strings (`@ : > ~ :: -> <- = <<< >>>`)

### 6.2 Alias style (config)

Pick one style; keep it stable.

Recommended default: **base-36 ids with prefix per namespace**

* scope: `s` + base36(i)  → `s0, s1, ...`
* key:   `k` + base36(i)  → `k0, k1, ...`
* value: `v` + base36(i)  → `v0, v1, ...`
* text:  `t` + base36(i)  → `t0, t1, ...`

Pros:

* extremely compact
* deterministic
* avoids accidental “real word” collisions

Cons:

* less human-readable (but you said reversibility isn’t needed)

Example:

* `authentication` → `s0`
* `rate_limit` → `k3`
* `oauth2` → `v1`

### 6.3 Human-readable variant (optional)

If you want readability, use:

* abbreviation based on initial letters, then disambiguate with suffix

Example:

* `authentication` → `auth`
* if `auth` taken → `auth1`
* if `auth1` taken → `auth2`, etc.

This can still be deterministic but tends to collide more.

### 6.4 Final alias assignment

For each candidate in ranked order:

1. propose alias according to selected style
2. if alias in RESERVED, increment `i` until free
3. assign mapping and add alias to RESERVED

This guarantees collision-free output.

---

## 7. Namespace Assignment Rules

A token may appear in multiple namespaces. Decide deterministically:

### 7.1 Prefer safer, more structured namespaces

Priority:

1. key
2. scope
3. value
4. text

Example:
If `timeout_seconds` appears as a key and also in text, map it as a key.

### 7.2 Optional: allow multi-namespace mapping

You MAY emit the same mapping in multiple namespaces if:

* alias does not collide
* config `allow_multi=true`

But default is single-namespace to keep dictionary compact.

---

## 8. Deterministic Reproducibility Guarantees

To guarantee identical output:

* Always sort input files by canonical path/filename
* Always use stable tokenization rules
* Always use stable tie-breakers
* Avoid hash-randomization (don’t use unordered dict iteration)

Output ordering within each namespace:

* sort by source token lexicographically (or by rank). Pick one and document it.

Recommended:

* order by rank (most impactful first), tie-break lexicographically.

---

## 9. Configuration (`CFG_auto`) Defaults

```json
{
  "min_len": 6,
  "min_freq": 3,
  "max_entries": 256,
  "min_gain": 10,
  "tokenize": {
    "split_on": "whitespace",
    "keep_chars": "A-Za-z0-9_\\-.",
    "lowercase": true
  },
  "namespaces": {
    "scope": true,
    "key": true,
    "value": true,
    "text": true,
    "global": false
  },
  "protect": {
    "negations": ["no", "not", "never"],
    "modals": ["must", "should", "may"]
  },
  "stoplist": ["the", "a", "an", "and", "or", "to", "of", "in", "for"],
  "alias_style": "ns_base36",
  "strict_no_alias_equals_source_token": true,
  "ignore_blocks": true
}
```

---

## 10. Example

Corpus tokens:

* `authentication` freq 120
* `authorization` freq 70
* `configuration` freq 55
* `rate_limit` freq 40

AUTO picks top 4, assigns:

* scope:

  * `authentication` → `s0`
  * `authorization` → `s1`
* key:

  * `configuration` → `k0`
  * `rate_limit` → `k1`

Generated dictionary snippet:

```json
{
  "version": "1.0",
  "policy": {
    "case": "smart",
    "match": "token",
    "longest_match": true,
    "normalize_unicode": "NFKC",
    "max_passes": 1
  },
  "maps": {
    "scope": { "authentication": "s0", "authorization": "s1" },
    "key": { "configuration": "k0", "rate_limit": "k1" },
    "value": {},
    "text": {},
    "type": {}
  }
}
```

---

## 11. Safety vs Compression Knobs

If you want *more* compression:

* allow `text` namespace more aggressively
* allow `min_len` down to 5
* allow `max_entries` bigger
* allow code-token extraction (dangerous; off by default)

If you want *more* safety:

* disable `text` namespace
* restrict to `scope` + `key` only (still big wins, very safe)

---

## 12. Recommended “Production Modes”

### Mode SAFE (default)

* namespaces: scope + key + value(enums only)
* text disabled
* alias style: ns_base36

### Mode BALANCED

* text enabled, but only tokens ≥ 8 chars and freq ≥ 10

### Mode AGGRESSIVE

* text enabled broadly
* allow multi-namespace
* larger dictionary
