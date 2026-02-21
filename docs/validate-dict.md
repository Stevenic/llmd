# validate-dict — DCS Dictionary Validator

Validates a DCS dictionary file against the JSON Schema.

Available as both `node tools/js/validate-dict.js` and `python tools/py/validate_dict.py`.

---

## Usage

```bash
# JavaScript (requires AJV)
cd tools/js && node validate-dict.js ../../dict/llmd-core.dict.json

# Python (requires jsonschema)
cd tools/py && python validate_dict.py ../../dict/llmd-core.dict.json
```

---

## Arguments

| Position | Description |
|----------|-------------|
| 1 | Path to the dictionary file to validate |

---

## Behavior

- Loads the schema from `llmd-dcs-dictionary.schema.json` (expected in the current directory — run from `tools/js/` or `tools/py/`)
- Validates the dictionary against the schema
- Prints a success or failure message with error details
- Exits with code `0` on success, `1` on failure

---

## Schema Reference

The validator uses the [DCS Dictionary Schema](../schema/llmd-dcs-dictionary.schema.json) (JSON Schema Draft 2020-12).

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `version` | string | Must match `1.0` or `1.0.x` |
| `maps` | object | Must contain all five namespace maps |

### Maps (all required)

| Namespace | Applies to | Example |
|-----------|------------|---------|
| `scope` | `@Scope` labels | `"authentication"` → `"auth"` |
| `key` | `:k=v` attribute keys | `"methods"` → `"m"` |
| `value` | Enum-like values in `:k=v` | `"OAuth2"` → `"O2"` |
| `text` | `>` item text tokens | `"service"` → `"svc"` |
| `type` | `::type` block labels | `"javascript"` → `"js"` |

### Optional Fields

| Field | Description |
|-------|-------------|
| `policy` | Matching behavior: `case`, `match` mode, `longest_match`, `max_passes`, `protect`, `value_eligibility`, `enable_global` |
| `units` | Unit normalization map (e.g., `"requests per minute"` → `"/m"`) |
| `stop` | Stopword lists for c2 and c3, plus protected words |

### Policy Options

| Key | Values | Default | Description |
|-----|--------|---------|-------------|
| `case` | `lower`, `preserve`, `smart` | `smart` | Case handling during matching |
| `match` | `word`, `token`, `substring` | `token` | Matching mode |
| `longest_match` | bool | `true` | Prefer longest applicable key |
| `normalize_unicode` | `NFC`, `NFD`, `NFKC`, `NFKD` | — | Unicode normalization before matching |
| `max_passes` | 1–10 | `1` | Maximum replacement passes |
| `enable_global` | bool | `false` | Enable `maps.global` namespace |

### Protect Options

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `negations` | bool | `true` | Protect `no`, `not`, `never` |
| `modals` | bool | `true` | Protect `must`, `should`, `may` |
| `numbers` | bool | `true` | Protect purely numeric tokens |

### Value Eligibility

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `allow_enums` | bool | `true` | Allow substitution of enum-like tokens |
| `allow_paths` | bool | `false` | Allow substitution of path-like strings |
| `allow_free_text` | bool | `false` | Allow substitution inside free text |
| `deny_patterns` | string[] | — | Regex patterns that disqualify values |

---

## Dependencies

| Runtime | Package |
|---------|---------|
| Node.js | `ajv` ^8.17.1 (JSON Schema Draft 2020-12 support) |
| Python | `jsonschema` >=4.20.0 |
