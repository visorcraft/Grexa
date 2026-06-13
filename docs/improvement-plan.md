# Grexa Improvement Plan

Generated after validating five candidate tasks with Claude CLI. This document
records the feedback, the final scope adjustments, and a concrete implementation
plan for the legitimate work.

## Validation summary

| # | Candidate task | Verdict | Action |
|---|----------------|---------|--------|
| 1 | Enforce 512 MiB read cap in replace | ✅ Legitimate hardening | Proceed, refocus on container path and shared constant |
| 2 | Cache per-container grep availability | ✅ Legitimate | Implemented; global `(runtime_kind, container_id)` cache with test-level invalidation |
| 3 | Finish wiring reduced-motion accessibility | ✅ Legitimate bug fix | Proceed; 6 hard-coded lines, split into token swaps and infinite-spinner design |
| 4 | Smoke-test AppImage in release CI | ✅ Legitimate, highest value | Proceed first; add offscreen-platform smoke run |
| 5 | Expose Fluent bundle to QML + backfill | ✅ Legitimate | Implemented; generic `i18n()` / `i18nPlural()` helpers, all `qsTr()` migrated, locales synced |

## Recommended execution order

1. **AppImage CI smoke test** — contained, guards a documented, repeated failure mode.
2. **Reduced-motion accessibility** — fixes a user-visible promise the UI already makes.
3. **Replace read-cap hardening** — closes a real container-only gap.
4. **Container grep caching** — implemented with a global `(runtime_kind, container_id)` cache.
5. **QML Fluent backfill + `i18n()` helper** — implemented; all shipped QML now routes through Fluent.

---

## Task 1: AppImage CI smoke test

### Goal
Ensure the release pipeline executes the produced AppImage so a broken
QML bundle fails the build before artifacts are published.

### Scope
- Add one job step (or extend the existing `linux-tarball` / AppImage job) that
  runs the AppImage with the system QML import path removed.
- Use `QT_QPA_PLATFORM=offscreen` because the runner has no display.
- Run for a bounded time; success means the process does not exit with code 2
  (the documented "QML payload did not instantiate" failure mode).

### Acceptance criteria
- `.github/workflows/release.yml` contains a step that executes
  `env -u QML2_IMPORT_PATH QT_QPA_PLATFORM=offscreen timeout 12 target/appimage/Grexa-*-x86_64.AppImage`
  and expects exit code `124` (timeout) or non-`2`.
- A deliberate break (e.g. omitting `QML_SOURCES_PATHS`) would fail the release
  workflow before artifacts are uploaded.

### Key files
- `.github/workflows/release.yml`
- `packaging/appimage/build.sh`
- `AGENTS.md`

---

## Task 2: Finish reduced-motion accessibility wiring

### Goal
Honor the existing "Reduce motion" setting for every animation in the GUI.

### Scope
Only six hard-coded durations remain. Split into two groups:

1. **Token swap (one line)**
   - `apps/grexa-gui/qml/AppCheckBox.qml:37` — change `duration: 110` to
     `duration: DesignTokens.durationSnap`.

2. **Infinite busy spinners (five lines)**
   - `apps/grexa-gui/qml/SearchBar.qml:254`
   - `apps/grexa-gui/qml/SearchPage.qml:691,692`
   - `apps/grexa-gui/qml/SearchPage.qml:1008,1009`
   - These loops must not simply swap to `0` (which freezes the indicator).
     Instead, when reduced motion is enabled, stop/hide the spinner or replace
     it with a static "busy" indicator. `SettingsPage.qml:542` already promises
     the user that reduced motion "disables busy spinners," so hiding them is
     the intended behavior.

### Acceptance criteria
- With "Reduce motion" off, all existing animations still run.
- With "Reduce motion" on, the six listed animations are either instantaneous
  (token swap) or hidden/disabled (infinite spinners).
- A quick grep for `duration:` or `loops: Animation.Infinite` in the listed
  files finds no remaining hard-coded durations.

### Key files
- `apps/grexa-gui/qml/DesignTokens.qml`
- `apps/grexa-gui/qml/SearchPage.qml`
- `apps/grexa-gui/qml/SearchBar.qml`
- `apps/grexa-gui/qml/AppCheckBox.qml`

---

## Task 3: Enforce 512 MiB read cap in replace

### Goal
Prevent `replace.rs` from loading multi-gigabyte files into memory.

### Scope adjustment
The dominant `replace_with` path is already protected upstream because it
replays search results. The real reachable gap is the container path, which
calls `replace_file` directly without going through `search_with`.

- Promote `MAX_SEARCH_FILE_BYTES` out of `crates/grexa-core/src/search.rs` into
  a shared location (e.g. `crates/grexa-core/src/models.rs` or a new
  `crates/grexa-core/src/constants.rs`).
- Add a size guard in `crates/grexa-core/src/replace.rs` inside
  `read_regular_text` / `rewrite_one_pre_read` that rejects files larger than
  the shared cap.
- Ensure `replace_file` and `replace_with` surface a clear error for oversize
  files rather than silently skipping.

### Acceptance criteria
- A 1 GiB test file passed to `replace_file` returns an error, not an OOM.
- Container replace also rejects oversize files through the same guard.
- `MAX_SEARCH_FILE_BYTES` is defined once and used by both search and replace.
- Existing replace tests still pass; a new unit test covers the oversize case.

### Key files
- `crates/grexa-core/src/search.rs`
- `crates/grexa-core/src/replace.rs`
- `crates/grexa-core/src/models.rs` (or new constants module)
- `crates/grexa-containers/src/search.rs`

---

## Task 4: Cache per-container grep availability

### Implementation
A global `OnceLock<Mutex<HashMap<(ContainerRuntimeKind, String), bool>>>` now
stores the result of `which grep` per `(runtime_kind, container_id)`. The cache
is populated on first probe and reused for the lifetime of the process. A
`clear_grep_availability_cache()` helper keeps container unit tests isolated
by giving each fake container a unique id.

---

## Task 5: QML Fluent backfill + `i18n()` helper

### Implementation
`SearchController` now exposes `i18n(key)` and `i18n_plural(key, n)` invokables
that read the workspace Fluent bundle. `Main.qml` re-exports these as
`app.i18n()` and `app.i18nPlural()` so every QML file can use them. All
`qsTr()` calls in shipped QML were migrated to Fluent keys, and `de`/`ja`
catalogs were backfilled to keep the locale-sync gate green.

`docs/translations.md` now documents the QML pattern, and
`scripts/check_locale_sync.py` treats zero `qsTr()` calls as the expected
post-migration state.

### Acceptance criteria
- Switching `ui_language` in settings changes all user-visible strings in the
  GUI, not just status-bar plural counts.
- `scripts/check_locale_sync.py` and the `every_locale_has_same_key_set_as_english`
  test continue to pass.
- No `qsTr()` calls remain in shipped QML.

### Key files
- `apps/grexa-gui/src/qobjects/search.rs`
- `apps/grexa-gui/src/workspace.rs`
- `apps/grexa-gui/qml/*.qml`
- `crates/grexa-i18n/src/lib.rs`
- `crates/grexa-i18n/locales/*/grexa.ftl`
- `docs/translations.md`

---

## Next step recommendation

Start with **Task 1 (AppImage CI smoke test)**, then **Task 2 (reduced-motion)**,
then **Task 3 (replace read cap)**. Each is small, well-scoped, and ships
independent value. Treat Task 5 as a larger follow-up with its own design doc.