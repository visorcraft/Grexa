// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Context-preview dialog. Renders the configured surrounding lines
// through the controller's previewAt() invokable. The output already
// has `>` on the match line; we apply our match tint on that line.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Controls.Dialog {
    id: dialog
    modal: true
    standardButtons: Controls.Dialog.Close
    width: Math.min(parent ? parent.width * 0.85 : 900, 900)
    height: Math.min(parent ? parent.height * 0.75 : 540, 540)

    property string path: ""
    property int lineNumber: 0

    function baseName(p) {
        if (!p) return ""
        const idx = p.lastIndexOf("/")
        return idx >= 0 ? p.substring(idx + 1) : p
    }

    function format(raw) {
        if (!raw) return ""
        const lines = raw.split("\n")
        let html = "<div style='font-family:monospace; font-size:13px;'>"
        for (let i = 0; i < lines.length; ++i) {
            const ln = lines[i]
            if (!ln) continue
            const isMatch = ln.startsWith(">")
            const escaped = String(ln)
                .replace(/&/g, "&amp;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;")
            if (isMatch) {
                html += "<div style='background-color:"
                    + app.rgbaCss(app.tokens.matchTint)
                    + "; padding:2px 6px;'>"
                    + escaped + "</div>"
            } else {
                html += "<div style='padding:2px 6px; opacity:0.7;'>"
                    + escaped + "</div>"
            }
        }
        html += "</div>"
        return html
    }

    onOpened: {
        if (dialog.path.length > 0 && dialog.lineNumber > 0) {
            const raw = app.searchController.previewAt(dialog.path, dialog.lineNumber)
            previewBody.text = format(raw)
        } else {
            previewBody.text = ""
        }
    }

    header: Rectangle {
        color: app.tokens.surface1
        implicitHeight: 56
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 1
            color: app.tokens.separator
        }
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: app.tokens.spaceL
            anchors.rightMargin: app.tokens.spaceL
            spacing: app.tokens.spaceM

            Kirigami.Icon {
                source: app.tokens.iconForPath(dialog.path)
                implicitWidth: 24
                implicitHeight: 24
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0
                Controls.Label {
                    text: dialog.baseName(dialog.path)
                    font.pixelSize: app.tokens.textSubheading
                    font.weight: app.tokens.weightBold
                }
                Controls.Label {
                    text: dialog.path
                    font.pixelSize: app.tokens.textCaption
                    font.family: app.tokens.monoFamily
                    opacity: 0.55
                    elide: Text.ElideMiddle
                    Layout.fillWidth: true
                }
            }
            Rectangle {
                radius: app.tokens.radiusPill
                color: app.tokens.accentMute
                border.color: app.tokens.accent
                border.width: 1
                implicitHeight: 24
                implicitWidth: lineLabel.implicitWidth + app.tokens.spaceM * 2
                Controls.Label {
                    id: lineLabel
                    anchors.centerIn: parent
                    text: app.i18n("ui-line-1-630b65").arg(dialog.lineNumber)
                    font.pixelSize: app.tokens.textCaption
                    font.weight: app.tokens.weightMedium
                    color: app.tokens.accent
                }
            }
        }
    }

    contentItem: Rectangle {
        color: app.tokens.surface0
        Controls.ScrollView {
            anchors.fill: parent
            anchors.margins: app.tokens.spaceM
            Controls.Label {
                id: previewBody
                width: parent.width
                textFormat: Text.RichText
                wrapMode: Text.NoWrap
            }
        }
    }
}
