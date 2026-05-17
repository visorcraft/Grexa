#!/usr/bin/env bash
# Compare grexa-cli throughput against ripgrep on a representative
# tree. Requires `hyperfine` and `rg` on $PATH.
#
# Usage:
#   scripts/bench_vs_rg.sh [path] [term]
#
# Defaults to searching the current repo for "TODO".

set -euo pipefail

ROOT="${1:-$(pwd)}"
TERM="${2:-TODO}"
BIN="$(cd "$(dirname "$0")/.." && pwd)/target/release/grexa-cli"

if ! command -v hyperfine >/dev/null; then
    echo "hyperfine not found. Install via `cargo install hyperfine` or your package manager." >&2
    exit 1
fi
if ! command -v rg >/dev/null; then
    echo "rg (ripgrep) not found." >&2
    exit 1
fi
if [ ! -x "$BIN" ]; then
    echo "Building grexa-cli in release mode..."
    (cd "$(dirname "$0")/.." && cargo build --release -p grexa-cli)
fi

echo "Tree: $ROOT"
echo "Term: $TERM"
echo

hyperfine \
    --warmup 2 \
    --runs 10 \
    --export-markdown bench-results.md \
    "$BIN --quiet '$ROOT' '$TERM'" \
    "rg --quiet '$TERM' '$ROOT'"

echo
echo "Results written to bench-results.md"
