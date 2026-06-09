// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Themed flat Button wrapper — see AppTextField.qml for the why.
// qqc2-desktop-style's Button has `Kirigami.Theme.inherit: false`
// with `colorSet: Button`, so its text + icon stay locked to the
// host palette regardless of our page-level overrides. Forcing
// inherit and re-stating the Button colors from our tokens lets
// flat actions stay readable on a Light surface.

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

Controls.Button {
    id: btn
    flat: true
    Accessible.role: Accessible.Button
    Accessible.name: text
    icon.color: app.tokens.textPrimary
    Kirigami.Theme.inherit: true
    Kirigami.Theme.colorSet: Kirigami.Theme.Button
    Kirigami.Theme.backgroundColor: app.tokens.surface1
    Kirigami.Theme.textColor: app.tokens.textPrimary
    Kirigami.Theme.highlightColor: app.tokens.accent
    Kirigami.Theme.highlightedTextColor: app.tokens.accentText

    palette.button:     app.tokens.surface1
    palette.buttonText: app.tokens.textPrimary
    palette.windowText: app.tokens.textPrimary
}
