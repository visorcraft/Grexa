// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Regex Builder. Preset chips + pattern editor + sample text with
// inline match highlights + match-count badge. The match
// highlighting is computed in QML JS (cheap enough for the test
// strings users typically paste here) so we don't need a separate
// "highlight pattern" invokable on the controller.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.Page {
    id: page
    padding: 0
    titleDelegate: Item {}
    globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

    property var controller: app.regexController
    property int activePreset: -1

    function evaluate() {
        controller.pattern = patternField.text
        controller.sample = sampleArea.text
        controller.caseInsensitive = caseInsensitive.checked
        controller.evaluate()
        refreshHighlight()
    }

    function refreshHighlight() {
        const sample = sampleArea.text
        if (patternField.text.length === 0 || controller.error.length > 0) {
            sampleHighlight.text = ""
            return
        }
        // Single source of truth: ask the Rust controller for the
        // exact byte ranges its engine matched. The JS regex engine
        // is no longer in the loop, so the highlight cannot drift
        // from the match-count badge.
        let ranges = []
        try {
            ranges = JSON.parse(controller.matchRangesJson())
        } catch (_e) {
            sampleHighlight.text = ""
            return
        }
        if (ranges.length === 0) {
            sampleHighlight.text = ""
            return
        }
        let html = ""
        let last = 0
        const tint = app.rgbaCss(app.tokens.matchTint)
        for (let i = 0; i < ranges.length; ++i) {
            const start = ranges[i][0]
            const end   = ranges[i][1]
            if (start < last) continue   // overlapping ranges — keep monotonic
            html += escapeHtml(sample.substring(last, start))
            html += "<span style='background-color:" + tint
                + "; padding:0 2px;'>"
                + escapeHtml(sample.substring(start, end))
                + "</span>"
            last = end
        }
        html += escapeHtml(sample.substring(last))
        sampleHighlight.text = "<pre style='font-family:monospace;'>" + html + "</pre>"
    }

    function escapeHtml(s) {
        return String(s)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/\n/g, "<br>")
    }

    readonly property var presets: [
        { name: qsTr("Email"),  pattern: "[\\w.%+-]+@[\\w.-]+\\.[A-Za-z]{2,}" },
        { name: qsTr("Phone"),  pattern: "\\+?\\d{1,3}[-. ]?\\(?\\d{1,4}\\)?[-. ]?\\d{1,9}[-. ]?\\d{1,9}" },
        { name: qsTr("Date"),   pattern: "\\d{4}-\\d{2}-\\d{2}" },
        { name: qsTr("Digits"), pattern: "\\d+" },
        { name: qsTr("URL"),    pattern: "https?://\\S+" },
        { name: qsTr("IPv4"),   pattern: "(\\d{1,3}\\.){3}\\d{1,3}" },
        { name: qsTr("Hex"),    pattern: "0x[0-9a-fA-F]+" },
        { name: qsTr("UUID"),   pattern: "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}" },
    ]

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // -- Header
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 76
            color: app.tokens.surface0
            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: 1
                color: app.tokens.separator
            }
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: app.tokens.spaceXL
                anchors.rightMargin: app.tokens.spaceXL
                spacing: app.tokens.spaceM
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 1
                    Controls.Label {
                        text: qsTr("Regex Builder")
                        font.pixelSize: app.tokens.textHeading
                        font.weight: app.tokens.weightBold
                        font.family: app.tokens.sansFamily
                        font.letterSpacing: -0.3
                    }
                    Controls.Label {
                        text: qsTr("Test patterns against sample text — same engine the search uses.")
                        font.pixelSize: app.tokens.textCaption + 1
                        font.family: app.tokens.sansFamily
                        opacity: 0.6
                    }
                }
                Rectangle {
                    radius: app.tokens.radiusPill
                    color: page.controller.error.length > 0
                        ? Qt.rgba(0.75, 0.23, 0.17, 0.2)
                        : page.controller.matchCount > 0
                            ? app.tokens.accentMute
                            : Qt.rgba(0, 0, 0, 0)
                    border.color: page.controller.error.length > 0
                        ? app.tokens.error
                        : page.controller.matchCount > 0
                            ? app.tokens.accent
                            : "transparent"
                    border.width: 1
                    implicitHeight: 28
                    implicitWidth: badgeLabel.implicitWidth + app.tokens.spaceL * 2
                    visible: patternField.text.length > 0
                    Controls.Label {
                        id: badgeLabel
                        anchors.centerIn: parent
                        text: page.controller.error.length > 0
                            ? qsTr("invalid")
                            : qsTr("%1 match(es)").arg(page.controller.matchCount)
                        font.pixelSize: app.tokens.textCaption
                        font.weight: app.tokens.weightMedium
                        color: page.controller.error.length > 0
                            ? app.tokens.error
                            : app.tokens.accent
                    }
                }
            }
        }

        // -- Preset chips
        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceL
            spacing: app.tokens.spaceS

            Controls.Label {
                text: qsTr("Presets")
                font.pixelSize: app.tokens.textCaption
                opacity: 0.65
            }
            Flow {
                Layout.fillWidth: true
                spacing: app.tokens.spaceS
                Repeater {
                    model: page.presets
                    delegate: Rectangle {
                        radius: app.tokens.radiusPill
                        property bool selected: page.activePreset === index
                        color: selected ? app.tokens.accentMute
                            : chipMouse.containsPress ? app.tokens.surface2
                            : chipMouse.containsMouse ? app.tokens.accentMute
                            : app.tokens.surface1
                        border.color: selected || chipMouse.containsMouse
                            ? app.tokens.accent : app.tokens.separatorStrong
                        border.width: 1
                        implicitHeight: 32
                        implicitWidth: presetLabel.implicitWidth + app.tokens.spaceL * 2
                        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
                        Behavior on border.color { ColorAnimation { duration: app.tokens.durationSnap } }
                        Controls.Label {
                            id: presetLabel
                            anchors.centerIn: parent
                            text: modelData.name
                            font.pixelSize: app.tokens.textCaption + 1
                            font.family: app.tokens.sansFamily
                            font.weight: selected ? app.tokens.weightSemibold : app.tokens.weightMedium
                            color: selected || chipMouse.containsMouse ? app.tokens.accent : Kirigami.Theme.textColor
                            opacity: selected || chipMouse.containsMouse ? 1.0 : 0.85
                            Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
                        }
                        MouseArea {
                            id: chipMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                page.activePreset = index
                                patternField.text = modelData.pattern
                                page.evaluate()
                            }
                        }
                    }
                }
            }
        }

        // -- Pattern editor
        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceL
            spacing: app.tokens.spaceXS

            Controls.Label {
                text: qsTr("Pattern")
                font.pixelSize: app.tokens.textCaption
                opacity: 0.65
            }
            RowLayout {
                Layout.fillWidth: true
                spacing: app.tokens.spaceS
                Controls.TextField {
                    id: patternField
                    Layout.fillWidth: true
                    placeholderText: qsTr("e.g.  fn\\s+\\w+_test")
                    font.family: app.tokens.monoFamily
                    onTextChanged: page.evaluate()
                }
                Controls.ToolButton {
                    id: caseInsensitive
                    checkable: true
                    text: "i"
                    font.family: app.tokens.monoFamily
                    Controls.ToolTip.text: qsTr("Case-insensitive")
                    Controls.ToolTip.visible: hovered
                    onToggled: page.evaluate()
                }
            }
        }

        // -- Error banner
        Kirigami.InlineMessage {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceS
            visible: page.controller.error.length > 0
            type: Kirigami.MessageType.Error
            text: page.controller.error
        }

        // -- Sample text
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceL
            Layout.bottomMargin: app.tokens.spaceL
            spacing: app.tokens.spaceXS

            Controls.Label {
                text: qsTr("Sample text")
                font.pixelSize: app.tokens.textCaption
                opacity: 0.65
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: app.tokens.radiusCard
                color: app.tokens.surface1
                border.color: app.tokens.separator
                border.width: 1

                Controls.ScrollView {
                    id: sampleScroll
                    anchors.fill: parent
                    anchors.margins: app.tokens.spaceS
                    visible: patternField.text.length === 0 || sampleHighlight.text.length === 0
                    Controls.TextArea {
                        id: sampleArea
                        placeholderText: qsTr("Paste sample text and watch the matches light up.")
                        font.family: app.tokens.monoFamily
                        font.pixelSize: app.tokens.textBody
                        wrapMode: TextEdit.Wrap
                        background: null
                        onTextChanged: page.evaluate()
                    }
                }
                // Read-only highlight overlay: when a pattern is active
                // we show this RichText-rendered version with <mark>
                // spans. Click anywhere to put focus back into the
                // editable area underneath.
                Flickable {
                    anchors.fill: parent
                    anchors.margins: app.tokens.spaceS
                    visible: !sampleScroll.visible
                    contentWidth: width
                    contentHeight: sampleHighlight.implicitHeight
                    clip: true
                    Controls.Label {
                        id: sampleHighlight
                        width: parent.width
                        textFormat: Text.RichText
                        wrapMode: Text.Wrap
                        font.family: app.tokens.monoFamily
                        font.pixelSize: app.tokens.textBody
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            sampleHighlight.text = ""   // hand back the editor
                            sampleArea.forceActiveFocus()
                        }
                    }
                }
            }
        }
    }
}
