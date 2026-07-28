Name:           grexa
Version:        1.11.2
Release:        1%{?dist}
Summary:        Fast Linux file content search with tabs, replace, and AI assistance

License:        GPL-3.0-only
URL:            https://github.com/visorcraft/Grexa
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  qt6-qtbase-devel
BuildRequires:  qt6-qtdeclarative-devel
BuildRequires:  qt6-qttools-devel
BuildRequires:  kf6-kirigami-devel
BuildRequires:  pkgconf-pkg-config
BuildRequires:  desktop-file-utils
BuildRequires:  libappstream-glib

Requires:       qt6-qtbase
Requires:       qt6-qtdeclarative
Requires:       kf6-kirigami
Recommends:     poppler-utils
Recommends:     podman
Suggests:       docker-ce
Suggests:       gnome-keyring
Suggests:       kwalletmanager

%description
Grexa is a fast, precise grep-style search workbench for Linux developers.
It feels at home on KDE Plasma, integrates with the Breeze icon set, and
respects the system color scheme by default. The CLI is available as
grexa-cli; the GUI as grexa.

%prep
%setup -q -n %{name}-%{version}

%build
cargo build --workspace --release --locked

%install
install -Dm755 target/release/grexa %{buildroot}%{_bindir}/grexa
install -Dm755 target/release/grexa-cli %{buildroot}%{_bindir}/grexa-cli

install -Dm644 packaging/com.visorcraft.Grexa.desktop \
    %{buildroot}%{_datadir}/applications/com.visorcraft.Grexa.desktop
install -Dm644 packaging/com.visorcraft.Grexa.metainfo.xml \
    %{buildroot}%{_datadir}/metainfo/com.visorcraft.Grexa.metainfo.xml
install -Dm644 packaging/icons/scalable/com.visorcraft.Grexa.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/com.visorcraft.Grexa.svg
for sz in 16 24 32 48 64 96 128 192 256 512; do
    install -Dm644 packaging/icons/${sz}x${sz}/apps/com.visorcraft.Grexa.png \
        %{buildroot}%{_datadir}/icons/hicolor/${sz}x${sz}/apps/com.visorcraft.Grexa.png
done

target/release/grexa-cli manpage > grexa-cli.1
install -Dm644 grexa-cli.1 %{buildroot}%{_mandir}/man1/grexa-cli.1

install -d %{buildroot}%{_datadir}/bash-completion/completions
install -d %{buildroot}%{_datadir}/zsh/site-functions
install -d %{buildroot}%{_datadir}/fish/vendor_completions.d
target/release/grexa-cli completions bash \
    > %{buildroot}%{_datadir}/bash-completion/completions/grexa-cli
target/release/grexa-cli completions zsh \
    > %{buildroot}%{_datadir}/zsh/site-functions/_grexa-cli
target/release/grexa-cli completions fish \
    > %{buildroot}%{_datadir}/fish/vendor_completions.d/grexa-cli.fish

# Optional validators — skipped if the helper isn't installed (non-Fedora
# rpmbuild hosts).
command -v desktop-file-validate >/dev/null && \
    desktop-file-validate %{buildroot}%{_datadir}/applications/com.visorcraft.Grexa.desktop || :
command -v appstream-util >/dev/null && \
    appstream-util validate-relax \
        %{buildroot}%{_datadir}/metainfo/com.visorcraft.Grexa.metainfo.xml || :

%files
%license LICENSE
%doc README.md docs/*.md
%{_bindir}/grexa
%{_bindir}/grexa-cli
%{_datadir}/applications/com.visorcraft.Grexa.desktop
%{_datadir}/metainfo/com.visorcraft.Grexa.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/com.visorcraft.Grexa.svg
%{_datadir}/icons/hicolor/*x*/apps/com.visorcraft.Grexa.png
%{_mandir}/man1/grexa-cli.1*
%{_datadir}/bash-completion/completions/grexa-cli
%{_datadir}/zsh/site-functions/_grexa-cli
%{_datadir}/fish/vendor_completions.d/grexa-cli.fish

%changelog
* Tue Jul 28 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.11.2-1
- Keep pages at the full page-stack width so content no longer collapses into
  a narrow column.

* Mon Jul 27 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.11.1-1
- Add a documentation index and rebuild the public guides around current GUI,
  CLI, storage, AI, container, and packaging behavior.
- Expand settings, security, resource-limit, accessibility, migration, and
  troubleshooting documentation.
- Correct cxx-qt bridge documentation and refresh third-party credits.

* Fri Jul 24 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.11.0-1
- Align the full cxx-qt stack to 0.9 (cxx-qt, cxx-qt-lib, cxx-qt-build,
  qt-build-utils) so the GUI build and runtime share one generation path.
- Upgrade third-party crates and CI actions, including quick-xml 0.41
  (RUSTSEC), notify 8, regex 1.13, ignore, anyhow, clap_complete, cxx,
  vergen-git2, actions/checkout 7, and cargo-deny-action.
- Bump crossbeam-epoch past RUSTSEC-2026-0204 and restore a green CI gate.

* Sun Jul 20 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.10.2-1
- Keep the Search page alive when switching sidebar pages so path/term/tabs
  survive navigation (e.g. About → Search).
- Open the folder browser on the path already in the search bar.
- Apply Browse → Open into the path field; recent-path history updates no
  longer clear the editable path combo.

* Fri Jul 10 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.10.1-1
- Track grexa-db 1.10.0 so the engine dependency stays version-aligned with
  Grexa. No functional changes from 1.10.0.

* Fri Jul 10 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.10.0-1
- Bound per-tab result snapshots (tab limit + aggregate row budget) and stop
  duplicating the active tab's rows on restore; fixes session-long memory
  growth that forced a restart.
- Coalesce search streaming to one GUI update per frame and thread cancellation
  into per-line matching; cap and cancel the replace collectors.

* Sat Jun 27 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.9.1-1
- Bound the regex replace scan per file (CPU deadline + match cap) so a
  pathological pattern can no longer peg a core on a large file, matching the
  search hot path's existing guards.
- Remove hardcoded absolute build paths from committed config and docs.
- Pin the grexa-db git dependency to a release tag for reproducible builds and
  bring the cargo-deny policy check back to green.

* Sat Jun 20 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.9.0-1
- AI "Summarize results" now packs a few lines of context around each match
  (not just the matched line), with the hit flagged, so the assistant can
  answer about the surrounding code. Budget-bounded and disclosed as before.

* Fri Jun 19 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.8.1-1
- Add AI "Summarize results": the AI Search panel summarizes the on-screen
  matches (path, line, matched text), packed breadth-first into a bounded
  prompt budget. Settings adds an AI excerpt-budget slider to tune it.
- Build on the Rust 1.96 toolchain (workspace pin and release CI).

* Thu Jun 18 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.7.1-1
- Fix the GUI failing to launch: the database browser page used signal-handler
  names that did not match the controller's signals, which prevented the main
  window from instantiating. No other changes.

* Thu Jun 18 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.7.0-1
- Extract the grexa-db storage engine into its own repository; consume it as a
  pinned git dependency instead of an in-tree workspace member. No user-visible
  change.
- Harden reference-path validation and view deletion in the storage engine.

* Wed Jun 17 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.6.0-1
- Refresh all third-party crates to their latest compatible versions.
- Migrate API-key storage to keyring-core with the zbus secret-service backend;
  same KWallet / GNOME Keyring storage, no user-visible change.
- Reconcile the in-app Credits and Third-Party Licenses with the current
  dependency tree.

* Wed Jun 17 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.5.4-1
- Harden the container command runner: per-command timeout and output-size cap
  with reliable process-group cleanup; large container searches report capped
  results instead of failing.
- Tighten search/replace resource bounds and surface walk-depth truncation.
- Localize the remaining search status and notification messages.

* Sat Jun 13 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.5.3-1
- Skip the read-only-directory replace failure test when running as root,
  so CI passes in containerized release builds.

* Fri Jun 12 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.5.2-1
- Expose generic QML i18n helpers and migrate all qsTr() calls to Fluent.
- Cache per-container grep availability in container search.
- Honor the reduced-motion setting for all busy spinners.
- Enforce the shared 512 MiB read cap in the replace pipeline.
- Smoke-test the AppImage in release CI.

* Fri Jun 12 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.5.1-1
- Version bump to 1.5.1.

* Thu Jun 11 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.5.0-1
- Add whole-word search matching and a CLI replace subcommand with full
  search flag parity.
- Harden search/replace correctness: normalized offset mapping, capture
  expansion, and whole-word boundary handling.
- Improve container search: flag forwarding, binary replace guard, and
  correct column offsets.
- Cache GUI search options and deduplicate per-tab results.
- Add accessibility settings and keyboard shortcuts.

* Tue Jun 02 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.4.1-1
- Refresh the pinned linuxdeploy continuous hash so the AppImage release
  artifact builds again (1.4.0 did not publish). No functional change
  from 1.4.0 — still ships the bundled runtime-component license texts
  viewable in Credits and the Licenses view.

* Tue Jun 02 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.4.0-1
- Bundle the full license texts for the system/runtime components (Qt,
  KDE Frameworks/Kirigami, Poppler, the Docker/Podman CLIs, and the
  Secret Service backends) and surface them in-app: a per-component
  "View license" action in Credits and a searchable "Runtime
  components" tab in the Licenses view.

* Fri May 29 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.3.0-1
- Rename the application ID to com.visorcraft.Grexa and set the
  organization domain to visorcraft.com. Breaking identity change:
  earlier installs do not upgrade in place; stored API keys are not
  migrated to the new keyring service.
* Fri May 29 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.2.0-1
- Security hardening: replace pipeline refuses to write outside the
  search root and restores permissions via the file descriptor; AI API
  keys are never sent over plaintext HTTP and are redacted from logs;
  bounded regex backtracking, a 512 MiB search read cap, and a pdftotext
  watchdog guard pathological inputs; container exec/cp argument-injection
  hardening; CLI terminal-escape sanitization.
- API keys now use the pure-Rust Secret Service keyring backend.
- Hardened release CI (pinned actions + image digests, build provenance).
- Removed dead code and unused dependencies.

* Wed May 20 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.0.1-1
- Expands the GitHub release pipeline to publish tarball, AppImage,
  Arch/CachyOS, Debian/Ubuntu, and Fedora/RHEL artifacts.
- Hardens live Docker/Podman tests against container startup races.

* Tue May 19 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.0.0-1
- v1.0.0 stable release — feature-complete against Grex on Linux.
- Promotes v0.3 polish (per-tab isolation, responsive toolbar,
  auto-saved Settings, Fluent plurals, taskbar icon) to the 1.0 line.

* Mon May 18 2026 VisorCraft LLC <maintainer@visorcraft.com> - 0.3.0-1
- v0.3.0 polish and responsiveness release.

* Sat May 16 2026 VisorCraft LLC <maintainer@visorcraft.com> - 0.1.0-1
- Initial Fedora package.
