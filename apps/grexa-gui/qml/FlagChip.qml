// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Toggle-able flag chip used inside the SearchBar (.* for regex,
// Aa for case-sensitive). Monospace label, soft rounded square,
// accent-tinted when checked. Slightly larger than v1 — the
// previous 30×26 felt cramped against the new bar height.

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

Item {
    id: root
    property string label: ""
    property string tooltip: ""
    property bool checked: false
    signal toggled()

    implicitWidth: 34
    implicitHeight: 30

    Rectangle {
        anchors.fill: parent
        anchors.margins: 3
        radius: app.tokens.radiusButton
        color: root.checked ? app.tokens.accentMute
            : mouse.containsPress ? app.tokens.surface2
            : mouse.containsMouse ? app.tokens.surface1
            : "transparent"
        border.color: root.checked ? app.tokens.accent : "transparent"
        border.width: 1
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
        Behavior on border.color { ColorAnimation { duration: app.tokens.durationSnap } }
    }

    Controls.Label {
        anchors.centerIn: parent
        text: root.label
        font.family: app.tokens.monoFamily
        font.pixelSize: app.tokens.textCaption + 1
        font.weight: root.checked ? app.tokens.weightSemibold : app.tokens.weightMedium
        color: root.checked ? app.tokens.accent : Kirigami.Theme.textColor
        opacity: root.checked ? 1.0 : 0.7
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        // Emit `toggled()` only — the parent owns the boolean state
        // and the `checked: <parent.flag>` declarative binding flows
        // the new value back. Imperatively assigning `root.checked`
        // here would break that binding, so any future external
        // change to the parent's flag wouldn't propagate to the
        // chip's visual state.
        onClicked: root.toggled()
    }

    Controls.ToolTip.text: root.tooltip
    Controls.ToolTip.visible: mouse.containsMouse && root.tooltip.length > 0
}
