<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Contributing to Grexa

Thank you for helping improve Grexa. This project is a Rust workspace
with a Qt 6 / Kirigami GUI, a scriptable CLI, and a shared search
engine. Changes should be small, tested, and aligned with the existing
crate boundaries.

## Contribution workflow

1. Fork the repository on GitHub.
2. Clone your fork:

   ```bash
   git clone https://github.com/<you>/grexa.git
   cd grexa
   ```

3. Create a focused branch:

   ```bash
   git checkout -b fix-search-filter
   ```

4. Install the development prerequisites from
   [docs/build-and-test.md](docs/build-and-test.md).
5. Make the smallest change that fully solves the issue.
6. Add or update tests and documentation.
7. Run the local gate:

   ```bash
   just ci
   ```

8. Push your branch and open a pull request against `main`.

Pull requests should include a clear summary, the tests you ran, and
screenshots or terminal output when the change affects the GUI or CLI
output.

## Project layout

- `crates/grexa-core` contains search, replace, filtering, encoding,
  document extraction, settings, history, profiles, and desktop helper
  logic.
- `crates/grexa-cli` contains the `grexa-cli` binary and command-line
  interface.
- `crates/grexa-containers` contains Docker and Podman adapters.
- `crates/grexa-ai` contains the optional OpenAI-compatible client and
  Secret Service key storage.
- `crates/grexa-i18n` contains Fluent catalogs and localization
  helpers.
- `apps/grexa-gui` contains the Qt 6 / Kirigami app and cxx-qt bridge.
- `docs/` contains user docs, architecture docs, and Grex behavior
  audits.
- `packaging/` contains distro, Flatpak, AppImage, desktop, metainfo,
  and icon assets.

Keep algorithmic behavior in `grexa-core`. The GUI and CLI should call
shared core APIs instead of reimplementing search or replace logic.

## Local development

Use `just` when available:

```bash
just build          # cargo build --workspace
just build-release  # cargo build --workspace --release
just test           # cargo test --workspace
just lint           # cargo clippy --workspace --all-targets -- -D warnings
just fmt            # cargo fmt --all
just ci             # fmt check + clippy + tests
```

Direct `cargo` commands are acceptable when `just` is not installed.

Qt is required only for the GUI crate. CLI and core work can be built
and tested without Qt:

```bash
cargo build -p grexa-cli --release
cargo test -p grexa-core
```

## Coding standards

- Use stable Rust 1.95+ and edition 2024. Do not require nightly.
- Keep `grexa-core` synchronous. Do not introduce an async runtime.
- Follow the existing crate boundaries and adjacent code style.
- Prefer focused, explicit code over broad abstractions.
- Use `tracing::*!` for structured logs.
- Route user-facing Rust strings through `grexa-i18n::Bundle`.
- Route user-facing QML strings through `i18n(...)`.
- Add concise comments only when the reason is not obvious from the
  code.
- Do not hand-edit `Cargo.lock`; let Cargo update it.
- Do not add speculative feature flags.
- Do not reintroduce `qmetaobject`; cxx-qt is the Rust / Qt bridge.
- Do not add CMake requirements. The workspace builds through Cargo.

Every new source file must include the SPDX short header used by the
repository:

```text
SPDX-FileCopyrightText: 2026 VisorCraft LLC
SPDX-License-Identifier: GPL-3.0-only
```

Use the comment syntax appropriate for the file type.

## GUI and QML changes

- cxx-qt bridge modules live under `apps/grexa-gui/src/qobjects/`.
- Each QObject should keep Rust-side state in its backing `*Rust`
  struct so logic can be unit-tested without a Qt runtime.
- New QML files must be listed in `apps/grexa-gui/build.rs`; otherwise
  they will not be bundled into the binary.
- QML files are compiled into Qt resources at build time. Rebuild after
  editing QML.
- Keep GUI strings localized through Fluent keys.

## Tests

Match test coverage to the risk of the change:

- Core search, replace, filtering, settings, history, profile, and
  encoding changes need Rust unit or integration tests.
- CLI flag or output changes need CLI integration coverage in
  `crates/grexa-cli/tests/`.
- Container behavior should use the `CommandRunner` mock boundary
  unless the test explicitly belongs under the live container feature.
- GUI bridge logic should be tested against Rust backing structs where
  possible.
- Filesystem behavior should use `tempfile::TempDir` against the real
  filesystem.

Run the same gate CI uses before opening a pull request:

```bash
just ci
```

When changing dependencies, also run:

```bash
just deny
```

## Localization

English is the source catalog:

```text
crates/grexa-i18n/locales/en/grexa.ftl
```

When adding or renaming a Fluent key:

1. Add the English key.
2. Add the same key to every shipped locale.
3. Use a placeholder translation if a final translation is not ready.
4. Run the locale sync check:

   ```bash
   python3 scripts/check_locale_sync.py
   ```

The same parity check runs in the test suite.

## Documentation

Update documentation in the same pull request when behavior changes.

- User workflows belong in [docs/usage.md](docs/usage.md).
- CLI flags, settings, paths, and shortcuts belong in
  [docs/reference.md](docs/reference.md).
- Architecture or crate-boundary changes belong in
  [docs/architecture.md](docs/architecture.md).
- GUI bridge changes belong in [docs/gui-design.md](docs/gui-design.md).
- Intentional divergences from upstream Grex belong in
  [docs/linux-decisions.md](docs/linux-decisions.md).
- Behavior covered by a `docs/grex-*-audit.md` file must keep that
  audit document accurate.

## Dependency policy

Grexa is GPL-3.0-only. New dependencies must be compatible with the
allowlist in `deny.toml`. If a dependency needs a license
clarification, include the reason in the pull request and update
`deny.toml` in the same change.

Avoid new dependencies unless they clearly reduce complexity or provide
well-tested domain behavior that should not be maintained locally.

## Pull request expectations

A good pull request:

- Has one clear purpose.
- Describes user-visible behavior changes.
- Calls out migrations or compatibility risks.
- Includes tests, or explains why tests are not practical.
- Updates docs and localization when needed.
- Passes `just ci`.
- Avoids unrelated formatting or refactoring churn.

Maintainers may ask for smaller commits, additional tests, or docs
updates before merging.

## Security

Do not report security issues through public issues or pull requests.
Follow the disclosure policy in [docs/security.md](docs/security.md).
