#
# spec file for package grexa
#
# Copyright (c) 2026 VisorCraft LLC
# Licensed under GPL-3.0-only.
#

Name:           grexa
Version:        1.2.0
Release:        0
Summary:        Fast Linux file content search with tabs, replace, and AI assistance
License:        GPL-3.0-only
Group:          Productivity/Other
URL:            https://github.com/visorcraft/grexa
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.95
BuildRequires:  cargo
BuildRequires:  pkgconfig(Qt6Core) >= 6.6
BuildRequires:  pkgconfig(Qt6Quick) >= 6.6
BuildRequires:  pkgconfig(KF6Kirigami)
BuildRequires:  appstream-glib
BuildRequires:  desktop-file-utils
BuildRequires:  update-desktop-files

Requires:       libqt6-qtbase6
Requires:       libqt6-qtdeclarative6
Requires:       kirigami6
Recommends:     poppler-tools
Recommends:     podman
Suggests:       docker
Suggests:       gnome-keyring
Suggests:       kwalletmanager5

%description
Grexa is a fast, precise grep-style search workbench for Linux developers.
It feels at home on KDE Plasma, integrates with the Breeze icon set, and
respects the system color scheme by default.

%package -n %{name}-cli
Summary:        Headless command-line companion to Grexa
Group:          Productivity/Other
Recommends:     poppler-tools

%description -n %{name}-cli
Provides grexa-cli, the same search engine without the Qt GUI.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --workspace --release --frozen

%install
install -Dm755 target/release/grexa %{buildroot}%{_bindir}/grexa
install -Dm755 target/release/grexa-cli %{buildroot}%{_bindir}/grexa-cli

install -Dm644 packaging/io.visorcraft.Grexa.desktop \
    %{buildroot}%{_datadir}/applications/io.visorcraft.Grexa.desktop
install -Dm644 packaging/io.visorcraft.Grexa.metainfo.xml \
    %{buildroot}%{_datadir}/metainfo/io.visorcraft.Grexa.metainfo.xml
install -Dm644 packaging/icons/scalable/io.visorcraft.Grexa.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.visorcraft.Grexa.svg
for sz in 16 24 32 48 64 96 128 192 256 512; do
    install -Dm644 packaging/icons/${sz}x${sz}/apps/io.visorcraft.Grexa.png \
        %{buildroot}%{_datadir}/icons/hicolor/${sz}x${sz}/apps/io.visorcraft.Grexa.png
done

target/release/grexa-cli manpage > grexa-cli.1
install -Dm644 grexa-cli.1 %{buildroot}%{_mandir}/man1/grexa-cli.1
gzip -9 %{buildroot}%{_mandir}/man1/grexa-cli.1

install -d %{buildroot}%{_datadir}/bash-completion/completions
install -d %{buildroot}%{_datadir}/zsh/site-functions
install -d %{buildroot}%{_datadir}/fish/vendor_completions.d
target/release/grexa-cli completions bash \
    > %{buildroot}%{_datadir}/bash-completion/completions/grexa-cli
target/release/grexa-cli completions zsh \
    > %{buildroot}%{_datadir}/zsh/site-functions/_grexa-cli
target/release/grexa-cli completions fish \
    > %{buildroot}%{_datadir}/fish/vendor_completions.d/grexa-cli.fish

%suse_update_desktop_file %{name}

%check
cargo test --workspace --release --frozen

%files
%license LICENSE
%doc README.md docs/*.md
%{_bindir}/grexa
%{_datadir}/applications/io.visorcraft.Grexa.desktop
%{_datadir}/metainfo/io.visorcraft.Grexa.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/io.visorcraft.Grexa.svg
%{_datadir}/icons/hicolor/*x*/apps/io.visorcraft.Grexa.png

%files -n %{name}-cli
%{_bindir}/grexa-cli
%{_mandir}/man1/grexa-cli.1.gz
%{_datadir}/bash-completion/completions/grexa-cli
%{_datadir}/zsh/site-functions/_grexa-cli
%{_datadir}/fish/vendor_completions.d/grexa-cli.fish

%changelog
* Fri May 29 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.2.0-0
- Security hardening: replace pipeline refuses to write outside the
  search root and restores permissions via the file descriptor; AI API
  keys are never sent over plaintext HTTP and are redacted from logs;
  bounded regex backtracking, a 512 MiB search read cap, and a pdftotext
  watchdog guard pathological inputs; container exec/cp argument-injection
  hardening; CLI terminal-escape sanitization.
- API keys now use the pure-Rust Secret Service keyring backend.
- Hardened release CI (pinned actions + image digests, build provenance).
- Removed dead code and unused dependencies.

* Wed May 20 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.0.1-0
- Expands the GitHub release pipeline to publish tarball, AppImage,
  Arch/CachyOS, Debian/Ubuntu, and Fedora/RHEL artifacts.
- Hardens live Docker/Podman tests against container startup races.

* Tue May 19 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.0.0-0
- v1.0.0 stable release — feature-complete against Grex on Linux.
- Promotes v0.3 polish (per-tab isolation, responsive toolbar,
  auto-saved Settings, Fluent plurals, taskbar icon) to the 1.0 line.

* Mon May 18 2026 VisorCraft LLC <maintainer@visorcraft.com> - 0.3.0-0
- v0.3.0 polish and responsiveness release.

* Sat May 16 2026 VisorCraft LLC <maintainer@visorcraft.com> - 0.1.0-0
- Initial openSUSE package.
