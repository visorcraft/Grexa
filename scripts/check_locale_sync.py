#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 VisorCraft LLC
# SPDX-License-Identifier: GPL-3.0-only
"""Verify every Grexa Fluent locale defines the same key set as English.

Also checks that QML ``qsTr()`` string arguments are consistent across
all QML files (duplicate strings with different source texts are flagged).

Run from the repo root:

    python3 scripts/check_locale_sync.py

CI exit codes:
  0 — all locales are in sync and QML strings are consistent.
  1 — one or more locales are missing keys, define extra keys, or QML
      strings are inconsistent.

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

REPO_ROOT = Path(__file__).resolve().parent.parent
LOCALES_ROOT = REPO_ROOT / "crates" / "grexa-i18n" / "locales"
QML_ROOT = REPO_ROOT / "apps" / "grexa-gui" / "qml"
CANONICAL_LOCALE = "en"
MESSAGE_ID = re.compile(r"^([A-Za-z][A-Za-z0-9_-]*)\s*=")
QSTR_CALL = re.compile(r'qsTr\(\s*"((?:[^"\\]|\\.)*)"\s*\)')


def keys_in(path: Path) -> set[str]:
    """Return every Fluent message id defined in `path`."""
    keys: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith(" ") or line.startswith("\t"):
            continue
        if not line or line.startswith("#"):
            continue
        match = MESSAGE_ID.match(line)
        if not match:
            continue
        keys.add(match.group(1))
    return keys


def qstr_strings_in(path: Path) -> list[str]:
    """Return every ``qsTr("...")`` string argument in a QML file."""
    text = path.read_text(encoding="utf-8")
    return QSTR_CALL.findall(text)


def check_qml_files_listed() -> bool:
    """Verify every QML file in the qml/ directory is listed in build.rs."""
    if not QML_ROOT.exists():
        return True

    build_rs = REPO_ROOT / "apps" / "grexa-gui" / "build.rs"
    if not build_rs.exists():
        return True

    build_content = build_rs.read_text(encoding="utf-8")

    ok = True
    qml_files = sorted(QML_ROOT.glob("*.qml"))
    for qml_file in qml_files:
        name = qml_file.name
        if f"qml/{name}" not in build_content:
            ok = False
            print(
                f"  {name} exists in qml/ but is not listed in build.rs (won't ship)",
                file=sys.stderr,
            )

    return ok


def check_qml_strings() -> bool:
    """Verify QML qsTr() strings are present and non-empty.

    Returns True if ok, False if problems were found.
    """
    if not QML_ROOT.exists():
        print(f"QML root missing: {QML_ROOT}", file=sys.stderr)
        return False

    ok = True
    total_strings = 0
    qml_files = sorted(QML_ROOT.glob("*.qml"))

    for qml_file in qml_files:
        strings = qstr_strings_in(qml_file)
        total_strings += len(strings)

        empty = [i + 1 for i, s in enumerate(strings) if not s.strip()]
        if empty:
            ok = False
            print(
                f"  {qml_file.name}: {len(empty)} empty qsTr() call(s) at occurrence(s) {empty}",
                file=sys.stderr,
            )

    print(f"QML qsTr() check: {total_strings} strings across {len(qml_files)} files")

    if total_strings == 0:
        print("  (all qsTr() strings have been migrated to Fluent)")

    return ok


def main() -> int:
    if not LOCALES_ROOT.exists():
        print(f"locale root missing: {LOCALES_ROOT}", file=sys.stderr)
        return 1

    canonical_path = LOCALES_ROOT / CANONICAL_LOCALE / "grexa.ftl"
    if not canonical_path.exists():
        print(f"canonical catalog missing: {canonical_path}", file=sys.stderr)
        return 1

    ok = True

    # --- Fluent locale sync ---
    canonical = keys_in(canonical_path)
    print(f"canonical {CANONICAL_LOCALE} has {len(canonical)} keys")

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

    # --- QML structural checks ---
    if not check_qml_files_listed():
        ok = False
    if not check_qml_strings():
        ok = False

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
