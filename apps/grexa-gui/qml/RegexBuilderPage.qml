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

    // See SettingsPage.qml — Pages render under the View colorSet.
    Kirigami.Theme.inherit: false
    Kirigami.Theme.colorSet: Kirigami.Theme.View
    Kirigami.Theme.backgroundColor: app.tokens.surface0
    Kirigami.Theme.textColor: app.tokens.textPrimary
    Kirigami.Theme.highlightColor: app.tokens.accent
    Kirigami.Theme.highlightedTextColor: app.tokens.accentText

    palette.window:          app.tokens.surface0
    palette.windowText:      app.tokens.textPrimary
    palette.base:            app.tokens.surface1
    palette.alternateBase:   app.tokens.surface2
    palette.text:            app.tokens.textPrimary
    palette.button:          app.tokens.surface1
    palette.buttonText:      app.tokens.textPrimary
    palette.brightText:      app.tokens.accentText
    palette.highlight:       app.tokens.accent
    palette.highlightedText: app.tokens.accentText
    palette.toolTipBase:     app.tokens.surface2
    palette.toolTipText:     app.tokens.textPrimary
    palette.mid:             app.tokens.separator
    palette.midlight:        app.tokens.surface1
    palette.light:           app.tokens.surface2
    palette.dark:            app.tokens.surface0
    palette.shadow:          app.tokens.shadowFar
    palette.placeholderText: Qt.rgba(app.tokens.textPrimary.r,
                                     app.tokens.textPrimary.g,
                                     app.tokens.textPrimary.b, 0.55)

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
        matchesModel.clear()
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
        // Populate the side-panel match list and the highlight overlay
        // in a single pass. The matches list trims each entry to a
        // reasonable preview so a giant match doesn't blow up the panel.
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
            const captured = sample.substring(start, end)
            matchesModel.append({
                index: i + 1,
                start: start,
                end: end,
                text: captured.length > 64 ? captured.substring(0, 64) + "…" : captured
            })
        }
        html += escapeHtml(sample.substring(last))
        sampleHighlight.text = "<pre style='font-family:monospace;'>" + html + "</pre>"
    }

    // Side-panel match list — populated by refreshHighlight().
    ListModel { id: matchesModel }

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
                AppTextField {
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

        // -- Sample text + live match list (Mailspring-class
        // two-column layout — input on the left, the list of what
        // the pattern matched on the right).
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceL
            Layout.bottomMargin: app.tokens.spaceL
            spacing: app.tokens.spaceL

            // -- Sample text card
            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: app.tokens.spaceS

                RowLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceS
                    Controls.Label {
                        text: qsTr("SAMPLE TEXT")
                        font.pixelSize: 10
                        font.weight: app.tokens.weightSemibold
                        font.letterSpacing: 1.6
                        opacity: 0.5
                    }
                    Item { Layout.fillWidth: true }
                    AppFlatButton {
                        visible: sampleArea.text.length > 0
                        icon.name: "edit-clear-symbolic"
                        icon.color: app.tokens.textPrimary
                        text: qsTr("Clear")
                        display: Controls.AbstractButton.TextBesideIcon
                        font.pixelSize: app.tokens.textCaption
                        onClicked: { sampleArea.text = ""; page.evaluate() }
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: app.tokens.radiusCard
                    color: app.tokens.surface1
                    border.color: sampleArea.activeFocus ? app.tokens.accent : app.tokens.separatorStrong
                    border.width: 1
                    Behavior on border.color { ColorAnimation { duration: app.tokens.durationSnap } }

                    Controls.ScrollView {
                        id: sampleScroll
                        anchors.fill: parent
                        anchors.margins: app.tokens.spaceL
                        visible: patternField.text.length === 0 || sampleHighlight.text.length === 0
                        Controls.TextArea {
                            id: sampleArea
                            placeholderText: qsTr("Paste sample text and watch the matches light up…")
                            font.family: app.tokens.monoFamily
                            font.pixelSize: app.tokens.textBody + 1
                            wrapMode: TextEdit.Wrap
                            background: null
                            onTextChanged: page.evaluate()
                        }
                    }
                    // Read-only highlight overlay: when a pattern is
                    // active and matched, we render the RichText
                    // version. Click anywhere to put focus back into
                    // the editable area underneath.
                    Flickable {
                        anchors.fill: parent
                        anchors.margins: app.tokens.spaceL
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
                            font.pixelSize: app.tokens.textBody + 1
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: {
                                sampleHighlight.text = ""
                                sampleArea.forceActiveFocus()
                            }
                        }
                    }
                }
            }

            // -- Matches side-panel
            ColumnLayout {
                Layout.preferredWidth: 320
                Layout.minimumWidth: 240
                Layout.fillHeight: true
                spacing: app.tokens.spaceS

                RowLayout {
                    Layout.fillWidth: true
                    Controls.Label {
                        text: qsTr("MATCHES")
                        font.pixelSize: 10
                        font.weight: app.tokens.weightSemibold
                        font.letterSpacing: 1.6
                        opacity: 0.5
                    }
                    Item { Layout.fillWidth: true }
                    Rectangle {
                        visible: matchesModel.count > 0
                        radius: app.tokens.radiusPill
                        color: app.tokens.accentMute
                        border.color: app.tokens.accent
                        border.width: 1
                        implicitHeight: 20
                        implicitWidth: matchTotal.implicitWidth + app.tokens.spaceM * 2
                        Controls.Label {
                            id: matchTotal
                            anchors.centerIn: parent
                            text: matchesModel.count + ""
                            font.pixelSize: app.tokens.textCaption
                            font.weight: app.tokens.weightSemibold
                            font.family: app.tokens.monoFamily
                            color: app.tokens.accent
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: app.tokens.radiusCard
                    color: app.tokens.surface1
                    border.color: app.tokens.separator
                    border.width: 1

                    Controls.ScrollView {
                        anchors.fill: parent
                        anchors.margins: app.tokens.spaceXS
                        visible: matchesModel.count > 0
                        ListView {
                            anchors.fill: parent
                            model: matchesModel
                            spacing: 0
                            clip: true
                            delegate: Rectangle {
                                width: ListView.view.width
                                height: 38
                                color: matchMouse.containsMouse ? app.tokens.surface2 : "transparent"
                                Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
                                Rectangle {
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.bottom: parent.bottom
                                    height: 1
                                    color: app.tokens.separator
                                    opacity: 0.6
                                }
                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: app.tokens.spaceM
                                    anchors.rightMargin: app.tokens.spaceM
                                    spacing: app.tokens.spaceS
                                    Controls.Label {
                                        text: "#" + model.index
                                        font.family: app.tokens.monoFamily
                                        font.pixelSize: app.tokens.textCaption
                                        color: app.tokens.accent
                                        opacity: 0.8
                                        Layout.preferredWidth: 28
                                    }
                                    Controls.Label {
                                        Layout.fillWidth: true
                                        text: model.text
                                        font.family: app.tokens.monoFamily
                                        font.pixelSize: app.tokens.textBody
                                        elide: Text.ElideRight
                                    }
                                    Controls.Label {
                                        text: model.start + "‥" + model.end
                                        font.family: app.tokens.monoFamily
                                        font.pixelSize: app.tokens.textCaption
                                        opacity: 0.5
                                    }
                                }
                                MouseArea {
                                    id: matchMouse
                                    anchors.fill: parent
                                    hoverEnabled: true
                                }
                            }
                        }
                    }

                    // Empty side-panel state.
                    ColumnLayout {
                        anchors.centerIn: parent
                        width: parent.width - app.tokens.spaceXL * 2
                        spacing: app.tokens.spaceS
                        visible: matchesModel.count === 0
                        Kirigami.Icon {
                            source: "edit-find-symbolic"
                            implicitWidth: 32
                            implicitHeight: 32
                            opacity: 0.32
                            Layout.alignment: Qt.AlignHCenter
                            isMask: true
                            color: Kirigami.Theme.textColor
                        }
                        Controls.Label {
                            text: page.controller.error.length > 0
                                ? qsTr("Invalid pattern")
                                : patternField.text.length === 0
                                    ? qsTr("Enter a pattern")
                                    : sampleArea.text.length === 0
                                        ? qsTr("Add sample text")
                                        : qsTr("No matches")
                            font.pixelSize: app.tokens.textCaption + 1
                            font.weight: app.tokens.weightMedium
                            font.family: app.tokens.sansFamily
                            opacity: 0.55
                            horizontalAlignment: Text.AlignHCenter
                            Layout.alignment: Qt.AlignHCenter
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                        }
                    }
                }
            }
        }
    }
}
