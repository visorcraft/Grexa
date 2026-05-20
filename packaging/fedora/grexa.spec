Name:           grexa
Version:        1.0.0
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
    desktop-file-validate %{buildroot}%{_datadir}/applications/io.visorcraft.Grexa.desktop || :
command -v appstream-util >/dev/null && \
    appstream-util validate-relax \
        %{buildroot}%{_datadir}/metainfo/io.visorcraft.Grexa.metainfo.xml || :

%files
%license LICENSE
%doc README.md docs/*.md
%{_bindir}/grexa
%{_bindir}/grexa-cli
%{_datadir}/applications/io.visorcraft.Grexa.desktop
%{_datadir}/metainfo/io.visorcraft.Grexa.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/io.visorcraft.Grexa.svg
%{_datadir}/icons/hicolor/*x*/apps/io.visorcraft.Grexa.png
%{_mandir}/man1/grexa-cli.1*
%{_datadir}/bash-completion/completions/grexa-cli
%{_datadir}/zsh/site-functions/_grexa-cli
%{_datadir}/fish/vendor_completions.d/grexa-cli.fish

%changelog
* Tue May 19 2026 VisorCraft LLC <maintainer@visorcraft.com> - 1.0.0-1
- v1.0.0 stable release — feature-complete against Grex on Linux.
- Promotes v0.3 polish (per-tab isolation, responsive toolbar,
  auto-saved Settings, Fluent plurals, taskbar icon) to the 1.0 line.

* Mon May 18 2026 VisorCraft LLC <maintainer@visorcraft.com> - 0.3.0-1
- v0.3.0 polish and responsiveness release.

* Sat May 16 2026 VisorCraft LLC <maintainer@visorcraft.com> - 0.1.0-1
- Initial Fedora package.
