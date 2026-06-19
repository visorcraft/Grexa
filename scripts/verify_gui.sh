#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 VisorCraft LLC
# SPDX-License-Identifier: GPL-3.0-only
#
# GUI launch gate. Headless-launches a built grexa GUI binary and FAILS
# (non-zero exit) unless the main window actually instantiates. This is the
# guard against the "taskbar pin does nothing" bug: a green `cargo build` can
# still ship a GUI that dies before the event loop, because cxx-qt-lib 0.8
# swallows the QML error and the binary exits 2 with only a generic
# "QML payload did not instantiate" line. See AGENTS.md "Build + install".
#
# Usage:
#   scripts/verify_gui.sh                      # verify target/release/grexa
#   scripts/verify_gui.sh path/to/grexa        # verify a specific binary
#   scripts/verify_gui.sh pkg:path/to/x.pkg.tar.zst   # verify the binary
#                                              #   INSIDE a built Arch package
#                                              #   (the exact bytes installed)
#
# Needs a Qt install with the `offscreen` platform plugin (ships with qtbase)
# and, for pkg: mode, `unzstd` (ships with zstd). Both are present on any box
# that can build Grexa.

set -euo pipefail

ARG="${1:-target/release/grexa}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

case "$ARG" in
    pkg:*)
        PKG="${ARG#pkg:}"
        [ -f "$PKG" ] || { echo "verify-gui: no such package: $PKG" >&2; exit 1; }
        tar --use-compress-program=unzstd -xf "$PKG" -C "$TMP" usr/bin/grexa
        BIN="$TMP/usr/bin/grexa"
        echo "verify-gui: extracted GUI binary from $PKG"
        ;;
    *)
        BIN="$ARG"
        ;;
esac

[ -x "$BIN" ] || { echo "verify-gui: not an executable: $BIN" >&2; exit 1; }

echo "verify-gui: launching $BIN headless (offscreen)…"
# Isolate XDG_RUNTIME_DIR so the single-instance flock lands on a fresh,
# private lockfile — otherwise a Grexa already running on this box would make
# even a healthy build exit early ("Another Grexa instance is already
# running") and the gate would mis-report. Offscreen QPA needs no compositor
# socket, so a throwaway runtime dir is safe.
mkdir -p "$TMP/run"
chmod 700 "$TMP/run"
set +e
OUT="$(timeout 15 env XDG_RUNTIME_DIR="$TMP/run" QT_QPA_PLATFORM=offscreen GREXA_LOG=info "$BIN" 2>&1)"
CODE=$?
set -e
echo "------------------------------------------------------------"
echo "$OUT"
echo "------------------------------------------------------------"

# A healthy GUI prints "Grexa GUI shell starting" and then SITS in the Qt
# event loop forever — so `timeout` has to kill it (exit 124). Any other
# outcome means it fell over before opening a window.
fail() { echo "verify-gui: FAIL — $1" >&2; exit 1; }

grep -q "Grexa GUI shell starting" <<<"$OUT" \
    || fail "binary never reached GUI startup (wrong binary, or it crashed instantly)"
grep -qE "did not instantiate|QML failed to load" <<<"$OUT" \
    && fail "the QML payload did not instantiate — this is the dead-pin bug, do NOT ship this build"
[ "$CODE" -eq 124 ] \
    || fail "GUI exited on its own (code $CODE); a working window holds the event loop until killed"

echo "verify-gui: OK — window instantiated and held the event loop"
