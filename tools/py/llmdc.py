#!/usr/bin/env python3
# ============================================================
# llmdc — LLMD Compiler (Python)
# Spec: LLMD v0.2 + Compiler Design v0.2
# ============================================================
import argparse
import json
import os
import re
import sys
import unicodedata
from math import floor


def die(msg):
    print("error: " + msg, file=sys.stderr)
    sys.exit(1)


def read_json(p):
    with open(p, "r", encoding="utf-8") as f:
        return json.load(f)


def list_files(inputs):
    out = []
    for p in inputs:
        if os.path.isdir(p):
            for root, _, files in os.walk(p):
                for fn in files:
                    fp = os.path.join(root, fn)
                    if re.search(r"\.(md|markdown|llmd)$", fp, re.IGNORECASE):
                        out.append(fp)
        elif os.path.isfile(p):
            if re.search(r"\.(md|markdown|llmd)$", p, re.IGNORECASE):
                out.append(p)
    out.sort()
    return out


# ============================================================
# Stage 0: Normalize
# ============================================================
def stage0(text):
    text = unicodedata.normalize("NFKC", text)
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    return [line.rstrip() for line in text.split("\n")]


# ============================================================
# Stage 1: Extract / Protect Blocks
# ============================================================
def stage1(lines):
    blocks = []
    out = []
    in_block = False
    lang = ""
    buf = []
    fence = ""

    for line in lines:
        if not in_block:
            m = re.match(r"^(`{3,})(\w*)\s*$", line)
            if m:
                in_block = True
                fence = m.group(1)
                lang = m.group(2) or ""
                buf = []
                continue
            out.append(line)
        else:
            if line.rstrip() == fence:
                idx = len(blocks)
                blocks.append({"index": idx, "lang": lang, "content": "\n".join(buf)})
                out.append(f"\u27E6BLOCK:{idx}\u27E7")
                in_block = False
                fence = ""
                lang = ""
                buf = []
            else:
                buf.append(line)

    if in_block and buf:
        idx = len(blocks)
        blocks.append({"index": idx, "lang": lang, "content": "\n".join(buf)})
        out.append(f"\u27E6BLOCK:{idx}\u27E7")

    return out, blocks


# ============================================================
# Stage 2: Parse to IR
# ============================================================
RE_HEADING = re.compile(r"^(#{1,6})\s+(.+)$")
RE_UL = re.compile(r"^(\s*)([-*+])\s+(.+)$")
RE_OL = re.compile(r"^(\s*)(\d+)\.\s+(.+)$")
RE_BLOCK_REF = re.compile(r"^\u27E6BLOCK:(\d+)\u27E7$")
RE_KV = re.compile(r"^([A-Za-z][A-Za-z0-9 _-]{0,63})\s*:\s+(.+)$")


def is_structural(line):
    t = line.strip()
    if not t:
        return True
    if RE_HEADING.match(t):
        return True
    if RE_UL.match(t) or RE_OL.match(t):
        return True
    if RE_BLOCK_REF.match(t):
        return True
    if "|" in t:
        return True
    if RE_KV.match(t) and not t.startswith("http://") and not t.startswith("https://"):
        return True
    return False


def parse_table_row(row):
    cells = [c.strip() for c in row.split("|")]
    if cells and cells[0] == "":
        cells = cells[1:]
    if cells and cells[-1] == "":
        cells = cells[:-1]
    return cells


def stage2(lines):
    ir = []
    i = 0
    n = len(lines)

    while i < n:
        line = lines[i]
        t = line.strip()

        if t == "":
            ir.append({"type": "blank"})
            i += 1
            continue

        # Skip thematic breaks (---, ***, ___)
        if re.match(r"^[-*_]{3,}$", t):
            i += 1
            continue

        m = RE_BLOCK_REF.match(t)
        if m:
            ir.append({"type": "block_ref", "index": int(m.group(1))})
            i += 1
            continue

        m = RE_HEADING.match(t)
        if m:
            ir.append({"type": "heading", "level": len(m.group(1)), "text": m.group(2).strip()})
            i += 1
            continue

        # Table detection
        if "|" in t and i + 1 < n:
            next_line = lines[i + 1].strip()
            if re.match(r"^\|?[\s:-]+\|", next_line) and "---" in next_line:
                rows = [parse_table_row(t)]
                i += 2  # skip header + delimiter
                while i < n and "|" in lines[i] and lines[i].strip():
                    rows.append(parse_table_row(lines[i].strip()))
                    i += 1
                ir.append({"type": "table", "rows": rows})
                continue

        m = RE_UL.match(line)
        if m:
            depth = floor(len(m.group(1)) / 2)
            ir.append({"type": "list_item", "depth": depth, "text": m.group(3).strip(), "ordered": False})
            i += 1
            continue

        m = RE_OL.match(line)
        if m:
            depth = floor(len(m.group(1)) / 2)
            ir.append({"type": "list_item", "depth": depth, "text": m.group(3).strip(), "ordered": True})
            i += 1
            continue

        m = RE_KV.match(t)
        if m and not t.startswith("http://") and not t.startswith("https://"):
            ir.append({"type": "kv", "key": m.group(1), "value": m.group(2).strip()})
            i += 1
            continue

        # Paragraph: merge consecutive non-structural lines
        para = [t]
        i += 1
        while i < n:
            nl = lines[i].strip()
            if not nl or is_structural(lines[i]):
                break
            para.append(nl)
            i += 1
        ir.append({"type": "paragraph", "text": " ".join(para)})

    return ir


# ============================================================
# Inline markdown processing
# ============================================================
def strip_inline_markdown(text):
    text = re.sub(r"\*\*(.+?)\*\*", r"\1", text)
    text = re.sub(r"__(.+?)__", r"\1", text)
    text = re.sub(r"(?<!\*)\*(?!\*)(.+?)(?<!\*)\*(?!\*)", r"\1", text)
    text = re.sub(r"`([^`]+)`", r"\1", text)
    text = re.sub(r"~~(.+?)~~", r"\1", text)
    return text


def process_links(text, keep_urls):
    if keep_urls:
        text = re.sub(r"!\[([^\]]*)\]\(([^)]+)\)", r"\1<\2>", text)
        text = re.sub(r"\[([^\]]*)\]\(([^)]+)\)", r"\1<\2>", text)
    else:
        text = re.sub(r"!\[([^\]]*)\]\(([^)]+)\)", r"\1", text)
        text = re.sub(r"\[([^\]]*)\]\(([^)]+)\)", r"\1", text)
    return text


def process_inline(text, compression, keep_urls):
    text = strip_inline_markdown(text)
    text = process_links(text, compression < 2 or keep_urls)
    return text


# ============================================================
# Scope normalization
# ============================================================
def norm_scope_name(text, compression):
    s = re.sub(r"\s+", "_", text.strip())
    s = re.sub(r"[^A-Za-z0-9_-]", "", s)
    if compression >= 2:
        s = s.lower()
    return s


def norm_key(text):
    s = re.sub(r"\s+", "_", text.strip().lower())
    s = re.sub(r"[^a-z0-9_-]", "", s)
    s = s.strip("-")
    return s


# ============================================================
# Stages 3+4: Scope Resolution + Emit LLMD
# ============================================================
def emit_llmd(ir, blocks, config):
    compression = config.get("compression", 2)
    scope_mode = config.get("scope_mode", "flat")
    keep_urls = config.get("keep_urls", False)
    sentence_split = config.get("sentence_split", False)

    out = []
    current_scope = None
    heading_stack = []  # [(level, name)]

    def resolve_scope(level, text):
        name = norm_scope_name(text, compression)
        while heading_stack and heading_stack[-1][0] >= level:
            heading_stack.pop()
        heading_stack.append((level, name))

        if scope_mode == "flat":
            return name
        if scope_mode in ("concat", "stacked"):
            return "_".join(h[1] for h in heading_stack)
        return name

    def emit_scope(scope):
        nonlocal current_scope
        if scope and scope != current_scope:
            out.append("@" + scope)
            current_scope = scope

    def ensure_scope():
        if not current_scope:
            emit_scope("root")

    def process_text(text):
        return process_inline(text, compression, keep_urls)

    GENERIC_HEADERS = {"value", "description", "details", "info", "notes", "default", "type"}

    def is_informative_header(header):
        return bool(header) and header.strip().lower() not in GENERIC_HEADERS

    def classify_table(rows):
        """Classify table: 'property' (2-col KV), 'keyed_multi' (3+ cols with
        unique identifier-like first column), or 'raw'."""
        if len(rows) < 2:
            return "raw"
        num_cols = len(rows[0])
        if not all(len(r) == num_cols for r in rows[1:]):
            return "raw"
        if num_cols < 2:
            return "raw"
        # Check if first column values are unique and identifier-like
        first_col_vals = set()
        all_identifier = True
        for r in rows[1:]:
            val = r[0].strip()
            if val in first_col_vals:
                all_identifier = False
                break
            first_col_vals.add(val)
            if not re.match(r"^[A-Za-z._-]", val) or len(val.split()) > 4:
                all_identifier = False
                break
        if not all_identifier:
            return "raw"
        if num_cols == 2:
            return "property"
        return "keyed_multi"

    def split_sentences(text):
        if not sentence_split or compression < 2:
            return [text]
        sentences = re.split(r"(?<=[.!?])\s+(?=[A-Z])", text)
        return [s for s in sentences if s.strip()]

    # Boolean/enum value compression
    bool_compress_enabled = config.get("bool_compress", True) and compression >= 2
    BOOL_MAP = {
        "yes": "Y", "no": "N",
        "true": "T", "false": "F",
        "enabled": "Y", "disabled": "N",
    }

    def compress_bool_value(val):
        if not bool_compress_enabled:
            return val
        return BOOL_MAP.get(val.strip().lower(), val)

    max_kv_per_line = config.get("max_kv_per_line", 4)
    prefix_extraction = config.get("prefix_extraction", True)
    min_prefix_len = config.get("min_prefix_len", 6)
    min_prefix_pct = config.get("min_prefix_pct", 0.6)
    kv_buffer = []

    def find_common_prefix(keys):
        if len(keys) < 2:
            return ""
        prefix = keys[0]
        for k in keys[1:]:
            while not k.startswith(prefix):
                prefix = prefix[:-1]
                if not prefix:
                    return ""
        # Trim to last separator boundary
        last_sep = max(prefix.rfind("-"), prefix.rfind("_"), prefix.rfind("."))
        if last_sep > 0:
            prefix = prefix[:last_sep + 1]
        else:
            prefix = ""
        return prefix

    def flush_kv():
        nonlocal kv_buffer
        if not kv_buffer:
            return

        # Try prefix extraction at c1+
        if compression >= 1 and prefix_extraction and len(kv_buffer) >= 3:
            keys = [kv["key"] for kv in kv_buffer]
            prefix = find_common_prefix(keys)
            if len(prefix) >= min_prefix_len:
                match_count = sum(1 for k in keys if k.startswith(prefix))
                if match_count / len(keys) >= min_prefix_pct:
                    out.append(":_pfx=" + prefix)
                    adjusted = [
                        {"key": kv["key"][len(prefix):] if kv["key"].startswith(prefix) else kv["key"],
                         "value": kv["value"]}
                        for kv in kv_buffer
                    ]
                    for i in range(0, len(adjusted), max_kv_per_line):
                        chunk = adjusted[i:i + max_kv_per_line]
                        pairs = [kv["key"] + "=" + kv["value"] for kv in chunk]
                        out.append(":" + " ".join(pairs))
                    kv_buffer = []
                    return

        if compression >= 1:
            for i in range(0, len(kv_buffer), max_kv_per_line):
                chunk = kv_buffer[i:i + max_kv_per_line]
                pairs = [kv["key"] + "=" + kv["value"] for kv in chunk]
                out.append(":" + " ".join(pairs))
        else:
            for kv in kv_buffer:
                out.append(":" + kv["key"] + "=" + kv["value"])
        kv_buffer = []

    for node in ir:
        if node["type"] != "kv":
            flush_kv()

        if node["type"] == "heading":
            scope = resolve_scope(node["level"], node["text"])
            emit_scope(scope)

        elif node["type"] == "paragraph":
            ensure_scope()
            text = process_text(node["text"])
            for s in split_sentences(text):
                s = s.strip()
                if s:
                    out.append(s)

        elif node["type"] == "list_item":
            ensure_scope()
            text = process_text(node["text"])
            depth_dots = "." * node["depth"]
            out.append("-" + depth_dots + (" " if depth_dots else "") + text)

        elif node["type"] == "kv":
            ensure_scope()
            k = norm_key(node["key"])
            v = process_text(node["value"])
            if k:
                kv_buffer.append({"key": k, "value": v})
            else:
                out.append(process_text(node["key"] + ": " + node["value"]))

        elif node["type"] == "table":
            ensure_scope()
            rows = node["rows"]
            table_type = classify_table(rows)

            # Detect boolean columns for compression
            bool_cols = set()
            if bool_compress_enabled and len(rows) > 1:
                for c in range(1, len(rows[0])):
                    all_bool = all(
                        (r[c] if c < len(r) else "").strip().lower() in BOOL_MAP
                        for r in rows[1:]
                    )
                    if all_bool:
                        bool_cols.add(c)

            def process_cell(cell, col_idx):
                text = process_text(cell)
                return compress_bool_value(text) if col_idx in bool_cols else text

            if table_type == "property":
                # Emit column header if informative
                if len(rows[0]) >= 2 and is_informative_header(rows[0][1]):
                    col_header = norm_key(rows[0][1])
                    if col_header:
                        out.append(":_col=" + col_header)
                for r in rows[1:]:
                    k = norm_key(r[0])
                    v = process_cell(r[1], 1)
                    if k:
                        kv_buffer.append({"key": k, "value": v})
                    else:
                        out.append(process_text(r[0] + "¦" + r[1]))
            elif table_type == "keyed_multi":
                # Emit column headers
                col_headers = "¦".join(norm_key(h) for h in rows[0])
                out.append(":_cols=" + col_headers)
                for r in rows[1:]:
                    k = norm_key(r[0])
                    vals = [process_cell(c, ci + 1) for ci, c in enumerate(r[1:])]
                    if k:
                        kv_buffer.append({"key": k, "value": "¦".join(vals)})
                    else:
                        cells = [process_cell(c, ci) for ci, c in enumerate(r)]
                        out.append("¦".join(cells))
            else:
                # Raw: emit column headers then c1¦c2¦c3
                if len(rows[0]) >= 2:
                    col_headers = "¦".join(norm_key(h) for h in rows[0])
                    out.append(":_cols=" + col_headers)
                for r in rows[1:]:
                    cells = [process_cell(c, ci) for ci, c in enumerate(r)]
                    out.append("¦".join(cells))

        elif node["type"] == "block_ref":
            ensure_scope()
            block = blocks[node["index"]]
            lang = block["lang"] or "code"
            out.append("::" + lang)
            out.append("<<<")
            out.append(block["content"])
            out.append(">>>")

    flush_kv()
    return out


# ============================================================
# Stage 5: Compression Passes
# ============================================================
def compress_c0(lines):
    out = []
    for line in lines:
        t = re.sub(r"\s+", " ", line).strip()
        if not t:
            continue
        # Strip any residual horizontal rules
        if re.match(r"^[-*_]{3,}$", t) or t == ">---":
            continue
        out.append(t)
    return out


def compress_c1(lines):
    return compress_c0(lines)


def _escape_regex(s):
    return re.escape(s)


def _is_text_line(line):
    """A line is a text line if it does not start with any known prefix."""
    if not line:
        return False
    for prefix in ("@", ":", "-", "~", "::", "<<<", ">>>", "\u2192", "\u2190", "="):
        if line.startswith(prefix):
            return False
    return True


def compress_c2(lines, config):
    stopwords = set(s.lower() for s in config.get("stopwords", []))
    protect = set(s.lower() for s in config.get("protect_words", []))
    phrase_map = config.get("phrase_map", {})
    units = config.get("units", {})

    # Sort by length desc for longest match
    phrases_sorted = sorted(phrase_map.keys(), key=len, reverse=True)
    units_sorted = sorted(units.keys(), key=len, reverse=True)

    in_block = False
    out = []

    for line in lines:
        if line == "<<<":
            in_block = True
            out.append(line)
            continue
        if line == ">>>":
            in_block = False
            out.append(line)
            continue
        if in_block:
            out.append(line)
            continue
        if line.startswith("::") or line.startswith("@"):
            out.append(line)
            continue

        text = line

        # Determine line type
        is_text = _is_text_line(text)
        is_list = text.startswith("-")
        is_attr = text.startswith(":")

        if is_text:
            line_prefix = ""
            body = text
        elif is_list:
            line_prefix = "-"
            body = text[1:]
        elif is_attr:
            line_prefix = ":"
            body = text[1:]
        else:
            out.append(text)
            continue

        # Apply phrase map on text, list, and attribute lines
        for phrase in phrases_sorted:
            body = re.sub(_escape_regex(phrase), phrase_map[phrase], body, flags=re.IGNORECASE)

        for unit in units_sorted:
            # "N unit" pattern
            body = re.sub(
                r"(\d+)\s+" + _escape_regex(unit),
                r"\1" + units[unit],
                body,
                flags=re.IGNORECASE,
            )
            # Standalone
            body = re.sub(_escape_regex(unit), units[unit], body, flags=re.IGNORECASE)

        text = line_prefix + body

        # Stopword removal on text and list lines
        if is_text or is_list:
            prefix2 = "-" if is_list else ""
            body2 = text[1:] if is_list else text
            tokens = body2.split()
            filtered = []
            for t in tokens:
                low = re.sub(r"[^a-z]", "", t.lower())
                if not low:
                    filtered.append(t)
                    continue
                if low in protect:
                    filtered.append(t)
                    continue
                if low not in stopwords:
                    filtered.append(t)
            text = prefix2 + " ".join(filtered)

        # Trailing period stripping on text and list lines
        if is_text or is_list:
            if text.endswith(".") and not text.endswith("...") and not text.endswith("e.g.") and not text.endswith("i.e.") and not text.endswith("etc."):
                text = text[:-1]

        out.append(text)

    return out


# ============================================================
# Stage 6: Post-processing
# ============================================================
def stage6(lines, config):
    anchor_every = config.get("anchor_every", 0)

    # Validation
    first_scope = False
    in_block = False
    errors = []

    for i, line in enumerate(lines):
        if line == "<<<":
            in_block = True
            continue
        if line == ">>>":
            in_block = False
            continue
        if in_block:
            continue
        if line.startswith("@"):
            first_scope = True
            continue
        if line.startswith("~"):
            continue
        if not first_scope and (line.startswith(":") or line.startswith("-") or line.startswith("\u2192") or line.startswith("\u2190") or line.startswith("=") or _is_text_line(line)):
            errors.append(f"line {i + 1}: scoped line before first @scope")

    if errors:
        print("validation warnings:", file=sys.stderr)
        for e in errors:
            print("  " + e, file=sys.stderr)

    # Anchors
    if anchor_every > 0:
        current_scope = None
        lines_since = 0
        out = []
        for line in lines:
            if line.startswith("@"):
                current_scope = line
                lines_since = 0
                out.append(line)
                continue
            lines_since += 1
            if lines_since >= anchor_every and current_scope:
                out.append(current_scope)
                lines_since = 0
            out.append(line)
        return out

    return lines


# ============================================================
# Main Compile Pipeline
# ============================================================
def compile_text(text, config):
    compression = config.get("compression", 2)

    # Stage 0
    lines = stage0(text)

    # Stage 1
    clean_lines, blocks = stage1(lines)

    # Stage 2
    ir = stage2(clean_lines)

    # Stages 3+4
    output = emit_llmd(ir, blocks, config)

    # Stage 5
    if compression >= 0:
        output = compress_c0(output)
    if compression >= 1:
        output = compress_c1(output)
    if compression >= 2:
        output = compress_c2(output, config)

    # Stage 6
    output = stage6(output, config)

    return "\n".join(output) + "\n"


# ============================================================
# CLI
# ============================================================
def main():
    parser = argparse.ArgumentParser(
        prog="llmdc",
        description="LLMD Compiler — compile Markdown to LLMD format",
    )
    parser.add_argument("inputs", nargs="+", help="Input file(s) or directory")
    parser.add_argument("-o", "--output", help="Output file (default: stdout)")
    parser.add_argument("-c", "--compression", type=int, choices=[0, 1, 2], help="Compression level (default: from config or 2)")
    parser.add_argument("--scope-mode", choices=["flat", "concat", "stacked"], help="Scope mode (default: flat)")
    parser.add_argument("--keep-urls", action="store_true", help="Keep URLs at c2+")
    parser.add_argument("--sentence-split", action="store_true", help="Split sentences into separate > lines at c2+")
    parser.add_argument("--anchor-every", type=int, help="Re-emit @scope every N lines (default: 0 = off)")
    parser.add_argument("--config", help="Config file path")

    args = parser.parse_args()

    # Load config
    config = {}
    if args.config:
        config = read_json(args.config)
    else:
        for p in ["llmdc.config.json", "config/llmdc.config.json"]:
            if os.path.isfile(p):
                config = read_json(p)
                break

    # CLI overrides
    if args.compression is not None:
        config["compression"] = args.compression
    if args.scope_mode is not None:
        config["scope_mode"] = args.scope_mode
    if args.keep_urls:
        config["keep_urls"] = True
    if args.sentence_split:
        config["sentence_split"] = True
    if args.anchor_every is not None:
        config["anchor_every"] = args.anchor_every
    config.setdefault("compression", 2)

    # Collect files
    files = list_files(args.inputs)
    if not files:
        die("no input files found")

    # Compile
    all_text = ""
    for fp in files:
        if all_text:
            all_text += "\n"
        with open(fp, "r", encoding="utf-8") as f:
            all_text += f.read()

    result = compile_text(all_text, config)

    if args.output:
        with open(args.output, "w", encoding="utf-8") as f:
            f.write(result)
        tokens = len([t for t in result.split() if t])
        print(
            f"compiled {len(files)} file(s) -> {args.output} (c{config['compression']}, ~{tokens} tokens)",
            file=sys.stderr,
        )
    else:
        sys.stdout.write(result)


if __name__ == "__main__":
    main()
