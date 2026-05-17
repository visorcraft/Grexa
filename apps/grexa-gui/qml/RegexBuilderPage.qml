// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Regex Builder — backed by `grexa_core::pattern::PatternEngine`.
// The QML side is a two-pane editor; the Rust side handles compile,
// match enumeration, and the live breakdown that highlights groups.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.Page {
    title: i18n("Regex Builder")
    padding: Kirigami.Units.smallSpacing

    ColumnLayout {
        anchors.fill: parent
        spacing: Kirigami.Units.smallSpacing

        // Presets row
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            Label { text: i18n("Presets:") }
            ButtonGroup { id: presetGroup; exclusive: true }
            Repeater {
                model: [
                    { name: i18n("Email"), pattern: "[\\w.%+-]+@[\\w.-]+\\.[A-Za-z]{2,}" },
                    { name: i18n("Phone"), pattern: "\\+?\\d{1,3}[-. ]?\\(?\\d{1,4}\\)?[-. ]?\\d{1,9}[-. ]?\\d{1,9}" },
                    { name: i18n("Date"),  pattern: "\\d{4}-\\d{2}-\\d{2}" },
                    { name: i18n("Digits"), pattern: "\\d+" },
                    { name: i18n("URL"),   pattern: "https?://\\S+" },
                ]
                ToolButton {
                    text: modelData.name
                    onClicked: patternField.text = modelData.pattern
                }
            }
        }

        // Toggles row
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            CheckBox { id: caseInsensitive; text: i18n("Case-insensitive") }
            CheckBox { id: multiline;       text: i18n("Multiline (^/$ per line)") }
            CheckBox { id: globalMatch;     text: i18n("Global"); checked: true }
        }

        // Pattern input
        TextField {
            id: patternField
            Layout.fillWidth: true
            placeholderText: i18n("Regular expression")
            font.family: "monospace"
        }

        // Sample text + match list pane
        SplitView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            orientation: Qt.Vertical

            ScrollView {
                SplitView.preferredHeight: parent.height * 0.45
                TextArea {
                    id: sampleArea
                    placeholderText: i18n("Paste sample text here…")
                    font.family: "monospace"
                    wrapMode: TextEdit.Wrap
                }
            }

            // Live matches
            ListView {
                SplitView.fillHeight: true
                clip: true
                model: ListModel { id: matchModel }
                delegate: Kirigami.SubtitleDelegate {
                    text: i18n("Match %1: %2", model.index ?? 0, model.text ?? "")
                    subtitle: model.captures ?? ""
                }

                Kirigami.PlaceholderMessage {
                    anchors.centerIn: parent
                    visible: matchModel.count === 0 && patternField.text.length > 0
                    text: i18n("No matches.")
                }
                Kirigami.PlaceholderMessage {
                    anchors.centerIn: parent
                    visible: patternField.text.length === 0
                    text: i18n("Enter a pattern to see matches.")
                }
            }
        }

        // Action row
        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            Button {
                text: i18n("Apply to Search tab")
                icon.name: "edit-find"
                enabled: patternField.text.length > 0
            }
        }
    }
}
