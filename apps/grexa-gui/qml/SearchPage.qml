// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Search workspace. Unified address-bar style SearchBar at the top
// (path + term + flags + primary action in one card), result list
// below, status footer pinned to the bottom.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs as Dialogs
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
    ListModel { id: containersModel }
    property var runtimesList: []

    function refreshContainers() {
        // Kick off the off-thread probe — result lands on
        // controller.containersJson via the `containersJsonChanged`
        // signal, which the Connections{} block below handles.
        page.controller.refreshContainers()
    }

    // Populate the model whenever the controller's cached JSON
    // updates. This runs on the GUI thread after the worker has
    // already done the slow `docker ps` / `podman ps` work.
    function applyContainersJson() {
        containersModel.clear()
        runtimesList = []
        const raw = page.controller.containersJson
        if (!raw || raw.length === 0) return
        try {
            const data = JSON.parse(raw)
            runtimesList = data.runtimes || []
            const containers = data.containers || []
            for (let i = 0; i < containers.length; ++i) {
                const c = containers[i]
                containersModel.append({
                    kind: c.kind,
                    containerId: c.id,
                    label: c.name + " · " + c.image + " (" + c.status + ")"
                })
            }
        } catch (e) {}
    }

    Connections {
        target: page.controller
        function onContainersJsonChanged() {
            page.applyContainersJson()
            if (targetSelector) targetSelector.rebuildTargetModel()
        }
    }

    // Re-apply the view rules whenever the search-within filter or
    // result-mode toggle changes. The controller re-projects
    // `rows` → `visible` and emits a model reset.
    function refreshView() { page.controller.refreshView() }

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
        // Action toolbar — target selector, mode toggle, filter +
        // replace buttons, Stop / Clear / AI assist + counter pill.
        // ============================================================
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceM
            Layout.bottomMargin: app.tokens.spaceS
            spacing: app.tokens.spaceS

            // Target selector: Local vs Docker vs Podman containers.
            // Reads from `controller.containersJson()` on demand and
            // populates `containersModel`. Gated on the Settings
            // toggle so the dropdown stays minimal when the user
            // hasn't opted into container search.
            Controls.ComboBox {
                id: targetSelector
                Layout.preferredWidth: 220
                model: ListModel {
                    id: targetModel
                    ListElement { label: "Local files";    kind: 0; containerId: "" }
                }
                textRole: "label"
                Component.onCompleted: rebuildTargetModel()
                onActivated: {
                    page.controller.targetKind = targetModel.get(currentIndex).kind
                    page.controller.selectedContainerId = targetModel.get(currentIndex).containerId
                }
                function rebuildTargetModel() {
                    const prevKind = page.controller.targetKind
                    const prevId = page.controller.selectedContainerId
                    targetModel.clear()
                    targetModel.append({ label: qsTr("Local files"), kind: 0, containerId: "" })
                    if (app.settingsController.enableContainerSearch) {
                        page.refreshContainers()
                        for (let i = 0; i < containersModel.count; ++i) {
                            const c = containersModel.get(i)
                            targetModel.append({
                                label: kindLabel(c.kind) + " · " + c.label,
                                kind: c.kind,
                                containerId: c.containerId
                            })
                        }
                    }
                    // Restore the previous selection where possible.
                    for (let j = 0; j < targetModel.count; ++j) {
                        const m = targetModel.get(j)
                        if (m.kind === prevKind && m.containerId === prevId) {
                            currentIndex = j
                            return
                        }
                    }
                    currentIndex = 0
                    page.controller.targetKind = 0
                    page.controller.selectedContainerId = ""
                }
                function kindLabel(k) {
                    return k === 1 ? qsTr("Docker")
                         : k === 2 ? qsTr("Podman rootless")
                         : k === 3 ? qsTr("Podman rootful")
                                   : qsTr("Container")
                }
                // Re-list when the user flips the container toggle.
                Connections {
                    target: app.settingsController
                    function onEnableContainerSearchChanged() { targetSelector.rebuildTargetModel() }
                }
            }

            // Result mode segmented toggle — Content vs Files. The
            // model dedupes when `result_mode == 1`.
            Controls.ButtonGroup { id: modeGroup; exclusive: true }
            Controls.Button {
                Layout.preferredWidth: 96
                Controls.ButtonGroup.group: modeGroup
                checkable: true
                checked: page.controller.resultMode === 0
                text: qsTr("Content")
                onClicked: {
                    page.controller.resultMode = 0
                    page.refreshView()
                }
            }
            Controls.Button {
                Layout.preferredWidth: 72
                Controls.ButtonGroup.group: modeGroup
                checkable: true
                checked: page.controller.resultMode === 1
                text: qsTr("Files")
                onClicked: {
                    page.controller.resultMode = 1
                    page.refreshView()
                }
            }

            // Filter button → opens the filter drawer.
            Controls.Button {
                flat: true
                icon.name: "view-filter-symbolic"
                text: qsTr("Filters")
                display: Controls.AbstractButton.TextBesideIcon
                onClicked: filterDrawer.open()
            }

            // Replace button → opens the replace dialog. Disabled for
            // container targets and before any search has run.
            Controls.Button {
                flat: true
                icon.name: "edit-find-replace-symbolic"
                text: qsTr("Replace…")
                display: Controls.AbstractButton.TextBesideIcon
                enabled: page.controller.targetKind === 0 && page.controller.hasSearched && !page.controller.busy
                onClicked: replaceDialog.open()
            }

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
        // Within-filter row — narrows the visible result rows by a
        // substring or regex against the preview line. Wired to the
        // controller's `within_filter` / `within_regex` qproperties.
        // ============================================================
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.bottomMargin: app.tokens.spaceS
            spacing: app.tokens.spaceS
            visible: page.controller.matchCount > 0 || page.controller.withinFilter.length > 0

            Kirigami.Icon {
                source: "view-filter-symbolic"
                implicitWidth: 14
                implicitHeight: 14
                isMask: true
                color: Kirigami.Theme.textColor
                opacity: 0.5
            }
            Controls.TextField {
                id: withinField
                Layout.fillWidth: true
                placeholderText: qsTr("Filter results — substring or regex")
                text: page.controller.withinFilter
                onTextEdited: {
                    page.controller.withinFilter = text
                    page.refreshView()
                }
            }
            Controls.CheckBox {
                text: qsTr("regex")
                checked: page.controller.withinRegex
                onToggled: {
                    page.controller.withinRegex = checked
                    page.refreshView()
                }
            }
            Controls.Button {
                flat: true
                icon.name: "edit-clear-symbolic"
                display: Controls.AbstractButton.IconOnly
                Controls.ToolTip.text: qsTr("Clear filter")
                Controls.ToolTip.visible: hovered
                enabled: page.controller.withinFilter.length > 0
                onClicked: {
                    page.controller.withinFilter = ""
                    page.refreshView()
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
                focus: true
                keyNavigationEnabled: true
                keyNavigationWraps: false

                // Space on the focused row opens the context preview —
                // mirrors Grex's keyboard-friendly result navigation.
                Keys.onPressed: function(event) {
                    if (event.key === Qt.Key_Space && currentIndex >= 0) {
                        const path = page.controller.rowFullPath(currentIndex)
                        const lineRaw = model.data(model.index(currentIndex, 0), 0x0102) || "0"
                        contextPreview.path = path
                        contextPreview.lineNumber = parseInt(String(lineRaw), 10)
                        contextPreview.open()
                        event.accepted = true
                    }
                }

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
                    fullPath: page.controller.rowFullPath(index)
                    onOpenPreview: {
                        contextPreview.path = page.controller.rowFullPath(index)
                        contextPreview.lineNumber = parseInt(model.line, 10)
                        contextPreview.open()
                    }
                }
            }

            // "Haven't searched yet" — show example chips so the user
            // has a one-click way to validate the install.
            EmptyState {
                anchors.centerIn: parent
                width: parent.width
                height: parent.height
                visible: resultList.count === 0 && !page.controller.busy && !page.controller.hasSearched
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

            // "Searched, no matches" — distinct copy + offer to widen.
            Kirigami.PlaceholderMessage {
                anchors.centerIn: parent
                width: parent.width - 2 * app.tokens.spaceXL
                visible: resultList.count === 0 && !page.controller.busy && page.controller.hasSearched
                icon.name: "edit-find-symbolic"
                text: qsTr("No matches found")
                explanation: page.controller.withinFilter.length > 0
                    ? qsTr("The result filter '%1' hid every row. Clear it to see the raw matches, or widen the search.").arg(page.controller.withinFilter)
                    : qsTr("Try a shorter term, drop a filter, or pick a broader folder. Hidden files, gitignored paths, and binary content are excluded by default — flip those toggles in the Filters drawer.")
                helpfulAction: Kirigami.Action {
                    text: qsTr("Open Filters")
                    icon.name: "view-filter-symbolic"
                    onTriggered: filterDrawer.open()
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

    // Native folder picker. On KDE Plasma this is the Breeze chooser;
    // on Wayland under Flatpak it routes through the XDG portal
    // automatically. We feed the dialog an already-encoded
    // `file://` URL so paths with spaces, accents, or tilde
    // expansion don't fail at the portal layer.
    function pathToFileUrl(path) {
        if (!path || path.length === 0) return ""
        // Tilde + relative paths get resolved Rust-side; this is a
        // best-effort GUI prepass so the dialog opens *somewhere*
        // sensible rather than rejecting `~/code` outright.
        if (path.charAt(0) !== "/") return ""
        // `encodeURI` preserves "/" but escapes spaces, accents,
        // and other reserved characters per RFC 3986.
        return "file://" + encodeURI(path)
    }

    Dialogs.FolderDialog {
        id: browseDialog
        title: qsTr("Choose folder")
        currentFolder: page.pathToFileUrl(searchBar.pathText)
        onAccepted: {
            // `selectedFolder` is a `url`. Convert to a string and
            // run it through `decodeURIComponent` so percent-encoded
            // characters (e.g. `My%20Code`) come back as literal text.
            const u = selectedFolder.toString()
            let decoded = u.replace(/^file:\/\//, "")
            try { decoded = decodeURIComponent(decoded) } catch (e) {}
            searchBar.pathText = decoded
            page.controller.addRecentPath(decoded)
        }
    }

    // -----------------------------------------------------------------
    // Filter drawer — per-search overrides bound to SettingsController.
    // Toggling here also updates the persisted defaults for the next
    // session, matching how Grex treats its filter pane.
    // -----------------------------------------------------------------
    Controls.Drawer {
        id: filterDrawer
        edge: Qt.RightEdge
        modal: false
        interactive: true
        dim: false
        width: Math.min(page.width * 0.42, 480)
        height: page.height

        Rectangle {
            anchors.fill: parent
            color: app.tokens.surface1
            border.color: app.tokens.separator
            border.width: 1

            Controls.ScrollView {
                anchors.fill: parent
                anchors.margins: app.tokens.spaceL
                clip: true

                ColumnLayout {
                    width: filterDrawer.width - app.tokens.spaceL * 2
                    spacing: app.tokens.spaceM

                    RowLayout {
                        Layout.fillWidth: true
                        Controls.Label {
                            text: qsTr("Filters")
                            font.pixelSize: app.tokens.textSubheading
                            font.weight: app.tokens.weightBold
                            Layout.fillWidth: true
                        }
                        Controls.Button {
                            flat: true
                            icon.name: "window-close-symbolic"
                            display: Controls.AbstractButton.IconOnly
                            onClicked: filterDrawer.close()
                        }
                    }
                    Controls.Label {
                        Layout.fillWidth: true
                        text: qsTr("Changes apply to the next search and also persist as defaults for new sessions.")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.65
                        wrapMode: Text.WordWrap
                    }

                    Controls.CheckBox {
                        text: qsTr("Respect .gitignore")
                        checked: app.settingsController.respectGitignore
                        onToggled: { app.settingsController.respectGitignore = checked; app.settingsController.apply() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Include hidden files (dotfiles)")
                        checked: app.settingsController.includeHidden
                        onToggled: { app.settingsController.includeHidden = checked; app.settingsController.apply() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Include binary / extracted docs")
                        checked: app.settingsController.includeBinary
                        onToggled: { app.settingsController.includeBinary = checked; app.settingsController.apply() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Include system files")
                        checked: app.settingsController.includeSystemFiles
                        onToggled: { app.settingsController.includeSystemFiles = checked; app.settingsController.apply() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Include subfolders (recursive)")
                        checked: app.settingsController.includeSubfolders
                        onToggled: { app.settingsController.includeSubfolders = checked; app.settingsController.apply() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Follow symbolic links")
                        checked: app.settingsController.includeSymbolicLinks
                        onToggled: { app.settingsController.includeSymbolicLinks = checked; app.settingsController.apply() }
                    }

                    Controls.Label {
                        text: qsTr("Match file names")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.65
                        Layout.topMargin: app.tokens.spaceS
                    }
                    Controls.TextField {
                        Layout.fillWidth: true
                        placeholderText: "*.rs|*.toml|-target*"
                        text: app.settingsController.defaultMatchFiles
                        onEditingFinished: {
                            app.settingsController.defaultMatchFiles = text
                            app.settingsController.apply()
                        }
                    }

                    Controls.Label {
                        text: qsTr("Exclude directories")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.65
                    }
                    Controls.TextField {
                        Layout.fillWidth: true
                        placeholderText: "node_modules, target, .venv"
                        text: app.settingsController.defaultExcludeDirs
                        onEditingFinished: {
                            app.settingsController.defaultExcludeDirs = text
                            app.settingsController.apply()
                        }
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // Replace dialog — confirmation + irreversible warning. Reads the
    // controller's `last_path` + `last_term` indirectly via the
    // currently-displayed result count.
    // -----------------------------------------------------------------
    Controls.Dialog {
        id: replaceDialog
        modal: true
        title: qsTr("Replace matches")
        standardButtons: Controls.Dialog.Cancel
        width: Math.min(page.width * 0.6, 520)

        Connections {
            target: page.controller
            function onReplaceCompleted(success) {
                if (success) {
                    replaceDialog.close()
                    replaceSummaryDialog.open()
                }
            }
        }

        contentItem: ColumnLayout {
            spacing: app.tokens.spaceM

            Controls.Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: qsTr("Replace every match in %1 files. The original files are rewritten in place — there is no undo.").arg(page.controller.filesMatched)
            }

            Controls.Label {
                text: qsTr("Replacement")
                font.pixelSize: app.tokens.textCaption
                opacity: 0.65
            }
            Controls.TextField {
                id: replacementField
                Layout.fillWidth: true
                placeholderText: qsTr("Replacement text (regex captures: $1, ${name})")
            }

            Controls.Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                font.pixelSize: app.tokens.textCaption
                opacity: 0.6
                text: qsTr("A journal of rewritten files lives at $XDG_STATE_HOME/grexa/replace-journal.json until grexa exits cleanly.")
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: app.tokens.spaceS
                Item { Layout.fillWidth: true }
                Controls.Button {
                    text: qsTr("Cancel")
                    onClicked: replaceDialog.close()
                }
                PrimaryButton {
                    text: page.controller.replacing ? qsTr("Replacing…") : qsTr("Replace All")
                    icon.name: "edit-find-replace-symbolic"
                    enabled: !page.controller.replacing && replacementField.text.length > 0
                    onClicked: page.controller.startReplace(replacementField.text)
                }
            }
        }
    }

    // Result dialog shown when replace completes successfully.
    Controls.Dialog {
        id: replaceSummaryDialog
        modal: true
        title: qsTr("Replace complete")
        standardButtons: Controls.Dialog.Ok
        Controls.Label {
            // Strip the JSON wrapper and surface the counts as plain
            // text. `last_replace_summary` is a JSON object; parse it
            // and pull the numeric fields.
            text: {
                try {
                    const r = JSON.parse(page.controller.lastReplaceSummary || "{}")
                    return qsTr("Modified %1 files · %2 matches replaced · %3 failures · %4 ms")
                        .arg(r.files_modified || 0)
                        .arg(r.matches_replaced || 0)
                        .arg(r.failure_count || 0)
                        .arg(r.elapsed_ms || 0)
                } catch (e) {
                    return qsTr("Replace finished.")
                }
            }
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
