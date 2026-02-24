# schema2llmd — JSON Schema to LLMD Converter

Converts JSON Schema files into compressed LLMD format. Extracts object definitions, their properties, types, descriptions, and allowed values, then emits a compact two-scope LLMD document suitable for LLM context windows.

Available as `node tools/js/schema2llmd.js`, `python tools/py/schema2llmd.py`, and `cargo run --bin schema2llmd`.

---

## Usage

```bash
# Basic conversion (output to stdout)
schema2llmd schema.json

# Convert to file
schema2llmd schema.json -o output.llmd

# With explicit config
schema2llmd schema.json -o output.llmd --config config/llmdc.config.json
```

---

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `<schema.json>` | Input JSON Schema file (required) | — |
| `-o, --output <path>` | Output file (stdout if omitted) | stdout |
| `--config <path>` | Config file path | auto-detect |
| `-h, --help` | Show help | |

---

## Config

Uses the same `llmdc.config.json` as the main compiler (auto-detected from `llmdc.config.json` or `config/llmdc.config.json`). The following config keys apply to schema2llmd output:

| Key | Used for |
|-----|----------|
| `stopwords` | Removed from property descriptions |
| `protect_words` | Never removed from descriptions |
| `phrase_map` | Phrase replacements in descriptions |
| `units` | Unit normalizations in descriptions |
| `bool_compress` | Boolean value compression in descriptions |

---

## How It Works

### 1. Schema Parsing

Reads a JSON Schema file and walks `definitions` (the `definitions` key at the root). Each definition is classified as either an "object definition" (has `properties`, `allOf`, or `type: "object"`) or a scalar/enum (skipped).

`$ref` pointers are resolved recursively. `allOf`, `anyOf`, and `oneOf` branches are merged to collect the full property set.

### 2. Output Structure

The tool emits a two-scope LLMD document:

```
@Objects.Properties
Required properties marked with `!`.
:ObjectA.properties=prop1!, prop2, prop3
:ObjectB.properties=prop1, prop4!
@Properties
-prop1 (string): Description text [allowed, values]
-prop2 (array of Item): Description text Default: "foo".
-prop3 (boolean): Description text
-prop4 (number): Description text
```

**`@Objects.Properties`** — One `:` attribute per object definition, listing its property names. Required properties are marked with `!`.

**`@Properties`** — One `-` list item per unique property across all definitions, showing:
- Property name
- Type (resolved from `type`, `$ref`, `oneOf`/`anyOf`)
- Compressed description (from `description` field)
- Default value if present
- Allowed values extracted from `const`, `enum`, `oneOf`, or `pattern`

### 3. Type Resolution

| Schema pattern | Emitted type |
|---------------|-------------|
| `"type": "string"` | `string` |
| `"type": "array"` with `items.$ref` → `Foo` | `array of Foo` |
| `"$ref"` to a string definition | `string` |
| `"oneOf"` with mixed types | `string¦number` (¦-separated) |
| No type info | `any` |

### 4. Pattern Value Extraction

When a property has a `pattern` field (regex), the tool attempts to extract literal allowed values. It handles:

- Top-level alternation: `^(foo|bar|baz)$` → `[foo, bar, baz]`
- Character class patterns: `^[A-a][B-b]$` → uses first character of each class
- Nested groups with balanced parentheses
- Combined with `$ref` resolution for pattern-based enums

### 5. Description Compression

Property descriptions are compressed using the same c2 pipeline as the main compiler:

1. **Phrase map** — longest-first, case-insensitive replacement (e.g., "in order to" → "to")
2. **Unit normalization** — e.g., "1000 requests per minute" → "1000/m"
3. **Boolean compression** — Yes/No → Y/N, true/false → T/F
4. **Stopword removal** — common words removed, protected words preserved
5. **Trailing period stripping** — preserves `...`, `e.g.`, `i.e.`, `etc.`
6. **Markdown link stripping** — `[text](url)` → `text`
7. **Truncation** — descriptions capped at 200 characters

---

## Example

### Input (excerpt from a JSON Schema)

```json
{
  "definitions": {
    "TextBlock": {
      "type": "object",
      "properties": {
        "type": { "const": "TextBlock" },
        "text": {
          "type": "string",
          "description": "The text to display."
        },
        "size": {
          "type": "string",
          "description": "The size of the text.",
          "oneOf": [
            { "const": "small" },
            { "const": "default" },
            { "const": "medium" },
            { "const": "large" }
          ]
        },
        "wrap": {
          "type": "boolean",
          "description": "Whether the text should wrap.",
          "default": false
        }
      },
      "required": ["type", "text"]
    }
  }
}
```

### Output

```
@Objects.Properties
Required properties marked with `!`.
:TextBlock.properties=type!, text!, size, wrap
@Properties
-type (string): [TextBlock]
-text (string): text display
-size (string): size text. [small, default, medium, large]
-wrap (boolean): Whether text should wrap. Default: false
```

---

## Differences from llmdc

| Aspect | llmdc | schema2llmd |
|--------|-------|-------------|
| Input | Markdown files/directories | Single JSON Schema file |
| Parsing | 6-stage Markdown pipeline | JSON Schema `definitions` walk |
| Scope model | Headings → `@scope` | Fixed two-scope structure |
| Compression | Full c0/c1/c2 pipeline | c2 description compression only |
| Tables | Classified and converted | N/A (no tables in schema) |
| Code blocks | Preserved in `<<<`/`>>>` | N/A |
