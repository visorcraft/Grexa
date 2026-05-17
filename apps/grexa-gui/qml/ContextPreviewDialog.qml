// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Context-preview dialog. The Rust side hands it a
// `grexa_core::ContextPreviewResult`; the QML side renders gutter line
// numbers + match-line highlight + match-substring underline.
//
// Keyboard shortcuts:
//   - Space (from result list) ⇒ open
//   - Escape                   ⇒ close
//   - Enter                    ⇒ open in editor

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Dialog {
    id: dialog
    modal: true
    title: model && model.matchLineNumber ? i18n("%1 — line %2", model.fileName, model.matchLineNumber) : i18n("Preview")
    standardButtons: Dialog.Open | Dialog.Close
    width: Math.min(parent.width * 0.8, 900)
    height: Math.min(parent.height * 0.8, 500)

    property var model

    contentItem: ScrollView {
        clip: true

        ListView {
            id: listView
            anchors.fill: parent
            model: dialog.model ? dialog.model.lines : []

            delegate: RowLayout {
                spacing: 0
                width: parent ? parent.width : 0

                // Match indicator strip
                Rectangle {
                    width: 4
                    color: modelData.isMatch ? "#3DAEE9" : "transparent"
                    height: rowText.implicitHeight
                }

                // Gutter (line number)
                Label {
                    text: modelData.lineNumber
                    Layout.preferredWidth: 50
                    horizontalAlignment: Text.AlignRight
                    color: Kirigami.Theme.disabledTextColor
                    font.family: "monospace"
                    rightPadding: 8
                }

                // Content
                Label {
                    id: rowText
                    Layout.fillWidth: true
                    text: modelData.content
                    font.family: "monospace"
                    wrapMode: Text.NoWrap
                    background: Rectangle {
                        color: modelData.isMatch ? "#283D4F" : "transparent"
                        opacity: modelData.isMatch ? 0.35 : 0.0
                    }
                }
            }
        }
    }

    onAccepted: {
        // Rust hook: invoke openInEditor(match-line)
    }
}
