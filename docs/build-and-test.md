# Building and Testing Grexa

## Prerequisites

| Distro                 | Install command |
| ---------------------- | --------------- |
| Arch / Manjaro         | `pacman -S rust qt6-base qt6-declarative kirigami extra-cmake-modules poppler` |
| Fedora                 | `dnf install rust cargo qt6-qtbase-devel qt6-qtdeclarative-devel kf6-kirigami-devel poppler-utils` |
| Debian / Ubuntu        | `apt install rustc cargo qt6-base-dev qt6-declarative-dev qt6-tools-dev qml6-module-org-kde-kirigami clang poppler-utils` |
| openSUSE               | `zypper install rust cargo qt6-base-devel qt6-declarative-devel kirigami6-devel poppler-tools` |

Notes:

- **Rust 1.95+** is required (Rust 2024 edition). Install via your
  distro or via [rustup](https://rustup.rs/).
- **Qt 6.6+** is required only for the GUI. The CLI builds with
  the Rust toolchain alone.
- **`pdftotext`** (Poppler) is optional but unlocks PDF search.
- **`podman` or `docker`** is optional but unlocks container search.

## Build everything

```bash
cargo build --workspace --release
```

Binaries land at:

- `target/release/grexa-cli` — headless CLI
- `target/release/grexa` — Qt 6 / Kirigami GUI built via cxx-qt 0.8.
  Smoke-test with `QT_QPA_PLATFORM=offscreen target/release/grexa`.

## Tests

```bash
just ci             # format + clippy + tests, same gate CI uses

# Individual stages:
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Specific test groups (291 tests total as of v0.3):

```bash
cargo test -p grexa-core                      # 190 total (122 unit + integration)
cargo test --test gitignore_parity            # 61 cases
cargo test --test property                    # 4 proptest properties
cargo test --test root_safety                 # 3 pseudo-FS tests
cargo test -p grexa-cli                       # 16 CLI integration tests
cargo test -p grexa-containers                # 24 mocked container tests
cargo test -p grexa-ai                        # 17 mocked HTTP tests
cargo test -p grexa-i18n                      # 8 locale tests
cargo test -p grexa                           # 36 GUI controller tests (no Qt runtime)
```

## Locale sync

Whenever you add a new translation key, the English catalog at
`crates/grexa-i18n/locales/en/grexa.ftl` is the source of truth.

```bash
python3 scripts/check_locale_sync.py
```

The same check runs as a `cargo test` (`every_locale_has_same_key_set_as_english`)
so a `just ci` pass is sufficient.

## Generated artifacts

```bash
just manpage        # writes target/man/grexa-cli.1
just completions    # writes target/completions/{grexa-cli.bash,_grexa-cli,grexa-cli.fish}
```

## Dependency policy

```bash
just deny           # cargo-deny: licenses + advisories + bans
just audit          # cargo-audit: RustSec database
```

Both require `cargo install cargo-deny` / `cargo install cargo-audit`.

## Container search live tests

The default container test suite uses `MockCommandRunner` so it runs
without a daemon. To exercise the live path automatically, enable the
`container-live` Cargo feature:

```bash
# Run the live-daemon integration tests (requires podman or docker
# on $PATH; tests no-op when neither is reachable).
cargo test -p grexa-containers --features container-live -- live::
```

The live tests spawn a throwaway Alpine container, run direct grep
inside it, exercise the archive mirror fallback, and clean up on the
way out.

You can also exercise the live path manually:

```bash
# Start a podman container and search inside it.
podman run -d --name web alpine sleep 600
podman exec web sh -c 'echo "TODO inside container" > /etc/grexa-todo.txt'
grexa-cli /etc TODO --container web --runtime podman
podman rm -f web
```

## GUI prerequisites + dev cycle

The Qt/Kirigami shell is the `grexa` binary under `apps/grexa-gui/`:

```bash
# Iterate on QML / Rust together:
cargo run -p grexa

# Or with cargo-watch for an auto-rebuild on save:
cargo install cargo-watch
cargo watch -x 'run -p grexa'
```

Note: QML files are bundled into the binary as Qt resources at
build time via `cxx-qt-build`'s `qrc_resources`. Editing a `.qml`
file requires a `cargo build` — there is no filesystem hot-reload
path. New `.qml` files must be added to the `qml_files` list in
`apps/grexa-gui/build.rs` or they won't ship.

## Cross-distro container build

For reproducible packaging builds:

```bash
podman build -t grexa-builder -f packaging/Dockerfile.builder .
podman run --rm -v "$PWD:/src" -w /src grexa-builder \
    cargo build --workspace --release
```

## What CI runs

See [`.github/workflows/ci.yml`](../.github/workflows/ci.yml):

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace --all-features`
4. `cargo deny check`
5. `appstreamcli validate packaging/io.visorcraft.Grexa.metainfo.xml`
6. `desktop-file-validate packaging/io.visorcraft.Grexa.desktop`

## Performance baselines

Run the search engine against a representative source tree:

```bash
cargo build --release -p grexa-cli
hyperfine --warmup 2 \
    'target/release/grexa-cli ~/code/linux TODO --quiet' \
    'rg --quiet TODO ~/code/linux'
```

A spreadsheet of comparison results lives at
[memory-budgets.md](memory-budgets.md).

## Troubleshooting

| Symptom | Likely cause |
| ------- | ------------ |
| `pdftotext: not found` skipped PDFs | install Poppler (`pdftotext`) |
| `keyring backend unavailable` AI errors | start KWallet / GNOME Keyring; on headless boxes, AI features are off by design |
| Slow regex search on lookaround patterns | the cascade fell through to `fancy-regex`; `tracing::info!` logs note this |
| Search misses files because of `.gitignore` | the engine respects gitignore even without `.git/`; pass `--include-system` to ignore the rules |
| Container search exits with status 125 | `docker`/`podman` CLI returned a permission error; check the socket perms |
