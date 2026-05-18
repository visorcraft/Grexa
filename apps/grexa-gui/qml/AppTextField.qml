// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Themed TextField wrapper. The qqc2-desktop-style TextField in
// /usr/lib/qt6/qml/org/kde/desktop/TextField.qml hardcodes
// `Kirigami.Theme.inherit: false`, which makes its View-colorSet bg
// fall back to the host palette regardless of any page-level Theme
// override we apply. Forcing inherit: true at the instance level
// wins over the component default, and re-stating the View colors
// from our tokens lets a Light theme actually paint these inputs
// light. Use this instead of `Controls.TextField` everywhere.

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

Controls.TextField {
    Kirigami.Theme.inherit: true
    Kirigami.Theme.colorSet: Kirigami.Theme.View
    Kirigami.Theme.backgroundColor: app.tokens.surface1
    Kirigami.Theme.textColor: app.tokens.textPrimary
    Kirigami.Theme.highlightColor: app.tokens.accent
    Kirigami.Theme.highlightedTextColor: app.tokens.accentText
}
