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
    exit 0
fi

linuxdeploy --appdir "$appdir" --plugin qt --output appimage \
    --custom-apprun "$apprun_src" \
    --desktop-file "${appdir}/usr/share/applications/com.visorcraft.Grexa.desktop"

mv Grexa*.AppImage "$output"
echo "wrote $output"
