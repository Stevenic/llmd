# LLMD — LLM-optimized Deterministic Markdown

LLMD is a deterministic compiler system that converts Markdown into a compact, token-efficient format designed for LLM context windows. It replaces verbose hierarchical Markdown with implicit scoping, structured attributes, and configurable compression — reducing token counts while preserving semantic recoverability.

**Author:** Steven Ickman | **License:** MIT

---

## Quick Start

```bash
# JavaScript (Node.js 18+)
node tools/js/llmdc.js docs/llmdc.md -c 2 -o docs/llmdc.llmd

# Python (3.10+)
python tools/py/llmdc.py docs/llmdc.md -c 2 -o docs/llmdc.llmd

# Rust (single binary, no runtime needed)
cargo run --manifest-path tools/rust/Cargo.toml -- docs/llmdc.md -c 2 -o docs/llmdc.llmd
```

All three implementations produce identical output. Use whichever fits your environment.

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
 @authentication
  >The API supports authentication via OAuth2 and API keys.
  >Use OAuth2 for user-facing apps.
  >Use API keys for server-to-server.
  :rate_limit=1000 requests per minute.
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
When you need to fit several documents into a single context window, compile a directory at c2. The compiler handles file ordering deterministically and merges everything into one `.llmd` output.

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
│
├── .architecture/
│   └── ARCHITECTURE.md                            # System overview and diagrams
│
├── docs/                                          # Tool reference documentation
│   ├── llmdc.md                                   # Compiler reference
│   └── llmdc.llmd                                 # Pre-compiled LLMD version
│
├── config/
│   └── llmdc.config.json                          # Compiler config (stopwords, phrases, units)
│
├── tools/
│   ├── js/                                        # Node.js implementations
│   │   └── llmdc.js                               # Compiler
│   ├── py/                                        # Python implementations
│   │   └── llmdc.py                               # Compiler
│   └── rust/                                      # Rust implementation
│       └── src/                                   # Compiler (single binary)
│
└── corpora/
    └── samples/                                   # Sample documents for testing
        ├── api-spec.md
        └── fluentlm-components.md
```

---

## Tools

| Tool | JS | Python | Rust | Purpose |
|------|-----|--------|------|---------|
| **llmdc** | `tools/js/llmdc.js` | `tools/py/llmdc.py` | `tools/rust/` | Compile Markdown → LLMD |

Full reference docs: [`docs/`](docs/)

### Performance

All three implementations produce identical output. Measured on Windows 11 (median of 5 runs, c2 compression):

| File | JS (Node 22) | Python 3.10 | Rust (release) |
|------|-------------|-------------|----------------|
| api-spec.md (1.3 KB) | 140 ms | 238 ms | 61 ms |
| fluentlm-components.md (45 KB) | 243 ms | 354 ms | 73 ms |

Run `pwsh tools/bench.ps1` or `bash tools/bench.sh` to reproduce.

---

## Compression Levels

| Level | Name | What it does |
|-------|------|--------------|
| **c0** | Structural normalize | Whitespace cleanup, structure conversion |
| **c1** | Compact structure | Merge `:k=v` pairs, collapse blanks, prefix extraction |
| **c2** | Token compaction | Stopword removal, phrase/unit normalization, boolean compression |

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
# Compile your docs at c2 (good default)
node tools/js/llmdc.js my-docs/ -c 2 -o context.llmd

# Or compile at c0/c1 for less aggressive compression
node tools/js/llmdc.js my-docs/ -c 0 -o context.llmd
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

### Let the agent run compilation

For Claude Code, you can ask it to compile in one shot:

```
Compile corpora/samples/ at c2 and tell me the token savings.
```

The agent can execute the shell commands, read the output, and summarize results without you needing to remember the CLI arguments.

### Tips for best results

- **Point the agent at `docs/`** — The reference docs in `docs/*.md` describe every CLI option and config key. An agent that reads these can run any tool correctly.
- **Use the config file** — `config/llmdc.config.json` is self-documenting. Agents can read and modify it for tuning.
- **Batch compile on change** — Set up a hook or ask the agent to recompile whenever source docs change, so `.llmd` versions stay current.

---

## Specifications

| Document | Description |
|----------|-------------|
| [LLMD Specification v0.1](LLMD%20Specification%20-%20v0.1.md) | Format definition: line types, scoping model, normalization rules, compression levels |
| [Compiler Design v0.1](LLMD%20Compiler%20Design%20v0.1.md) | 6-stage pipeline architecture, table classification, prefix extraction, config reference |
| [Architecture](/.architecture/ARCHITECTURE.md) | System overview, component diagrams, file relationships |
