#!/usr/bin/env node
import fs from "fs";
import path from "path";

function die(msg){ console.error(msg); process.exit(1); }
function readJSON(p){ return JSON.parse(fs.readFileSync(p,"utf8")); }

function listFilesRecursive(inputs) {
  const out = [];
  for (const p of inputs) {
    const st = fs.statSync(p);
    if (st.isDirectory()) {
      const entries = fs.readdirSync(p).map(e => path.join(p, e));
      out.push(...listFilesRecursive(entries));
    } else if (st.isFile()) out.push(p);
  }
  out.sort((a,b)=>a.localeCompare(b));
  return out;
}

function normalizeUnicodeNFKC(s){ return s.normalize("NFKC"); }

function canonicalizeToLLMDLines(text, cfg) {
  text = normalizeUnicodeNFKC(text).replace(/\r\n/g, "\n");
  const raw = text.split("\n");

  let lines = [];
  if (cfg.ignore_blocks) {
    let inFence = false;
    for (const ln of raw) {
      const t = ln.trim();
      if (t.startsWith("```")) { inFence = !inFence; continue; }
      if (inFence) continue;
      lines.push(ln);
    }
  } else lines = raw;

  const out = [];
  let cur = null;
  const emit = (s) => { if (s && s !== cur) { out.push("@"+s); cur = s; } };
  const normScope = s => s.trim().replace(/\s+/g,"_").toLowerCase();
  const normKey = s => s.trim().toLowerCase().replace(/\s+/g,"_").replace(/[^a-z0-9_]/g,"");

  for (const ln0 of lines) {
    const ln = ln0.trim();
    if (!ln) continue;

    const mh = ln.match(/^(#{1,6})\s+(.*)$/);
    if (mh) { emit(normScope(mh[2])); continue; }

    const mb = ln.match(/^[-*+]\s+(.*)$/) || ln.match(/^\d+\.\s+(.*)$/);
    if (mb) { if (!cur) emit("root"); out.push(">"+mb[1].trim()); continue; }

    const mkv = ln.match(/^([A-Za-z0-9 _-]{2,64})\s*:\s*(.+)$/);
    if (mkv) { if (!cur) emit("root"); const k = normKey(mkv[1]); out.push(k?(":"+k+"="+mkv[2].trim()):(">"+ln)); continue; }

    if (!cur) emit("root");
    out.push(">"+ln);
  }
  return out;
}

function estTokensLine(line) {
  // ignore blocks (none here), sum ceil(len(tok)/4) over whitespace toks
  const toks = line.split(/\s+/).filter(Boolean);
  let sum = 0;
  for (const t of toks) sum += Math.ceil(t.length / 4);
  return sum;
}

function applyDictTokenMode(lines, dict) {
  const maps = dict.maps || {};
  const scopeMap = maps.scope || {};
  const keyMap = maps.key || {};
  const valueMap = maps.value || {};
  const textMap = maps.text || {};
  const typeMap = maps.type || {};

  const out = [];
  for (const line of lines) {
    if (line.startsWith("@")) {
      const s = line.slice(1).trim().toLowerCase();
      out.push("@"+(scopeMap[s] ?? s));
      continue;
    }
    if (line.startsWith(":")) {
      const parts = line.slice(1).trim().split(/\s+/).filter(Boolean);
      const nParts = parts.map(p => {
        const i = p.indexOf("=");
        if (i <= 0) return p;
        const k = p.slice(0,i).toLowerCase();
        const v = p.slice(i+1);
        const nk = keyMap[k] ?? k;
        // enum split on | and , then remap eligible tokens
        const vv = v.split(/([|,])/g).map(chunk => {
          const c = chunk.trim();
          const low = c.toLowerCase();
          if (chunk === "|" || chunk === ",") return chunk;
          if (/^[a-z][a-z0-9._-]*$/.test(low) && valueMap[low]) return valueMap[low];
          return c;
        }).join("");
        return nk + "=" + vv;
      });
      out.push(":" + nParts.join(" "));
      continue;
    }
    if (line.startsWith(">")) {
      const toks = line.slice(1).trim().split(/\s+/).filter(Boolean);
      const nt = toks.map(t => {
        const low = t.toLowerCase().replace(/^[^a-z0-9._-]+/g,"").replace(/[^a-z0-9._-]+$/g,"");
        return textMap[low] ?? t;
      });
      out.push(">" + nt.join(" "));
      continue;
    }
    if (line.startsWith("::")) {
      const t = line.slice(2).trim().toLowerCase();
      out.push("::" + (typeMap[t] ?? t));
      continue;
    }
    out.push(line);
  }
  return out;
}

// CLI: node bench.js <config.json> <dict.json> <inputs...>
const argv = process.argv.slice(2);
if (argv.length < 3) die("Usage: node bench.js <config.json> <dict.json> <inputs...>");
const cfg = readJSON(argv[0]);
const dict = readJSON(argv[1]);
const files = listFilesRecursive(argv.slice(2));
if (!files.length) die("No input files found.");

let before = 0;
let after = 0;

for (const fp of files) {
  const txt = fs.readFileSync(fp, "utf8");
  const ll = canonicalizeToLLMDLines(txt, cfg);
  const ll2 = applyDictTokenMode(ll, dict);

  for (const ln of ll) before += estTokensLine(ln);
  for (const ln of ll2) after += estTokensLine(ln);
}

const saved = before - after;
const pctRed = before ? (saved / before) * 100 : 0;
const pctSize = before ? (after / before) * 100 : 0;

console.log(`Files: ${files.length}`);
console.log(`Est tokens BEFORE: ${before}`);
console.log(`Est tokens AFTER : ${after}`);
console.log(`Saved: ${saved} (${pctRed.toFixed(1)}% reduction, final size ${pctSize.toFixed(1)}%)`);