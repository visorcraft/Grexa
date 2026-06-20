<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

<p align="center">
  <img src="packaging/icons/512x512/apps/com.visorcraft.Grexa.png" alt="Grexa logo" width="250">
</p>

<h1 align="center">Grexa</h1>

<p align="center">
  <strong>Fast, precise file-content search for Linux.</strong>
</p>

<p align="center">
  A Qt 6 / Kirigami desktop app and scriptable Rust CLI for searching,
  filtering, previewing, and safely replacing text across local files,
  documents, and containers.
</p>

## What is Grexa?

Grexa is a Linux-native port of
[Grex](https://github.com/visorcraft/grex), rebuilt as a Rust workspace
with a Qt 6 / Kirigami interface. It is designed for developers and
power users who need fast local search with predictable filters,
grep-style automation, and a polished desktop workflow.

Grexa can:

- Search by literal text or regex, including advanced regex features
  through a fast `regex` / `fancy-regex` cascade.
- Respect `.gitignore`, hidden-file settings, glob filters, size
  filters, binary-file rules, symlinks, and recursive directory
  options.
- Preview matches with file path, line, column, encoding, modified
  time, and sorted result views.
- Replace text safely with atomic file writes and a replace journal.
- Search extracted text from OOXML, ODF, and PDF documents.
- Search inside Docker or Podman containers.
- Run as either the `grexa` desktop app or the `grexa-cli` command.
- Chat with an optional, opt-in AI panel (any OpenAI-compatible
  endpoint) that can summarize the current search results over the
  matched lines.
- Store API keys in the Linux Secret Service when optional AI features
  are configured.

## Plain files all the way down

Most apps bury your data in an opaque blob — a SQLite file, a settings
database, a binary cache — that only the app can read. Grexa does the
opposite. Its storage layer is **[grexa-db](https://github.com/visorcraft/grexa-db)**,
a standalone (`Apache-2.0`) flat-file database engine where **every record is a
plain Markdown file** and **a query is just the filesystem**. That one decision
shows up in Grexa two ways.

### It remembers everything — in files you own

Your **recent folders**, **search history**, and **saved search profiles**
aren't trapped in a binary blob. Each is a human-readable Markdown record under
`~/.local/share/grexa/db/`, with YAML frontmatter and a schema you can read:

```console
$ cat ~/.local/share/grexa/db/recent_paths/entry-*.md
---
path: /home/you/projects/kernel
added_at: 1718706000000000000
---
```

So your own data is greppable, diffable, versionable, and portable with the
tools you already have — `grep` it, track it in `git`, back it up with `cp`,
sync it with Syncthing. **If Grexa vanished tomorrow, every byte is still
readable.** No export step, no proprietary format, no lock-in. (Grexa writes
these records atomically — temp file + rename — so a crash never leaves a torn
entry.)

### A database browser, built in

Open **Tools → Database** in the desktop app and point it at *any* grexa-db
directory — including Grexa's own. From there you can list collections, run
typed filter queries, validate records against their schema, and **materialize
a query as a real directory of symlinks** you can open in any file manager.
Point it at `~/.local/share/grexa/db` and you'll browse the very history and
profiles the app has been quietly writing.

### Benchmarks: where flat files beat a binary database

grexa-db is not trying to out-race SQLite on a million-row `JOIN` — its own
[design spec](https://github.com/visorcraft/grexa-db/blob/master/docs/grexa-db-design.md)
says a real database wins past ~250k records. What the flat-file design *buys*
you is everything below. Every number is measured by a deterministic,
fixed-seed benchmark holding the **same 5,000 records** three ways: as grexa-db
records, as a SQLite database, and as a single JSON blob (what Grexa used
*before* grexa-db).

| # | Property | grexa-db | The "standard" way |
|---|----------|----------|--------------------|
| 1 | Records readable with **zero database software** | **5,000 / 5,000** (`cat note.md`) | SQLite: **0** — it's a binary blob |
| 2 | Tools that can query it with **no driver** | **7** — `grep` `rg` `awk` `find` `git` `sed` `fzf` | SQLite: **1** (`sqlite3`, SQL only) |
| 3 | A one-field edit is a **human-reviewable diff** | **2-line** `git diff` | SQLite: `Binary files differ` — **0** reviewable lines |
| 4 | **Incremental backup** of that one-field edit | **195 bytes** re-transmitted | SQLite: **4,064 bytes** — ≈ **21× more** |
| 5 | **Blast radius** of one corrupted byte | **1 record** lost; **4,999** still readable | SQLite: header byte → **whole DB unreadable** (0/5,000) |
| 6 | Engine **footprint / supply chain** | **19** pure-Rust crates, **0** C libraries | SQLite: a ~1.6 MB C library linked in + a C build step |
| 7 | **Peak RAM** to scan-filter **40,000** records | **7.7 MB** — streams one record at a time | Load-the-blob: **53 MB** — ≈ **7× more** |
| 8 | **Open + first answer** | **0.8 ms** | Parse-the-blob: **30 ms** — ≈ **37× slower** |
| 9 | **Crash mid-write** (`SIGKILL`) | **0** partial records across **47,000** writes | In-place JSON rewrite: file truncated to **0 bytes** |
| 10 | **Merge** two datasets | `cp -r` — **0** lines of SQL, ~2 ms | SQLite: `ATTACH` + `INSERT…SELECT`, or dump + reload |
| 11 | Query results are **real directories** | **983** symlinks in `views/by-rating/5/`, usable by `ls`/`find`/`du`/`fzf` | SQL views: **0** filesystem objects |
| 12 | **Add a new field** | **0** `ALTER TABLE`, **0** rows rewritten; old records still query | SQLite: `ALTER TABLE` + a migration |

Reproduce all of it (deterministic — fixed seed, writes `bench-results.json`):

```bash
git clone https://github.com/visorcraft/grexa-db && cd grexa-db
cargo build --release -p grexa-db-cli
python3 scripts/bench.py          # optional: N=20000 N_BIG=100000 python3 scripts/bench.py
```

> Absolute numbers are from one CachyOS / AMD box (5,000 records; the memory
> test uses 40,000). Your machine's numbers will differ — the **ratios** are
> the point, and the script regenerates them on your hardware.

### Scaling

The table above is grexa-db's behavior *today*. Read+parse dominates query time
and **now runs in parallel across all cores** (shipped in grexa-db; results are
byte-identical to the serial path, verified). Measured on **200,000 records**
(16-core box):

| Operation | Before (serial) | After (parallel) | Gain |
|---|---|---|---|
| selective filter (100 hits) | 1005 ms | **208 ms** | **4.8×** |
| broad filter (80k hits) | 1017 ms | **276 ms** | **3.7×** |
| list all records | 992 ms | **375 ms** | **2.6×** |
| `order_by` (sort 200k) | 1669 ms | **851 ms** | **2.0×** |

A **secondary index** is also shipped — and wired into the **Database browser**,
which holds it in memory and keeps it fresh via `inotify`. It's a derived
`.grexa-index/` sidecar (rebuildable — delete it and every record is still
intact). Held that way, a **selective query drops from 188 ms to 0.63 ms
(297×)**, byte-identical to a scan, with verify-on-read so a stale index can
never return a wrong match and a selectivity guard so broad queries fall back to
the parallel scan. The directory walk is linear to 1M records (no cliff). A
frontmatter fast-path (→ ~10× on the cold scan) remains prototyped. Full method,
numbers, and design in grexa-db's
[scaling R&D](https://github.com/visorcraft/grexa-db/blob/master/docs/grexa-db-scaling-rnd.md).

## Setup

### Requirements

- Linux on Wayland or X11. KDE Plasma 6 is the primary desktop target.
- Qt 6.6+ and Kirigami 6 for the GUI.
- Rust 1.96+ only when building from source.
- Optional: `pdftotext` from Poppler for PDF search.
- Optional: Docker or Podman for container search.
- Optional: KWallet or GNOME Keyring for AI-provider keys.

### Install development packages

Use your distro's package manager before building from source. The
development packages also satisfy the GUI runtime requirements on most
systems.

| Distro | Command |
| ------ | ------- |
| Debian / Ubuntu | `sudo apt install rustc cargo qt6-base-dev qt6-declarative-dev qt6-tools-dev clang poppler-utils` |
| Fedora | `sudo dnf install rust cargo qt6-qtbase-devel qt6-qtdeclarative-devel kf6-kirigami-devel clang poppler-utils` |
| Arch / Manjaro | `sudo pacman -S rust qt6-base qt6-declarative kirigami clang poppler` |
| openSUSE | `sudo zypper install rust cargo qt6-base-devel qt6-declarative-devel kirigami6-devel clang poppler-tools` |

Ubuntu 24.04 does not currently package the Qt 6/KF6 Kirigami QML
runtime used by the GUI. The Debian / Ubuntu package list above is
enough for source builds and CLI development; run the GUI on a distro
that ships Kirigami 6, or install Kirigami 6 from KDE/distro packages
when available.

The repository uses [`just`](https://just.systems/) for common tasks.
If it is not installed, the equivalent `cargo` commands still work.

```bash
cargo install just
```

## Install

### From a GitHub Release

Download the latest `grexa-<version>-linux-x86_64.tar.gz` from the
repository's GitHub Releases page, then unpack it:

```bash
tar -xzf grexa-<version>-linux-x86_64.tar.gz
cd grexa-<version>-linux-x86_64

./bin/grexa
./bin/grexa-cli --help
```

To install the archive into `/usr/local`:

```bash
sudo install -Dm755 bin/grexa /usr/local/bin/grexa
sudo install -Dm755 bin/grexa-cli /usr/local/bin/grexa-cli
sudo cp -a share/. /usr/local/share/
```

### From source

```bash
git clone https://github.com/visorcraft/grexa.git
cd grexa

just ci
just build-release

target/release/grexa
target/release/grexa-cli --help
```

CLI-only builds do not need Qt:

```bash
cargo build -p grexa-cli --release
target/release/grexa-cli ~/code TODO
```

### Packaging

Packaging recipes live under [`packaging/`](packaging/), including
Flatpak, AppImage, Debian, Fedora, openSUSE, and Arch/CachyOS
metadata. See [docs/build-and-test.md](docs/build-and-test.md) for
packaging commands and release automation details.

## Tweak Grexa

### Common CLI workflows

```bash
# Basic content search
grexa-cli ~/code TODO

# Regex search
grexa-cli ~/code 'fn\s+\w+_test' --regex --case-sensitive

# JSON output for scripts
grexa-cli ~/code TODO --format json | jq '.[] | .full_path'

# Search inside a Podman container
grexa-cli /etc TODO --container web --runtime podman

# Generate shell completions
grexa-cli completions bash > ~/.local/share/bash-completion/completions/grexa-cli
```

### Desktop settings

The GUI settings page auto-saves changes. Grexa stores local app data
under standard XDG locations:

| Data | Default path |
| ---- | ------------ |
| Settings | `~/.config/grexa/settings.json` |
| Recent paths, history, profiles | `~/.local/share/grexa/` |
| Logs and replace journal | `~/.local/state/grexa/` |

Set `GREXA_LOG` to tune logging:

```bash
GREXA_LOG=debug grexa
```

### Optional integrations

- PDF search uses `pdftotext` when available.
- Container search uses Docker or Podman from `PATH`.
- AI-provider keys are stored in the system Secret Service, not in QML
  or plain-text config files.
- Localization currently ships English, German, and Japanese catalogs.

Full usage details are in [docs/usage.md](docs/usage.md). CLI flags,
settings, paths, and keyboard shortcuts are in
[docs/reference.md](docs/reference.md).

## Contribute

Contributions are welcome through the standard fork-and-pull-request
workflow. Start with [CONTRIBUTING.md](CONTRIBUTING.md), which covers
local setup, coding standards, tests, documentation expectations,
localization rules, dependency policy, and pull request requirements.

The short version:

```bash
git clone https://github.com/<you>/grexa.git
cd grexa
git checkout -b fix-or-feature-name

just ci
```

Before opening a pull request, include focused tests for behavior
changes, update relevant docs, and make sure `just ci` passes.

## Documentation

- [docs/features.md](docs/features.md) — feature inventory
- [docs/usage.md](docs/usage.md) — user workflows
- [docs/reference.md](docs/reference.md) — settings and CLI reference
- [docs/build-and-test.md](docs/build-and-test.md) — build, test, and packaging guide
- [docs/architecture.md](docs/architecture.md) — workspace architecture
- [docs/gui-design.md](docs/gui-design.md) — Qt / cxx-qt bridge design
- [docs/translations.md](docs/translations.md) — localization workflow
- [docs/SECURITY.md](docs/SECURITY.md) — threat model and disclosure policy
- [docs/feature-parity.md](docs/feature-parity.md) — Grex / Grexa parity matrix

## License

Grexa is licensed under GPL-3.0-only, matching the upstream Grex
project. See [LICENSE](LICENSE) for the full text and
[CREDITS.md](CREDITS.md) for third-party attribution.
