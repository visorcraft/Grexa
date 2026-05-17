// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Surface card with optional title and subtitle. Used for Settings
// sections and any grouped content that should visually float above
// the page background.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Rectangle {
    id: root
    default property alias contentChildren: contentRow.children
    property string title: ""
    property string subtitle: ""

    Layout.fillWidth: true
    color: app.tokens.surface1
    radius: app.tokens.radiusCard
    border.color: app.tokens.separator
    border.width: 1
    implicitHeight: column.implicitHeight + app.tokens.spaceL * 2

    ColumnLayout {
        id: column
        anchors.fill: parent
        anchors.margins: app.tokens.spaceL
        spacing: app.tokens.spaceM

        ColumnLayout {
            visible: root.title.length > 0
            Layout.fillWidth: true
            spacing: app.tokens.spaceXS

            Controls.Label {
                text: root.title
                font.pixelSize: app.tokens.textSubheading
                font.weight: app.tokens.weightBold
            }
            Controls.Label {
                visible: root.subtitle.length > 0
                text: root.subtitle
                font.pixelSize: app.tokens.textBody
                opacity: 0.65
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        ColumnLayout {
            id: contentRow
            Layout.fillWidth: true
            spacing: app.tokens.spaceM
        }
    }
}
