#!/usr/bin/env bash
# AppImage builder for Grexa.
#
# Usage: packaging/appimage/build.sh
#
# Produces ./Grexa-<version>-x86_64.AppImage. Requires linuxdeploy and
# linuxdeploy-plugin-qt on $PATH; on most distros:
#   curl -L -o linuxdeploy https://github.com/linuxdeploy/linuxdeploy/releases/latest/download/linuxdeploy-x86_64.AppImage
#   chmod +x linuxdeploy
#
# Compared to Flatpak, AppImage bundles Qt + Kirigami + a static Rust binary
# in one file. The trade-off: no portal-mediated permissions; the bundle has
# whatever access the launching user has.

set -euo pipefail
cd "$(dirname "$0")/../.."

VERSION="$(grep '^version' crates/grexa-core/Cargo.toml | head -1 | cut -d'"' -f2)"
APP_DIR="target/appimage/Grexa.AppDir"

rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/usr/bin" "$APP_DIR/usr/share/applications" \
    "$APP_DIR/usr/share/metainfo" "$APP_DIR/usr/share/icons/hicolor/scalable/apps" \
    "$APP_DIR/usr/share/man/man1"

cargo build --release --workspace

install -m755 target/release/grexa "$APP_DIR/usr/bin/grexa"
install -m755 target/release/grexa-cli "$APP_DIR/usr/bin/grexa-cli"

install -m644 packaging/io.visorcraft.Grexa.desktop \
    "$APP_DIR/usr/share/applications/io.visorcraft.Grexa.desktop"
install -m644 packaging/io.visorcraft.Grexa.metainfo.xml \
    "$APP_DIR/usr/share/metainfo/io.visorcraft.Grexa.metainfo.xml"
install -m644 packaging/icons/scalable/io.visorcraft.Grexa.svg \
    "$APP_DIR/usr/share/icons/hicolor/scalable/apps/io.visorcraft.Grexa.svg"

"$APP_DIR/usr/bin/grexa-cli" manpage > "$APP_DIR/usr/share/man/man1/grexa-cli.1"

# AppRun shim — launches the GUI binary.
cat > "$APP_DIR/AppRun" <<'APPRUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="$HERE/usr/bin:$PATH"
export QT_PLUGIN_PATH="$HERE/usr/lib/qt6/plugins${QT_PLUGIN_PATH:+:$QT_PLUGIN_PATH}"
exec "$HERE/usr/bin/grexa" "$@"
APPRUN
chmod +x "$APP_DIR/AppRun"

cp packaging/icons/scalable/io.visorcraft.Grexa.svg "$APP_DIR/io.visorcraft.Grexa.svg"
cp packaging/io.visorcraft.Grexa.desktop "$APP_DIR/io.visorcraft.Grexa.desktop"

linuxdeploy --appdir "$APP_DIR" --plugin qt --output appimage --custom-apprun "$APP_DIR/AppRun"
mv Grexa*.AppImage "Grexa-${VERSION}-x86_64.AppImage"
echo "wrote Grexa-${VERSION}-x86_64.AppImage"
