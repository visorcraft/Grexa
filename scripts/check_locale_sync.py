#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 VisorCraft LLC
# SPDX-License-Identifier: GPL-3.0-only
"""Verify every Grexa Fluent locale defines the same key set as English.

Run from the repo root:

    python3 scripts/check_locale_sync.py

CI exit codes:
  0 — all locales are in sync.
  1 — one or more locales are missing keys or define extra keys.

The script is deliberately stand-alone — it has no external dependencies
so packaging recipes can run it during a build without provisioning a
Cargo toolchain. The same logic lives in `keys_in` inside
`crates/grexa-i18n/src/lib.rs`; this script is the "stop the merge"
gate, the Rust test is the "rebuild green" gate.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

LOCALES_ROOT = Path(__file__).resolve().parent.parent / "crates" / "grexa-i18n" / "locales"
CANONICAL_LOCALE = "en"
MESSAGE_ID = re.compile(r"^([A-Za-z][A-Za-z0-9_-]*)\s*=")


def keys_in(path: Path) -> set[str]:
    """Return every Fluent message id defined in `path`."""
    keys: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith(" ") or line.startswith("\t"):
            # Continuation of the previous message.
            continue
        if not line or line.startswith("#"):
            continue
        match = MESSAGE_ID.match(line)
        if not match:
            continue
        keys.add(match.group(1))
    return keys


def main() -> int:
    if not LOCALES_ROOT.exists():
        print(f"locale root missing: {LOCALES_ROOT}", file=sys.stderr)
        return 1

    canonical_path = LOCALES_ROOT / CANONICAL_LOCALE / "grexa.ftl"
    if not canonical_path.exists():
        print(f"canonical catalog missing: {canonical_path}", file=sys.stderr)
        return 1

    canonical = keys_in(canonical_path)
    print(f"canonical {CANONICAL_LOCALE} has {len(canonical)} keys")

    ok = True
    for locale_dir in sorted(LOCALES_ROOT.iterdir()):
        if not locale_dir.is_dir() or locale_dir.name == CANONICAL_LOCALE:
            continue
        catalog = locale_dir / "grexa.ftl"
        if not catalog.exists():
            print(f"  {locale_dir.name}: missing grexa.ftl", file=sys.stderr)
            ok = False
            continue
        keys = keys_in(catalog)
        missing = sorted(canonical - keys)
        extra = sorted(keys - canonical)
        if missing or extra:
            ok = False
            print(f"  {locale_dir.name}: {len(keys)} keys", file=sys.stderr)
            if missing:
                print(f"    missing: {', '.join(missing)}", file=sys.stderr)
            if extra:
                print(f"    extra:   {', '.join(extra)}", file=sys.stderr)
        else:
            print(f"  {locale_dir.name}: in sync ({len(keys)} keys)")

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
