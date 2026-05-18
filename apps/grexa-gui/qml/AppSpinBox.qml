// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Themed SpinBox wrapper — see AppTextField.qml for the why.

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

Controls.SpinBox {
    Kirigami.Theme.inherit: true
    Kirigami.Theme.colorSet: Kirigami.Theme.View
    Kirigami.Theme.backgroundColor: app.tokens.surface1
    Kirigami.Theme.textColor: app.tokens.textPrimary
    Kirigami.Theme.highlightColor: app.tokens.accent
    Kirigami.Theme.highlightedTextColor: app.tokens.accentText
}
