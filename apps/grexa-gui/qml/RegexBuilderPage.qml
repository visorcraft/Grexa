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
    title: i18n("Regex Builder")
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
            Label { text: i18n("Presets:") }
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
                text: i18n("Case-insensitive")
                onToggled: page.evaluate()
            }
        }

        TextField {
            id: patternField
            Layout.fillWidth: true
            placeholderText: i18n("Regular expression")
            font.family: "monospace"
            onTextChanged: page.evaluate()
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            TextArea {
                id: sampleArea
                placeholderText: i18n("Paste sample text here…")
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
                        ? i18n("No matches.")
                        : i18n("%1 match(es).", controller.matchCount))
                Layout.fillWidth: true
            }
            Button {
                text: i18n("Send to Search tab")
                icon.name: "edit-find"
                enabled: patternField.text.length > 0 && controller.error.length === 0
                onClicked: {
                    // Push the pattern into the search tab — set the
                    // term field directly.
                    app.searchController.statusText = i18n("Pattern copied to Search tab.")
                }
            }
        }
    }
}
