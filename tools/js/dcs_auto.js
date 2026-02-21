#!/usr/bin/env node
import fs from "fs";
import path from "path";

function die(msg) { console.error(msg); process.exit(1); }

function readJSON(p) {
  return JSON.parse(fs.readFileSync(p, "utf8"));
}

function listFilesRecursive(inputs) {
  const out = [];
  for (const p of inputs) {
    const st = fs.statSync(p);
    if (st.isDirectory()) {
      const entries = fs.readdirSync(p).map(e => path.join(p, e));
      out.push(...listFilesRecursive(entries));
    } else if (st.isFile()) {
      out.push(p);
    }
  }
  // Deterministic order
  out.sort((a,b) => a.localeCompare(b));
  return out;
}

function base36(n) {
  // Deterministic lowercase base36 without prefixes
  return n.toString(36);
}

const RESERVED_PREFIXES = ["~", "@", ":", ">", "::", "->", "<-", "=", "<<<", ">>>"];

function isReservedPrefix(s) {
  return RESERVED_PREFIXES.some(p => s.startsWith(p));
}

function normalizeUnicodeNFKC(s) {
  // Node supports normalize()
  return s.normalize("NFKC");
}

function stripBlocksLLMD(lines) {
  const out = [];
  let inBlock = false;
  for (const line of lines) {
    if (!inBlock && line.startsWith("<<<")) { inBlock = true; continue; }
    if (inBlock) {
      if (line.startsWith(">>>")) inBlock = false;
      continue;
    }
    out.push(line);
  }
  return out;
}

// Very lightweight canonicalization: Markdown -> LLMD-like lines (c1-ish), without dict.
// Enough for AUTO token extraction; conservative.
function canonicalizeToLLMDLines(text, ext, cfg) {
  text = normalizeUnicodeNFKC(text).replace(/\r\n/g, "\n");
  const rawLines = text.split("\n");

  // Ignore fenced code blocks for AUTO by default
  let lines = [];
  if (cfg.ignore_blocks) {
    let inFence = false;
    for (const ln of rawLines) {
      const t = ln.trim();
      if (t.startsWith("```")) { inFence = !inFence; continue; }
      if (inFence) continue;
      lines.push(ln);
    }
  } else {
    lines = rawLines;
  }

  const out = [];
  let currentScope = null;

  const emitScope = (scope) => {
    if (!scope) return;
    if (scope !== currentScope) {
      out.push("@" + scope);
      currentScope = scope;
    }
  };

  const normScope = (s) => {
    // normalize like LLMD: trim, spaces->_, lowercase
    return s.trim().replace(/\s+/g, "_").toLowerCase();
  };

  const normKey = (s) => {
    return s.trim().toLowerCase().replace(/\s+/g, "_").replace(/[^a-z0-9_]/g, "");
  };

  for (const ln0 of lines) {
    const ln = ln0.trim();
    if (!ln) continue;

    // Markdown headings
    const mHead = ln.match(/^(#{1,6})\s+(.*)$/);
    if (mHead) {
      const name = normScope(mHead[2]);
      emitScope(name);
      continue;
    }

    // Bullet list
    const mBul = ln.match(/^[-*+]\s+(.*)$/);
    if (mBul) {
      if (!currentScope) emitScope("root");
      out.push(">" + mBul[1].trim());
      continue;
    }

    // Numbered list
    const mNum = ln.match(/^\d+\.\s+(.*)$/);
    if (mNum) {
      if (!currentScope) emitScope("root");
      out.push(">" + mNum[1].trim());
      continue;
    }

    // Simple Key: Value line => attribute candidate
    const mKV = ln.match(/^([A-Za-z0-9 _-]{2,64})\s*:\s*(.+)$/);
    if (mKV) {
      if (!currentScope) emitScope("root");
      const k = normKey(mKV[1]);
      const v = mKV[2].trim();
      if (k) out.push(":" + k + "=" + v);
      else out.push(">" + ln);
      continue;
    }

    // Paragraph => item
    if (!currentScope) emitScope("root");
    out.push(">" + ln);
  }

  return out;
}

function tokenizeTextToTokens(s) {
  // Token-mode: split on whitespace; then normalize token chars
  // Keep A-Za-z0-9_.- and hyphen/underscore; strip leading/trailing non-kept.
  const raw = s.split(/\s+/).filter(Boolean);
  const keepRe = /^[A-Za-z0-9._-]+$/;
  const cleaned = [];
  for (let tok of raw) {
    tok = tok.toLowerCase();
    tok = tok.replace(/^[^a-z0-9._-]+/g, "").replace(/[^a-z0-9._-]+$/g, "");
    if (!tok) continue;
    // If token has internal disallowed chars, split further conservatively
    if (!keepRe.test(tok)) {
      const parts = tok.split(/[^a-z0-9._-]+/g).filter(Boolean);
      cleaned.push(...parts);
    } else {
      cleaned.push(tok);
    }
  }
  return cleaned;
}

function isNumericLike(tok) {
  return /^[0-9]+([./:_-][0-9]+)*$/.test(tok);
}
function isUrlEmailLike(tok) {
  return /^https?:\/\//.test(tok) || tok.includes("@");
}

function estTokensFromString(s) {
  // deterministic heuristic: sum ceil(len(token)/4) over whitespace tokens
  const toks = s.split(/\s+/).filter(Boolean);
  let sum = 0;
  for (const t of toks) sum += Math.ceil(t.length / 4);
  return sum;
}

function splitEnumValues(v) {
  // Split on | or , for enum-like values
  return v.split(/[|,]/g).map(x => x.trim()).filter(Boolean);
}

function buildAutoDict(files, cfg, baseDict) {
  const protect = new Set([...(cfg.protect?.negations ?? []), ...(cfg.protect?.modals ?? [])].map(s => s.toLowerCase()));
  const stop = new Set((cfg.stoplist ?? []).map(s => s.toLowerCase()));

  const namespacesEnabled = cfg.namespaces ?? { scope:true, key:true, value:true, text:true };

  const freqByNS = {
    scope: new Map(),
    key: new Map(),
    value: new Map(),
    text: new Map()
  };

  const allSourceTokens = new Set();

  function bump(map, tok) {
    map.set(tok, (map.get(tok) ?? 0) + 1);
    allSourceTokens.add(tok);
  }

  for (const fp of files) {
    const ext = path.extname(fp).toLowerCase();
    const content = fs.readFileSync(fp, "utf8");
    const lines = canonicalizeToLLMDLines(content, ext, cfg);

    let ll = lines;
    if (cfg.ignore_blocks) ll = stripBlocksLLMD(ll);

    for (const line of ll) {
      if (line.startsWith("~")) continue;

      if (line.startsWith("@") && namespacesEnabled.scope) {
        const scope = line.slice(1).trim().toLowerCase();
        if (scope) bump(freqByNS.scope, scope);
        continue;
      }

      if (line.startsWith(":")) {
        // :k=v k=v ...
        const body = line.slice(1).trim();
        const pairs = body.split(/\s+/).filter(Boolean);
        for (const p of pairs) {
          const idx = p.indexOf("=");
          if (idx <= 0) continue;
          const k = p.slice(0, idx).toLowerCase();
          const v = p.slice(idx + 1);

          if (namespacesEnabled.key && k) bump(freqByNS.key, k);

          if (namespacesEnabled.value && v) {
            // Only consider enum-like pieces
            for (const piece0 of splitEnumValues(v)) {
              const piece = piece0.toLowerCase();
              if (!piece) continue;
              if (!/^[a-z][a-z0-9._-]*$/.test(piece)) continue;
              if (isNumericLike(piece) || isUrlEmailLike(piece)) continue;
              bump(freqByNS.value, piece);
            }
          }
        }
        continue;
      }

      if (line.startsWith(">") && namespacesEnabled.text) {
        const text = line.slice(1).trim();
        for (const tok of tokenizeTextToTokens(text)) bump(freqByNS.text, tok);
        continue;
      }

      // relations and blocks ignored for AUTO v1.0 in this reference
    }
  }

  // Build reserved sets from base dictionary
  const reservedAliases = new Set();
  const reservedKeys = new Set();
  if (baseDict?.maps) {
    for (const ns of Object.keys(baseDict.maps)) {
      const m = baseDict.maps[ns] || {};
      for (const k of Object.keys(m)) reservedKeys.add(k.toLowerCase());
      for (const v of Object.values(m)) reservedAliases.add(String(v).toLowerCase());
    }
  }

  // Candidate collection
  const candidates = []; // {ns, tok, freq, gain}
  const minLen = cfg.min_len ?? 6;
  const minFreq = cfg.min_freq ?? 3;
  const minGain = cfg.min_gain ?? 10;

  function considerNS(ns, map) {
    for (const [tok, f] of map.entries()) {
      if (tok.length < minLen) continue;
      if (f < minFreq) continue;
      if (stop.has(tok) || protect.has(tok)) continue;
      if (isNumericLike(tok) || isUrlEmailLike(tok)) continue;
      if (reservedKeys.has(tok)) continue;

      // Estimate savings using ns_base36 aliases (approx 2–4 chars): compute later after alias chosen.
      // For ranking before alias assignment, assume alias length ~ 3.
      const assumedAlias = (ns[0] + "0"); // conservative short
      const gainPerUse = Math.max(0, estTokensFromString(tok) - estTokensFromString(assumedAlias));
      const totalGain = f * gainPerUse;

      if (totalGain >= minGain) {
        candidates.push({ ns, tok, freq: f, totalGain });
      }
    }
  }

  if (namespacesEnabled.scope) considerNS("scope", freqByNS.scope);
  if (namespacesEnabled.key)   considerNS("key",   freqByNS.key);
  if (namespacesEnabled.value) considerNS("value", freqByNS.value);
  if (namespacesEnabled.text)  considerNS("text",  freqByNS.text);

  // Namespace priority: key > scope > value > text
  const nsPri = { key: 1, scope: 2, value: 3, text: 4 };

  // If same token appears in multiple namespaces, keep only best (highest priority), deterministic.
  // Do this by grouping by tok.
  const bestByTok = new Map();
  for (const c of candidates) {
    const prev = bestByTok.get(c.tok);
    if (!prev) { bestByTok.set(c.tok, c); continue; }
    const a = prev, b = c;
    const better =
      (nsPri[b.ns] < nsPri[a.ns]) ||
      (nsPri[b.ns] === nsPri[a.ns] && b.totalGain > a.totalGain) ||
      (nsPri[b.ns] === nsPri[a.ns] && b.totalGain === a.totalGain && b.freq > a.freq) ||
      (nsPri[b.ns] === nsPri[a.ns] && b.totalGain === a.totalGain && b.freq === a.freq && b.tok.localeCompare(a.tok) < 0);
    if (better) bestByTok.set(c.tok, c);
  }

  let final = Array.from(bestByTok.values());

  // Rank: totalGain desc, len(tok) desc, freq desc, tok asc
  final.sort((a,b) =>
    (b.totalGain - a.totalGain) ||
    (b.tok.length - a.tok.length) ||
    (b.freq - a.freq) ||
    a.tok.localeCompare(b.tok)
  );

  // Cap entries
  const maxEntries = cfg.max_entries ?? 256;
  final = final.slice(0, maxEntries);

  // Assign aliases (ns_base36)
  const usedAliases = new Set([...reservedAliases].map(x => x.toLowerCase()));
  const usedSourceTokens = cfg.strict_no_alias_equals_source_token ? allSourceTokens : new Set();

  function nextAlias(ns, i) {
    const prefix = ns === "scope" ? "s" :
                   ns === "key"   ? "k" :
                   ns === "value" ? "v" : "t";
    return prefix + base36(i);
  }

  const maps = { scope:{}, key:{}, value:{}, text:{}, type:{} };

  const counters = { scope:0, key:0, value:0, text:0 };
  for (const c of final) {
    let i = counters[c.ns];
    while (true) {
      const a = nextAlias(c.ns, i).toLowerCase();
      if (isReservedPrefix(a)) { i++; continue; }
      if (usedAliases.has(a)) { i++; continue; }
      if (cfg.strict_no_alias_equals_source_token && usedSourceTokens.has(a)) { i++; continue; }
      // accept
      maps[c.ns][c.tok] = a;
      usedAliases.add(a);
      counters[c.ns] = i + 1;
      break;
    }
  }

  // For stable output, sort keys in each namespace lexicographically (deterministic)
  function sortObj(o) {
    const keys = Object.keys(o).sort((a,b) => a.localeCompare(b));
    const n = {};
    for (const k of keys) n[k] = o[k];
    return n;
  }

  const dictOut = {
    version: "1.0",
    policy: {
      case: "smart",
      match: "token",
      longest_match: true,
      normalize_unicode: "NFKC",
      max_passes: 1,
      enable_global: false
    },
    maps: {
      scope: sortObj(maps.scope),
      key: sortObj(maps.key),
      value: sortObj(maps.value),
      text: sortObj(maps.text),
      type: {}
    }
  };

  return dictOut;
}

// CLI
// node auto_gen.js <config.json> <out.dict.json> <inputs...> [--base base.dict.json]
const argv = process.argv.slice(2);
if (argv.length < 3) {
  die("Usage: node auto_gen.js <config.json> <out.dict.json> <inputs...> [--base base.dict.json]");
}
const cfgPath = argv[0];
const outPath = argv[1];

let basePath = null;
let inputs = argv.slice(2);
const baseIdx = inputs.indexOf("--base");
if (baseIdx >= 0) {
  basePath = inputs[baseIdx + 1];
  inputs = inputs.slice(0, baseIdx);
}

const cfg = readJSON(cfgPath);
const files = listFilesRecursive(inputs);
if (files.length === 0) die("No input files found.");

const baseDict = basePath ? readJSON(basePath) : null;
const dict = buildAutoDict(files, cfg, baseDict);

fs.writeFileSync(outPath, JSON.stringify(dict, null, 2) + "\n", "utf8");
console.log(`✅ Wrote ${outPath} (${Object.keys(dict.maps.scope).length + Object.keys(dict.maps.key).length + Object.keys(dict.maps.value).length + Object.keys(dict.maps.text).length} entries)`);