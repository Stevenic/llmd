# LLMD — LLM-optimized Deterministic Markdown

LLMD is a deterministic compiler system that converts Markdown into a compact, token-efficient format designed for LLM context windows. It replaces verbose hierarchical Markdown with implicit scoping, structured attributes, and configurable compression — reducing token counts while preserving semantic recoverability.

**Author:** Steven Ickman | **License:** MIT

---

## Quick Start

```bash
# Install JS dependencies
cd tools/js && npm install && cd ../..

# Compile a Markdown file at compression level 2
node tools/js/llmdc.js docs/llmdc.md -c 2 -o docs/llmdc.llmd

# Compile an entire directory
node tools/js/llmdc.js corpora/samples/ -c 2 -o output.llmd

# Generate a dictionary from a corpus, then compile at c3
node tools/js/dcs_auto.js config/auto_config.json dict/llmd-auto.dict.json corpora/samples/
node tools/js/llmdc.js corpora/samples/ -c 3 --dict dict/llmd-core.dict.json --dict dict/llmd-auto.dict.json -o output.llmd
```

Python equivalents are available in `tools/py/` (requires `pip install -r tools/py/requirements.txt` for dictionary validation).

---

## What It Looks Like

**Markdown input:**
```markdown
## Authentication
The API supports authentication via OAuth2 and API keys.
- Use OAuth2 for user-facing apps.
- Use API keys for server-to-server.
Rate limit: 1000 requests per minute.
```

**LLMD output (c2):**
```
@auth
:rate_limit=1000/m
>API supports authentication via OAuth2 API keys.
>OAuth2 user-facing apps.
>API keys server-to-server.
```

Every line starts with a type prefix: `@` scope, `:` attribute, `>` content, `::` code block, `->` relation.

---

## When to Use LLMD

### Stuffing reference docs into LLM context
You have API docs, component libraries, or style guides that need to fit in a system prompt or RAG chunk. LLMD strips markdown formatting overhead while keeping the content machine-readable.

### CSS/design system references
Component tables with class names like `flm-button--primary` compress well — the compiler preserves hyphens in keys, extracts common prefixes, and retains column semantics so the LLM can generate correct markup.

### API specification compression
Endpoint tables, parameter lists, and status code references convert naturally to `:k=v` attributes and `>` items. Code examples pass through untouched inside `::` blocks.

### Multi-document context packing
When you need to fit several documents into a single context window, compile a directory at c2 or c3. The compiler handles file ordering deterministically and merges everything into one `.llmd` output.

### Agentic tool context
Feed `.llmd` files as tool/function descriptions or system instructions to agents. The format is designed so LLMs can parse the scoped structure without explicit instructions.

---

## Project Structure

```
llmd/
├── README.md                                      # This file
├── LICENSE                                        # MIT
│
├── LLMD Specification - v0.1.md                   # Format spec (line types, scoping, normalization)
├── LLMD Compiler Design v0.1.md                   # 6-stage pipeline spec
├── LLMD Dictionary Compression System (DCS) v1.0.md  # DCS spec
├── DCS-AUTO v0.1.md                               # Auto dictionary generation spec
│
├── .architecture/
│   └── ARCHITECTURE.md                            # System overview and diagrams
│
├── docs/                                          # Tool reference documentation
│   ├── llmdc.md                                   # Compiler reference
│   ├── dcs-auto.md                                # Dictionary generator reference
│   ├── validate-dict.md                           # Dictionary validator reference
│   ├── bench.md                                   # Benchmark tool reference
│   └── *.llmd                                     # Pre-compiled LLMD versions
│
├── config/
│   ├── llmdc.config.json                          # Compiler config (stopwords, phrases, units)
│   └── auto_config.json                           # DCS-AUTO generator settings
│
├── schema/
│   └── llmd-dcs-dictionary.schema.json            # JSON Schema for dictionary files
│
├── dict/
│   └── llmd-core.dict.json                        # Hand-curated core dictionary
│
├── tools/
│   ├── js/                                        # Node.js implementations
│   │   ├── llmdc.js                               # Compiler
│   │   ├── dcs_auto.js                            # Dictionary generator
│   │   ├── validate-dict.js                       # Dictionary validator
│   │   └── bench.js                               # Token reduction benchmark
│   └── py/                                        # Python implementations
│       ├── llmdc.py                               # Compiler
│       ├── dcs_auto.py                            # Dictionary generator
│       ├── validate_dict.py                       # Dictionary validator
│       └── bench.py                               # Token reduction benchmark
│
└── corpora/
    └── samples/                                   # Sample documents for testing
        ├── api-spec.md
        └── fluentlm-components.md
```

---

## Tools

| Tool | JS | Python | Purpose |
|------|-----|--------|---------|
| **llmdc** | `tools/js/llmdc.js` | `tools/py/llmdc.py` | Compile Markdown → LLMD |
| **dcs_auto** | `tools/js/dcs_auto.js` | `tools/py/dcs_auto.py` | Generate dictionaries from corpora |
| **validate-dict** | `tools/js/validate-dict.js` | `tools/py/validate_dict.py` | Validate dictionary against schema |
| **bench** | `tools/js/bench.js` | `tools/py/bench.py` | Measure token reduction |

Full reference docs: [`docs/`](docs/)

---

## Compression Levels

| Level | Name | What it does |
|-------|------|--------------|
| **c0** | Structural normalize | Whitespace cleanup, structure conversion |
| **c1** | Compact structure | Merge `:k=v` pairs, collapse blanks, prefix extraction |
| **c2** | Token compaction | Stopword removal, phrase/unit normalization, boolean compression |
| **c3** | Symbolic | Apply DCS dictionaries (scope/key/value/text/type maps) |

---

## Key Features

- **Hyphen-preserving key normalization** — CSS class names like `flm-button--primary` survive compilation intact
- **Table classification** — 2-column property tables emit `:k=v`, 3+ column tables with identifier keys emit `:key=v1|v2`, others emit `>` rows with `:_cols=` headers
- **Common prefix extraction** — When keys share a prefix (e.g., `flm-text--`), it's factored out as `:_pfx=` to avoid repetition
- **Chunked KV emission** — Large attribute groups split across multiple lines (`max_kv_per_line`, default 4)
- **Boolean compression** — Columns of `Yes/No`, `true/false`, `enabled/disabled` → `Y/N`, `T/F`
- **Column header preservation** — `:_col=` and `:_cols=` meta-attributes retain table column semantics
- **Code block passthrough** — Fenced code blocks preserved exactly inside `::lang` / `<<<` / `>>>` delimiters
- **Deterministic output** — Same input + config always produces identical output

---

## Typical Workflow

```bash
# 1. Compile your docs at c2 (good default)
node tools/js/llmdc.js my-docs/ -c 2 -o context.llmd

# 2. If you need more compression, generate a domain dictionary
node tools/js/dcs_auto.js config/auto_config.json dict/my-auto.dict.json my-docs/

# 3. Validate the generated dictionary
cd tools/js && node validate-dict.js ../../dict/my-auto.dict.json && cd ../..

# 4. Benchmark the dictionary's impact
node tools/js/bench.js config/auto_config.json dict/my-auto.dict.json my-docs/

# 5. Compile at c3 with both dictionaries
node tools/js/llmdc.js my-docs/ -c 3 \
  --dict dict/llmd-core.dict.json \
  --dict dict/my-auto.dict.json \
  -o context.llmd
```

---

## Using with Agentic Coding Tools

LLMD works well with AI coding assistants like Claude Code and GitHub Copilot. These tools can automate the compilation workflow, and `.llmd` files make efficient context for AI-driven tasks.

### Feed .llmd as context

Compiled `.llmd` files are smaller and cheaper to include in system prompts, tool descriptions, or RAG results. If your agent needs a component reference or API spec, give it the `.llmd` version instead of raw Markdown.

### Automate compilation in your workflow

Add instructions to your project's `CLAUDE.md` or Copilot instructions file:

```markdown
## LLMD Compilation
When documentation files in `docs/` are modified, recompile them:
- Run `node tools/js/llmdc.js docs/ -c 2 -o docs/compiled.llmd`
- The compiled output goes to `docs/compiled.llmd`
- Always compile at c2 unless asked otherwise
```

### Let the agent run the full pipeline

For Claude Code, you can ask it to run the full compile-benchmark cycle in one shot:

```
Compile corpora/samples/ at c2 and c3, benchmark the difference,
and tell me the token savings.
```

The agent can execute the shell commands, read the output, and summarize results without you needing to remember the CLI arguments.

### Generate domain dictionaries interactively

```
Generate a DCS dictionary from my API docs in docs/api/,
validate it, then show me the top 10 entries by token savings.
```

The agent handles the `dcs_auto` → `validate-dict` → `bench` pipeline and presents results conversationally.

### Tips for best results

- **Point the agent at `docs/`** — The reference docs in `docs/*.md` describe every CLI option and config key. An agent that reads these can run any tool correctly.
- **Use the config files** — `config/llmdc.config.json` and `config/auto_config.json` are self-documenting. Agents can read and modify them for tuning.
- **Check the schema** — `schema/llmd-dcs-dictionary.schema.json` defines valid dictionary structure. Agents can use it to generate or validate dictionaries programmatically.
- **Compare c2 vs c3** — Ask the agent to compile at both levels and compare. c2 is lossless for identifiers; c3 trades readability for more compression.
- **Batch compile on change** — Set up a hook or ask the agent to recompile whenever source docs change, so `.llmd` versions stay current.

---

## Specifications

| Document | Description |
|----------|-------------|
| [LLMD Specification v0.1](LLMD%20Specification%20-%20v0.1.md) | Format definition: line types, scoping model, normalization rules, compression levels |
| [Compiler Design v0.1](LLMD%20Compiler%20Design%20v0.1.md) | 6-stage pipeline architecture, table classification, prefix extraction, config reference |
| [DCS v1.0](LLMD%20Dictionary%20Compression%20System%20(DCS)%20v1.0.md) | Dictionary format, namespace maps, matching policies, safety mechanisms |
| [DCS-AUTO v0.1](DCS-AUTO%20v0.1.md) | Automatic dictionary generation algorithm, scoring, alias assignment |
| [Architecture](/.architecture/ARCHITECTURE.md) | System overview, component diagrams, file relationships |
