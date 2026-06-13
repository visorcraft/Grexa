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
    property string runtimeText: ""
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
        page.runtimeText = app.settingsController.runtimeLicensesText()
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
        case 1: return app.i18n("ui-thirdparty-licenses")
        case 2: return app.i18n("ui-acknowledgments")
        case 3: return app.i18n("ui-runtime-components")
        default: return app.i18n("ui-grexa-license")
        }
    }

    function documentSubtitle(index) {
        switch (index) {
        case 1:
            return app.i18n("ui-the-cargoaboutgenerated-bundle-with-every-direct-d02cc5")
        case 2:
            return app.i18n("ui-narrative-attribution-for-grexa-grex-runtime-9cb532")
        case 3:
            return app.i18n("ui-full-license-texts-for-the-qt-7c5dad")
        default:
            return app.i18n("ui-the-complete-gpl30only-license-text-bundled-237019")
        }
    }

    function documentBody(index) {
        switch (index) {
        case 1: return page.thirdPartyText
        case 2: return page.creditsText
        case 3: return page.runtimeText
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
            return app.i18n("ui-no-matches-for-query").arg(query)
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
                    text: app.i18n("ui-licenses")
                    color: app.tokens.textPrimary
                    font.pixelSize: 24
                    font.weight: app.tokens.weightBold
                    font.family: app.tokens.sansFamily
                    font.letterSpacing: 0
                    Layout.fillWidth: true
                }

                Controls.Label {
                    text: app.i18n("ui-bundled-license-and-attribution-documents-available-9098e4")
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
                        text: app.i18n("ui-grexa-license")
                        width: implicitWidth + app.tokens.spaceL
                    }
                    Controls.TabButton {
                        text: app.i18n("ui-thirdparty")
                        width: implicitWidth + app.tokens.spaceL
                    }
                    Controls.TabButton {
                        text: app.i18n("ui-acknowledgments")
                        width: implicitWidth + app.tokens.spaceL
                    }
                    Controls.TabButton {
                        text: app.i18n("ui-runtime-components")
                        width: implicitWidth + app.tokens.spaceL
                    }
                }

                AppFlatButton {
                    text: app.i18n("ui-copy")
                    icon.name: "edit-copy-symbolic"
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: app.searchController.copyToClipboard(page.currentBody)
                    Controls.ToolTip.text: app.i18n("ui-copy-the-current-document")
                    Controls.ToolTip.visible: hovered
                }

                AppFlatButton {
                    visible: page.activeDocument === 0
                    text: app.i18n("ui-dialog")
                    icon.name: "document-preview-symbolic"
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: page.gplTextRequested()
                    Controls.ToolTip.text: app.i18n("ui-open-the-gpl-text-in-a")
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
                        ? app.i18n("ui-1-matches-ac30b9").arg(page.matchingLineCount)
                        : app.i18n("ui-1-lines-9b1ae5").arg(page.currentLineCount)
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
                    placeholderText: app.i18n("ui-find-by-crate-package-license-or")
                    onTextChanged: page.filterText = text
                    Accessible.name: app.i18n("ui-find-in-license-document")
                }

                Controls.CheckBox {
                    id: wrapToggle
                    text: app.i18n("ui-wrap")
                    checked: page.wrapText
                    onToggled: page.wrapText = checked
                    font.pixelSize: app.tokens.textCaption + 1
                    palette.windowText: app.tokens.textPrimary
                    palette.text: app.tokens.textPrimary
                }

                AppFlatButton {
                    enabled: page.filterText.length > 0
                    text: app.i18n("ui-clear")
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
