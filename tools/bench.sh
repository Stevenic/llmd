#!/usr/bin/env bash
# bench.sh — Benchmark all LLMD compiler implementations
# Usage: bash tools/bench.sh [runs]
# Requires: node (18+), python3 (3.10+), cargo build --release in tools/rust/

set -euo pipefail

RUNS="${1:-5}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CONFIG="config/llmdc.config.json"
RUST_BIN="tools/rust/target/release/llmdc"
TMPOUT=$(mktemp)
trap "rm -f '$TMPOUT'" EXIT

# Check prerequisites
missing=()
command -v node &>/dev/null || missing+=("node")
command -v python3 &>/dev/null || missing+=("python3")
[ -x "$RUST_BIN" ] || missing+=("Rust binary (run: cargo build --release --manifest-path tools/rust/Cargo.toml)")
if [ ${#missing[@]} -gt 0 ]; then
    echo "Missing: ${missing[*]}" >&2
    exit 1
fi

bench() {
    local label="$1"
    shift
    local times=()
    for ((i = 0; i < RUNS; i++)); do
        local start end elapsed
        start=$(date +%s%N 2>/dev/null || python3 -c "import time; print(int(time.time()*1e9))")
        "$@" > "$TMPOUT" 2>/dev/null || true
        end=$(date +%s%N 2>/dev/null || python3 -c "import time; print(int(time.time()*1e9))")
        elapsed=$(( (end - start) / 1000000 ))
        times+=("$elapsed")
    done
    # Sort and pick median
    local sorted
    sorted=$(printf '%s\n' "${times[@]}" | sort -n)
    local median
    median=$(echo "$sorted" | sed -n "$((RUNS / 2 + 1))p")
    printf "  %-8s  %sms  (runs: %s)\n" "$label" "$median" "$(IFS=', '; echo "${times[*]}")"
    echo "$median"
}

echo ""
echo "LLMD Compiler Benchmark ($RUNS runs per tool, median reported)"
echo "============================================================="

declare -A results

for sample in "api-spec.md" "fluentlm-components.md"; do
    input="corpora/samples/$sample"
    size=$(wc -c < "$input" | tr -d ' ')
    size_kb=$(echo "scale=1; $size / 1024" | bc)
    echo ""
    echo "$sample ($size_kb KB)"
    echo "--------------------------------------------------"

    js_ms=$(bench "JS" node tools/js/llmdc.js "$input" --config "$CONFIG" -o "$TMPOUT")
    py_ms=$(bench "Python" python3 tools/py/llmdc.py "$input" --config "$CONFIG" -o "$TMPOUT")
    rs_ms=$(bench "Rust" "$RUST_BIN" "$input" --config "$CONFIG" -o "$TMPOUT")

    results["$sample,JS"]=$js_ms
    results["$sample,Python"]=$py_ms
    results["$sample,Rust"]=$rs_ms
    results["$sample,size"]=$size_kb
done

echo ""
echo "Summary"
echo "-------"
echo ""
printf "| %-35s | %8s | %8s | %8s |\n" "File" "JS" "Python" "Rust"
printf "| %-35s | %8s | %8s | %8s |\n" "-----------------------------------" "--------" "--------" "--------"
for sample in "api-spec.md" "fluentlm-components.md"; do
    label="$sample (${results[$sample,size]} KB)"
    printf "| %-35s | %5s ms | %5s ms | %5s ms |\n" \
        "$label" "${results[$sample,JS]}" "${results[$sample,Python]}" "${results[$sample,Rust]}"
done
echo ""
