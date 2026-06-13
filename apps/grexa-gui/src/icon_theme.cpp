// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

#include <QCoreApplication>
#include <QDir>
#include <QIcon>
#include <QString>
#include <QStringList>

extern "C" void grexa_set_icon_theme(const char *name) {
    // AppImage mounts at a temporary path; Qt's default icon search paths
    // point at the host filesystem, so add the bundled AppDir/share/icons
    // path before setting the theme name.
    const QString app_dir = QCoreApplication::applicationDirPath();
    const QString bundled_icons = QDir(app_dir).filePath("../share/icons");

    QStringList paths = QIcon::themeSearchPaths();
    if (!paths.contains(bundled_icons)) {
        paths.prepend(bundled_icons);
        QIcon::setThemeSearchPaths(paths);
    }

    QIcon::setThemeName(QString::fromUtf8(name));
}
