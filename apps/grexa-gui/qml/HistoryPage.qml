// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Search history page — every search that completed lives here
// keyed by the seven-field dedupe. Clicking an entry repopulates the
// Search page form; the user still has to hit Search to actually run
// the query (so the history page never silently runs an expensive
// search-everything against `/`).

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

    ListModel { id: historyModel }
    property string filterText: ""

    // Debounce filter typing so a 500-entry history doesn't
    // rebuild on every keystroke. 120ms is below the perceptual
    // threshold for "typing while seeing results update".
    Timer {
        id: filterDebounce
        interval: 120
        repeat: false
        onTriggered: page.refresh()
    }

    Component.onCompleted: refresh()

    function rowMatchesFilter(term, path) {
        const f = filterText.trim().toLowerCase()
        if (f.length === 0) return true
        return term.toLowerCase().includes(f) || path.toLowerCase().includes(f)
    }

    function refresh() {
        historyModel.clear()
        try {
            const arr = JSON.parse(app.searchController.historyJson())
            for (let i = 0; i < arr.length; ++i) {
                const e = arr[i]
                const term = e.search_term || ""
                const path = e.search_path || ""
                if (!rowMatchesFilter(term, path)) continue
                historyModel.append({
                    term: term,
                    path: path,
                    matches: e.result_count || 0,
                    regex: e.regex_search || false,
                    filesMode: e.files_search || false,
                    caseSensitive: e.search_case_sensitive || false,
                    timestamp: e.timestamp_unix || 0,
                    raw: JSON.stringify(e)
                })
            }
        } catch (e) {}
    }

    ColumnLayout {
        width: page.width
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
                        text: qsTr("History")
                        font.pixelSize: app.tokens.textHeading
                        font.weight: app.tokens.weightBold
                        font.family: app.tokens.sansFamily
                        font.letterSpacing: -0.3
                    }
                    Controls.Label {
                        text: qsTr("Every completed search, deduped on the seven-field Grex key.")
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
                placeholderText: qsTr("Filter history by term or path")
                text: page.filterText
                onTextEdited: { page.filterText = text; filterDebounce.restart() }
            }
            AppFlatButton {
                icon.name: "edit-clear-symbolic"
                icon.color: app.tokens.textPrimary
                display: Controls.AbstractButton.IconOnly
                enabled: page.filterText.length > 0
                // Clear is immediate — the user shouldn't wait
                // for the debounce when they explicitly hit X.
                onClicked: { page.filterText = ""; filterDebounce.stop(); page.refresh() }
            }
        }

        // -- Empty state
        Kirigami.PlaceholderMessage {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: app.tokens.spaceXL * 2
            visible: historyModel.count === 0
            icon.name: "history-symbolic"
            icon.color: app.tokens.textPrimary
            text: page.filterText.length > 0
                ? qsTr("No history entries match “%1”").arg(page.filterText)
                : qsTr("No search history yet")
            explanation: page.filterText.length > 0
                ? qsTr("Try a shorter filter, or clear it to see every saved search.")
                : qsTr("Run a search from the Search page and it'll land here.")
        }

        // -- List
        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceL
            Layout.bottomMargin: app.tokens.spaceL
            spacing: app.tokens.spaceS

            Repeater {
                model: historyModel
                delegate: Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 64
                    radius: app.tokens.radiusCard
                    color: rowHover.containsMouse ? app.tokens.surface2 : app.tokens.surface1
                    border.color: app.tokens.separator
                    border.width: 1
                    Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }

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
                                text: model.term
                                font.family: app.tokens.monoFamily
                                font.pixelSize: app.tokens.textBodyEmphasis
                                font.weight: app.tokens.weightSemibold
                                elide: Text.ElideRight
                            }
                            Controls.Label {
                                Layout.fillWidth: true
                                // Plural form goes through Qt's `qsTr(..., "", n)`
                                // overload so translators can drive the singular /
                                // plural inflection per locale instead of inheriting
                                // English rules from an inline ternary.
                                text: qsTr("%1 · %2%3%4").arg(model.path)
                                    .arg(qsTr("%n match(es)", "", model.matches))
                                    .arg(model.regex ? " · regex" : "")
                                    .arg(model.caseSensitive ? " · case" : "")
                                font.pixelSize: app.tokens.textCaption
                                opacity: 0.6
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
                                // Switch to Search and populate the
                                // form so the user can choose to
                                // re-run (or edit) the search.
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
                            Controls.ToolTip.text: qsTr("Forget this entry")
                            Controls.ToolTip.visible: hovered
                            onClicked: {
                                app.searchController.removeHistoryEntry(model.raw)
                                page.refresh()
                            }
                        }
                    }
                    MouseArea {
                        id: rowHover
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.NoButton
                    }
                }
            }
        }
    }
}
