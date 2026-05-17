// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Search workspace — the primary tab. The structure here matches the
// Phase 4 Search UI MVP contract; the Rust side hands it real data via
// the cxx-qt bindings landing in a follow-up PR. This file is
// validated by qmllint and visually reviewed; the controller logic it
// drives is unit-tested in apps/grexa-gui/src/{tab,workspace}.rs.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.Page {
    title: i18n("Search")
    padding: Kirigami.Units.smallSpacing

    // --- Top input area --------------------------------------------------
    header: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing

        // Tab strip
        TabBar {
            id: tabBar
            Layout.fillWidth: true
            TabButton {
                text: i18n("New tab")
                width: Math.max(120, contentItem.implicitWidth + 16)
            }
        }

        // Path + term row
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            ComboBox {
                id: pathInput
                Layout.fillWidth: true
                editable: true
                model: ListModel { id: recentPaths }
                placeholderText: i18n("Path to search")
                // Enter applies the picker selection; Down arrow opens
                // the recent-paths AutoSuggest list (populated from the
                // Rust `RecentPathStore`).
                Keys.onReturnPressed: searchButton.clicked()
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
                Keys.onReturnPressed: searchButton.clicked()
            }
            ToolButton {
                checkable: true
                icon.name: "code-context"
                text: i18n("Regex")
                ToolTip.text: i18n("Treat the search term as a regular expression (PCRE-style)")
                ToolTip.visible: hovered
            }
            ToolButton {
                checkable: true
                icon.name: "format-text-italic"
                text: i18n("Aa")
                ToolTip.text: i18n("Case-sensitive match")
                ToolTip.visible: hovered
            }
        }

        RowLayout {
            id: replacementRow
            Layout.fillWidth: true
            visible: false  // toggled by the Replace button below
            spacing: Kirigami.Units.smallSpacing

            TextField {
                Layout.fillWidth: true
                placeholderText: i18n("Replacement (use $1, $name for regex captures)")
                Keys.onReturnPressed: replaceButton.clicked()
            }
        }

        // Command strip
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            Button { id: searchButton; icon.name: "edit-find"; text: i18n("Search") }
            Button { id: stopButton; icon.name: "process-stop"; text: i18n("Stop"); enabled: false }
            Button { id: aiButton; icon.name: "tools-symbolic"; text: i18n("AI") }
            Button { id: replaceButton; icon.name: "edit-find-replace"; text: i18n("Replace"); checkable: true }
            Button { icon.name: "view-refresh"; text: i18n("Reset") }
            Button { id: filterButton; icon.name: "filter-symbolic"; text: i18n("Filter Options"); checkable: true }
            Button { icon.name: "favorites"; text: i18n("Profiles") }
            Button { icon.name: "view-history"; text: i18n("History") }
            Button { icon.name: "document-export"; text: i18n("Export") }
        }

        // Filter pane (collapsible)
        GroupBox {
            id: filterPane
            Layout.fillWidth: true
            visible: filterButton.checked
            title: i18n("Filter options")

            GridLayout {
                columns: 3
                rowSpacing: Kirigami.Units.smallSpacing
                columnSpacing: Kirigami.Units.largeSpacing

                CheckBox { text: i18n("Respect .gitignore") }
                CheckBox { text: i18n("Include hidden") }
                CheckBox { text: i18n("Include system") }
                CheckBox { text: i18n("Include subfolders"); checked: true }
                CheckBox { text: i18n("Include binary/docs") }
                CheckBox { text: i18n("Follow symlinks") }

                Label { text: i18n("Match files:") }
                TextField {
                    Layout.columnSpan: 2
                    Layout.fillWidth: true
                    placeholderText: i18n("e.g. *.rs|*.toml|-target*")
                }

                Label { text: i18n("Exclude dirs:") }
                TextField {
                    Layout.columnSpan: 2
                    Layout.fillWidth: true
                    placeholderText: i18n("comma/semicolon separated; or regex")
                }

                Label { text: i18n("Size limit:") }
                ComboBox { model: [i18n("none"), i18n("less"), i18n("equal"), i18n("greater")] }
                RowLayout {
                    SpinBox {}
                    ComboBox { model: ["KB", "MB", "GB"] }
                }
            }
        }

        // Status bar
        RowLayout {
            Layout.fillWidth: true
            Label {
                id: statusLabel
                text: i18n("Ready")
                Layout.fillWidth: true
            }
            Label {
                id: elapsedLabel
                color: Kirigami.Theme.disabledTextColor
            }
        }
    }

    // --- Results area ----------------------------------------------------
    ColumnLayout {
        anchors.fill: parent
        spacing: Kirigami.Units.smallSpacing

        // Result mode + search-within
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            ButtonGroup { id: resultModeGroup }
            RadioButton { ButtonGroup.group: resultModeGroup; text: i18n("Content"); checked: true }
            RadioButton { ButtonGroup.group: resultModeGroup; text: i18n("Files") }

            Item { Layout.fillWidth: true }

            TextField {
                placeholderText: i18n("Search within results")
                Layout.preferredWidth: 240
            }
            ToolButton { checkable: true; text: ".*"; ToolTip.text: i18n("Regex"); ToolTip.visible: hovered }
        }

        // Virtualized result list — placeholder TableView with the
        // columns Phase 4 calls for. The model lands when cxx-qt wires
        // `TabState.view` into a QAbstractTableModel.
        Frame {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ListView {
                anchors.fill: parent
                clip: true
                model: ListModel { id: resultsModel }
                delegate: Kirigami.SubtitleDelegate {
                    text: model.fileName ?? ""
                    subtitle: model.snippet ?? ""
                }

                Kirigami.PlaceholderMessage {
                    anchors.centerIn: parent
                    visible: resultsModel.count === 0
                    text: i18n("Run a search to see results.")
                    explanation: i18n("Type a path, type a term, press Enter.")
                }
            }
        }
    }

    // --- Folder picker dialog -------------------------------------------
    Dialog {
        id: browseDialog
        modal: true
        title: i18n("Choose folder")
        standardButtons: Dialog.Open | Dialog.Cancel

        Label {
            text: i18n("Use the system file picker (portal) to choose a folder.")
            wrapMode: Text.WordWrap
        }

        onAccepted: {
            // Rust hook: emit `chooseFolder()` to controller; the result
            // populates `pathInput.editText`.
        }
    }
}
