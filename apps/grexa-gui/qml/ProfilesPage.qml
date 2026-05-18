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

    ListModel { id: profilesModel }

    Component.onCompleted: refresh()

    function refresh() {
        profilesModel.clear()
        try {
            const arr = JSON.parse(app.searchController.profilesJson())
            for (let i = 0; i < arr.length; ++i) {
                const p = arr[i]
                profilesModel.append({
                    name: p.name || "",
                    term: (p.search_options && p.search_options.search_term) || "",
                    path: (p.search_options && p.search_options.path) || "",
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
                Controls.Button {
                    flat: true
                    icon.name: "view-refresh"
                    text: qsTr("Refresh")
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: page.refresh()
                }
            }
        }

        Kirigami.PlaceholderMessage {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: app.tokens.spaceXL * 2
            visible: profilesModel.count === 0
            icon.name: "document-save-symbolic"
            text: qsTr("No saved profiles")
            explanation: qsTr("Open the Search page, fill in path + term + flags, then save the form as a named profile.")
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
                        Controls.Button {
                            flat: true
                            icon.name: "edit-find-symbolic"
                            text: qsTr("Open")
                            display: Controls.AbstractButton.TextBesideIcon
                            onClicked: {
                                app.goTo("search")
                                const p = app.pageStack.currentItem
                                if (p && p.searchBar) {
                                    p.searchBar.pathText = model.path
                                    p.searchBar.termText = model.term
                                    p.searchBar.regexEnabled = model.regex
                                    p.searchBar.caseSensitive = model.caseSensitive
                                    if (model.filesMode) p.controller.resultMode = 1
                                }
                            }
                        }
                        Controls.Button {
                            flat: true
                            icon.name: "edit-delete-symbolic"
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
