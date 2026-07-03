#!/usr/bin/env bash
# Reproduce the GoPlus benchmarks. Run from the repo root.
#
#   Benchmark A (token economy) needs python3 + tiktoken:
#       pip install tiktoken
#   Benchmarks B/C need the Go toolchain and a `goplus` binary (cargo build).
set -uo pipefail

BIN="${GOPLUS:-target/debug/goplus}"
[ -x "$BIN" ] || { echo "building goplus..."; cargo build >/dev/null 2>&1; BIN=target/debug/goplus; }
BIN="$(realpath "$BIN")"
ROOT="$(realpath "$(dirname "$0")/..")"
cd "$ROOT"

echo "########## A. Token economy: GoPlus (.gp) vs hand-written Go (.go) ##########"
if command -v python3 >/dev/null && python3 -c "import tiktoken" 2>/dev/null; then
  python3 benchmarks/count_tokens.py \
    benchmarks/token-economy/error_chain.gp   benchmarks/token-economy/error_chain.go \
    benchmarks/token-economy/state_machine.gp benchmarks/token-economy/state_machine.go \
    benchmarks/token-economy/decorated_api.gp benchmarks/token-economy/decorated_api.go
else
  echo "  (skipped: install python3 + tiktoken to measure token counts)"
fi

echo
echo "########## B. Runtime parity: GoPlus-generated Go vs hand-written Go ##########"
rm -rf /tmp/gp_parity; mkdir -p /tmp/gp_parity
"$BIN" transpile benchmarks/runtime-parity/parity.gp --out-dir /tmp/gp_parity >/dev/null 2>&1
GENDIR="$(dirname "$(find /tmp/gp_parity -name '*.go' | head -1)")"
cp benchmarks/runtime-parity/parity_test.go "$GENDIR/"
( cd "$GENDIR" && { [ -f go.mod ] || go mod init parity >/dev/null 2>&1; } && \
  go test -run TestParity -bench=. -benchmem -count=3 2>&1 | grep -E "PASS|FAIL|ok |Benchmark" )

echo
echo "########## C. Transpile pipeline speed (criterion) ##########"
cargo bench --bench pipeline 2>&1 | grep -E "^(parse_sample|analyze_sample|codegen_sample)|time:" | head -20
echo "(A content-hash package cache, .goplus-package-cache.json, lets rebuilds skip unchanged packages.)"
