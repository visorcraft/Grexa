Name:           grexa
Version:        1.4.1
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
