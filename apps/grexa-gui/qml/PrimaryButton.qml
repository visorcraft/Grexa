// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Filled primary action button — gradient fill with a soft inner
// highlight, matching the SearchBar's embedded Search button so the
// app reads as one design language. Use sparingly: one primary per
// page (Search / Apply / Send); everything else should be flat.

import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

Controls.Button {
    id: root
    property color baseColor: app.tokens.accent
    property color hoverColor: app.tokens.accentHover

    Accessible.role: Accessible.Button
    Accessible.name: text

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
            font.weight: app.tokens.weightSemibold
            font.pixelSize: app.tokens.textBody
            font.family: app.tokens.sansFamily
            anchors.verticalCenter: parent.verticalCenter
        }
    }

    background: Rectangle {
        radius: app.tokens.radiusButton
        gradient: Gradient {
            GradientStop {
                position: 0.0
                color: !root.enabled ? Qt.darker(root.baseColor, 1.6)
                    : root.pressed ? app.tokens.accentPressed
                    : root.hovered ? root.hoverColor
                    : root.baseColor
            }
            GradientStop {
                position: 1.0
                color: !root.enabled ? Qt.darker(root.baseColor, 1.8)
                    : root.pressed ? app.tokens.accentDeep
                    : root.hovered ? root.baseColor
                    : app.tokens.accentPressed
            }
        }
        opacity: root.enabled ? 1.0 : 0.5

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 1
            height: parent.height * 0.5
            radius: parent.radius - 1
            gradient: Gradient {
                GradientStop { position: 0.0; color: Qt.rgba(1, 1, 1, 0.18) }
                GradientStop { position: 1.0; color: Qt.rgba(1, 1, 1, 0.0) }
            }
        }
    }
}
