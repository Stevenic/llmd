#!/usr/bin/env python3
import json, os, re, sys
from math import ceil

def die(msg):
    print(msg, file=sys.stderr)
    sys.exit(1)

def read_json(p):
    with open(p, "r", encoding="utf-8") as f:
        return json.load(f)

def list_files_recursive(inputs):
    out = []
    for p in inputs:
        if os.path.isdir(p):
            for root, _, files in os.walk(p):
                for fn in files:
                    out.append(os.path.join(root, fn))
        elif os.path.isfile(p):
            out.append(p)
    out.sort()
    return out

def canonicalize_to_llmd_lines(text: str, cfg: dict) -> list[str]:
    text = text.replace("\r\n", "\n")
    raw_lines = text.split("\n")

    lines = []
    if cfg.get("ignore_blocks", True):
        in_fence = False
        for ln in raw_lines:
            t = ln.strip()
            if t.startswith("```"):
                in_fence = not in_fence
                continue
            if in_fence:
                continue
            lines.append(ln)
    else:
        lines = raw_lines

    out = []
    current = None

    def emit(scope: str):
        nonlocal current
        if scope and scope != current:
            out.append("@" + scope)
            current = scope

    def norm_scope(s: str) -> str:
        return re.sub(r"\s+", "_", s.strip()).lower()

    def norm_key(s: str) -> str:
        s = re.sub(r"\s+", "_", s.strip().lower())
        s = re.sub(r"[^a-z0-9_]", "", s)
        return s

    for ln0 in lines:
        ln = ln0.strip()
        if not ln:
            continue

        m = re.match(r"^(#{1,6})\s+(.*)$", ln)
        if m:
            emit(norm_scope(m.group(2)))
            continue

        m = re.match(r"^[-*+]\s+(.*)$", ln) or re.match(r"^\d+\.\s+(.*)$", ln)
        if m:
            if not current:
                emit("root")
            out.append(">" + m.group(1).strip())
            continue

        m = re.match(r"^([A-Za-z0-9 _-]{2,64})\s*:\s*(.+)$", ln)
        if m:
            if not current:
                emit("root")
            k = norm_key(m.group(1))
            v = m.group(2).strip()
            out.append(":" + k + "=" + v if k else ">" + ln)
            continue

        if not current:
            emit("root")
        out.append(">" + ln)

    return out

def est_tokens_line(line: str) -> int:
    toks = [t for t in re.split(r"\s+", line.strip()) if t]
    return sum(ceil(len(t)/4) for t in toks)

def apply_dict_token_mode(lines: list[str], d: dict) -> list[str]:
    maps = d.get("maps", {})
    scope_map = maps.get("scope", {})
    key_map = maps.get("key", {})
    value_map = maps.get("value", {})
    text_map = maps.get("text", {})
    type_map = maps.get("type", {})

    out = []
    for line in lines:
        if line.startswith("@"):
            s = line[1:].strip().lower()
            out.append("@" + scope_map.get(s, s))
            continue

        if line.startswith(":"):
            parts = [p for p in re.split(r"\s+", line[1:].strip()) if p]
            new_parts = []
            for p in parts:
                if "=" not in p:
                    new_parts.append(p)
                    continue
                k, v = p.split("=", 1)
                k2 = key_map.get(k.lower(), k.lower())

                # Split enum chunks but keep delimiters
                chunks = re.split(r"([|,])", v)
                out_chunks = []
                for ch in chunks:
                    if ch in ("|", ","):
                        out_chunks.append(ch)
                        continue
                    low = ch.strip().lower()
                    if re.fullmatch(r"[a-z][a-z0-9._-]*", low) and low in value_map:
                        out_chunks.append(value_map[low])
                    else:
                        out_chunks.append(ch.strip())
                v2 = "".join(out_chunks)
                new_parts.append(f"{k2}={v2}")

            out.append(":" + " ".join(new_parts))
            continue

        if line.startswith(">"):
            toks = [t for t in re.split(r"\s+", line[1:].strip()) if t]
            nt = []
            for t in toks:
                low = t.lower()
                low = re.sub(r"^[^a-z0-9._-]+", "", low)
                low = re.sub(r"[^a-z0-9._-]+$", "", low)
                nt.append(text_map.get(low, t))
            out.append(">" + " ".join(nt))
            continue

        if line.startswith("::"):
            tp = line[2:].strip().lower()
            out.append("::" + type_map.get(tp, tp))
            continue

        out.append(line)

    return out

# CLI: python bench.py <config.json> <dict.json> <inputs...>
argv = sys.argv[1:]
if len(argv) < 3:
    die("Usage: python bench.py <config.json> <dict.json> <inputs...>")

cfg = read_json(argv[0])
d = read_json(argv[1])
files = list_files_recursive(argv[2:])
if not files:
    die("No input files found.")

before = 0
after = 0

for fp in files:
    with open(fp, "r", encoding="utf-8") as f:
        txt = f.read()
    ll = canonicalize_to_llmd_lines(txt, cfg)
    ll2 = apply_dict_token_mode(ll, d)
    before += sum(est_tokens_line(x) for x in ll)
    after += sum(est_tokens_line(x) for x in ll2)

saved = before - after
pct_red = (saved / before * 100) if before else 0.0
pct_size = (after / before * 100) if before else 0.0

print(f"Files: {len(files)}")
print(f"Est tokens BEFORE: {before}")
print(f"Est tokens AFTER : {after}")
print(f"Saved: {saved} ({pct_red:.1f}% reduction, final size {pct_size:.1f}%)")