# Credits and Attribution

## Copyright

Grexa is © VisorCraft LLC and contributors, distributed under the
[GNU General Public License v3.0](LICENSE).

## Upstream project

Grexa is a Linux/Qt port of **[Grex](https://github.com/visorcraft/grex)**,
the original Windows/WinUI search-and-replace tool by VisorCraft.
Every behavior contract, search algorithm, and visual decision is
inherited from Grex unless `docs/linux-decisions.md` documents a
deliberate divergence. Grex is also GPL-3.0-licensed.

## Runtime dependencies

Grexa links against the following system runtimes at execution time.
The runtime binaries themselves are still provided by downstream
packagers (Flatpak, AppImage, distro repos), who handle their
redistribution. The full license texts for these components are now
bundled under the top-level [`LICENSES/`](LICENSES/) directory and are
viewable in-app — per-component via Credits ("View license") and
collectively under the Licenses dialog's "Runtime components" tab.

| Component | License | Project |
| --------- | ------- | ------- |
| Qt 6 (Core, Qml, Gui, Quick) | LGPL-3.0 / GPL-3.0 / commercial | https://www.qt.io |
| KDE Frameworks 6 — Kirigami | LGPL-2.1+ | https://invent.kde.org/frameworks/kirigami |
| Poppler (`pdftotext`) | GPL-2.0+ | https://poppler.freedesktop.org |
| Docker / Podman CLI | Apache-2.0 / various | https://www.docker.com, https://podman.io |
| Secret Service / KWallet / GNOME Keyring | various | freedesktop.org |

## Rust crate dependencies

Grexa pulls in the following directly-used crates from crates.io.
The full machine-generated transitive supplement — every crate, its
exact version, and the full text of every distinct license — lives
in the in-app **View Licenses** page and is mirrored at
[`docs/credits-third-party.md`](docs/credits-third-party.md).
Regenerate it via `just credits` (which runs `cargo-about` over
`Cargo.lock`). `cargo deny check` (configured in `deny.toml`) enforces
license compatibility on every CI run.

### Qt / GUI bridge

| Crate | License | Project |
| ----- | ------- | ------- |
| `cxx-qt`, `cxx-qt-lib`, `cxx-qt-build`, `cxx-qt-gen`, `cxx-qt-macro`, `qt-build-utils` | MIT OR Apache-2.0 | [KDAB/cxx-qt](https://github.com/KDAB/cxx-qt) |
| `cxx`, `cxx-build`, `cxxbridge-macro` | MIT OR Apache-2.0 | [dtolnay/cxx](https://github.com/dtolnay/cxx) |

### Search engine + filesystem

| Crate | License | Project |
| ----- | ------- | ------- |
| `regex`, `regex-automata`, `regex-syntax` | MIT OR Apache-2.0 | [rust-lang/regex](https://github.com/rust-lang/regex) |
| `fancy-regex` | MIT | [fancy-regex/fancy-regex](https://github.com/fancy-regex/fancy-regex) |
| `ignore`, `globset` | Unlicense OR MIT | [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) |
| `aho-corasick`, `memchr`, `bstr` (transitive — performance-critical search primitives) | Unlicense OR MIT | [BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick), [BurntSushi/memchr](https://github.com/BurntSushi/memchr), [BurntSushi/bstr](https://github.com/BurntSushi/bstr) |
| `encoding_rs` | (Apache-2.0 OR MIT) AND BSD-3-Clause | [hsivonen/encoding_rs](https://github.com/hsivonen/encoding_rs) |
| `chardetng` | Apache-2.0 OR MIT | [hsivonen/chardetng](https://github.com/hsivonen/chardetng) |
| `unicode-normalization` | MIT OR Apache-2.0 | [unicode-rs/unicode-normalization](https://github.com/unicode-rs/unicode-normalization) |

### Unicode / localization

| Crate | License | Project |
| ----- | ------- | ------- |
| `icu_casemap`, `icu_locale_core`, `icu_normalizer`, `icu_properties`, `icu_provider`, `icu_collections` | Unicode-3.0 | [unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `fluent`, `fluent-bundle`, `fluent-syntax`, `fluent-langneg`, `intl-memoizer`, `intl_pluralrules` | Apache-2.0 OR MIT | [projectfluent/fluent-rs](https://github.com/projectfluent/fluent-rs) |
| `unic-langid` | MIT OR Apache-2.0 | [zbraniecki/unic-locale](https://github.com/zbraniecki/unic-locale) |

### Document extraction

| Crate | License | Project |
| ----- | ------- | ------- |
| `zip` | MIT | [zip-rs/zip2](https://github.com/zip-rs/zip2) |
| `quick-xml` | MIT | [tafia/quick-xml](https://github.com/tafia/quick-xml) |

### Networking + secrets

| Crate | License | Project |
| ----- | ------- | ------- |
| `ureq` | MIT OR Apache-2.0 | [algesten/ureq](https://github.com/algesten/ureq) |
| `rustls`, `rustls-pki-types` | Apache-2.0 OR ISC OR MIT | [rustls/rustls](https://github.com/rustls/rustls) |
| `rustls-webpki` | ISC | [rustls/webpki](https://github.com/rustls/webpki) |
| `webpki-roots` | CDLA-Permissive-2.0 | [rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| `keyring`, `linux-keyutils` | MIT OR Apache-2.0 | [hwchen/keyring-rs](https://github.com/hwchen/keyring-rs) |

### Serialization + CLI plumbing

| Crate | License | Project |
| ----- | ------- | ------- |
| `serde`, `serde_derive`, `serde_json`, `serde_repr` | MIT OR Apache-2.0 | [serde-rs/serde](https://github.com/serde-rs/serde) |
| `clap`, `clap_complete`, `clap_mangen`, `clap_derive` | MIT OR Apache-2.0 | [clap-rs/clap](https://github.com/clap-rs/clap) |
| `anyhow`, `thiserror` | MIT OR Apache-2.0 | [dtolnay/anyhow](https://github.com/dtolnay/anyhow), [dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `tempfile` | MIT OR Apache-2.0 | [Stebalien/tempfile](https://github.com/Stebalien/tempfile) |
| `ctrlc` | MIT OR Apache-2.0 | [Detegr/rust-ctrlc](https://github.com/Detegr/rust-ctrlc) |

### Logging

| Crate | License | Project |
| ----- | ------- | ------- |
| `tracing`, `tracing-subscriber`, `tracing-appender`, `tracing-attributes`, `tracing-core` | MIT | [tokio-rs/tracing](https://github.com/tokio-rs/tracing) |

### Dev / test-only

| Crate | License | Project |
| ----- | ------- | ------- |
| `proptest` | MIT OR Apache-2.0 | [proptest-rs/proptest](https://github.com/proptest-rs/proptest) |
| `assert_cmd`, `predicates` | MIT OR Apache-2.0 | [assert-rs](https://github.com/assert-rs) |

## License compatibility

GPL-3.0-only is compatible with all licenses listed above. Specifically:

- MIT / Apache-2.0 / BSD-3-Clause / ISC are permissive and combine freely.
- `Unicode-3.0` (ICU4X) is FSF-approved as GPL-compatible.
- `Unlicense` (ripgrep components) is FSF-approved as GPL-compatible.
- `Zlib` (used by `foldhash`, `miniz_oxide`, `tinyvec`) is FSF-approved as GPL-compatible.
- `CDLA-Permissive-2.0` (used by `webpki-roots`) is a permissive license whose terms are compatible with redistribution under GPL-3.0.

The deny.toml allowlist enforces this. New licenses outside the
allowlist fail the `cargo deny check` step in CI.

## Reporting attribution gaps

If you find code or assets in this repository that we have failed to
credit, please open an issue at
<https://github.com/visorcraft/grexa/issues> and we will correct the
record.
