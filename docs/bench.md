# bench — Token Reduction Benchmark

Measures token reduction from applying a DCS dictionary to a corpus. Canonicalizes Markdown to LLMD c1-style lines, applies the dictionary, and reports estimated token savings.

Available as both `node tools/js/bench.js` and `python tools/py/bench.py`.

---

## Usage

```bash
# JavaScript
node tools/js/bench.js config/auto_config.json dict/llmd-core.dict.json corpora/samples/

# Python
python tools/py/bench.py config/auto_config.json dict/llmd-core.dict.json corpora/samples/
```

---

## Arguments

| Position | Description |
|----------|-------------|
| 1 | Config file (JSON — same format as `auto_config.json`) |
| 2 | Dictionary file (JSON — DCS dictionary format) |
| 3+ | Input files or directories |

---

## Output

```
Files: 2
Est tokens BEFORE: 1450
Est tokens AFTER : 1180
Saved: 270 (18.6% reduction, final size 81.4%)
```

| Metric | Description |
|--------|-------------|
| Files | Number of input files processed |
| Est tokens BEFORE | Estimated tokens after c1 canonicalization (before dictionary) |
| Est tokens AFTER | Estimated tokens after dictionary application |
| Saved | Token delta, percentage reduction, and final size percentage |

---

## Algorithm

1. **Collect files** — Recursively find all files in input paths, sorted lexicographically
2. **Canonicalize** — Convert each file to LLMD c1-style lines:
   - Headings → `@scope` (lowercased, spaces→`_`)
   - Lists → `>text`
   - KV lines → `:key=value`
   - Everything else → `>text`
   - Code blocks optionally stripped (`ignore_blocks` config)
3. **Apply dictionary** — Token-mode replacement across namespaces:
   - `@scope` lines → scope map
   - `:k=v` lines → key map on keys, value map on enum-like value parts
   - `>text` lines → text map on individual tokens
   - `::type` lines → type map
4. **Estimate tokens** — Heuristic: `sum(ceil(len(token) / 4))` per whitespace-split token
5. **Report** — Before/after token counts and reduction percentage

---

## Config

Uses the same config format as `dcs_auto`. The relevant field is:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ignore_blocks` | bool | `true` | Skip fenced code blocks during canonicalization |

---

## Dictionary Format

Expects a JSON file conforming to the [DCS Dictionary Schema](../schema/llmd-dcs-dictionary.schema.json). The bench tool uses the `maps` object (namespace→{source→alias} mappings) to perform token-mode replacements.

---

## Notes

- The token estimation heuristic (`ceil(len/4)`) approximates BPE tokenization without requiring a tokenizer dependency
- Dictionary application uses simple whole-token matching (case-insensitive for scope/key, case-preserving for text)
- Value substitution only applies to enum-like tokens: must start with a letter, contain only `[A-Za-z0-9._-]`, and not be numeric or URL-like
