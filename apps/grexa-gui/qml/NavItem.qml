// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Sidebar navigation entry.
//
// 32px tall · 18px icon · 13px label · subtle hover/press/active
// states · 3px accent bar on the left of the active item, eased in.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Item {
    id: root
    height: 32

    property string label: ""
    property string iconName: ""
    property bool active: false
    signal triggered()

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 6
        anchors.rightMargin: 6
        radius: app.tokens.radiusButton
        color: {
            if (root.active) return app.tokens.accentMute
            if (mouseArea.containsPress) return app.tokens.surface2
            if (mouseArea.containsMouse) return app.tokens.surface1
            return "transparent"
        }
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
    }

    // Active accent bar (just to the right of the left padding)
    Rectangle {
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: 2
        width: 3
        height: 16
        radius: 1.5
        color: app.tokens.accent
        opacity: root.active ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: app.tokens.durationSnap } }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: app.tokens.spaceL
        anchors.rightMargin: app.tokens.spaceL
        spacing: app.tokens.spaceM

        Kirigami.Icon {
            source: root.iconName
            implicitWidth: 16
            implicitHeight: 16
            color: root.active ? app.tokens.accent : Kirigami.Theme.textColor
            opacity: root.active ? 1.0 : 0.75
            isMask: true
        }
        Controls.Label {
            text: root.label
            font.pixelSize: app.tokens.textBody
            font.weight: root.active ? app.tokens.weightMedium : app.tokens.weightNormal
            opacity: root.active ? 1.0 : 0.85
            Layout.fillWidth: true
        }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.triggered()
    }
}
