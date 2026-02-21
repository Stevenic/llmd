# dcs_auto — DCS Automatic Dictionary Generator

Generates DCS dictionaries from source corpora using frequency analysis. No LLM calls required.

Available as both `node tools/js/dcs_auto.js` and `python tools/py/dcs_auto.py`.

---

## Usage

```bash
# Generate dictionary from a corpus directory
node tools/js/dcs_auto.js config/auto_config.json dict/llmd-auto.dict.json corpora/samples/

# With a base dictionary (auto-generated entries won't collide with it)
node tools/js/dcs_auto.js config/auto_config.json dict/llmd-auto.dict.json corpora/ --base dict/llmd-core.dict.json

# Python equivalent
python tools/py/dcs_auto.py config/auto_config.json dict/llmd-auto.dict.json corpora/
```

---

## Arguments

| Position | Description |
|----------|-------------|
| 1 | Config file path (JSON) |
| 2 | Output dictionary file path |
| 3+ | Input files or directories |

| Flag | Description |
|------|-------------|
| `--base <path>` | Base dictionary to avoid alias/key collisions with |

---

## Config File

See [`config/auto_config.json`](../config/auto_config.json) for the default configuration.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `min_len` | int | `6` | Minimum token length to consider |
| `min_freq` | int | `3` | Minimum frequency to consider |
| `max_entries` | int | `256` | Maximum dictionary entries |
| `min_gain` | int | `10` | Minimum total token gain to include |
| `alias_style` | string | `"ns_base36"` | Alias naming strategy |
| `namespaces` | object | all `true` | Enable/disable namespace extraction |
| `ignore_blocks` | bool | `true` | Skip fenced code blocks |
| `strict_no_alias_equals_source_token` | bool | `true` | Prevent aliases that match source tokens |
| `stoplist` | string[] | see config | Words excluded from candidacy |
| `protect.negations` | string[] | `["no","not","never"]` | Protected negation words |
| `protect.modals` | string[] | `["must","should","may"]` | Protected modal words |

---

## Algorithm

1. **Canonicalize** — Each source file is converted to LLMD c1-style lines (headings→`@scope`, lists→`>text`, KV→`:k=v`)
2. **Extract tokens** — Tokens extracted by namespace (`@scope`, `:key`, `:value`, `>text`), lowercased
3. **Filter** — Reject tokens shorter than `min_len`, below `min_freq`, in stoplist/protect lists, numeric, URLs, or already in base dictionary keys
4. **Score** — `total_gain = frequency × max(0, est_tokens(token) - est_tokens(alias))`
5. **Deduplicate** — If a token appears in multiple namespaces, keep only the highest-priority one (key > scope > value > text)
6. **Rank** — Sort by `total_gain` desc, token length desc, frequency desc, token asc
7. **Cap** — Take top `max_entries` candidates
8. **Assign aliases** — Deterministic `ns_base36` format: `s0`, `s1`, ... (scope), `k0`, `k1`, ... (key), etc. Aliases skip reserved prefixes and collisions with base dictionary or source tokens

---

## Output Format

Produces a JSON file conforming to the [DCS Dictionary Schema](../schema/llmd-dcs-dictionary.schema.json):

```json
{
  "version": "1.0",
  "policy": {
    "case": "smart",
    "match": "token",
    "longest_match": true,
    "normalize_unicode": "NFKC",
    "max_passes": 1,
    "enable_global": false
  },
  "maps": {
    "scope": { "authentication": "s0" },
    "key":   { "methods": "k0" },
    "value": { "oauth2": "v0" },
    "text":  { "service": "t0" },
    "type":  {}
  }
}
```

All map keys are sorted lexicographically for deterministic output.

---

## Token Estimation

Uses a heuristic: `sum(ceil(len(token) / 4))` over whitespace-split tokens. This approximates BPE token counts without requiring a tokenizer.

---

## Namespace Priority

When the same token appears in multiple namespaces, only the highest-priority namespace is kept:

1. `key` (highest)
2. `scope`
3. `value`
4. `text` (lowest)
