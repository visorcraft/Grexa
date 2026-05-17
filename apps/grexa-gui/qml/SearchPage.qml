// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Search workspace — the primary tab.
//
// Bound to `app.searchController` (declared in Main.qml). The
// controller drives:
//   - status_text / match_count / busy / recent_path_count properties
//   - the row list (it IS the model — QAbstractListModel subclass)
//   - start_search / cancel / clear_results invokables
//   - history_changed / search_completed signals

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.Page {
    id: page
    title: i18n("Search")
    padding: Kirigami.Units.smallSpacing

    property var controller: app.searchController

    Connections {
        target: controller
        function onHistoryChanged() {
            page.refreshRecentPaths()
        }
        function onSearchCompleted(cancelled) {
            // Re-enable stop button explicitly even though `busy` flips
            // — QML's binding sees it but a fresh signal makes it
            // obvious in event logs.
            stopButton.enabled = false
        }
    }

    Component.onCompleted: refreshRecentPaths()

    function refreshRecentPaths() {
        recentPaths.clear()
        try {
            const arr = JSON.parse(controller.recentPathsJson())
            for (let i = 0; i < arr.length; ++i) {
                recentPaths.append({ pathText: arr[i] })
            }
        } catch (e) {
            // controller returned something that wasn't an array; leave the
            // model empty
        }
    }

    function launchSearch() {
        const path = pathInput.editText
        const term = termInput.text
        if (path.length === 0 || term.length === 0) return
        controller.startSearch(path, term, regexToggle.checked, caseToggle.checked, false)
    }

    header: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing

        TabBar {
            id: tabBar
            Layout.fillWidth: true
            TabButton {
                text: i18n("Search")
                width: Math.max(120, contentItem.implicitWidth + 16)
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            ComboBox {
                id: pathInput
                Layout.fillWidth: true
                editable: true
                textRole: "pathText"
                model: ListModel { id: recentPaths }
                Keys.onReturnPressed: page.launchSearch()
            }
            Button {
                icon.name: "folder-open"
                text: i18n("Browse")
                onClicked: browseDialog.open()
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            TextField {
                id: termInput
                Layout.fillWidth: true
                placeholderText: i18n("Search term")
                Keys.onReturnPressed: page.launchSearch()
            }
            ToolButton {
                id: regexToggle
                checkable: true
                icon.name: "code-context"
                text: i18n("Regex")
                ToolTip.text: i18n("Treat the search term as a regular expression (PCRE-style)")
                ToolTip.visible: hovered
            }
            ToolButton {
                id: caseToggle
                checkable: true
                icon.name: "format-text-italic"
                text: i18n("Aa")
                ToolTip.text: i18n("Case-sensitive match")
                ToolTip.visible: hovered
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            Button {
                id: searchButton
                icon.name: "edit-find"
                text: i18n("Search")
                enabled: !controller.busy
                onClicked: page.launchSearch()
            }
            Button {
                id: stopButton
                icon.name: "process-stop"
                text: i18n("Stop")
                enabled: controller.busy
                onClicked: controller.cancel()
            }
            Button {
                icon.name: "edit-clear"
                text: i18n("Clear")
                enabled: !controller.busy && controller.matchCount > 0
                onClicked: controller.clearResults()
            }
            Item { Layout.fillWidth: true }
            Label {
                text: i18n("%1 matches in %2 files", controller.matchCount, controller.filesMatched)
                visible: controller.matchCount > 0
                color: Kirigami.Theme.disabledTextColor
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Label {
                id: statusLabel
                text: controller.statusText.length > 0 ? controller.statusText : i18n("Ready")
                Layout.fillWidth: true
                color: controller.busy ? Kirigami.Theme.activeTextColor : Kirigami.Theme.textColor
            }
            BusyIndicator {
                running: controller.busy
                visible: controller.busy
                implicitHeight: statusLabel.implicitHeight
                implicitWidth: implicitHeight
            }
        }
    }

    Frame {
        anchors.fill: parent

        ListView {
            id: resultList
            anchors.fill: parent
            clip: true
            model: page.controller
            spacing: 2

            delegate: ItemDelegate {
                width: ListView.view ? ListView.view.width : 0
                ColumnLayout {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.margins: Kirigami.Units.smallSpacing
                    spacing: 2
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Kirigami.Units.smallSpacing
                        Label {
                            text: model.relativePath
                            font.weight: Font.Bold
                            elide: Text.ElideMiddle
                            Layout.fillWidth: true
                        }
                        Label {
                            text: model.line + ":" + model.column
                            color: Kirigami.Theme.disabledTextColor
                        }
                    }
                    Label {
                        text: model.previewBefore + " ⟨" + model.previewMatch + "⟩ " + model.previewAfter
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                        color: Kirigami.Theme.disabledTextColor
                    }
                }
                onClicked: {
                    contextPreview.path = page.controller.rowFullPath(index)
                    contextPreview.lineNumber = model.line
                    contextPreview.open()
                }
            }

            Kirigami.PlaceholderMessage {
                anchors.centerIn: parent
                visible: parent.count === 0 && !page.controller.busy
                text: i18n("Run a search to see results.")
                explanation: i18n("Type a path, type a term, press Enter.")
            }
        }
    }

    Dialog {
        id: browseDialog
        modal: true
        title: i18n("Choose folder")
        standardButtons: Dialog.Cancel
        Label {
            text: i18n("The portal file picker integration lands in Phase 5. Type the path directly for now.")
            wrapMode: Text.WordWrap
        }
    }

    ContextPreviewDialog {
        id: contextPreview
    }
}
