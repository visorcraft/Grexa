// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Toggle-able flag chip used inside the SearchBar (.* for regex,
// Aa for case-sensitive). Monospace label, pill shape, accent-tinted
// when checked.

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

Item {
    id: root
    property string label: ""
    property string tooltip: ""
    property bool checked: false
    signal toggled()

    implicitWidth: 30
    implicitHeight: 26

    Rectangle {
        anchors.fill: parent
        anchors.margins: 2
        radius: app.tokens.radiusInput
        color: root.checked ? app.tokens.accentMute
            : mouse.containsPress ? app.tokens.surface2
            : mouse.containsMouse ? app.tokens.surface1
            : "transparent"
        border.color: root.checked ? app.tokens.accent : "transparent"
        border.width: 1
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
    }

    Controls.Label {
        anchors.centerIn: parent
        text: root.label
        font.family: app.tokens.monoFamily
        font.pixelSize: app.tokens.textCaption
        font.weight: root.checked ? app.tokens.weightMedium : app.tokens.weightNormal
        color: root.checked ? app.tokens.accent : Kirigami.Theme.textColor
        opacity: root.checked ? 1.0 : 0.75
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            root.checked = !root.checked
            root.toggled()
        }
    }

    Controls.ToolTip.text: root.tooltip
    Controls.ToolTip.visible: mouse.containsMouse && root.tooltip.length > 0
}
