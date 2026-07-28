# Building and Testing Grexa

This guide covers local prerequisites, CLI-only and full-workspace builds,
tests, GUI validation, generated artifacts, packaging, releases, performance
checks, and common failures.

## Toolchain

The workspace uses:

- stable Rust 1.96, pinned by [`rust-toolchain.toml`](../rust-toolchain.toml);
- Rust edition 2024;
- Cargo only, with no project CMake build;
- Qt 6.4 or newer for the GUI;
- Kirigami 6 for the GUI runtime;
- `clang` and a C++ toolchain for cxx-qt generated code.

[`just`](https://just.systems/) is the command runner used by the repository.
Every recipe is a thin wrapper around Cargo or a packaging script.

```bash
cargo install just
```

The CLI and core crates do not need Qt:

```bash
cargo build -p grexa-cli
cargo test -p grexa-core
```

The first build of a fresh checkout needs network access because grexa-db is a
pinned Git dependency.

## Distro prerequisites

Package names change between distro releases. These commands match the
repository's current CI/package targets and include optional Poppler support:

### Arch, CachyOS, or Manjaro

```bash
sudo pacman -S --needed \
  rust just pkgconf clang ninja \
  qt6-base qt6-declarative qt6-tools qt6-svg \
  kirigami breeze-icons poppler
```

### Fedora

```bash
sudo dnf install \
  rust cargo just pkgconf-pkg-config clang ninja-build \
  qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qttools-devel \
  kf6-kirigami-devel poppler-utils
```

### Debian

Current Debian releases with Kirigami 6:

```bash
sudo apt install \
  rustc cargo just pkg-config clang ninja-build \
  qt6-base-dev qt6-declarative-dev qt6-tools-dev \
  qml6-module-org-kde-kirigami libgl1-mesa-dev poppler-utils
```

On a release without the Kirigami 6 QML module, CLI/core work still builds
with Rust alone. Use the AppImage or Flatpak for the GUI.

### openSUSE

```bash
sudo zypper install \
  rust cargo just pkgconf clang ninja \
  qt6-base-devel qt6-declarative-devel qt6-tools-devel \
  kirigami6-devel poppler-tools
```

If a distro Rust package is older than 1.96, install
[rustup](https://rustup.rs/). Entering the repository then selects the pinned
toolchain automatically.

## Build

Debug workspace:

```bash
just build
```

Release workspace:

```bash
just build-release
```

Binaries:

```text
target/debug/grexa
target/debug/grexa-cli
target/release/grexa
target/release/grexa-cli
```

Run through `just`:

```bash
just run-gui
just run-cli ~/code TODO --gitignore
```

Equivalent Cargo commands:

```bash
cargo build --workspace
cargo build --workspace --release
cargo run -p grexa
cargo run -p grexa-cli -- ~/code TODO
```

## Local quality gates

The required local CI-parity gate is:

```bash
just ci
```

It runs, in order:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Individual recipes:

```bash
just fmt
just fmt-check
just lint
just test
just check
```

The broader release preflight is:

```bash
just preflight
```

It adds:

```bash
cargo deny --all-features check
cargo audit
cargo about generate about.hbs --output-file docs/credits-third-party.md
```

Install the optional tools first:

```bash
cargo install cargo-deny
cargo install cargo-audit
cargo install cargo-about --features cli
```

`just preflight` regenerates the checked-in third-party supplement. Review that
diff rather than hand-editing it.

## Targeted tests

```bash
# Core unit and integration tests
cargo test -p grexa-core

# Grex-compatible ignore behavior
cargo test -p grexa-core --test gitignore_parity

# Property tests
cargo test -p grexa-core --test property

# Root and pseudo-filesystem safety
cargo test -p grexa-core --test root_safety

# Spawned CLI integration tests
cargo test -p grexa-cli

# Mocked container-runtime tests
cargo test -p grexa-containers

# Mocked HTTP, evidence, and secret-backend tests
cargo test -p grexa-ai

# Fluent bundle and locale parity
cargo test -p grexa-i18n

# Rust backing logic for QObjects; no display server needed
cargo test -p grexa
```

Do not hardcode total test counts in release checks. The suite grows and
`cargo test` is the source of truth.

## Locale validation

English is the source catalog:

```text
crates/grexa-i18n/locales/en/grexa.ftl
```

Run both the catalog/QML checker and crate tests after changing strings:

```bash
python3 scripts/check_locale_sync.py
cargo test -p grexa-i18n
```

The crate test runs in `just ci`; the Python checker adds QML-specific checks
and should be run directly for localization work.

## GUI development

The GUI crate is `grexa` under `apps/grexa-gui/`.

```bash
cargo run -p grexa
```

cxx-qt compiles every QML file into the binary's Qt resource module. Editing
QML therefore requires another Cargo build. There is no production
filesystem-hot-reload mode.

New QML files must also be added to the `qml_files` list in
[`apps/grexa-gui/build.rs`](../apps/grexa-gui/build.rs). If omitted, the file
may work from a source tree during debugging but will not ship in a package.

### Verify that the window really starts

A successful compile is not enough. Qt can reject the root QML object at
runtime while the Rust/C++ build remains green.

After a release build:

```bash
just verify-gui
```

For another binary:

```bash
just verify-gui target/debug/grexa
just verify-gui pkg:packaging/arch/grexa-<version>-1-x86_64.pkg.tar.zst
```

The verifier launches offscreen with an isolated runtime directory. Success
means the main window instantiates and the event loop remains alive. The
generic failure is:

```text
QML payload did not instantiate
```

Use this gate after every QML/controller signature change and before handing
off a package.

### Diagnose a QML load failure

1. Build the debug GUI:

   ```bash
   cargo build -p grexa
   ```

2. Locate generated QML modules:

   ```bash
   ls -d target/debug/build/grexa-*/out/qt-build-utils/qml_modules
   ```

3. Run `qmllint` with that directory:

   ```bash
   qmllint \
     -I target/debug/build/grexa-*/out/qt-build-utils/qml_modules \
     apps/grexa-gui/qml/Main.qml
   ```

4. Check cxx-qt naming. Rust snake_case becomes QML camelCase:

   ```text
   record_paths_ready -> onRecordPathsReady
   list_views         -> listViews()
   ```

A handler for a nonexistent signal is fatal to QML object creation.

## Generated CLI artifacts

```bash
just manpage
just completions
```

Outputs:

```text
target/man/grexa-cli.1
target/completions/grexa-cli.bash
target/completions/_grexa-cli
target/completions/grexa-cli.fish
```

The release workflow installs them into the standard package locations.

## Container live tests

Default tests use `MockCommandRunner` and need no daemon.

To run the opt-in live test:

```bash
cargo test -p grexa-containers --features container-live -- live::
```

The test uses Docker or Podman when reachable and skips otherwise. It creates
and removes a temporary Alpine container.

Manual Podman smoke:

```bash
podman run -d --name grexa-smoke alpine sleep 600
podman exec grexa-smoke sh -c 'echo "TODO inside container" > /tmp/grexa.txt'
target/debug/grexa-cli /tmp TODO --container grexa-smoke --runtime podman
podman rm -f grexa-smoke
```

## Packaging

The release workflow is the canonical reproducible packaging implementation.
Local helpers cover the main developer targets:

| Artifact | Command | Output |
| -------- | ------- | ------ |
| Arch / CachyOS | `just arch-package` | `packaging/arch/grexa-<version>-1-x86_64.pkg.tar.zst` |
| Fedora RPM in Fedora container | `just fedora-pkg` | path printed by the container build |
| Flatpak bundle | `just flatpak-bundle` | `target/release/grexa.flatpak` |
| AppImage or staged AppDir | `bash packaging/appimage/build.sh` | `target/appimage/Grexa-<version>-x86_64.AppImage` or `Grexa.AppDir` |

`just arch-package` also runs the packaged-GUI launch gate. It does not install
the package:

```bash
sudo pacman -U packaging/arch/grexa-<version>-1-x86_64.pkg.tar.zst
```

Debian and openSUSE recipes live under `packaging/debian/` and
`packaging/opensuse/`. Their exact clean-container commands are maintained in
[the release workflow](../.github/workflows/release.yml).

### AppImage requirements

To produce a packed AppImage, put these on `PATH`:

- `linuxdeploy`
- `linuxdeploy-plugin-qt`
- `qmake6`

The build host also needs Breeze icons and Qt SVG plugins for complete
Kirigami icon rendering. If `linuxdeploy` is absent, the script deliberately
stages only the unpacked AppDir and exits successfully with a notice.

The script's `QML_SOURCES_PATHS`, two-pass linuxdeploy flow, `QMAKE=qmake6`,
`NO_STRIP=1` retry, Breeze icons, and Qt SVG staging are required. Removing
them can produce an AppImage that builds but exits before showing a window.

Verify the result without host QML fallback:

```bash
env -u QML2_IMPORT_PATH \
  timeout 12 target/appimage/Grexa-<version>-x86_64.AppImage
```

Exit `124` or `143` means the event loop stayed alive until timeout. Exit `2`
means the QML payload failed.

### Flatpak

Install the pinned runtimes once:

```bash
flatpak remote-add --user --if-not-exists flathub \
  https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y flathub \
  org.kde.Platform//6.10 \
  org.kde.Sdk//6.10 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08
```

Then:

```bash
just flatpak-vendor
just flatpak
just flatpak-bundle
```

`flatpak-vendor` writes dependencies under `target/flatpak/vendor`; it does not
permanently redirect host Cargo. The sandbox build is offline and frozen.

The manifest grants home, `/run/media`, network for opt-in AI, Secret Service,
file-manager, and notification access. It does not expose Docker or Podman
sockets, so the Flatpak does not provide container search.

The freedesktop 25.08 Rust extension is one minor release behind the workspace
pin. Keep code compatible with its Rust 1.95 compiler until the runtime
extension catches up, and verify Flatpak after any toolchain bump.

The Flatpak intentionally installs PNG icons rather than the SVG because some
host librsvg/gdk-pixbuf combinations reject the SVG during export.

## Release process

Before tagging:

1. Update `[workspace.package].version` in `Cargo.toml`.
2. Update sibling Grexa path-dependency versions in the GUI, AI, CLI, and
   container crate manifests.
3. Update `packaging/arch/PKGBUILD`, resetting `pkgrel=1`.
4. Prepend the release in
   `packaging/com.visorcraft.Grexa.metainfo.xml`.
5. Update `packaging/debian/changelog`.
6. Update Fedora and openSUSE spec versions and changelogs.
7. Run `cargo metadata >/dev/null` to let Cargo refresh `Cargo.lock`.
8. Confirm the lockfile changed only for Grexa workspace packages.
9. Run `just preflight`.
10. Build the target package and run its GUI launch gate.

Never hand-edit `Cargo.lock`.

For a semantic version tag, the tag version must match the workspace version:

```bash
git tag -a v<version> -m "Grexa v<version>"
git push origin v<version>
```

The tag-triggered workflow builds:

- Linux x86_64 tarball;
- AppImage;
- Arch package;
- Debian packages;
- Fedora RPM;
- Flatpak bundle;
- SHA-256 checksum files.

The tarball is an unpackaged tree built against the Arch job's Qt/Kirigami
stack. It is not ABI-portable across arbitrary distributions; use the AppImage
when a self-contained GUI is needed.

It validates formatting, Clippy, all-feature tests, desktop metadata, AppStream
metadata, and GUI launch before publication.

## GitHub CI

[`ci.yml`](../.github/workflows/ci.yml) runs independent jobs for:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace --all-features`
4. release GUI build and offscreen smoke test on Arch
5. `cargo-deny`
6. AppStream metadata validation
7. desktop-file validation

`just ci` is parity for the formatting, Clippy, and default-feature workspace
test sequence. Run `just preflight` when dependencies, licenses, or release
artifacts change.

## Performance checks

Build the optimized CLI, then compare a trusted path and term with ripgrep:

```bash
cargo build --release -p grexa-cli
scripts/bench_vs_rg.sh ~/code TODO
```

The script requires `hyperfine` and `rg`, performs ten runs after warmup, and
writes `bench-results.md`. Use trusted arguments because the benchmark tool
receives command strings.

Memory ceilings and GUI snapshot policy are documented in
[Result-set memory budgets](memory-budgets.md).

## Troubleshooting

| Symptom or error | Cause and next check |
| ---------------- | -------------------- |
| Qt headers or `qmake6` not found | Install Qt 6 development packages and set `QMAKE=qmake6`. |
| Kirigami module missing at runtime | Install Kirigami 6, or use the AppImage/Flatpak. |
| `QML payload did not instantiate` | Run `qmllint`; check new QML is in `build.rs` and Rust signal/method names are camelCase in QML. |
| GUI build passes but no window appears | Run `just verify-gui`; a successful compile is not a launch test. |
| AppImage starts only with `QML2_IMPORT_PATH=/usr/lib/qt6/qml` | The bundle missed a QML module; check `QML_SOURCES_PATHS` and the two linuxdeploy passes. |
| AppImage `strip` fails on `.relr.dyn` | Re-run with `NO_STRIP=1`. |
| PDF files are skipped | Install Poppler `pdftotext`; use `GREXA_LOG=debug` for extraction errors. |
| AI key save reports backend unavailable | Start/unlock KWallet, GNOME Keyring, or another Secret Service on the session bus. |
| Container runtime not detected | Check `docker ps` or `podman ps`, socket permissions, `DOCKER_HOST`, and `XDG_RUNTIME_DIR`. |
| Container command times out | Narrow the container path/term; commands are killed after 30 seconds. |
| Search unexpectedly follows ignore files | `--gitignore` was enabled; omit it. `--include-system` is a separate control. |
| Search omits dependency/system paths | Add `--include-system`, then use explicit globs to control scope. |
| Extended regex is slow or capped | Narrow the path/globs, simplify the pattern, or use `--regex-engine fast` to reject unsupported constructs. |
| `just deny`, `audit`, or `credits` is missing | Install `cargo-deny`, `cargo-audit`, or `cargo-about` as shown above. |
