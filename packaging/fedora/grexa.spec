Name:           grexa
Version:        0.3.0
Release:        1%{?dist}
Summary:        Fast Linux file content search with tabs, replace, and AI assistance

License:        GPL-3.0-only
URL:            https://github.com/visorcraft/grexa
Source0:        https://github.com/visorcraft/grexa/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  rust >= 1.95
BuildRequires:  cargo
BuildRequires:  qt6-qtbase-devel
BuildRequires:  qt6-qtdeclarative-devel
BuildRequires:  kf6-kirigami-devel
BuildRequires:  pkgconf
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
%autosetup -n %{name}-%{version}

%build
cargo build --workspace --release --frozen

%check
cargo test --workspace --release --frozen

%install
install -Dm755 target/release/grexa %{buildroot}%{_bindir}/grexa
install -Dm755 target/release/grexa-cli %{buildroot}%{_bindir}/grexa-cli

install -Dm644 packaging/io.visorcraft.Grexa.desktop \
    %{buildroot}%{_datadir}/applications/io.visorcraft.Grexa.desktop
install -Dm644 packaging/io.visorcraft.Grexa.metainfo.xml \
    %{buildroot}%{_metainfodir}/io.visorcraft.Grexa.metainfo.xml
install -Dm644 packaging/icons/scalable/io.visorcraft.Grexa.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.visorcraft.Grexa.svg
for sz in 16 24 32 48 64 96 128 192 256 512; do
    install -Dm644 packaging/icons/${sz}x${sz}/apps/io.visorcraft.Grexa.png \
        %{buildroot}%{_datadir}/icons/hicolor/${sz}x${sz}/apps/io.visorcraft.Grexa.png
done

target/release/grexa-cli manpage > grexa-cli.1
install -Dm644 grexa-cli.1 %{buildroot}%{_mandir}/man1/grexa-cli.1

target/release/grexa-cli completions bash \
    > %{buildroot}%{_datadir}/bash-completion/completions/grexa-cli
target/release/grexa-cli completions zsh \
    > %{buildroot}%{_datadir}/zsh/site-functions/_grexa-cli
target/release/grexa-cli completions fish \
    > %{buildroot}%{_datadir}/fish/vendor_completions.d/grexa-cli.fish

desktop-file-validate %{buildroot}%{_datadir}/applications/io.visorcraft.Grexa.desktop
appstream-util validate-relax \
    %{buildroot}%{_metainfodir}/io.visorcraft.Grexa.metainfo.xml

%files
%license LICENSE
%doc README.md docs/*.md
%{_bindir}/grexa
%{_bindir}/grexa-cli
%{_datadir}/applications/io.visorcraft.Grexa.desktop
%{_metainfodir}/io.visorcraft.Grexa.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/io.visorcraft.Grexa.svg
%{_datadir}/icons/hicolor/*x*/apps/io.visorcraft.Grexa.png
%{_mandir}/man1/grexa-cli.1*
%{_datadir}/bash-completion/completions/grexa-cli
%{_datadir}/zsh/site-functions/_grexa-cli
%{_datadir}/fish/vendor_completions.d/grexa-cli.fish

%changelog
* Mon May 18 2026 VisorCraft LLC <maintainer@visorcraft.com> - 0.3.0-1
- v0.3.0 polish and responsiveness release.

* Sat May 16 2026 VisorCraft LLC <maintainer@visorcraft.com> - 0.1.0-1
- Initial Fedora package.
