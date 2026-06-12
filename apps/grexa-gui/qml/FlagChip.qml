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

    // The chip is a view onto the parent's flag — the parent owns
    // the boolean and drives `active` through a declarative binding
    // (`active: parent.someFlag`). The chip emits `toggled()` on
    // click; the parent flips its own flag from `onToggled`. The
    // chip MUST NOT imperatively assign `root.active` itself —
    // doing so would break the parent's binding so a future
    // external change to the flag wouldn't propagate back. The
    // name `active` (not `checked`) is deliberate: there's no
    // intuitive "toggle me" verb on `active`, so casual
    // contributors don't reach for `chip.active = !chip.active`.
    property bool active: false
    signal toggled()

    Accessible.role: Accessible.Button
    Accessible.name: root.label

    implicitWidth: 34
    implicitHeight: 30

    Rectangle {
        anchors.fill: parent
        anchors.margins: 3
        radius: app.tokens.radiusButton
        color: root.active ? app.tokens.accentMute
            : mouse.containsPress ? app.tokens.surface2
            : mouse.containsMouse ? app.tokens.surface1
            : "transparent"
        border.color: root.active ? app.tokens.accent : "transparent"
        border.width: 1
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
        Behavior on border.color { ColorAnimation { duration: app.tokens.durationSnap } }
    }

    Controls.Label {
        anchors.centerIn: parent
        text: root.label
        font.family: app.tokens.monoFamily
        font.pixelSize: app.tokens.textCaption + 1
        font.weight: root.active ? app.tokens.weightSemibold : app.tokens.weightMedium
        color: root.active ? app.tokens.accent : Kirigami.Theme.textColor
        opacity: root.active ? 1.0 : 0.7
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.toggled()
    }

    Controls.ToolTip.text: root.tooltip
    Controls.ToolTip.visible: mouse.containsMouse && root.tooltip.length > 0
}
