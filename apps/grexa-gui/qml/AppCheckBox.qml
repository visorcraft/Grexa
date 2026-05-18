// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Themed CheckBox wrapper — see AppTextField.qml for the why.

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

Controls.CheckBox {
    id: cb
    Kirigami.Theme.inherit: true
    Kirigami.Theme.colorSet: Kirigami.Theme.View
    Kirigami.Theme.backgroundColor: app.tokens.surface1
    Kirigami.Theme.textColor: app.tokens.textPrimary
    Kirigami.Theme.highlightColor: app.tokens.accent
    Kirigami.Theme.highlightedTextColor: app.tokens.accentText

    // qqc2-desktop-style's CheckIndicator (the indicator delegate)
    // uses a native QStyle-rendered StyleItem whose colors come
    // from the widget's QPalette — not from Kirigami.Theme. Replace
    // the indicator with a hand-rolled themed Rectangle so the
    // checkbox box matches our surface/accent.
    indicator: Rectangle {
        implicitWidth: 18
        implicitHeight: 18
        x: cb.leftPadding
        y: parent.height / 2 - height / 2
        radius: 4
        color: cb.checked ? app.tokens.accent : app.tokens.surface1
        border.color: cb.checked
            ? app.tokens.accent
            : (cb.hovered ? app.tokens.accent : app.tokens.separator)
        border.width: cb.checked ? 0 : 1
        Behavior on color { ColorAnimation { duration: 110 } }

        // Checkmark glyph
        Kirigami.Icon {
            anchors.centerIn: parent
            visible: cb.checked
            source: "check-symbolic"
            implicitWidth: 12
            implicitHeight: 12
            color: app.tokens.accentText
            isMask: true
        }
    }
}
