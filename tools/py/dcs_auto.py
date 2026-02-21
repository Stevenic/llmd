#!/usr/bin/env python3
import json, os, re, sys
from math import ceil

RESERVED_PREFIXES = ["~", "@", ":", ">", "::", "->", "<-", "=", "<<<", ">>>"]

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

def base36(n: int) -> str:
    chars = "0123456789abcdefghijklmnopqrstuvwxyz"
    if n == 0:
        return "0"
    s = ""
    while n > 0:
        n, r = divmod(n, 36)
        s = chars[r] + s
    return s

def is_reserved_prefix(s: str) -> bool:
    return any(s.startswith(p) for p in RESERVED_PREFIXES)

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

    def emit_scope(scope: str):
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
            emit_scope(norm_scope(m.group(2)))
            continue

        m = re.match(r"^[-*+]\s+(.*)$", ln) or re.match(r"^\d+\.\s+(.*)$", ln)
        if m:
            if not current:
                emit_scope("root")
            out.append(">" + m.group(1).strip())
            continue

        m = re.match(r"^([A-Za-z0-9 _-]{2,64})\s*:\s*(.+)$", ln)
        if m:
            if not current:
                emit_scope("root")
            k = norm_key(m.group(1))
            v = m.group(2).strip()
            out.append(":" + k + "=" + v if k else ">" + ln)
            continue

        if not current:
            emit_scope("root")
        out.append(">" + ln)

    return out

def tokenize_text_to_tokens(s: str) -> list[str]:
    raw = re.split(r"\s+", s.strip())
    toks = []
    for tok in raw:
        if not tok:
            continue
        tok = tok.lower()
        tok = re.sub(r"^[^a-z0-9._-]+", "", tok)
        tok = re.sub(r"[^a-z0-9._-]+$", "", tok)
        if not tok:
            continue
        if re.fullmatch(r"[a-z0-9._-]+", tok):
            toks.append(tok)
        else:
            toks.extend([p for p in re.split(r"[^a-z0-9._-]+", tok) if p])
    return toks

def is_numeric_like(tok: str) -> bool:
    return re.fullmatch(r"[0-9]+([./:_-][0-9]+)*", tok) is not None

def is_url_email_like(tok: str) -> bool:
    return tok.startswith("http://") or tok.startswith("https://") or ("@" in tok)

def est_tokens_from_string(s: str) -> int:
    toks = [t for t in re.split(r"\s+", s.strip()) if t]
    return sum(ceil(len(t)/4) for t in toks)

def split_enum_values(v: str) -> list[str]:
    parts = [p.strip() for p in re.split(r"[|,]", v) if p.strip()]
    return parts

def build_auto_dict(files: list[str], cfg: dict, base_dict: dict | None):
    protect = set([*(cfg.get("protect", {}).get("negations", [])),
                   *(cfg.get("protect", {}).get("modals", []))])
    protect = {p.lower() for p in protect}
    stop = {s.lower() for s in cfg.get("stoplist", [])}

    ns_enabled = cfg.get("namespaces", {"scope": True, "key": True, "value": True, "text": True})

    freq = { "scope": {}, "key": {}, "value": {}, "text": {} }
    all_source_tokens = set()

    def bump(ns: str, tok: str):
        freq[ns][tok] = freq[ns].get(tok, 0) + 1
        all_source_tokens.add(tok)

    # reserved from base dict
    reserved_aliases = set()
    reserved_keys = set()
    if base_dict and "maps" in base_dict:
        for ns, mp in base_dict["maps"].items():
            if not isinstance(mp, dict):
                continue
            for k in mp.keys():
                reserved_keys.add(str(k).lower())
            for v in mp.values():
                reserved_aliases.add(str(v).lower())

    for fp in files:
        with open(fp, "r", encoding="utf-8") as f:
            txt = f.read()
        lines = canonicalize_to_llmd_lines(txt, cfg)

        for line in lines:
            if line.startswith("~"):
                continue
            if line.startswith("@") and ns_enabled.get("scope", True):
                s = line[1:].strip().lower()
                if s:
                    bump("scope", s)
                continue
            if line.startswith(":"):
                body = line[1:].strip()
                pairs = [p for p in re.split(r"\s+", body) if p]
                for p in pairs:
                    if "=" not in p:
                        continue
                    k, v = p.split("=", 1)
                    k = k.lower()
                    if ns_enabled.get("key", True) and k:
                        bump("key", k)
                    if ns_enabled.get("value", True) and v:
                        for piece0 in split_enum_values(v):
                            piece = piece0.lower()
                            if not re.fullmatch(r"[a-z][a-z0-9._-]*", piece):
                                continue
                            if is_numeric_like(piece) or is_url_email_like(piece):
                                continue
                            bump("value", piece)
                continue
            if line.startswith(">") and ns_enabled.get("text", True):
                text = line[1:].strip()
                for tok in tokenize_text_to_tokens(text):
                    bump("text", tok)
                continue

    # Candidate filtering + scoring
    min_len = cfg.get("min_len", 6)
    min_freq = cfg.get("min_freq", 3)
    min_gain = cfg.get("min_gain", 10)
    max_entries = cfg.get("max_entries", 256)

    ns_priority = {"key": 1, "scope": 2, "value": 3, "text": 4}

    candidates = []  # (tok, ns, f, totalGain)
    def consider(ns: str):
        for tok, f in freq[ns].items():
            if len(tok) < min_len or f < min_freq:
                continue
            if tok in stop or tok in protect:
                continue
            if is_numeric_like(tok) or is_url_email_like(tok):
                continue
            if tok in reserved_keys:
                continue
            assumed_alias = ns[0] + "0"
            gain_per = max(0, est_tokens_from_string(tok) - est_tokens_from_string(assumed_alias))
            total_gain = f * gain_per
            if total_gain >= min_gain:
                candidates.append((tok, ns, f, total_gain))

    if ns_enabled.get("scope", True): consider("scope")
    if ns_enabled.get("key", True): consider("key")
    if ns_enabled.get("value", True): consider("value")
    if ns_enabled.get("text", True): consider("text")

    # Keep best namespace per token
    best = {}
    for tok, ns, f, g in candidates:
        if tok not in best:
            best[tok] = (tok, ns, f, g)
            continue
        _, pns, pf, pg = best[tok]
        better = (
            ns_priority[ns] < ns_priority[pns]
            or (ns_priority[ns] == ns_priority[pns] and g > pg)
            or (ns_priority[ns] == ns_priority[pns] and g == pg and f > pf)
            or (ns_priority[ns] == ns_priority[pns] and g == pg and f == pf and tok < tok)  # no-op tie
        )
        if better:
            best[tok] = (tok, ns, f, g)

    final = list(best.values())
    # Rank: gain desc, len(tok) desc, freq desc, tok asc
    final.sort(key=lambda x: (-x[3], -len(x[0]), -x[2], x[0]))
    final = final[:max_entries]

    # Alias assignment (ns_base36)
    used_aliases = set(reserved_aliases)
    used_source = all_source_tokens if cfg.get("strict_no_alias_equals_source_token", True) else set()

    def next_alias(ns: str, i: int) -> str:
        prefix = "s" if ns == "scope" else "k" if ns == "key" else "v" if ns == "value" else "t"
        return prefix + base36(i)

    maps = {"scope": {}, "key": {}, "value": {}, "text": {}, "type": {}}
    counters = {"scope": 0, "key": 0, "value": 0, "text": 0}

    for tok, ns, f, g in final:
        i = counters[ns]
        while True:
            alias = next_alias(ns, i).lower()
            if is_reserved_prefix(alias) or alias in used_aliases or alias in used_source:
                i += 1
                continue
            maps[ns][tok] = alias
            used_aliases.add(alias)
            counters[ns] = i + 1
            break

    # Stable key ordering
    for ns in ["scope", "key", "value", "text"]:
        maps[ns] = {k: maps[ns][k] for k in sorted(maps[ns].keys())}

    return {
        "version": "1.0",
        "policy": {
            "case": "smart",
            "match": "token",
            "longest_match": True,
            "normalize_unicode": "NFKC",
            "max_passes": 1,
            "enable_global": False
        },
        "maps": {
            "scope": maps["scope"],
            "key": maps["key"],
            "value": maps["value"],
            "text": maps["text"],
            "type": {}
        }
    }

# CLI:
# python auto_gen.py <config.json> <out.dict.json> <inputs...> [--base base.dict.json]
argv = sys.argv[1:]
if len(argv) < 3:
    die("Usage: python auto_gen.py <config.json> <out.dict.json> <inputs...> [--base base.dict.json]")

cfg_path, out_path = argv[0], argv[1]
inputs = argv[2:]
base_path = None
if "--base" in inputs:
    i = inputs.index("--base")
    if i + 1 >= len(inputs):
        die("Missing path after --base")
    base_path = inputs[i + 1]
    inputs = inputs[:i]

cfg = read_json(cfg_path)
files = list_files_recursive(inputs)
if not files:
    die("No input files found.")

base_dict = read_json(base_path) if base_path else None
d = build_auto_dict(files, cfg, base_dict)

with open(out_path, "w", encoding="utf-8") as f:
    json.dump(d, f, indent=2, ensure_ascii=False)
    f.write("\n")

count = sum(len(d["maps"][ns]) for ns in ["scope","key","value","text"])
print(f"✅ Wrote {out_path} ({count} entries)")