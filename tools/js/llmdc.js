#!/usr/bin/env node
// ============================================================
// llmdc — LLMD Compiler (JavaScript)
// Spec: LLMD v0.1 + Compiler Design v0.1 + DCS v1.0
// ============================================================
import fs from "fs";
import path from "path";

function die(msg) { console.error("error: " + msg); process.exit(1); }
function readJSON(p) { return JSON.parse(fs.readFileSync(p, "utf8")); }

function listFiles(inputs) {
  const out = [];
  for (const p of inputs) {
    const st = fs.statSync(p);
    if (st.isDirectory()) {
      const entries = fs.readdirSync(p).map(e => path.join(p, e));
      out.push(...listFiles(entries));
    } else if (st.isFile() && /\.(md|markdown|llmd)$/i.test(p)) {
      out.push(p);
    }
  }
  out.sort((a, b) => a.localeCompare(b));
  return out;
}

// ============================================================
// Stage 0: Normalize
// ============================================================
function stage0(text) {
  text = text.normalize("NFKC");
  text = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  return text.split("\n").map(l => l.trimEnd());
}

// ============================================================
// Stage 1: Extract / Protect Blocks
// ============================================================
function stage1(lines) {
  const blocks = [];
  const out = [];
  let inBlock = false, lang = "", buf = [], fence = "";

  for (const line of lines) {
    if (!inBlock) {
      const m = line.match(/^(`{3,})(\w*)\s*$/);
      if (m) {
        inBlock = true;
        fence = m[1];
        lang = m[2] || "";
        buf = [];
        continue;
      }
      out.push(line);
    } else {
      if (line.trimEnd() === fence) {
        const idx = blocks.length;
        blocks.push({ index: idx, lang, content: buf.join("\n") });
        out.push(`\u27E6BLOCK:${idx}\u27E7`);
        inBlock = false;
        fence = "";
        lang = "";
        buf = [];
      } else {
        buf.push(line);
      }
    }
  }
  if (inBlock && buf.length > 0) {
    const idx = blocks.length;
    blocks.push({ index: idx, lang, content: buf.join("\n") });
    out.push(`\u27E6BLOCK:${idx}\u27E7`);
  }
  return { lines: out, blocks };
}

// ============================================================
// Stage 2: Parse to IR
// ============================================================
// IR types: heading, paragraph, list_item, table, kv, blank, block_ref

const RE_HEADING = /^(#{1,6})\s+(.+)$/;
const RE_UL = /^(\s*)([-*+])\s+(.+)$/;
const RE_OL = /^(\s*)(\d+)\.\s+(.+)$/;
const RE_BLOCK_REF = /^\u27E6BLOCK:(\d+)\u27E7$/;
const RE_KV = /^([A-Za-z][A-Za-z0-9 _-]{0,63})\s*:\s+(.+)$/;

function isStructural(line) {
  const t = line.trim();
  if (!t) return true;
  if (RE_HEADING.test(t)) return true;
  if (RE_UL.test(t) || RE_OL.test(t)) return true;
  if (RE_BLOCK_REF.test(t)) return true;
  if (t.includes("|")) return true; // potential table
  if (RE_KV.test(t) && !/^https?:\/\//.test(t)) return true;
  return false;
}

function parseTableRows(lines, startIdx) {
  const rows = [];
  let i = startIdx;
  const parseRow = (r) => {
    let cells = r.split("|").map(c => c.trim());
    if (cells.length > 0 && cells[0] === "") cells = cells.slice(1);
    if (cells.length > 0 && cells[cells.length - 1] === "") cells = cells.slice(0, -1);
    return cells;
  };
  // header
  rows.push(parseRow(lines[i]));
  i += 2; // skip delimiter
  while (i < lines.length) {
    const t = lines[i].trim();
    if (!t || !t.includes("|")) break;
    rows.push(parseRow(t));
    i++;
  }
  return { rows, nextIdx: i };
}

function stage2(lines) {
  const ir = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];
    const t = line.trim();

    if (t === "") { ir.push({ type: "blank" }); i++; continue; }

    const blockM = t.match(RE_BLOCK_REF);
    if (blockM) { ir.push({ type: "block_ref", index: parseInt(blockM[1]) }); i++; continue; }

    const headM = t.match(RE_HEADING);
    if (headM) { ir.push({ type: "heading", level: headM[1].length, text: headM[2].trim() }); i++; continue; }

    // Table: line with |, next line is delimiter
    if (t.includes("|") && i + 1 < lines.length) {
      const next = lines[i + 1].trim();
      if (/^\|?[\s:-]+\|/.test(next) && /---/.test(next)) {
        const { rows, nextIdx } = parseTableRows(lines, i);
        ir.push({ type: "table", rows });
        i = nextIdx;
        continue;
      }
    }

    const ulM = line.match(RE_UL);
    if (ulM) {
      ir.push({ type: "list_item", depth: Math.floor(ulM[1].length / 2), text: ulM[3].trim(), ordered: false });
      i++; continue;
    }

    const olM = line.match(RE_OL);
    if (olM) {
      ir.push({ type: "list_item", depth: Math.floor(olM[1].length / 2), text: olM[3].trim(), ordered: true });
      i++; continue;
    }

    const kvM = t.match(RE_KV);
    if (kvM && !/^https?:\/\//.test(t)) {
      ir.push({ type: "kv", key: kvM[1], value: kvM[2].trim() });
      i++; continue;
    }

    // Paragraph: merge consecutive non-structural lines
    const paraLines = [t];
    i++;
    while (i < lines.length) {
      const nl = lines[i].trim();
      if (!nl || isStructural(lines[i])) break;
      paraLines.push(nl);
      i++;
    }
    ir.push({ type: "paragraph", text: paraLines.join(" ") });
  }
  return ir;
}

// ============================================================
// Inline markdown processing
// ============================================================
function stripInlineMarkdown(text) {
  text = text.replace(/\*\*(.+?)\*\*/g, "$1");
  text = text.replace(/__(.+?)__/g, "$1");
  text = text.replace(/(?<!\*)\*(?!\*)(.+?)(?<!\*)\*(?!\*)/g, "$1");
  text = text.replace(/`([^`]+)`/g, "$1");
  text = text.replace(/~~(.+?)~~/g, "$1");
  return text;
}

function processLinks(text, keepUrls) {
  if (keepUrls) {
    text = text.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, "$1<$2>");
    text = text.replace(/\[([^\]]*)\]\(([^)]+)\)/g, "$1<$2>");
  } else {
    text = text.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, "$1");
    text = text.replace(/\[([^\]]*)\]\(([^)]+)\)/g, "$1");
  }
  return text;
}

function processInline(text, compression, keepUrls) {
  text = stripInlineMarkdown(text);
  text = processLinks(text, compression < 2 || keepUrls);
  return text;
}

// ============================================================
// Scope normalization
// ============================================================
function normScopeName(text, compression) {
  let s = text.trim().replace(/\s+/g, "_");
  s = s.replace(/[^A-Za-z0-9_-]/g, "");
  if (compression >= 2) s = s.toLowerCase();
  return s;
}

function normKey(text) {
  let s = text.trim().toLowerCase().replace(/\s+/g, "_");
  s = s.replace(/[^a-z0-9_-]/g, "");
  s = s.replace(/^-+|-+$/g, "");
  return s;
}

// ============================================================
// Stages 3+4: Scope Resolution + Emit LLMD
// ============================================================
function emitLLMD(ir, blocks, config) {
  const compression = config.compression ?? 2;
  const scopeMode = config.scope_mode ?? "flat";
  const keepUrls = config.keep_urls ?? false;
  const sentenceSplit = config.sentence_split ?? false;

  const out = [];
  let currentScope = null;
  const headingStack = []; // [{level, name}]

  function resolveScope(level, text) {
    const name = normScopeName(text, compression);
    // Trim stack to parent level
    while (headingStack.length > 0 && headingStack[headingStack.length - 1].level >= level) {
      headingStack.pop();
    }
    headingStack.push({ level, name });

    if (scopeMode === "flat") return name;
    if (scopeMode === "concat" || scopeMode === "stacked") {
      return headingStack.map(h => h.name).join("_");
    }
    return name;
  }

  function emitScope(scope) {
    if (scope && scope !== currentScope) {
      out.push("@" + scope);
      currentScope = scope;
    }
  }

  function ensureScope() {
    if (!currentScope) emitScope("root");
  }

  function processText(text) {
    return processInline(text, compression, keepUrls);
  }

  // Classify table: "property" (2-col key-value), "keyed_multi" (3+ cols with
  // unique identifier-like first column), or "raw"
  function classifyTable(rows) {
    if (rows.length < 2) return "raw";
    const numCols = rows[0].length;
    // Check consistent column count
    for (let r = 1; r < rows.length; r++) {
      if (rows[r].length !== numCols) return "raw";
    }
    if (numCols < 2) return "raw";
    // Check if first column values are unique and identifier-like
    const firstColVals = new Set();
    let allIdentifier = true;
    for (let r = 1; r < rows.length; r++) {
      const val = rows[r][0].trim();
      if (firstColVals.has(val)) { allIdentifier = false; break; }
      firstColVals.add(val);
      // Identifier-like: starts with letter/dot/-, contains no long prose
      if (!/^[A-Za-z._-]/.test(val) || val.split(/\s+/).length > 4) {
        allIdentifier = false; break;
      }
    }
    if (!allIdentifier) return "raw";
    if (numCols === 2) return "property";
    return "keyed_multi";
  }

  // Check if a column header is informative (not generic)
  const GENERIC_HEADERS = new Set(["value", "description", "details", "info", "notes", "default", "type"]);
  function isInformativeHeader(header) {
    return header && !GENERIC_HEADERS.has(header.trim().toLowerCase());
  }

  // Boolean/enum value compression
  const boolCompress = (config.bool_compress ?? true) && compression >= 2;
  const BOOL_MAP = {
    "yes": "Y", "no": "N",
    "true": "T", "false": "F",
    "enabled": "Y", "disabled": "N",
  };
  function compressBoolValue(val) {
    if (!boolCompress) return val;
    const low = val.trim().toLowerCase();
    return BOOL_MAP[low] ?? val;
  }

  // Sentence splitting for paragraphs at c2+
  function splitSentences(text) {
    if (!sentenceSplit || compression < 2) return [text];
    const sentences = text.split(/(?<=[.!?])\s+(?=[A-Z])/);
    return sentences.filter(s => s.trim());
  }

  // Buffer for merging consecutive KV lines at c1+
  const maxKVPerLine = config.max_kv_per_line ?? 4;
  const prefixExtraction = config.prefix_extraction ?? true;
  const minPrefixLen = config.min_prefix_len ?? 6;
  const minPrefixPct = config.min_prefix_pct ?? 0.6;
  let kvBuffer = [];

  function findCommonPrefix(keys) {
    if (keys.length < 2) return "";
    // Find longest common prefix among all keys
    let prefix = keys[0];
    for (let i = 1; i < keys.length; i++) {
      while (keys[i].indexOf(prefix) !== 0) {
        prefix = prefix.slice(0, -1);
        if (!prefix) return "";
      }
    }
    // Trim to last separator boundary (-, _, .)
    const lastSep = Math.max(prefix.lastIndexOf("-"), prefix.lastIndexOf("_"), prefix.lastIndexOf("."));
    if (lastSep > 0) prefix = prefix.slice(0, lastSep + 1);
    else prefix = "";
    return prefix;
  }

  function flushKV() {
    if (kvBuffer.length === 0) return;

    // Try prefix extraction at c1+
    if (compression >= 1 && prefixExtraction && kvBuffer.length >= 3) {
      const keys = kvBuffer.map(kv => kv.key);
      const prefix = findCommonPrefix(keys);
      if (prefix.length >= minPrefixLen) {
        const matchCount = keys.filter(k => k.startsWith(prefix)).length;
        if (matchCount / keys.length >= minPrefixPct) {
          out.push(":_pfx=" + prefix);
          // Emit with prefix stripped for matching keys, full for non-matching
          const adjusted = kvBuffer.map(kv => ({
            key: kv.key.startsWith(prefix) ? kv.key.slice(prefix.length) : kv.key,
            value: kv.value,
          }));
          for (let i = 0; i < adjusted.length; i += maxKVPerLine) {
            const chunk = adjusted.slice(i, i + maxKVPerLine);
            const pairs = chunk.map(kv => kv.key + "=" + kv.value);
            out.push(":" + pairs.join(" "));
          }
          kvBuffer = [];
          return;
        }
      }
    }

    if (compression >= 1) {
      // Merge consecutive KVs, chunked by maxKVPerLine
      for (let i = 0; i < kvBuffer.length; i += maxKVPerLine) {
        const chunk = kvBuffer.slice(i, i + maxKVPerLine);
        const pairs = chunk.map(kv => kv.key + "=" + kv.value);
        out.push(":" + pairs.join(" "));
      }
    } else {
      for (const kv of kvBuffer) {
        out.push(":" + kv.key + "=" + kv.value);
      }
    }
    kvBuffer = [];
  }

  for (const node of ir) {
    if (node.type !== "kv") flushKV();

    switch (node.type) {
      case "heading": {
        const scope = resolveScope(node.level, node.text);
        emitScope(scope);
        break;
      }
      case "paragraph": {
        ensureScope();
        const text = processText(node.text);
        const sentences = splitSentences(text);
        for (const s of sentences) {
          if (s.trim()) out.push(">" + s.trim());
        }
        break;
      }
      case "list_item": {
        ensureScope();
        const text = processText(node.text);
        const prefix = ".".repeat(node.depth);
        out.push(">" + prefix + (prefix ? " " : "") + text);
        break;
      }
      case "kv": {
        ensureScope();
        const k = normKey(node.key);
        const v = processText(node.value);
        if (k) {
          kvBuffer.push({ key: k, value: v });
        } else {
          out.push(">" + processText(node.key + ": " + node.value));
        }
        break;
      }
      case "table": {
        ensureScope();
        const rows = node.rows;
        const tableType = classifyTable(rows);
        // Detect boolean columns for compression
        const boolCols = new Set();
        if (boolCompress && rows.length > 1) {
          for (let c = 1; c < rows[0].length; c++) {
            const allBool = rows.slice(1).every(r => {
              const low = (r[c] || "").trim().toLowerCase();
              return low in BOOL_MAP;
            });
            if (allBool) boolCols.add(c);
          }
        }
        const processCell = (cell, colIdx) => {
          const text = processText(cell);
          return boolCols.has(colIdx) ? compressBoolValue(text) : text;
        };

        if (tableType === "property") {
          // Emit column header if informative
          if (rows[0].length >= 2 && isInformativeHeader(rows[0][1])) {
            const colHeader = normKey(rows[0][1]);
            if (colHeader) out.push(":_col=" + colHeader);
          }
          // Emit as :k=v
          for (let r = 1; r < rows.length; r++) {
            const k = normKey(rows[r][0]);
            const v = processCell(rows[r][1], 1);
            if (k) kvBuffer.push({ key: k, value: v });
            else out.push(">" + processText(rows[r][0] + "|" + rows[r][1]));
          }
        } else if (tableType === "keyed_multi") {
          // Emit column headers
          const colHeaders = rows[0].map(h => normKey(h)).join("|");
          out.push(":_cols=" + colHeaders);
          // Emit as :key=val1|val2|...
          for (let r = 1; r < rows.length; r++) {
            const k = normKey(rows[r][0]);
            const vals = rows[r].slice(1).map((c, ci) => processCell(c, ci + 1));
            if (k) kvBuffer.push({ key: k, value: vals.join("|") });
            else {
              const cells = rows[r].map((c, ci) => processCell(c, ci));
              out.push(">" + cells.join("|"));
            }
          }
        } else {
          // Raw: emit column headers then >c1|c2|c3
          if (rows[0].length >= 2) {
            const colHeaders = rows[0].map(h => normKey(h)).join("|");
            out.push(":_cols=" + colHeaders);
          }
          for (let r = 1; r < rows.length; r++) {
            const cells = rows[r].map((c, ci) => processCell(c, ci));
            out.push(">" + cells.join("|"));
          }
        }
        break;
      }
      case "block_ref": {
        ensureScope();
        const block = blocks[node.index];
        const lang = block.lang || "code";
        out.push("::" + lang);
        out.push("<<<");
        out.push(block.content);
        out.push(">>>");
        break;
      }
      case "blank":
        break;
      default:
        break;
    }
  }
  flushKV();
  return out;
}

// ============================================================
// Stage 5: Compression Passes
// ============================================================

// c0: whitespace normalize (already done by pipeline)
function compressC0(lines) {
  const out = [];
  for (const line of lines) {
    const t = line.replace(/\s+/g, " ").trim();
    if (t) out.push(t);
  }
  return out;
}

// c1: structural compaction — merge consecutive attributes, collapse blanks
function compressC1(lines) {
  return compressC0(lines);
  // Merging already handled during emission
}

// c2: token compaction — stopwords, phrase map, units
function compressC2(lines, config) {
  const stopwords = new Set((config.stopwords ?? []).map(s => s.toLowerCase()));
  const protect = new Set((config.protect_words ?? []).map(s => s.toLowerCase()));
  const phraseMap = config.phrase_map ?? {};
  const units = config.units ?? {};
  let inBlock = false;

  return lines.map(line => {
    if (line === "<<<") { inBlock = true; return line; }
    if (line === ">>>") { inBlock = false; return line; }
    if (inBlock) return line;
    if (line.startsWith("::") || line.startsWith("@")) return line;

    let text = line;

    // Apply phrase map (case-insensitive) on > lines and : value parts
    if (text.startsWith(">") || text.startsWith(":")) {
      let body = text.startsWith(">") ? text.slice(1) : text.slice(1);
      const prefix = text[0];

      // Sort phrase map keys by length desc for longest match
      const phrases = Object.keys(phraseMap).sort((a, b) => b.length - a.length);
      for (const phrase of phrases) {
        const re = new RegExp(escapeRegex(phrase), "gi");
        body = body.replace(re, phraseMap[phrase]);
      }

      // Apply unit normalization
      const unitKeys = Object.keys(units).sort((a, b) => b.length - a.length);
      for (const unit of unitKeys) {
        // Match "N unit" pattern e.g. "1000 requests per minute" → "1000/m"
        const reNum = new RegExp("(\\d+)\\s+" + escapeRegex(unit), "gi");
        body = body.replace(reNum, "$1" + units[unit]);
        // Also match standalone
        const re = new RegExp(escapeRegex(unit), "gi");
        body = body.replace(re, units[unit]);
      }

      text = prefix + body;
    }

    // Stopword removal on > lines only
    if (text.startsWith(">")) {
      const body = text.slice(1);
      const tokens = body.split(/\s+/).filter(Boolean);
      const filtered = tokens.filter(t => {
        const low = t.toLowerCase().replace(/[^a-z]/g, "");
        if (!low) return true;
        if (protect.has(low)) return true;
        return !stopwords.has(low);
      });
      text = ">" + filtered.join(" ");
    }

    return text;
  });
}

function escapeRegex(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// c3: DCS dictionary application
function compressC3(lines, dicts) {
  if (!dicts || dicts.length === 0) return lines;

  // Merge dictionaries (later overrides earlier)
  const merged = { scope: {}, key: {}, value: {}, text: {}, type: {} };
  let policy = { case: "smart", match: "token", longest_match: true, max_passes: 1 };
  let stopC3 = [];
  let protectSet = new Set(["no", "not", "never", "must", "should", "may"]);

  for (const dict of dicts) {
    if (dict.policy) {
      policy = { ...policy, ...dict.policy };
      if (dict.policy.protect) {
        if (dict.policy.protect.negations) {
          protectSet.add("no"); protectSet.add("not"); protectSet.add("never");
        }
        if (dict.policy.protect.modals) {
          protectSet.add("must"); protectSet.add("should"); protectSet.add("may");
        }
      }
    }
    if (dict.maps) {
      for (const ns of ["scope", "key", "value", "text", "type"]) {
        if (dict.maps[ns]) Object.assign(merged[ns], dict.maps[ns]);
      }
    }
    if (dict.stop?.c3) stopC3 = [...stopC3, ...dict.stop.c3];
  }

  const stopSet = new Set(stopC3.map(s => s.toLowerCase()));

  // Sort keys by length desc for longest-match
  const sortedMaps = {};
  for (const ns of ["scope", "key", "value", "text", "type"]) {
    sortedMaps[ns] = Object.entries(merged[ns]).sort((a, b) => b[0].length - a[0].length);
  }

  function applyMap(text, entries, mode) {
    if (entries.length === 0) return text;

    if (mode === "token") {
      // Token mode: split on whitespace, replace full tokens
      const tokens = text.split(/(\s+)/);
      return tokens.map(tok => {
        if (/^\s+$/.test(tok)) return tok;
        const low = tok.toLowerCase();
        if (protectSet.has(low)) return tok;
        if (/^\d+/.test(tok)) return tok; // protect numbers
        for (const [key, val] of entries) {
          const keyLow = policy.case === "preserve" ? key : key.toLowerCase();
          const cmp = policy.case === "preserve" ? tok : low;
          if (cmp === keyLow) return val;
        }
        return tok;
      }).join("");
    } else {
      // Word mode: use regex with word boundaries
      let result = text;
      for (const [key, val] of entries) {
        const flags = policy.case === "preserve" ? "g" : "gi";
        const re = new RegExp("(?<![A-Za-z0-9_./\\-])" + escapeRegex(key) + "(?![A-Za-z0-9_./\\-])", flags);
        result = result.replace(re, val);
      }
      return result;
    }
  }

  function isValueEligible(val) {
    if (/^[0-9]/.test(val)) return false;
    if (/^https?:\/\//.test(val)) return false;
    if (/^".*"$/.test(val)) return false;
    if (!/^[A-Za-z][A-Za-z0-9._-]*$/.test(val)) return false;
    return true;
  }

  const mode = policy.match || "token";
  let inBlock = false;

  const passes = Math.min(policy.max_passes || 1, 10);
  let result = [...lines];

  for (let pass = 0; pass < passes; pass++) {
    result = result.map(line => {
      if (line === "<<<") { inBlock = true; return line; }
      if (line === ">>>") { inBlock = false; return line; }
      if (inBlock) return line;

      // @scope
      if (line.startsWith("@")) {
        const scope = line.slice(1).trim();
        const replaced = applyMap(scope, sortedMaps.scope, mode);
        return "@" + replaced;
      }

      // ::type
      if (line.startsWith("::")) {
        const tp = line.slice(2).trim();
        const replaced = applyMap(tp, sortedMaps.type, mode);
        return "::" + replaced;
      }

      // :k=v pairs
      if (line.startsWith(":")) {
        const body = line.slice(1).trim();
        const pairs = body.split(/\s+/).filter(Boolean);
        const newPairs = pairs.map(p => {
          const eqIdx = p.indexOf("=");
          if (eqIdx <= 0) return p;
          const k = p.slice(0, eqIdx);
          const v = p.slice(eqIdx + 1);

          // Apply key map
          const newK = applyMap(k, sortedMaps.key, mode);

          // Apply value map to enum-split parts
          const vParts = v.split(/([|,])/);
          const newV = vParts.map(part => {
            if (part === "|" || part === ",") return part;
            const trimmed = part.trim();
            if (!isValueEligible(trimmed)) return trimmed;
            return applyMap(trimmed, sortedMaps.value, mode);
          }).join("");

          return newK + "=" + newV;
        });
        return ":" + newPairs.join(" ");
      }

      // >text
      if (line.startsWith(">")) {
        let body = line.slice(1);
        // Depth prefix preservation
        const depthMatch = body.match(/^(\.+\s)/);
        const depthPrefix = depthMatch ? depthMatch[0] : "";
        const textBody = depthPrefix ? body.slice(depthPrefix.length) : body;

        let newText = applyMap(textBody, sortedMaps.text, mode);

        // Remove c3 stopwords
        if (stopSet.size > 0) {
          const toks = newText.split(/\s+/).filter(Boolean);
          newText = toks.filter(t => {
            const low = t.toLowerCase();
            if (protectSet.has(low)) return true;
            return !stopSet.has(low);
          }).join(" ");
        }

        return ">" + depthPrefix + newText;
      }

      // ->relation targets: apply scope map
      if (line.startsWith("->")) {
        const target = line.slice(2).trim();
        const replaced = applyMap(target, sortedMaps.scope, mode);
        return "->" + replaced;
      }

      return line;
    });
  }

  return result;
}

// ============================================================
// Stage 6: Post-processing
// ============================================================
function stage6(lines, config) {
  const anchorEvery = config.anchor_every ?? 0;

  // Validation
  let firstScope = false;
  let inBlock = false;
  const errors = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line === "<<<") { inBlock = true; continue; }
    if (line === ">>>") { inBlock = false; continue; }
    if (inBlock) continue;

    if (line.startsWith("@")) { firstScope = true; continue; }
    if (line.startsWith("~")) continue;
    if (!firstScope && (line.startsWith(":") || line.startsWith(">") || line.startsWith("->"))) {
      errors.push(`line ${i + 1}: scoped line before first @scope`);
    }
  }

  if (errors.length > 0) {
    console.error("validation warnings:");
    for (const e of errors) console.error("  " + e);
  }

  // Anchors
  if (anchorEvery > 0) {
    let currentScope = null;
    let linesSinceAnchor = 0;
    const out = [];

    for (const line of lines) {
      if (line.startsWith("@")) {
        currentScope = line;
        linesSinceAnchor = 0;
        out.push(line);
        continue;
      }
      linesSinceAnchor++;
      if (anchorEvery > 0 && linesSinceAnchor >= anchorEvery && currentScope) {
        out.push(currentScope);
        linesSinceAnchor = 0;
      }
      out.push(line);
    }
    return out;
  }

  return lines;
}

// ============================================================
// Main Compile Pipeline
// ============================================================
function compile(text, config, dicts) {
  const compression = config.compression ?? 2;

  // Stage 0
  let lines = stage0(text);

  // Stage 1
  const { lines: cleanLines, blocks } = stage1(lines);

  // Stage 2
  const ir = stage2(cleanLines);

  // Stages 3+4
  let output = emitLLMD(ir, blocks, config);

  // Stage 5
  if (compression >= 0) output = compressC0(output);
  if (compression >= 1) output = compressC1(output);
  if (compression >= 2) output = compressC2(output, config);
  if (compression >= 3) output = compressC3(output, dicts);

  // Stage 6
  output = stage6(output, config);

  return output.join("\n") + "\n";
}

// ============================================================
// CLI
// ============================================================
function parseArgs(argv) {
  const args = {
    inputs: [],
    output: null,
    compression: null,
    dicts: [],
    scopeMode: null,
    keepUrls: null,
    sentenceSplit: null,
    anchorEvery: null,
    configPath: null,
  };

  let i = 0;
  while (i < argv.length) {
    const a = argv[i];
    if (a === "-o" || a === "--output") { args.output = argv[++i]; }
    else if (a === "-c" || a === "--compression") { args.compression = parseInt(argv[++i]); }
    else if (a === "--dict") { args.dicts.push(argv[++i]); }
    else if (a === "--scope-mode") { args.scopeMode = argv[++i]; }
    else if (a === "--keep-urls") { args.keepUrls = true; }
    else if (a === "--sentence-split") { args.sentenceSplit = true; }
    else if (a === "--anchor-every") { args.anchorEvery = parseInt(argv[++i]); }
    else if (a === "--config") { args.configPath = argv[++i]; }
    else if (a === "-h" || a === "--help") {
      console.log(`llmdc — LLMD Compiler

Usage: llmdc [options] <input...>

Options:
  -o, --output <path>       Output file (default: stdout)
  -c, --compression <0-3>   Compression level (default: from config or 2)
  --dict <path>             Dictionary file (repeatable, later overrides earlier)
  --scope-mode <mode>       Scope mode: flat, concat, stacked (default: flat)
  --keep-urls               Keep URLs at c2+ (default: strip)
  --sentence-split          Split sentences into separate > lines at c2+
  --anchor-every <n>        Re-emit @scope every N lines (default: 0 = off)
  --config <path>           Config file path
  -h, --help                Show this help`);
      process.exit(0);
    }
    else if (a.startsWith("-")) { die(`unknown option: ${a}`); }
    else { args.inputs.push(a); }
    i++;
  }
  return args;
}

function main() {
  const argv = process.argv.slice(2);
  if (argv.length === 0) {
    die("usage: llmdc [options] <input...>\nRun llmdc --help for details.");
  }

  const args = parseArgs(argv);
  if (args.inputs.length === 0) die("no input files specified");

  // Load config
  let config = {};
  if (args.configPath) {
    config = readJSON(args.configPath);
  } else {
    // Try default config locations
    const defaults = ["llmdc.config.json", "config/llmdc.config.json"];
    for (const p of defaults) {
      try { config = readJSON(p); break; } catch {}
    }
  }

  // CLI overrides
  if (args.compression !== null) config.compression = args.compression;
  if (args.scopeMode !== null) config.scope_mode = args.scopeMode;
  if (args.keepUrls !== null) config.keep_urls = args.keepUrls;
  if (args.sentenceSplit !== null) config.sentence_split = args.sentenceSplit;
  if (args.anchorEvery !== null) config.anchor_every = args.anchorEvery;
  if (config.compression === undefined) config.compression = 2;

  // Load dictionaries
  const dicts = args.dicts.map(p => readJSON(p));

  // Collect input files
  const files = listFiles(args.inputs);
  if (files.length === 0) die("no input files found");

  // Compile all files (sorted deterministically)
  let allText = "";
  for (const fp of files) {
    if (allText) allText += "\n";
    allText += fs.readFileSync(fp, "utf8");
  }

  const result = compile(allText, config, dicts);

  if (args.output) {
    fs.writeFileSync(args.output, result, "utf8");
    const tokens = result.split(/\s+/).filter(Boolean).length;
    console.error(`compiled ${files.length} file(s) -> ${args.output} (c${config.compression}, ~${tokens} tokens)`);
  } else {
    process.stdout.write(result);
  }
}

main();
