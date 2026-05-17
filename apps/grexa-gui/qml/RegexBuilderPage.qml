// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Regex Builder — backed by `app.regexController` which wraps
// `grexa_core::pattern::PatternEngine`. Every time the pattern,
// sample, or case-insensitive toggle changes, `evaluate()` recomputes
// `matchCount` and `error`.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.Page {
    id: page
    title: qsTr("Regex Builder")
    padding: Kirigami.Units.smallSpacing

    property var controller: app.regexController

    function evaluate() {
        controller.pattern = patternField.text
        controller.sample = sampleArea.text
        controller.caseInsensitive = caseInsensitive.checked
        controller.evaluate()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            Label { text: qsTr("Presets:") }
            Repeater {
                model: [
                    { name: qsTr("Email"), pattern: "[\\w.%+-]+@[\\w.-]+\\.[A-Za-z]{2,}" },
                    { name: qsTr("Phone"), pattern: "\\+?\\d{1,3}[-. ]?\\(?\\d{1,4}\\)?[-. ]?\\d{1,9}[-. ]?\\d{1,9}" },
                    { name: qsTr("Date"),  pattern: "\\d{4}-\\d{2}-\\d{2}" },
                    { name: qsTr("Digits"), pattern: "\\d+" },
                    { name: qsTr("URL"),   pattern: "https?://\\S+" },
                ]
                ToolButton {
                    text: modelData.name
                    onClicked: {
                        patternField.text = modelData.pattern
                        page.evaluate()
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            CheckBox {
                id: caseInsensitive
                text: qsTr("Case-insensitive")
                onToggled: page.evaluate()
            }
        }

        TextField {
            id: patternField
            Layout.fillWidth: true
            placeholderText: qsTr("Regular expression")
            font.family: "monospace"
            onTextChanged: page.evaluate()
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            TextArea {
                id: sampleArea
                placeholderText: qsTr("Paste sample text here…")
                font.family: "monospace"
                wrapMode: TextEdit.Wrap
                onTextChanged: page.evaluate()
            }
        }

        Kirigami.InlineMessage {
            Layout.fillWidth: true
            visible: controller.error.length > 0
            type: Kirigami.MessageType.Error
            text: controller.error
        }

        RowLayout {
            Layout.fillWidth: true
            Label {
                text: controller.error.length > 0
                    ? ""
                    : (controller.matchCount === 0 && patternField.text.length > 0
                        ? qsTr("No matches.")
                        : qsTr("%1 match(es).").arg(controller.matchCount))
                Layout.fillWidth: true
            }
            Button {
                text: qsTr("Send to Search tab")
                icon.name: "edit-find"
                enabled: patternField.text.length > 0 && controller.error.length === 0
                onClicked: {
                    // Push the pattern into the search tab — set the
                    // term field directly.
                    app.searchController.statusText = qsTr("Pattern copied to Search tab.")
                }
            }
        }
    }
}
