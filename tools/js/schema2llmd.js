#!/usr/bin/env node
// ============================================================
// schema2llmd — JSON Schema to LLMD converter
// Converts JSON Schema definitions into compressed LLMD format
// with llmdc config compression (stopwords, phrase_map, units).
// ============================================================
import fs from "fs";
import path from "path";

function die(msg) { console.error("error: " + msg); process.exit(1); }
function readJSON(p) { return JSON.parse(fs.readFileSync(p, "utf8")); }

// ============================================================
// CLI
// ============================================================
function parseArgs(argv) {
  const args = { schema: null, output: null, configPath: null };
  let i = 0;
  while (i < argv.length) {
    const a = argv[i];
    if (a === "-o" || a === "--output") { args.output = argv[++i]; }
    else if (a === "--config") { args.configPath = argv[++i]; }
    else if (a === "-h" || a === "--help") {
      console.log(`schema2llmd — JSON Schema to LLMD converter

Usage: node schema2llmd.js <schema.json> [options]

Options:
  -o, --output <path>   Output file (default: stdout)
  --config <path>       Config file path (auto-detect llmdc.config.json)
  -h, --help            Show this help`);
      process.exit(0);
    }
    else if (a.startsWith("-")) { die(`unknown option: ${a}`); }
    else if (!args.schema) { args.schema = a; }
    else { die(`unexpected argument: ${a}`); }
    i++;
  }
  return args;
}

// ============================================================
// Config loading (same auto-detect logic as llmdc.js)
// ============================================================
function loadConfig(configPath) {
  if (configPath) return readJSON(configPath);
  const defaults = ["llmdc.config.json", "config/llmdc.config.json"];
  for (const p of defaults) {
    try { return readJSON(p); } catch {}
  }
  return {};
}

// ============================================================
// Schema processing (from compress-schema.js)
// ============================================================
let schema;
let definitions;

function resolveRef(ref) {
  if (!ref) return null;
  const parts = ref.replace(/^#\//, '').split('/');
  let obj = schema;
  for (const p of parts) {
    obj = obj && obj[p];
  }
  return obj || null;
}

function collectProperties(node, visited = new Set()) {
  if (!node) return {};
  if (node.$ref) {
    const refKey = node.$ref;
    if (visited.has(refKey)) return {};
    visited.add(refKey);
    const resolved = resolveRef(refKey);
    return collectProperties(resolved, visited);
  }
  let props = {};
  if (node.allOf) {
    for (const sub of node.allOf) {
      Object.assign(props, collectProperties(sub, new Set(visited)));
    }
  }
  if (node.anyOf) {
    for (const sub of node.anyOf) {
      Object.assign(props, collectProperties(sub, new Set(visited)));
    }
  }
  if (node.oneOf) {
    for (const sub of node.oneOf) {
      Object.assign(props, collectProperties(sub, new Set(visited)));
    }
  }
  if (node.properties) {
    for (const [key, val] of Object.entries(node.properties)) {
      props[key] = val;
    }
  }
  return props;
}

function getRequired(node) {
  if (!node) return [];
  const req = new Set();
  if (node.required) {
    for (const r of node.required) req.add(r);
  }
  if (node.allOf) {
    for (const sub of node.allOf) {
      for (const r of getRequired(sub)) req.add(r);
    }
  }
  if (node.$ref) {
    const resolved = resolveRef(node.$ref);
    if (resolved) {
      for (const r of getRequired(resolved)) req.add(r);
    }
  }
  return req;
}

function getType(propSchema) {
  if (!propSchema) return 'any';
  if (propSchema.const) return 'string';
  if (propSchema.type === 'array') {
    if (propSchema.items) {
      const itemType = getItemTypeSummary(propSchema.items);
      return `array of ${itemType}`;
    }
    return 'array';
  }
  if (propSchema.type) return propSchema.type;
  if (propSchema.$ref) {
    const def = resolveRef(propSchema.$ref);
    if (def && def.type === 'string') return 'string';
    if (def && def.type === 'number') return 'number';
    if (def && def.type === 'boolean') return 'boolean';
    if (def && def.type === 'object') return 'object';
    return 'string';
  }
  if (propSchema.oneOf || propSchema.anyOf) {
    const options = propSchema.oneOf || propSchema.anyOf;
    const types = new Set();
    for (const opt of options) {
      if (opt.type) types.add(opt.type);
      else if (opt.$ref) {
        const resolved = resolveRef(opt.$ref);
        if (resolved && resolved.type) types.add(resolved.type);
        else types.add('object');
      }
    }
    if (types.size === 1) return [...types][0];
    return [...types].join('¦');
  }
  return 'any';
}

function getItemTypeSummary(items) {
  if (!items) return 'any';
  if (items.type === 'string') return 'string';
  if (items.type === 'number') return 'number';
  if (items.$ref) {
    const name = items.$ref.split('/').pop();
    return cleanDefName(name);
  }
  return 'any';
}

function cleanDefName(name) {
  return name.replace(/^\d+\./, '');
}

function describeProperty(propSchema) {
  if (!propSchema) return '';
  let desc = (propSchema.description || '').replace(/\n/g, ' ').replace(/\s+/g, ' ').trim();
  desc = desc.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');
  if (desc.length > 200) {
    desc = desc.substring(0, 197) + '...';
  }
  let parts = [];
  if (desc) parts.push(desc);
  if (propSchema.default !== undefined) {
    parts.push(`Default: ${JSON.stringify(propSchema.default)}.`);
  }
  const values = extractAllowedValues(propSchema);
  if (values.length > 0) {
    parts.push(`[${values.join(', ')}]`);
  }
  return parts.join(' ');
}

function extractAllowedValues(propSchema) {
  if (!propSchema) return [];
  if (propSchema.const) return [propSchema.const];
  if (propSchema.pattern) {
    return extractValuesFromPattern(propSchema.pattern);
  }
  if (propSchema.$ref) {
    const resolved = resolveRef(propSchema.$ref);
    if (resolved && resolved.pattern) {
      return extractValuesFromPattern(resolved.pattern);
    }
    if (resolved && (resolved.oneOf || resolved.anyOf)) {
      return extractAllowedValues(resolved);
    }
  }
  if (propSchema.oneOf || propSchema.anyOf) {
    const options = propSchema.oneOf || propSchema.anyOf;
    const vals = [];
    for (const opt of options) {
      if (opt.const) vals.push(opt.const);
      else if (opt.$ref) {
        const resolved = resolveRef(opt.$ref);
        if (resolved && resolved.const) vals.push(resolved.const);
      }
      if (opt.pattern) {
        vals.push(...extractValuesFromPattern(opt.pattern));
      }
    }
    return vals;
  }
  return [];
}

function extractValuesFromPattern(pattern) {
  if (!pattern) return [];
  const values = [];
  let p = pattern.replace(/^\^/, '').replace(/\$\s*$/, '');
  if (p.startsWith('(') && p.endsWith(')')) {
    p = p.slice(1, -1);
  }
  const alternatives = splitTopLevel(p, '|');
  for (const alt of alternatives) {
    let cleaned = alt.trim();
    while (cleaned.startsWith('(') && cleaned.endsWith(')') && isBalanced(cleaned.slice(1, -1))) {
      cleaned = cleaned.slice(1, -1).trim();
    }
    const charPatternMatch = cleaned.match(/^(\[[A-Za-z0-9]\|[A-Za-z0-9]\])+(\d*)$/);
    if (charPatternMatch) {
      let val = '';
      const charGroups = cleaned.matchAll(/\[([A-Za-z0-9])\|[A-Za-z0-9]\]/g);
      for (const m of charGroups) {
        val += m[1];
      }
      const trailingDigits = cleaned.match(/(\d+)$/);
      if (trailingDigits) {
        val += trailingDigits[1];
      }
      if (val) values.push(val);
    } else if (/^[A-Za-z0-9_:.*]+$/.test(cleaned)) {
      values.push(cleaned);
    }
  }
  return values;
}

function splitTopLevel(str, sep) {
  const parts = [];
  let depth = 0;
  let current = '';
  let inBracket = false;
  for (let i = 0; i < str.length; i++) {
    const c = str[i];
    if (c === '[') inBracket = true;
    if (c === ']') inBracket = false;
    if (c === '(' && !inBracket) depth++;
    if (c === ')' && !inBracket) depth--;
    if (c === sep && depth === 0 && !inBracket) {
      parts.push(current);
      current = '';
    } else {
      current += c;
    }
  }
  if (current) parts.push(current);
  return parts;
}

function isBalanced(str) {
  let depth = 0;
  for (const c of str) {
    if (c === '(') depth++;
    if (c === ')') depth--;
    if (depth < 0) return false;
  }
  return depth === 0;
}

function isObjectDefinition(defName, defSchema) {
  if (!defSchema) return false;
  if (defSchema.type === 'string' && !defSchema.properties) return false;
  if (defSchema.type === 'number' && !defSchema.properties) return false;
  if (defSchema.type === 'boolean' && !defSchema.properties) return false;
  if (!defSchema.properties && !defSchema.allOf && defSchema.type !== 'object') {
    if (defSchema.anyOf || defSchema.oneOf) {
      const branches = defSchema.anyOf || defSchema.oneOf;
      const hasObjBranch = branches.some(b => {
        if (b.$ref) {
          const resolved = resolveRef(b.$ref);
          return resolved && resolved.properties;
        }
        return b.properties;
      });
      if (!hasObjBranch) return false;
    } else {
      return false;
    }
  }
  if (defSchema.properties) return true;
  if (defSchema.type === 'object') return true;
  if (defSchema.allOf) return true;
  return false;
}

// ============================================================
// Compression (c2 optimizations from llmdc.js)
// ============================================================
function escapeRegex(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function compressDescription(text, config) {
  const stopwords = new Set((config.stopwords || []).map(s => s.toLowerCase()));
  const protect = new Set((config.protect_words || []).map(s => s.toLowerCase()));
  const phraseMap = config.phrase_map || {};
  const units = config.units || {};
  const boolCompress = config.bool_compress !== false;

  let body = text;

  // Apply phrase map (longest-first, case-insensitive)
  const phrases = Object.keys(phraseMap).sort((a, b) => b.length - a.length);
  for (const phrase of phrases) {
    const re = new RegExp(escapeRegex(phrase), "gi");
    body = body.replace(re, phraseMap[phrase]);
  }

  // Apply unit normalization
  const unitKeys = Object.keys(units).sort((a, b) => b.length - a.length);
  for (const unit of unitKeys) {
    const reNum = new RegExp("(\\d+)\\s+" + escapeRegex(unit), "gi");
    body = body.replace(reNum, "$1" + units[unit]);
    const re = new RegExp(escapeRegex(unit), "gi");
    body = body.replace(re, units[unit]);
  }

  // Boolean compression (Yes/No -> Y/N, true/false -> T/F, enabled/disabled -> Y/N)
  if (boolCompress) {
    const BOOL_MAP = {
      "yes": "Y", "no": "N",
      "true": "T", "false": "F",
      "enabled": "Y", "disabled": "N",
    };
    const tokens = body.split(/\s+/).filter(Boolean);
    body = tokens.map(t => {
      const low = t.toLowerCase();
      return BOOL_MAP[low] ?? t;
    }).join(' ');
  }

  // Stopword removal (skip protected words)
  const tokens = body.split(/\s+/).filter(Boolean);
  const filtered = tokens.filter(t => {
    const low = t.toLowerCase().replace(/[^a-z]/g, "");
    if (!low) return true;
    if (protect.has(low)) return true;
    return !stopwords.has(low);
  });
  body = filtered.join(' ');

  // Trailing period stripping (preserve ..., e.g., i.e., etc.)
  if (body.endsWith('.') && !body.endsWith('...') &&
      !body.endsWith('e.g.') && !body.endsWith('i.e.') && !body.endsWith('etc.')) {
    body = body.slice(0, -1);
  }

  // Clean up extra whitespace
  body = body.replace(/\s+/g, ' ').trim();

  return body;
}

// ============================================================
// LLMD Emission
// ============================================================
function generateLLMD(config) {
  const objectDefs = {};
  for (const [name, defSchema] of Object.entries(definitions)) {
    if (isObjectDefinition(name, defSchema)) {
      objectDefs[name] = defSchema;
    }
  }

  // Collect all unique properties across all objects
  const allProperties = {}; // propName -> { schemas[], objects[] }
  for (const [defName, defSchema] of Object.entries(objectDefs)) {
    const props = collectProperties(defSchema);
    for (const [propName, propSchema] of Object.entries(props)) {
      if (!allProperties[propName]) {
        allProperties[propName] = { schemas: [], objects: [] };
      }
      allProperties[propName].schemas.push(propSchema);
      allProperties[propName].objects.push(defName);
    }
  }

  const lines = [];

  // --- Objects section ---
  lines.push('@Objects.Properties');
  lines.push('Required properties marked with `!`.');

  for (const [defName, defSchema] of Object.entries(objectDefs)) {
    const props = collectProperties(defSchema);
    const required = getRequired(defSchema);
    const propNames = Object.keys(props);

    const propList = propNames.map(p => {
      if (required.has(p)) return `${p}!`;
      return p;
    }).join(', ');

    lines.push(`:${cleanDefName(defName)}.properties=${propList}`);
  }

  // --- Properties section ---
  lines.push('@Properties');

  const documentedProps = new Set();
  for (const [propName, info] of Object.entries(allProperties)) {
    if (documentedProps.has(propName)) continue;
    documentedProps.add(propName);

    // Pick the richest schema (longest description)
    let bestSchema = info.schemas[0];
    let bestDesc = '';
    for (const s of info.schemas) {
      const d = describeProperty(s);
      if (d.length > bestDesc.length) {
        bestDesc = d;
        bestSchema = s;
      }
    }

    const typeStr = getType(bestSchema);
    let description = bestDesc || describeProperty(bestSchema);

    // Apply compression to description text
    description = compressDescription(description, config);

    lines.push(`-${propName} (${typeStr}): ${description}`);
  }

  return lines.join('\n') + '\n';
}

// ============================================================
// Main
// ============================================================
function main() {
  const argv = process.argv.slice(2);
  if (argv.length === 0) {
    die("usage: node schema2llmd.js <schema.json> [options]\nRun with --help for details.");
  }

  const args = parseArgs(argv);
  if (!args.schema) die("no schema file specified");

  // Load schema
  const inputPath = path.resolve(args.schema);
  schema = JSON.parse(fs.readFileSync(inputPath, 'utf-8'));
  definitions = schema.definitions || {};

  // Load config
  const config = loadConfig(args.configPath);

  // Generate LLMD
  const result = generateLLMD(config);

  if (args.output) {
    fs.writeFileSync(args.output, result, 'utf8');
    const tokens = result.split(/\s+/).filter(Boolean).length;
    console.error(`schema2llmd: ${inputPath} -> ${args.output} (~${tokens} tokens)`);
  } else {
    process.stdout.write(result);
  }
}

main();
