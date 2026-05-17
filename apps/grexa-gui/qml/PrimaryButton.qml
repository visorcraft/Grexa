// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Filled primary action button. Use sparingly — there should be one
// primary on a page (Search / Apply / Send), and everything else
// should be `Controls.Button { flat: true }` or `Kirigami.Action`.

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

Controls.Button {
    id: root
    property color baseColor: app.tokens.accent
    property color hoverColor: app.tokens.accentHover

    leftPadding: app.tokens.spaceL
    rightPadding: app.tokens.spaceL
    topPadding: app.tokens.spaceS
    bottomPadding: app.tokens.spaceS
    icon.color: "white"
    display: Controls.AbstractButton.TextBesideIcon

    contentItem: Row {
        spacing: app.tokens.spaceS
        Kirigami.Icon {
            visible: root.icon.name.length > 0
            source: root.icon.name
            implicitWidth: 16
            implicitHeight: 16
            color: "white"
            isMask: true
            anchors.verticalCenter: parent.verticalCenter
        }
        Controls.Label {
            text: root.text
            color: "white"
            font.weight: app.tokens.weightMedium
            font.pixelSize: app.tokens.textBody
            anchors.verticalCenter: parent.verticalCenter
        }
    }

    background: Rectangle {
        radius: app.tokens.radiusButton
        color: !root.enabled ? Qt.darker(root.baseColor, 1.6)
            : root.pressed ? Qt.darker(root.baseColor, 1.15)
            : root.hovered ? root.hoverColor
            : root.baseColor
        opacity: root.enabled ? 1.0 : 0.5
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
    }
}
