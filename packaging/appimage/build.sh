#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 VisorCraft LLC
# SPDX-License-Identifier: GPL-3.0-only

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

VERSION="$(awk -F'"' '
    /^\[workspace\.package\]/ { in_section = 1; next }
    in_section && /^\[/ { exit }
    in_section && $1 ~ /^version[[:space:]]*=/ { print $2; exit }
' Cargo.toml)"

appdir="${1:-"${root}/target/appimage/Grexa.AppDir"}"
output="${2:-"${root}/target/appimage/Grexa-${VERSION}-x86_64.AppImage"}"

cargo build --release --workspace

rm -rf "$appdir"
install -Dm755 "${root}/target/release/grexa" "${appdir}/usr/bin/grexa"
install -Dm755 "${root}/target/release/grexa-cli" "${appdir}/usr/bin/grexa-cli"

install -Dm644 "${root}/packaging/com.visorcraft.Grexa.desktop" \
    "${appdir}/usr/share/applications/com.visorcraft.Grexa.desktop"
install -Dm644 "${root}/packaging/com.visorcraft.Grexa.metainfo.xml" \
    "${appdir}/usr/share/metainfo/com.visorcraft.Grexa.metainfo.xml"
install -Dm644 "${root}/packaging/icons/scalable/com.visorcraft.Grexa.svg" \
    "${appdir}/usr/share/icons/hicolor/scalable/apps/com.visorcraft.Grexa.svg"
for size in 16 24 32 48 64 96 128 192 256 512; do
    install -Dm644 "${root}/packaging/icons/${size}x${size}/apps/com.visorcraft.Grexa.png" \
        "${appdir}/usr/share/icons/hicolor/${size}x${size}/apps/com.visorcraft.Grexa.png"
done

mkdir -p "${appdir}/usr/share/man/man1"
"${appdir}/usr/bin/grexa-cli" manpage > "${appdir}/usr/share/man/man1/grexa-cli.1"

mkdir -p "${root}/target/appimage"
apprun_src="${root}/target/appimage/AppRun"
cat > "$apprun_src" <<'APPRUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="$HERE/usr/bin:$PATH"
export QT_PLUGIN_PATH="$HERE/usr/plugins${QT_PLUGIN_PATH:+:$QT_PLUGIN_PATH}"
export QML2_IMPORT_PATH="$HERE/usr/qml${QML2_IMPORT_PATH:+:$QML2_IMPORT_PATH}"
export LD_LIBRARY_PATH="$HERE/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$HERE/usr/bin/grexa" "$@"
APPRUN
chmod +x "$apprun_src"

cp "${root}/packaging/icons/scalable/com.visorcraft.Grexa.svg" \
    "${appdir}/com.visorcraft.Grexa.svg"
cp "${root}/packaging/com.visorcraft.Grexa.desktop" \
    "${appdir}/com.visorcraft.Grexa.desktop"

if ! command -v linuxdeploy >/dev/null 2>&1; then
    echo "linuxdeploy not on PATH; AppDir staged at ${appdir} but no AppImage produced." >&2
    echo "Install linuxdeploy + linuxdeploy-plugin-qt from https://github.com/linuxdeploy/." >&2
    exit 0
fi

# Force Qt6 qmake for linuxdeploy-plugin-qt. In mixed Qt5/Qt6 environments the
# first `qmake` on PATH is often Qt5's, which makes the plugin abort with
# "Could not find Qt modules to deploy".
QMAKE="${QMAKE:-$(command -v qmake6 || command -v qmake-qt6 || echo /usr/bin/qmake6)}"
export QMAKE

# Our QML is compiled into the binary as a Qt resource (qt_add_qml_module), so
# linuxdeploy-plugin-qt's qmlimportscanner has no on-disk QML to scan and would
# otherwise bundle ZERO QML modules. Point it at the QML sources so it discovers
# and deploys Kirigami / QtQuick.Controls / Dialogs / Layouts.
export QML_SOURCES_PATHS="${QML_SOURCES_PATHS:-${root}/apps/grexa-gui/qml}"

# linuxdeploy's bundled `strip` chokes on relr.dyn-only ELF objects on newer
# toolchains; retry unstripped (functionally identical, slightly larger). Set
# NO_STRIP=1 up front on hosts known to be affected to skip the doomed attempt.
run_linuxdeploy() { linuxdeploy --appdir "$appdir" "$@"; }
try_linuxdeploy() {
    if ! run_linuxdeploy "$@"; then
        if [ "${NO_STRIP:-}" = "1" ]; then
            echo "linuxdeploy failed with NO_STRIP=1 already set; giving up." >&2
            return 1
        fi
        echo "linuxdeploy failed (likely bundled strip vs relr.dyn); retrying with NO_STRIP=1 ..." >&2
        NO_STRIP=1 run_linuxdeploy "$@"
    fi
}

# linuxdeploy is run in TWO passes on purpose. A single combined
# `--plugin qt --output appimage` call deploys the QML-module plugin libraries
# (Kirigami, QtQuick.Controls, ...) only AFTER linuxdeploy's own dependency /
# rpath pass has already run, so those plugins ship with unresolved deps and the
# GUI dies at startup with "QML payload did not instantiate". Splitting deploy
# (phase 1) from packing (phase 2) makes the phase-2 invocation re-run the
# dependency resolution over the now-complete AppDir and fix up the QML plugins.
#
# Phase 1: deploy the Qt/QML stack into the AppDir (no AppImage yet).
try_linuxdeploy --plugin qt \
    --custom-apprun "$apprun_src" \
    --desktop-file "${appdir}/usr/share/applications/com.visorcraft.Grexa.desktop"

# Bundle the org.kde.desktop QtQuick Controls style (qqc2-desktop-style). It is
# a KDE module, not a Qt module, so linuxdeploy-plugin-qt never deploys it.
# NOT required to launch — without it the app still starts, but its
# QtQuick.Controls fall back to the plain "Basic" style instead of the native
# KDE look the QML targets. Its KF6 deps already arrive via Kirigami, so staging
# the module directory is enough.
qml_dir="$("$QMAKE" -query QT_INSTALL_QML 2>/dev/null || echo /usr/lib/qt6/qml)"
if [ -d "${qml_dir}/org/kde/desktop" ]; then
    cp -r "${qml_dir}/org/kde/desktop" "${appdir}/usr/qml/org/kde/desktop"
else
    echo "NOTE: ${qml_dir}/org/kde/desktop (qqc2-desktop-style) not found on the build" >&2
    echo "      host; the AppImage will render with the non-native Basic Controls style." >&2
fi

# Phase 2: pack the populated AppDir into the AppImage (AppRun already staged).
try_linuxdeploy --output appimage

mv Grexa*.AppImage "$output"
echo "wrote $output"
