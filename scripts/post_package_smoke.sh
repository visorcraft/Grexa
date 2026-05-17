#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 VisorCraft LLC
# SPDX-License-Identifier: GPL-3.0-only
# Post-package smoke test.
#
# Invoked after every packaging recipe (Flatpak, AppImage, distro RPM/deb)
# to verify the produced artifacts actually run. The script is intentionally
# tiny — it doesn't depend on cargo, only on the binaries the packager
# installed onto $PATH.
#
# Usage:
#   scripts/post_package_smoke.sh [path-to-grexa-cli]
#
# Exits non-zero on any failure; suitable as the last step of a CI job.

set -euo pipefail

CLI="${1:-grexa-cli}"
if ! command -v "$CLI" >/dev/null; then
    echo "post-package smoke: $CLI not on \$PATH" >&2
    exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "post-package smoke: $CLI"

# 1. Help renders.
"$CLI" --help >/dev/null
echo "  help ok"

# 2. Version renders.
"$CLI" --version >/dev/null
echo "  --version ok"

# 3. Search a tiny fixture tree.
mkdir -p "$TMP/tree/sub"
printf 'TODO write tests\nother content\n' > "$TMP/tree/notes.txt"
printf 'no marker here\n' > "$TMP/tree/sub/nope.log"

RESULT="$("$CLI" "$TMP/tree" TODO --quiet || true)"
if "$CLI" "$TMP/tree" TODO --quiet; then
    echo "  search matched ok"
else
    echo "post-package smoke: expected matches but got exit non-zero" >&2
    exit 1
fi

# 4. JSON output is valid JSON.
JSON="$("$CLI" "$TMP/tree" TODO --format json)"
if ! python3 -c "import json,sys; json.loads(sys.argv[1])" "$JSON" >/dev/null 2>&1; then
    echo "post-package smoke: JSON output failed parse: $JSON" >&2
    exit 1
fi
echo "  json output parses ok"

# 5. Count subcommand.
COUNT="$("$CLI" "$TMP/tree" TODO --count)"
if [ "$COUNT" != "1" ]; then
    echo "post-package smoke: expected count=1, got '$COUNT'" >&2
    exit 1
fi
echo "  count ok"

# 6. completions / manpage subcommands.
"$CLI" completions bash >/dev/null
"$CLI" completions fish >/dev/null
"$CLI" completions zsh >/dev/null
"$CLI" manpage >/dev/null
echo "  completions + manpage ok"

# 7. Exit code 1 when nothing matches.
if "$CLI" "$TMP/tree" XYZ_NEVER_PRESENT --quiet; then
    echo "post-package smoke: expected exit 1 when no match" >&2
    exit 1
fi
echo "  exit 1 on no match ok"

echo "post-package smoke: all checks passed"
