// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Search workspace. Unified address-bar style SearchBar at the top
// (path + term + flags + primary action in one card), result list
// below, status footer pinned to the bottom.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.visorcraft.Grexa 1.0

Kirigami.Page {
    id: page
    padding: 0
    titleDelegate: Item {}
    globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

    readonly property SearchController controller: app.searchController

    Connections {
        target: page.controller
        function onHistoryChanged() { page.refreshRecentPaths() }
    }

    Component.onCompleted: refreshRecentPaths()

    function refreshRecentPaths() {
        recentPaths.clear()
        try {
            const arr = JSON.parse(controller.recentPathsJson())
            for (let i = 0; i < arr.length; ++i) {
                recentPaths.append({ pathText: arr[i] })
            }
        } catch (e) {}
    }

    function launchSearch() {
        if (searchBar.pathText.length === 0 || searchBar.termText.length === 0) return
        controller.startSearch(searchBar.pathText, searchBar.termText,
                               searchBar.regexEnabled, searchBar.caseSensitive, false)
    }

    function applyExample(path, term) {
        searchBar.pathText = path
        searchBar.termText = term
        launchSearch()
    }

    ListModel { id: recentPaths }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // ============================================================
        // Toolbar / SearchBar strip
        // ============================================================
        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 92
            // Subtle top-bar gradient
            Rectangle {
                anchors.fill: parent
                gradient: Gradient {
                    GradientStop { position: 0.0; color: app.tokens.surface1 }
                    GradientStop { position: 1.0; color: app.tokens.surface0 }
                }
                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 1
                    color: app.tokens.separator
                }
            }
            SearchBar {
                id: searchBar
                anchors.fill: parent
                anchors.leftMargin: app.tokens.spaceXL
                anchors.rightMargin: app.tokens.spaceXL
                anchors.topMargin: app.tokens.spaceXL
                anchors.bottomMargin: app.tokens.spaceXL
                recentPathsModel: recentPaths
                busy: page.controller.busy
                onSubmitted: page.launchSearch()
                onBrowse: browseDialog.open()
            }
        }

        // ============================================================
        // Secondary action row — Stop / Clear + counter
        // ============================================================
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceM
            Layout.bottomMargin: app.tokens.spaceM
            spacing: app.tokens.spaceS

            Controls.Button {
                flat: true
                icon.name: "process-stop-symbolic"
                text: qsTr("Stop")
                display: Controls.AbstractButton.TextBesideIcon
                enabled: page.controller.busy
                onClicked: page.controller.cancel()
            }
            Controls.Button {
                flat: true
                icon.name: "edit-clear-symbolic"
                text: qsTr("Clear")
                display: Controls.AbstractButton.TextBesideIcon
                enabled: !page.controller.busy && page.controller.matchCount > 0
                onClicked: page.controller.clearResults()
            }
            Item { Layout.fillWidth: true }
            Rectangle {
                visible: page.controller.matchCount > 0
                radius: app.tokens.radiusPill
                color: app.tokens.accentMute
                border.color: app.tokens.accent
                border.width: 1
                implicitHeight: 22
                implicitWidth: matchCountLabel.implicitWidth + app.tokens.spaceM * 2
                Controls.Label {
                    id: matchCountLabel
                    anchors.centerIn: parent
                    text: qsTr("%1 matches · %2 files").arg(page.controller.matchCount).arg(page.controller.filesMatched)
                    font.pixelSize: app.tokens.textCaption
                    font.weight: app.tokens.weightMedium
                    color: app.tokens.accent
                }
            }
        }

        // ============================================================
        // Result area
        // ============================================================
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ListView {
                id: resultList
                anchors.fill: parent
                clip: true
                model: page.controller
                spacing: 0
                visible: count > 0

                add: Transition {
                    NumberAnimation { property: "opacity"; from: 0; to: 1; duration: app.tokens.durationSnap }
                    NumberAnimation { property: "y"; from: 6; duration: app.tokens.durationSnap; easing.type: Easing.OutCubic }
                }

                delegate: ResultRow {
                    width: ListView.view.width
                    relativePath: model.relativePath
                    line: parseInt(model.line, 10)
                    column: parseInt(model.column, 10)
                    previewBefore: model.previewBefore
                    previewMatch: model.previewMatch
                    previewAfter: model.previewAfter
                    onOpenPreview: {
                        contextPreview.path = page.controller.rowFullPath(index)
                        contextPreview.lineNumber = parseInt(model.line, 10)
                        contextPreview.open()
                    }
                }
            }

            EmptyState {
                anchors.centerIn: parent
                width: parent.width
                height: parent.height
                visible: resultList.count === 0 && !page.controller.busy
                title: qsTr("Search anywhere on your system")
                explanation: qsTr("Pick a folder, type a term, and we'll stream matches as they appear.")
                chipsModel: [
                    { label: qsTr("~/code · TODO"),       path: "/work/repos/visorcraft/grexa/crates", term: "TODO" },
                    { label: qsTr("~ · fn\\s+\\w+_test"), path: ".",                                  term: "fn\\s+\\w+_test" },
                    { label: qsTr("/etc · password"),     path: "/etc",                               term: "password" }
                ]
                onChipClicked: function(idx, data) {
                    page.applyExample(data.path, data.term)
                }
            }

            // Initial-search overlay
            Rectangle {
                anchors.fill: parent
                color: app.tokens.surface0
                opacity: page.controller.busy && resultList.count === 0 ? 0.85 : 0
                visible: opacity > 0
                Behavior on opacity { NumberAnimation { duration: app.tokens.durationNormal } }
                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: app.tokens.spaceM
                    Controls.BusyIndicator {
                        running: true
                        Layout.alignment: Qt.AlignHCenter
                    }
                    Controls.Label {
                        text: qsTr("Searching…")
                        font.pixelSize: app.tokens.textBody
                        opacity: 0.7
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }
        }

        // ============================================================
        // Status footer
        // ============================================================
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 30
            color: app.tokens.surface1
            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                height: 1
                color: app.tokens.separator
            }
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: app.tokens.spaceXL
                anchors.rightMargin: app.tokens.spaceXL
                spacing: app.tokens.spaceL

                Rectangle {
                    width: 8; height: 8; radius: 4
                    color: page.controller.busy ? app.tokens.warning
                         : page.controller.matchCount > 0 ? app.tokens.success
                         : app.tokens.separatorStrong
                    Behavior on color { ColorAnimation { duration: app.tokens.durationNormal } }
                }
                Controls.Label {
                    Layout.fillWidth: true
                    text: page.controller.statusText.length > 0 ? page.controller.statusText : qsTr("Ready")
                    font.pixelSize: app.tokens.textCaption
                    opacity: 0.75
                    elide: Text.ElideMiddle
                }
                Controls.Label {
                    visible: page.controller.filesScanned > 0
                    text: qsTr("scanned %1").arg(page.controller.filesScanned)
                    font.pixelSize: app.tokens.textCaption
                    opacity: 0.45
                    font.family: app.tokens.monoFamily
                }
                Controls.Label {
                    text: qsTr("recent %1").arg(page.controller.recentPathCount)
                    font.pixelSize: app.tokens.textCaption
                    opacity: 0.45
                    font.family: app.tokens.monoFamily
                }
            }
        }
    }

    Controls.Dialog {
        id: browseDialog
        modal: true
        title: qsTr("Choose folder")
        standardButtons: Controls.Dialog.Cancel
        Controls.Label {
            text: qsTr("The Portal file picker lands in Phase 5. Type the path directly for now.")
            wrapMode: Text.WordWrap
        }
    }

    ContextPreviewDialog {
        id: contextPreview
    }
}
