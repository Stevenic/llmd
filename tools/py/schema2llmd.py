#!/usr/bin/env python3
# ============================================================
# schema2llmd — JSON Schema to LLMD converter (Python)
# Converts JSON Schema definitions into compressed LLMD format
# with llmdc config compression (stopwords, phrase_map, units).
# ============================================================
import argparse
import json
import os
import re
import sys


def die(msg):
    print("error: " + msg, file=sys.stderr)
    sys.exit(1)


def read_json(p):
    with open(p, "r", encoding="utf-8") as f:
        return json.load(f)


# ============================================================
# Config loading (same auto-detect logic as llmdc.py)
# ============================================================
def load_config(config_path):
    if config_path:
        return read_json(config_path)
    for p in ["llmdc.config.json", "config/llmdc.config.json"]:
        if os.path.isfile(p):
            return read_json(p)
    return {}


# ============================================================
# Schema processing
# ============================================================
schema = None
definitions = {}


def resolve_ref(ref):
    if not ref:
        return None
    parts = re.sub(r"^#/", "", ref).split("/")
    obj = schema
    for p in parts:
        if isinstance(obj, dict):
            obj = obj.get(p)
        else:
            return None
    return obj


def collect_properties(node, visited=None):
    if visited is None:
        visited = set()
    if not node:
        return {}
    if "$ref" in node:
        ref_key = node["$ref"]
        if ref_key in visited:
            return {}
        visited.add(ref_key)
        resolved = resolve_ref(ref_key)
        return collect_properties(resolved, visited)
    props = {}
    if "allOf" in node:
        for sub in node["allOf"]:
            props.update(collect_properties(sub, set(visited)))
    if "anyOf" in node:
        for sub in node["anyOf"]:
            props.update(collect_properties(sub, set(visited)))
    if "oneOf" in node:
        for sub in node["oneOf"]:
            props.update(collect_properties(sub, set(visited)))
    if "properties" in node:
        for key, val in node["properties"].items():
            props[key] = val
    return props


def get_required(node):
    if not node:
        return set()
    req = set()
    if "required" in node:
        for r in node["required"]:
            req.add(r)
    if "allOf" in node:
        for sub in node["allOf"]:
            req.update(get_required(sub))
    if "$ref" in node:
        resolved = resolve_ref(node["$ref"])
        if resolved:
            req.update(get_required(resolved))
    return req


def get_type(prop_schema):
    if not prop_schema:
        return "any"
    if "const" in prop_schema:
        return "string"
    if prop_schema.get("type") == "array":
        if "items" in prop_schema:
            item_type = get_item_type_summary(prop_schema["items"])
            return f"array of {item_type}"
        return "array"
    if "type" in prop_schema:
        return prop_schema["type"]
    if "$ref" in prop_schema:
        defn = resolve_ref(prop_schema["$ref"])
        if defn and defn.get("type") in ("string", "number", "boolean", "object"):
            return defn["type"]
        return "string"
    options = prop_schema.get("oneOf") or prop_schema.get("anyOf")
    if options:
        types = set()
        for opt in options:
            if "type" in opt:
                types.add(opt["type"])
            elif "$ref" in opt:
                resolved = resolve_ref(opt["$ref"])
                if resolved and "type" in resolved:
                    types.add(resolved["type"])
                else:
                    types.add("object")
        if len(types) == 1:
            return next(iter(types))
        return "¦".join(sorted(types))
    return "any"


def get_item_type_summary(items):
    if not items:
        return "any"
    if items.get("type") == "string":
        return "string"
    if items.get("type") == "number":
        return "number"
    if "$ref" in items:
        name = items["$ref"].split("/")[-1]
        return clean_def_name(name)
    return "any"


def clean_def_name(name):
    return re.sub(r"^\d+\.", "", name)


def describe_property(prop_schema):
    if not prop_schema:
        return ""
    desc = (prop_schema.get("description") or "").replace("\n", " ")
    desc = re.sub(r"\s+", " ", desc).strip()
    desc = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", desc)
    if len(desc) > 200:
        desc = desc[:197] + "..."
    parts = []
    if desc:
        parts.append(desc)
    if "default" in prop_schema:
        parts.append(f"Default: {json.dumps(prop_schema['default'])}.")
    values = extract_allowed_values(prop_schema)
    if values:
        parts.append(f"[{', '.join(values)}]")
    return " ".join(parts)


def extract_allowed_values(prop_schema):
    if not prop_schema:
        return []
    if "const" in prop_schema:
        return [prop_schema["const"]]
    if "pattern" in prop_schema:
        return extract_values_from_pattern(prop_schema["pattern"])
    if "$ref" in prop_schema:
        resolved = resolve_ref(prop_schema["$ref"])
        if resolved:
            if "pattern" in resolved:
                return extract_values_from_pattern(resolved["pattern"])
            if "oneOf" in resolved or "anyOf" in resolved:
                return extract_allowed_values(resolved)
    options = prop_schema.get("oneOf") or prop_schema.get("anyOf")
    if options:
        vals = []
        for opt in options:
            if "const" in opt:
                vals.append(opt["const"])
            elif "$ref" in opt:
                resolved = resolve_ref(opt["$ref"])
                if resolved and "const" in resolved:
                    vals.append(resolved["const"])
            if "pattern" in opt:
                vals.extend(extract_values_from_pattern(opt["pattern"]))
        return vals
    return []


def extract_values_from_pattern(pattern):
    if not pattern:
        return []
    values = []
    p = re.sub(r"^\^", "", pattern)
    p = re.sub(r"\$\s*$", "", p)
    if p.startswith("(") and p.endswith(")"):
        p = p[1:-1]
    alternatives = split_top_level(p, "|")
    for alt in alternatives:
        cleaned = alt.strip()
        while cleaned.startswith("(") and cleaned.endswith(")") and is_balanced(cleaned[1:-1]):
            cleaned = cleaned[1:-1].strip()
        char_pattern_match = re.match(r"^(\[[A-Za-z0-9]\|[A-Za-z0-9]\])+(\d*)$", cleaned)
        if char_pattern_match:
            val = ""
            for m in re.finditer(r"\[([A-Za-z0-9])\|[A-Za-z0-9]\]", cleaned):
                val += m.group(1)
            trailing = re.search(r"(\d+)$", cleaned)
            if trailing:
                val += trailing.group(1)
            if val:
                values.append(val)
        elif re.match(r"^[A-Za-z0-9_:.*]+$", cleaned):
            values.append(cleaned)
    return values


def split_top_level(s, sep):
    parts = []
    depth = 0
    current = ""
    in_bracket = False
    for c in s:
        if c == "[":
            in_bracket = True
        if c == "]":
            in_bracket = False
        if c == "(" and not in_bracket:
            depth += 1
        if c == ")" and not in_bracket:
            depth -= 1
        if c == sep and depth == 0 and not in_bracket:
            parts.append(current)
            current = ""
        else:
            current += c
    if current:
        parts.append(current)
    return parts


def is_balanced(s):
    depth = 0
    for c in s:
        if c == "(":
            depth += 1
        if c == ")":
            depth -= 1
        if depth < 0:
            return False
    return depth == 0


def is_object_definition(def_name, def_schema):
    if not def_schema:
        return False
    typ = def_schema.get("type", "")
    has_props = "properties" in def_schema
    if typ in ("string", "number", "boolean") and not has_props:
        return False
    if not has_props and "allOf" not in def_schema and typ != "object":
        branches = def_schema.get("anyOf") or def_schema.get("oneOf")
        if branches:
            has_obj = any(
                (resolve_ref(b["$ref"]) or {}).get("properties") is not None
                if "$ref" in b
                else "properties" in b
                for b in branches
            )
            if not has_obj:
                return False
        else:
            return False
    if has_props:
        return True
    if typ == "object":
        return True
    if "allOf" in def_schema:
        return True
    return False


# ============================================================
# Compression (c2 optimizations from llmdc config)
# ============================================================
def _escape_regex(s):
    return re.escape(s)


def compress_description(text, config):
    stopwords = set(s.lower() for s in config.get("stopwords", []))
    protect = set(s.lower() for s in config.get("protect_words", []))
    phrase_map = config.get("phrase_map", {})
    units = config.get("units", {})
    bool_compress = config.get("bool_compress", True)

    body = text

    # Apply phrase map (longest-first, case-insensitive)
    phrases = sorted(phrase_map.keys(), key=len, reverse=True)
    for phrase in phrases:
        body = re.sub(_escape_regex(phrase), phrase_map[phrase], body, flags=re.IGNORECASE)

    # Apply unit normalization
    unit_keys = sorted(units.keys(), key=len, reverse=True)
    for unit in unit_keys:
        body = re.sub(
            r"(\d+)\s+" + _escape_regex(unit),
            r"\1" + units[unit],
            body,
            flags=re.IGNORECASE,
        )
        body = re.sub(_escape_regex(unit), units[unit], body, flags=re.IGNORECASE)

    # Boolean compression
    if bool_compress:
        bool_map = {
            "yes": "Y", "no": "N",
            "true": "T", "false": "F",
            "enabled": "Y", "disabled": "N",
        }
        tokens = body.split()
        body = " ".join(bool_map.get(t.lower(), t) for t in tokens if t)

    # Stopword removal (skip protected words)
    tokens = body.split()
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
    body = " ".join(filtered)

    # Trailing period stripping
    if body.endswith(".") and not body.endswith("...") and \
       not body.endswith("e.g.") and not body.endswith("i.e.") and not body.endswith("etc."):
        body = body[:-1]

    # Clean up extra whitespace
    body = re.sub(r"\s+", " ", body).strip()

    return body


# ============================================================
# LLMD Emission
# ============================================================
def generate_llmd(config):
    object_defs = {}
    for name, def_schema in definitions.items():
        if is_object_definition(name, def_schema):
            object_defs[name] = def_schema

    # Collect all unique properties across all objects
    all_properties = {}  # propName -> { schemas: [], objects: [] }
    for def_name, def_schema in object_defs.items():
        props = collect_properties(def_schema)
        for prop_name, prop_schema in props.items():
            if prop_name not in all_properties:
                all_properties[prop_name] = {"schemas": [], "objects": []}
            all_properties[prop_name]["schemas"].append(prop_schema)
            all_properties[prop_name]["objects"].append(def_name)

    lines = []

    # --- Objects section ---
    lines.append("@Objects.Properties")
    lines.append("Required properties marked with `!`.")

    for def_name, def_schema in object_defs.items():
        props = collect_properties(def_schema)
        required = get_required(def_schema)
        prop_names = list(props.keys())

        prop_list = ", ".join(
            f"{p}!" if p in required else p
            for p in prop_names
        )

        lines.append(f":{clean_def_name(def_name)}.properties={prop_list}")

    # --- Properties section ---
    lines.append("@Properties")

    documented = set()
    for prop_name, info in all_properties.items():
        if prop_name in documented:
            continue
        documented.add(prop_name)

        # Pick the richest schema (longest description)
        best_schema = info["schemas"][0]
        best_desc = ""
        for s in info["schemas"]:
            d = describe_property(s)
            if len(d) > len(best_desc):
                best_desc = d
                best_schema = s

        type_str = get_type(best_schema)
        description = best_desc or describe_property(best_schema)

        # Apply compression to description text
        description = compress_description(description, config)

        lines.append(f"-{prop_name} ({type_str}): {description}")

    return "\n".join(lines) + "\n"


# ============================================================
# Main
# ============================================================
def main():
    parser = argparse.ArgumentParser(
        prog="schema2llmd",
        description="schema2llmd — JSON Schema to LLMD converter",
    )
    parser.add_argument("schema", help="Input JSON Schema file")
    parser.add_argument("-o", "--output", help="Output file (default: stdout)")
    parser.add_argument("--config", help="Config file path (auto-detect llmdc.config.json)")

    args = parser.parse_args()

    global schema, definitions

    # Load schema
    input_path = os.path.abspath(args.schema)
    schema = read_json(input_path)
    definitions = schema.get("definitions", {})

    # Load config
    config = load_config(args.config)

    # Generate LLMD
    result = generate_llmd(config)

    if args.output:
        with open(args.output, "w", encoding="utf-8") as f:
            f.write(result)
        tokens = len([t for t in result.split() if t])
        print(f"schema2llmd: {input_path} -> {args.output} (~{tokens} tokens)", file=sys.stderr)
    else:
        sys.stdout.buffer.write(result.encode("utf-8"))


if __name__ == "__main__":
    main()
