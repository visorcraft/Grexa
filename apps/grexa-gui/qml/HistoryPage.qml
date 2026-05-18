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

    ListModel { id: historyModel }

    Component.onCompleted: refresh()

    function refresh() {
        historyModel.clear()
        try {
            const arr = JSON.parse(app.searchController.historyJson())
            for (let i = 0; i < arr.length; ++i) {
                const e = arr[i]
                historyModel.append({
                    term: e.search_term || "",
                    path: e.search_path || "",
                    matches: e.result_count || 0,
                    regex: e.regex_search || false,
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
                Controls.Button {
                    flat: true
                    icon.name: "view-refresh"
                    text: qsTr("Refresh")
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: page.refresh()
                }
            }
        }

        // -- Empty state
        Kirigami.PlaceholderMessage {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: app.tokens.spaceXL * 2
            visible: historyModel.count === 0
            icon.name: "history-symbolic"
            text: qsTr("No search history yet")
            explanation: qsTr("Run a search from the Search page and it'll land here.")
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
                                text: qsTr("%1 · %2 matches%3%4").arg(model.path)
                                    .arg(model.matches)
                                    .arg(model.regex ? " · regex" : "")
                                    .arg(model.caseSensitive ? " · case" : "")
                                font.pixelSize: app.tokens.textCaption
                                opacity: 0.6
                                elide: Text.ElideMiddle
                            }
                        }
                        Controls.Button {
                            flat: true
                            icon.name: "edit-find-symbolic"
                            text: qsTr("Open")
                            display: Controls.AbstractButton.TextBesideIcon
                            onClicked: {
                                // Switch to Search and populate the
                                // form so the user can choose to
                                // re-run (or edit) the search.
                                app.goTo("search")
                                const p = app.pageStack.currentItem
                                if (p && p.searchBar) {
                                    p.searchBar.pathText = model.path
                                    p.searchBar.termText = model.term
                                    p.searchBar.regexEnabled = model.regex
                                    p.searchBar.caseSensitive = model.caseSensitive
                                }
                            }
                        }
                        Controls.Button {
                            flat: true
                            icon.name: "edit-delete-symbolic"
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
