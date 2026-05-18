// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Saved-search profiles page. Each profile is a named snapshot of
// {path, term, regex, case-sensitive, files-mode}. The Search page
// lets the user save its current form as a profile; this page lists
// every saved profile and lets them re-load or delete.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
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

    ListModel { id: profilesModel }
    property string filterText: ""

    // See HistoryPage — debounce keystrokes so a large profile
    // list doesn't rebuild per character.
    Timer {
        id: filterDebounce
        interval: 120
        repeat: false
        onTriggered: page.refresh()
    }

    Component.onCompleted: refresh()

    function rowMatchesFilter(name, term, path) {
        const f = filterText.trim().toLowerCase()
        if (f.length === 0) return true
        return name.toLowerCase().includes(f)
            || term.toLowerCase().includes(f)
            || path.toLowerCase().includes(f)
    }

    function refresh() {
        profilesModel.clear()
        try {
            const arr = JSON.parse(app.searchController.profilesJson())
            for (let i = 0; i < arr.length; ++i) {
                const p = arr[i]
                const name = p.name || ""
                const term = (p.search_options && p.search_options.search_term) || ""
                const path = (p.search_options && p.search_options.path) || ""
                if (!rowMatchesFilter(name, term, path)) continue
                profilesModel.append({
                    name: name,
                    term: term,
                    path: path,
                    regex: (p.search_options && p.search_options.regex) || false,
                    caseSensitive: (p.search_options && p.search_options.case_sensitive) || false,
                    filesMode: p.files_search || false
                })
            }
        } catch (e) {}
    }

    ColumnLayout {
        width: page.width
        spacing: 0

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
                        text: qsTr("Profiles")
                        font.pixelSize: app.tokens.textHeading
                        font.weight: app.tokens.weightBold
                        font.family: app.tokens.sansFamily
                        font.letterSpacing: -0.3
                    }
                    Controls.Label {
                        text: qsTr("Named search presets. The Search page's “Save current as profile…” captures the active form here.")
                        font.pixelSize: app.tokens.textCaption + 1
                        font.family: app.tokens.sansFamily
                        opacity: 0.6
                    }
                }
                AppFlatButton {
                    icon.name: "view-refresh"
                    icon.color: app.tokens.textPrimary
                    text: qsTr("Refresh")
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: page.refresh()
                }
            }
        }

        // -- Filter row
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceM
            spacing: app.tokens.spaceS

            Kirigami.Icon {
                source: "view-filter-symbolic"
                implicitWidth: 14
                implicitHeight: 14
                isMask: true
                color: Kirigami.Theme.textColor
                opacity: 0.5
            }
            AppTextField {
                Layout.fillWidth: true
                placeholderText: qsTr("Filter profiles by name, term, or path")
                text: page.filterText
                onTextEdited: { page.filterText = text; filterDebounce.restart() }
            }
            AppFlatButton {
                icon.name: "edit-clear-symbolic"
                icon.color: app.tokens.textPrimary
                display: Controls.AbstractButton.IconOnly
                enabled: page.filterText.length > 0
                onClicked: { page.filterText = ""; filterDebounce.stop(); page.refresh() }
            }
        }

        Kirigami.PlaceholderMessage {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: app.tokens.spaceXL * 2
            visible: profilesModel.count === 0
            icon.name: "document-save-symbolic"
            icon.color: app.tokens.textPrimary
            text: page.filterText.length > 0
                ? qsTr("No profiles match “%1”").arg(page.filterText)
                : qsTr("No saved profiles")
            explanation: page.filterText.length > 0
                ? qsTr("Try a shorter filter, or clear it to see every saved profile.")
                : qsTr("Open the Search page, fill in path + term + flags, then save the form as a named profile.")
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceL
            Layout.bottomMargin: app.tokens.spaceL
            spacing: app.tokens.spaceS

            Repeater {
                model: profilesModel
                delegate: Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 68
                    radius: app.tokens.radiusCard
                    color: app.tokens.surface1
                    border.color: app.tokens.separator
                    border.width: 1
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: app.tokens.spaceL
                        anchors.rightMargin: app.tokens.spaceM
                        spacing: app.tokens.spaceM
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2
                            Controls.Label {
                                Layout.fillWidth: true
                                text: model.name
                                font.pixelSize: app.tokens.textBodyEmphasis
                                font.weight: app.tokens.weightSemibold
                                elide: Text.ElideRight
                            }
                            Controls.Label {
                                Layout.fillWidth: true
                                text: qsTr("%1 · “%2”%3%4%5").arg(model.path).arg(model.term)
                                    .arg(model.regex ? " · regex" : "")
                                    .arg(model.caseSensitive ? " · case" : "")
                                    .arg(model.filesMode ? " · files" : "")
                                font.family: app.tokens.monoFamily
                                font.pixelSize: app.tokens.textCaption
                                opacity: 0.65
                                elide: Text.ElideMiddle
                            }
                        }
                        AppFlatButton {
                            icon.name: "edit-find-symbolic"
                            icon.color: app.tokens.textPrimary
                            text: qsTr("Open")
                            display: Controls.AbstractButton.TextBesideIcon
                            onClicked: {
                                const path = model.path
                                const term = model.term
                                const regex = model.regex
                                const caseSensitive = model.caseSensitive
                                const filesMode = model.filesMode
                                app.goTo("search")
                                Qt.callLater(function() {
                                    const p = app.pageStack.currentItem
                                    if (p && p.searchBar) {
                                        p.searchBar.pathText = path
                                        p.searchBar.termText = term
                                        p.searchBar.regexEnabled = regex
                                        p.searchBar.caseSensitive = caseSensitive
                                        p.controller.resultMode = filesMode ? 1 : 0
                                        if (p.controller.busy) p.controller.cancel()
                                        p.controller.clearResults()
                                    }
                                })
                            }
                        }
                        AppFlatButton {
                            icon.name: "edit-delete-symbolic"
                            icon.color: app.tokens.textPrimary
                            display: Controls.AbstractButton.IconOnly
                            Controls.ToolTip.text: qsTr("Delete profile")
                            Controls.ToolTip.visible: hovered
                            onClicked: {
                                app.searchController.deleteProfile(model.name)
                                page.refresh()
                            }
                        }
                    }
                }
            }
        }
    }
}
