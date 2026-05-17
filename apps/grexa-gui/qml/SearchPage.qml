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
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 92
            color: app.tokens.surface0
            // Hairline bottom edge — subtler than v1.
            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: 1
                color: app.tokens.separator
            }
            SearchBar {
                id: searchBar
                anchors.fill: parent
                anchors.leftMargin: app.tokens.spaceXL
                anchors.rightMargin: app.tokens.spaceXL
                anchors.topMargin: app.tokens.spaceL + 2
                anchors.bottomMargin: app.tokens.spaceL + 2
                recentPathsModel: recentPaths
                busy: page.controller.busy
                onSubmitted: page.launchSearch()
                onBrowse: browseDialog.open()
            }
        }

        // ============================================================
        // Secondary action row — Stop / Clear / AI assist + counter
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
            Controls.Button {
                id: aiToggle
                flat: true
                checkable: true
                icon.name: "tools-symbolic"
                text: qsTr("AI assist")
                display: Controls.AbstractButton.TextBesideIcon
                enabled: app.settingsController.aiSearchEnabled
                Controls.ToolTip.visible: hovered && !enabled
                Controls.ToolTip.text: qsTr("Enable AI in Settings → AI Search to use this panel.")
                onCheckedChanged: checked ? aiDrawer.open() : aiDrawer.close()
            }
            Item { Layout.fillWidth: true }
            // Live status pill — match count + files, with a soft
            // accent fill and animated counter shimmer. Mailspring
            // uses similar pills for unread/important counters.
            Rectangle {
                visible: page.controller.matchCount > 0
                radius: app.tokens.radiusPill
                color: app.tokens.accentMute
                border.color: Qt.rgba(app.tokens.accent.r,
                                      app.tokens.accent.g,
                                      app.tokens.accent.b, 0.45)
                border.width: 1
                implicitHeight: 26
                implicitWidth: matchCountLabel.implicitWidth + app.tokens.spaceL * 2 + 16
                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: app.tokens.spaceM
                    anchors.rightMargin: app.tokens.spaceL
                    spacing: app.tokens.spaceXS
                    Rectangle {
                        Layout.preferredWidth: 6
                        Layout.preferredHeight: 6
                        radius: 3
                        color: app.tokens.accent
                        Layout.alignment: Qt.AlignVCenter
                        SequentialAnimation on opacity {
                            running: page.controller.busy
                            loops: Animation.Infinite
                            NumberAnimation { from: 1; to: 0.4; duration: 700 }
                            NumberAnimation { from: 0.4; to: 1; duration: 700 }
                        }
                    }
                    Controls.Label {
                        id: matchCountLabel
                        Layout.alignment: Qt.AlignVCenter
                        text: qsTr("%1 matches · %2 files").arg(page.controller.matchCount).arg(page.controller.filesMatched)
                        font.pixelSize: app.tokens.textCaption + 1
                        font.weight: app.tokens.weightSemibold
                        color: app.tokens.accent
                    }
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
                // Suggestion chips use tilde paths — `start_search` runs
                // them through `expand_tilde` before constructing
                // `SearchOptions`, so `~/code` resolves to $HOME/code
                // on whichever machine is running the binary.
                chipsModel: [
                    { label: qsTr("~/code · TODO"),       path: "~/code",   term: "TODO" },
                    { label: qsTr("~ · fn .* test"),      path: "~",        term: "fn\\s+\\w+_test" },
                    { label: qsTr("/etc · password"),     path: "/etc",     term: "password" }
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
        // Status footer — slim, generous side padding, semantic
        // status pip on the left and monospace counters on the right.
        // ============================================================
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 32
            color: app.tokens.surfaceSidebar
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
                    // Soft pulsing halo while busy
                    Rectangle {
                        anchors.centerIn: parent
                        width: parent.width + 6
                        height: parent.height + 6
                        radius: width / 2
                        color: "transparent"
                        border.color: app.tokens.warning
                        border.width: 1
                        opacity: page.controller.busy ? 0.6 : 0
                        SequentialAnimation on opacity {
                            running: page.controller.busy
                            loops: Animation.Infinite
                            NumberAnimation { from: 0.6; to: 0; duration: 900 }
                            NumberAnimation { from: 0; to: 0.6; duration: 0 }
                        }
                    }
                }
                Controls.Label {
                    Layout.fillWidth: true
                    text: page.controller.statusText.length > 0 ? page.controller.statusText : qsTr("Ready")
                    font.pixelSize: app.tokens.textCaption + 1
                    font.family: app.tokens.sansFamily
                    opacity: 0.72
                    elide: Text.ElideMiddle
                }
                Controls.Label {
                    visible: page.controller.filesScanned > 0
                    text: qsTr("scanned %1").arg(page.controller.filesScanned)
                    font.pixelSize: app.tokens.textCaption
                    opacity: 0.5
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

    // AI assist drawer — slides in from the right when the user
    // clicks the "AI assist" toolbar button. Disabled (and the
    // button greyed out) when ai_search_enabled is false; that
    // toggle is the audited opt-in.
    Controls.Drawer {
        id: aiDrawer
        edge: Qt.RightEdge
        modal: false
        interactive: true
        dim: false
        width: Math.min(page.width * 0.4, 460)
        height: page.height
        // `opened`/`position` are FINAL on Controls.Drawer — drive
        // visibility via the parent's `open()`/`close()` calls
        // (wired through `aiToggle.onCheckedChanged`) and reflect
        // closure back to the toggle here.
        onClosed: aiToggle.checked = false
        onOpened: aiToggle.checked = true

        Rectangle {
            anchors.fill: parent
            color: app.tokens.surface1
            border.color: app.tokens.separator
            border.width: 1

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: app.tokens.spaceL
                spacing: app.tokens.spaceM

                RowLayout {
                    Layout.fillWidth: true
                    Controls.Label {
                        text: qsTr("AI assist")
                        font.pixelSize: app.tokens.textSubheading
                        font.weight: app.tokens.weightBold
                        Layout.fillWidth: true
                    }
                    Controls.Button {
                        flat: true
                        icon.name: "window-close-symbolic"
                        display: Controls.AbstractButton.IconOnly
                        onClicked: aiDrawer.close()
                    }
                }
                Controls.Label {
                    Layout.fillWidth: true
                    text: qsTr("Ask about the codebase. Your query is sent to the configured endpoint only when the panel is enabled in Settings.")
                    font.pixelSize: app.tokens.textCaption
                    opacity: 0.6
                    wrapMode: Text.WordWrap
                }
                AiChatPanel {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                }
            }
        }
    }
}
