// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Sidebar navigation entry — Mailspring-style.
//
// 36px tall · 18px icon · 13px label · the active row paints a soft
// accent-tinted pill across its whole width, with the icon and label
// switching to the accent color. Hover gives a lighter wash; press
// drops one more step. No left accent bar — the full-row fill does
// the work.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Item {
    id: root
    height: app.tokens.navRowHeight

    property string label: ""
    property string iconName: ""
    property bool active: false
    // `compact` hides the text label so the row reads as an
    // icon-only entry — used when the sidebar is collapsed.
    property bool compact: false
    signal triggered()

    Accessible.role: Accessible.Button
    Accessible.name: label

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: app.tokens.spaceS
        anchors.rightMargin: app.tokens.spaceS
        anchors.topMargin: 1
        anchors.bottomMargin: 1
        radius: app.tokens.radiusInput
        color: {
            if (root.active) return app.tokens.accentMute
            if (mouseArea.containsPress) return app.tokens.surface2
            if (mouseArea.containsMouse) return app.tokens.surface1
            return "transparent"
        }
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }

        // Faint accent edge on the active row — adds depth without
        // a separate left bar. Mailspring uses something close to
        // this on the focused account row.
        border.color: root.active ? Qt.rgba(app.tokens.accent.r,
                                            app.tokens.accent.g,
                                            app.tokens.accent.b, 0.25)
                                  : "transparent"
        border.width: 1
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: app.tokens.spaceL
        anchors.rightMargin: app.tokens.spaceL
        spacing: app.tokens.spaceM

        Kirigami.Icon {
            source: root.iconName
            implicitWidth: 18
            implicitHeight: 18
            // Center the icon when there's no label to anchor against.
            Layout.alignment: root.compact ? Qt.AlignHCenter : Qt.AlignLeft
            Layout.fillWidth: root.compact
            color: root.active ? app.tokens.accent : app.tokens.textPrimary
            opacity: root.active ? 1.0 : 0.75
            isMask: true
            Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
        }
        Controls.Label {
            text: root.label
            font.pixelSize: app.tokens.textBody
            font.family: app.tokens.sansFamily
            font.weight: root.active ? app.tokens.weightSemibold : app.tokens.weightNormal
            color: root.active ? app.tokens.accent : app.tokens.textPrimary
            opacity: root.active ? 1.0 : 0.88
            Layout.fillWidth: true
            visible: !root.compact
            Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
        }
    }

    Controls.ToolTip.visible: root.compact && mouseArea.containsMouse
    Controls.ToolTip.text: root.label
    Controls.ToolTip.delay: 400

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.triggered()
    }
}
