// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.Page {
    id: page
    padding: 0
    titleDelegate: Item {}
    globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

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

    signal gplTextRequested()

    property int activeDocument: 0
    property string filterText: ""
    property string thirdPartyText: ""
    property string creditsText: ""
    property string gplText: ""
    property bool wrapText: false

    readonly property string currentTitle: page.documentTitle(page.activeDocument)
    readonly property string currentSubtitle: page.documentSubtitle(page.activeDocument)
    readonly property string currentBody: page.documentBody(page.activeDocument)
    readonly property int currentLineCount: page.lineCount(page.currentBody)
    readonly property int matchingLineCount: page.countMatchingLines(page.currentBody, page.filterText)
    readonly property string visibleBody: page.filteredBody(page.currentBody, page.filterText)

    background: Rectangle { color: app.tokens.surface0 }

    Component.onCompleted: page.loadDocuments()

    function loadDocuments() {
        page.thirdPartyText = page.decodeEntities(app.settingsController.thirdPartyLicensesText())
        page.creditsText = page.decodeEntities(app.settingsController.creditsText())
        page.gplText = app.settingsController.gplLicenseText()
    }

    function decodeEntities(text) {
        return String(text)
            .replace(/&quot;/g, "\"")
            .replace(/&#39;/g, "'")
            .replace(/&apos;/g, "'")
            .replace(/&lt;/g, "<")
            .replace(/&gt;/g, ">")
            .replace(/&amp;/g, "&")
    }

    function documentTitle(index) {
        switch (index) {
        case 1: return qsTr("Third-party licenses")
        case 2: return qsTr("Acknowledgments")
        default: return qsTr("Grexa License")
        }
    }

    function documentSubtitle(index) {
        switch (index) {
        case 1:
            return qsTr("The cargo-about-generated bundle with every direct and transitive Rust crate, grouped by license text.")
        case 2:
            return qsTr("Narrative attribution for Grexa, Grex, runtime components, and direct dependencies.")
        default:
            return qsTr("The complete GPL-3.0-only license text bundled into the application.")
        }
    }

    function documentBody(index) {
        switch (index) {
        case 1: return page.thirdPartyText
        case 2: return page.creditsText
        default: return page.gplText
        }
    }

    function lineCount(text) {
        if (!text || text.length === 0)
            return 0
        return String(text).split("\n").length
    }

    function lineNumber(value) {
        let s = String(value)
        while (s.length < 5)
            s = " " + s
        return s
    }

    function countMatchingLines(text, query) {
        const needle = String(query).trim().toLowerCase()
        if (needle.length === 0)
            return 0
        const lines = String(text).split("\n")
        let matches = 0
        for (let i = 0; i < lines.length; ++i) {
            if (lines[i].toLowerCase().indexOf(needle) !== -1)
                matches += 1
        }
        return matches
    }

    function filteredBody(text, query) {
        const source = String(text)
        const needle = String(query).trim().toLowerCase()
        if (needle.length === 0)
            return source

        const lines = source.split("\n")
        const matches = []
        for (let i = 0; i < lines.length; ++i) {
            if (lines[i].toLowerCase().indexOf(needle) !== -1)
                matches.push(page.lineNumber(i + 1) + "  " + lines[i])
        }

        if (matches.length === 0)
            return qsTr("No matches for \"%1\".").arg(query)
        return matches.join("\n")
    }

    function setActiveDocument(index) {
        if (page.activeDocument === index)
            return
        page.activeDocument = index
        page.filterText = ""
        filterField.text = ""
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 86
            color: app.tokens.surface1

            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: 1
                color: app.tokens.separator
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.leftMargin: app.tokens.spaceXL
                anchors.rightMargin: app.tokens.spaceXL
                spacing: app.tokens.spaceXS

                Item { Layout.fillHeight: true }

                Controls.Label {
                    text: qsTr("Licenses")
                    color: app.tokens.textPrimary
                    font.pixelSize: 24
                    font.weight: app.tokens.weightBold
                    font.family: app.tokens.sansFamily
                    font.letterSpacing: 0
                    Layout.fillWidth: true
                }

                Controls.Label {
                    text: qsTr("Bundled license and attribution documents, available without opening a browser.")
                    color: app.tokens.textPrimary
                    font.pixelSize: app.tokens.textCaption + 1
                    font.family: app.tokens.sansFamily
                    opacity: 0.62
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                }

                Item { Layout.fillHeight: true }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceL
            Layout.bottomMargin: app.tokens.spaceXL
            spacing: app.tokens.spaceM

            RowLayout {
                Layout.fillWidth: true
                spacing: app.tokens.spaceM

                Controls.TabBar {
                    id: tabs
                    Layout.fillWidth: true
                    currentIndex: page.activeDocument
                    onCurrentIndexChanged: page.setActiveDocument(currentIndex)

                    Controls.TabButton {
                        text: qsTr("Grexa License")
                        width: implicitWidth + app.tokens.spaceL
                    }
                    Controls.TabButton {
                        text: qsTr("Third-party")
                        width: implicitWidth + app.tokens.spaceL
                    }
                    Controls.TabButton {
                        text: qsTr("Acknowledgments")
                        width: implicitWidth + app.tokens.spaceL
                    }
                }

                AppFlatButton {
                    text: qsTr("Copy")
                    icon.name: "edit-copy-symbolic"
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: app.searchController.copyToClipboard(page.currentBody)
                    Controls.ToolTip.text: qsTr("Copy the current document")
                    Controls.ToolTip.visible: hovered
                }

                AppFlatButton {
                    visible: page.activeDocument === 0
                    text: qsTr("Dialog")
                    icon.name: "document-preview-symbolic"
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: page.gplTextRequested()
                    Controls.ToolTip.text: qsTr("Open the GPL text in a dialog")
                    Controls.ToolTip.visible: hovered
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: app.tokens.spaceM

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceXS

                    Controls.Label {
                        text: page.currentTitle
                        color: app.tokens.textPrimary
                        font.pixelSize: app.tokens.textSubheading
                        font.weight: app.tokens.weightBold
                        font.family: app.tokens.sansFamily
                        font.letterSpacing: 0
                        Layout.fillWidth: true
                    }

                    Controls.Label {
                        text: page.currentSubtitle
                        color: app.tokens.textPrimary
                        font.pixelSize: app.tokens.textCaption + 1
                        font.family: app.tokens.sansFamily
                        opacity: 0.62
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }

                Controls.Label {
                    text: page.filterText.trim().length > 0
                        ? qsTr("%1 matches").arg(page.matchingLineCount)
                        : qsTr("%1 lines").arg(page.currentLineCount)
                    color: app.tokens.textPrimary
                    font.pixelSize: app.tokens.textCaption
                    font.family: app.tokens.monoFamily
                    opacity: 0.62
                    Layout.alignment: Qt.AlignRight | Qt.AlignVCenter
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: app.tokens.spaceS

                AppTextField {
                    id: filterField
                    Layout.fillWidth: true
                    placeholderText: qsTr("Find by crate, package, license, or phrase...")
                    onTextChanged: page.filterText = text
                    Accessible.name: qsTr("Find in license document")
                }

                Controls.CheckBox {
                    id: wrapToggle
                    text: qsTr("Wrap")
                    checked: page.wrapText
                    onToggled: page.wrapText = checked
                    font.pixelSize: app.tokens.textCaption + 1
                    palette.windowText: app.tokens.textPrimary
                    palette.text: app.tokens.textPrimary
                }

                AppFlatButton {
                    enabled: page.filterText.length > 0
                    text: qsTr("Clear")
                    icon.name: "edit-clear-symbolic"
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: {
                        filterField.text = ""
                        page.filterText = ""
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 340
                radius: app.tokens.radiusCard
                color: app.tokens.surface1
                border.color: app.tokens.separator
                border.width: 1
                clip: true

                Controls.ScrollView {
                    anchors.fill: parent
                    anchors.margins: app.tokens.spaceM
                    clip: true

                    Controls.TextArea {
                        id: documentArea
                        text: page.visibleBody
                        readOnly: true
                        selectByMouse: true
                        persistentSelection: true
                        wrapMode: page.wrapText ? TextEdit.Wrap : TextEdit.NoWrap
                        textFormat: TextEdit.PlainText
                        color: app.tokens.textPrimary
                        selectedTextColor: app.tokens.accentText
                        selectionColor: app.tokens.accent
                        font.pixelSize: app.tokens.textCaption + 1
                        font.family: app.tokens.monoFamily
                        background: Rectangle { color: "transparent" }
                    }
                }
            }
        }
    }
}
