// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// A single search-result row.
//
// 64px tall · file-type icon column · semibold path with mid-elide
// · line:column pill badge right · 12px monospace preview with
// match span tinted in accent yellow.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Controls.ItemDelegate {
    id: root
    property string relativePath: ""
    property int line: 0
    property int column: 0
    property string previewBefore: ""
    property string previewMatch: ""
    property string previewAfter: ""
    signal openPreview()

    height: 64
    padding: 0
    hoverEnabled: true

    function escapeHtml(s) {
        return String(s)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
    }

    background: Rectangle {
        color: root.pressed ? app.tokens.surface2
            : root.hovered ? app.tokens.surface1
            : "transparent"
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
        // Bottom hairline divider
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: app.tokens.spaceXL + 36
            anchors.rightMargin: app.tokens.spaceXL
            height: 1
            color: app.tokens.separator
            opacity: 0.5
        }
    }

    contentItem: RowLayout {
        anchors.fill: parent
        anchors.leftMargin: app.tokens.spaceXL
        anchors.rightMargin: app.tokens.spaceXL
        spacing: app.tokens.spaceL

        // -- File-type icon column
        Rectangle {
            Layout.preferredWidth: 36
            Layout.preferredHeight: 36
            Layout.alignment: Qt.AlignVCenter
            radius: app.tokens.radiusButton
            color: app.tokens.surface1
            border.color: app.tokens.separator
            border.width: 1
            Kirigami.Icon {
                anchors.centerIn: parent
                source: app.tokens.iconForPath(root.relativePath)
                implicitWidth: 22
                implicitHeight: 22
                isMask: false
            }
        }

        // -- Path + preview column
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 3

            Controls.Label {
                Layout.fillWidth: true
                text: root.relativePath
                font.pixelSize: app.tokens.textBodyEmphasis
                font.weight: app.tokens.weightMedium
                elide: Text.ElideMiddle
            }
            Controls.Label {
                Layout.fillWidth: true
                font.family: app.tokens.monoFamily
                font.pixelSize: app.tokens.textCaption
                textFormat: Text.RichText
                wrapMode: Text.NoWrap
                elide: Text.ElideRight
                opacity: 0.85
                text: {
                    const before = root.escapeHtml(root.previewBefore)
                    const match = root.escapeHtml(root.previewMatch)
                    const after = root.escapeHtml(root.previewAfter)
                    const bg = app.tokens.matchTint.toString()
                    return "<span style='opacity:0.55'>" + before + "</span>"
                        + "<span style='background-color:" + bg
                        + "; padding:0 3px; border-radius:2px;'>"
                        + match + "</span>"
                        + "<span style='opacity:0.55'>" + after + "</span>"
                }
            }
        }

        // -- Line:column pill badge
        Rectangle {
            Layout.alignment: Qt.AlignVCenter
            radius: app.tokens.radiusPill
            color: app.tokens.surface1
            border.color: app.tokens.separator
            border.width: 1
            implicitHeight: 22
            implicitWidth: lineLabel.implicitWidth + app.tokens.spaceM * 2
            Controls.Label {
                id: lineLabel
                anchors.centerIn: parent
                text: root.line + ":" + root.column
                font.family: app.tokens.monoFamily
                font.pixelSize: app.tokens.textCaption
                font.weight: app.tokens.weightMedium
                opacity: 0.7
            }
        }

        // -- Chevron (only on hover)
        Kirigami.Icon {
            Layout.alignment: Qt.AlignVCenter
            source: "go-next-symbolic"
            implicitWidth: 14
            implicitHeight: 14
            color: app.tokens.accent
            isMask: true
            opacity: root.hovered ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: app.tokens.durationSnap } }
        }
    }

    onClicked: root.openPreview()
}
