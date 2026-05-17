# Grexa v0.1.0

Released: 2026-05-17

## Highlights

- Alpha 1: the Rust core, CLI, container search, AI integration, and
  Qt 6 / Kirigami GUI shell all ship working together for the first
  time.
- Pure Cargo build — no CMake — even with the cxx-qt 0.8 Rust ⇄ Qt
  bridge wiring `SearchController` through to QML.
- Feature parity matrix vs. upstream Grex (Windows/WinUI) tracked in
  [`docs/feature-parity.md`](feature-parity.md); divergences pinned
  in [`docs/linux-decisions.md`](linux-decisions.md).

## New

- **Search engine** — `grexa-core` ports every Grex search mode:
  literal, regex (Rust `regex` fast path + `fancy-regex` lookaround
  fallback), case-sensitive / insensitive, whole-word, multiline,
  in-line `include` / `exclude` globs, gitignore-aware walking
  (`require_git(false)` to match Grex's "respect .gitignore even
  outside a repo"), bin-extension blocklist, max-file-size cap,
  same-filesystem walking.
- **Replace pipeline** — atomic same-filesystem replace via
  `tempfile::NamedTempFile::persist`, replace journal at
  `$XDG_STATE_HOME/grexa/replace-journal.json`, CRLF + final-newline
  preservation, configurable backup directory.
- **Document extraction** — OOXML (`.docx`, `.xlsx`, `.pptx`), ODF
  (`.odt`, `.ods`, `.odp`), plain ZIP, RTF, and PDF (via Poppler's
  `pdftotext`). Each path falls back gracefully when the optional
  tool is missing.
- **Encoding detection** — `chardetng` heuristic + `encoding_rs` BOM
  / UTF-16 detection. Per-file confidence reported in JSON output.
- **Container search** — Docker and Podman, with `MockCommandRunner`
  + `SystemCommandRunner`. Direct `exec` grep emits one
  `ContainerSearchHit` per line with byte offset and column number;
  archive mirror fallback runs grep host-side on the file the
  container would have streamed. Live test matrix gated by the
  `container-live` Cargo feature (verified against rootless Podman 5.x).
- **AI assist (opt-in)** — OpenAI-compatible HTTP client over `ureq`
  + rustls, model discovery, Secret-Service-backed API key storage
  via `keyring`. AI mode disables itself on headless boxes with no
  secret backend, by design.
- **CLI (`grexa-cli`)** — every Grex search/replace flag, plus
  `--format json|csv`, `--container`, `--runtime`, shell completions
  (bash / zsh / fish) via `clap_complete`, man page generation via
  `clap_mangen`, profile save/load, `--include-system` to override
  gitignore.
- **GUI (`grexa`)** — Qt 6 / Kirigami shell launches via cxx-qt 0.8;
  Main / Search / Regex Builder / Settings / About / Context Preview
  / AI Chat / DesignTokens QML pages all ship. QML files are bundled
  into the binary at build time under the
  `com.visorcraft.Grexa 1.0` module.
- **Localization** — Fluent (`.ftl`) catalogs for English, German,
  and Japanese, with plural selectors. `scripts/check_locale_sync.py`
  enforces key parity in CI.
- **Packaging** — Flatpak manifest (`packaging/io.visorcraft.Grexa.*`),
  AppImage recipe, plus per-distro install scripts for Arch, Fedora,
  Debian/Ubuntu, and openSUSE.
- **Observability** — structured logging via `tracing` /
  `tracing-appender` to `$XDG_STATE_HOME/grexa/grexa-gui.log`,
  filterable through `GREXA_LOG`.

## Changed

- **Rust ⇄ Qt bridge: cxx-qt 0.8 (was qmetaobject 0.2).**
  `apps/grexa-gui/src/qobjects.rs` is now a `#[cxx_qt::bridge]`
  module; the `qmetaobject` dependency is gone. The change is
  invisible to QML — same `com.visorcraft.Grexa 1.0` import, same
  `SearchController` element with `status_text` / `match_count` /
  `busy` / `recent_path_count` properties — but compile-time
  generation gives sharper types and removes the runtime registration
  step. Documented in [`docs/gui-design.md`](gui-design.md).
- Renamed the spike fallback notes in PLAN.md from "Dedicated
  follow-up PR" to "Rust ⇄ Qt bridge resolution" so future readers
  understand the bridge is now live, not deferred.

## Fixed

- **gitignore parity bug** — `Gitignore::matched` doesn't match
  descendants of directory-only patterns. The fix routes those cases
  through a walker-based check in
  `crates/grexa-core/tests/gitignore_parity.rs`, and the search
  engine itself now passes `WalkBuilder::require_git(false)` so
  Grexa honors `.gitignore` files outside a repo (matches Grex
  behavior, divergence pinned in `docs/linux-decisions.md`).
- **Property test `glob_extension_match_or_miss` flake** —
  binary-extension inputs now `prop_assume!` away before the
  walker so the test doesn't surface a false negative on `.gz`.
- **`DetectedEncoding` round-trip** — switched call sites from
  `.copied()` to `.cloned()` and added `serde(default)` so saved
  search-history entries from before this release still load.

## Deprecated

- None.

## Removed

- **Windows GUI surface.** Grexa is Linux-only. The Windows port
  remains in the [`grex`](https://github.com/visorcraft/grex)
  repository; behavior contracts are inherited via
  [`docs/migration-from-grex.md`](migration-from-grex.md), not via
  source sharing.
- **`qml6`-spawn fallback host.** Replaced by the cxx-qt-based
  binary; the older host was only ever the spike-stage placeholder.

## Security

- API keys never round-trip through QML; the secret-service backend
  is the only entry point. AI features are off by design on
  headless boxes with no secret backend, so a misconfigured server
  can't leak keys through stderr or logs.
- `cargo deny check` enforces both the license allowlist
  (GPL-compatible only) and the RustSec advisory database on every
  CI run.

## Performance

- Search-engine baselines (`scripts/bench_vs_rg.sh`):
  `cargo build --release -p grexa-cli` → `hyperfine --warmup 2
  'target/release/grexa-cli ~/code/linux TODO --quiet' 'rg --quiet
  TODO ~/code/linux'`. Grexa is within 2× of `rg` on a 1.3M-file
  Linux kernel checkout on x86_64; the gap is dominated by Grexa's
  full-encoding detection pass, which is intentional. Detailed
  numbers land in `docs/perf-baselines.md` once the perf harness
  runs in CI.

## Developer notes

- New workspace deps: `cxx`, `cxx-qt`, `cxx-qt-lib`, `cxx-qt-build`
  (build-dep only). All MIT OR Apache-2.0, sourced from
  [KDAB/cxx-qt](https://github.com/KDAB/cxx-qt).
- `apps/grexa-gui/build.rs` is new. It calls
  `CxxQtBuilder::new_qml_module(...)` to register the QML module
  and bundle the `.qml` files via Qt's resource system.
- Removed workspace dep: `qmetaobject`.
- New CI jobs in `.github/workflows/ci.yml`: `build-gui` does a
  release build + offscreen smoke test under `xvfb`. The existing
  `lint` and `test` jobs now install Qt 6 dev packages so cxx-qt's
  C++ compile step can run.

## Verification

- `cargo test --workspace` → 283 tests pass (122 grexa-core unit +
  61 gitignore-parity + 4 property + 3 root-safety + 8 i18n + 16
  cli + 24 container + 17 ai + 28 GUI = 283) on Arch Linux x86_64
  with Qt 6.11.1, gcc 16.1, rustc 1.95.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo fmt --all -- --check` → clean.
- `cargo deny check` → advisories, bans, licenses, sources all OK.
- `QT_QPA_PLATFORM=offscreen target/release/grexa` → boots Qt 6,
  registers the `SearchController` QObject, loads
  `qrc:/qt/qml/com/visorcraft/Grexa/Main.qml`, and runs the Qt
  event loop until the process is killed.
- `python3 scripts/check_locale_sync.py` → all locales (en, de, ja)
  in sync.
- Live container suite: `cargo test -p grexa-containers --features
  container-live -- live::` → 4 tests pass against rootless
  Podman 5.x.

## Known issues

- The QML pages are contract-shaped placeholders. Only
  `SearchController.start_search` is wired through to actual search
  results; click-through wiring for Settings, Regex Builder,
  Context Preview, and AI Chat continues in Phase 4–18 follow-up
  PRs.
- Editing a `.qml` file in `apps/grexa-gui/qml/` requires a
  `cargo build` because cxx-qt bundles them into the binary at
  build time. Hot-reload via Qt's QML loader doesn't apply to the
  qrc-embedded path.

## Upgrade notes

- New users only — there is no prior Grexa release to upgrade from.
- Importing settings from Grex (the Windows project): follow
  [`docs/migration-from-grex.md`](migration-from-grex.md). The
  schema is intentionally JSON-compatible at the field level.

## Credits

- Authored by VisorCraft. Contributors listed in `git log`.
- Upstream lineage: [Grex](https://github.com/visorcraft/grex) on
  Windows/WinUI defined the behavior contract Grexa inherits.
- Major third-party libraries credited in
  [`CREDITS.md`](../CREDITS.md). Particular thanks to the
  [KDAB/cxx-qt](https://github.com/KDAB/cxx-qt) maintainers for
  the pure-Cargo flow that unlocked this release.

---

Template version: 2026-05-16.
