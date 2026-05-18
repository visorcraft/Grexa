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

        // Draw the mark ourselves instead of asking the icon theme
        // for "check-symbolic"; some themes resolve that name to a
        // document-style icon.
        Canvas {
            id: checkMark
            anchors.fill: parent
            visible: cb.checked
            opacity: cb.enabled ? 1 : 0.5
            property color markColor: app.tokens.accentText
            onMarkColorChanged: requestPaint()
            onVisibleChanged: if (visible) requestPaint()
            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
            onPaint: {
                const ctx = getContext("2d")
                ctx.clearRect(0, 0, width, height)
                ctx.strokeStyle = markColor
                ctx.lineWidth = 2.3
                ctx.lineCap = "round"
                ctx.lineJoin = "round"
                ctx.beginPath()
                ctx.moveTo(width * 0.28, height * 0.53)
                ctx.lineTo(width * 0.43, height * 0.68)
                ctx.lineTo(width * 0.74, height * 0.33)
                ctx.stroke()
            }
        }
    }
}
