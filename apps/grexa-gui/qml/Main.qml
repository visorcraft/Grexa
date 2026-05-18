// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Grexa GUI shell.
//
// Kirigami ApplicationWindow with a refined sidebar (grouped
// sections, 32px nav rows, version footer) and a page stack.
// Controllers are declared here once so every page reaches them
// via `app.tokens` / `app.searchController` etc.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.visorcraft.Grexa 1.0

Kirigami.ApplicationWindow {
    id: app
    title: qsTr("Grexa")
    width: 1180
    height: 760
    minimumWidth: 920
    minimumHeight: 600
    visible: true

    Component.onCompleted: {
        app.raise()
        app.requestActivate()
        // Surface a recovery dialog if the previous run left a
        // partial replace journal behind. Gated on the user opt-in
        // toggle in Settings → Replace.
        if (settingsController.replaceShowJournalOnStartup) {
            const j = searchController.residualJournalJson()
            if (j && j.length > 0) {
                try {
                    residualJournal.entry = JSON.parse(j)
                    residualJournal.open()
                } catch (e) {}
            }
        }
    }

    // ---- Shared singletons ----
    property alias tokens: tokens
    property alias searchController: searchController
    property alias settingsController: settingsController
    property alias regexController: regexController
    property alias aiController: aiController
    property string currentPageKey: "search"

    DesignTokens { id: tokens }

    // Render a Qt color as a CSS `rgba(R, G, B, A)` string. We avoid
    // `Qt.rgba(...).toString()` because it emits `#aarrggbb`, which
    // CSS doesn't reliably interpret — Qt's RichText subset and the
    // `<span style='background-color: …'>` path either treat it as
    // opaque RGB (dropping alpha) or as an invalid color.
    function rgbaCss(c) {
        const r = Math.round(c.r * 255)
        const g = Math.round(c.g * 255)
        const b = Math.round(c.b * 255)
        const a = c.a
        return "rgba(" + r + "," + g + "," + b + "," + a + ")"
    }

    SearchController { id: searchController }
    SettingsController {
        id: settingsController
        Component.onCompleted: reload()
    }
    RegexBuilderController { id: regexController }
    AiController {
        id: aiController
        // Pull endpoint + model + key state from settings on startup
        // so the chat panel doesn't first-launch with an empty endpoint.
        Component.onCompleted: reloadFromSettings()
    }

    pageStack.initialPage: searchPage
    pageStack.globalToolBar.style: Kirigami.ApplicationHeaderStyle.None

    // -- Global keyboard shortcuts -----------------------------------
    // F1 → About; Ctrl+, → Settings; Ctrl+1..4 jump to pages;
    // Esc cancels a running search; Ctrl+L focuses the search bar
    // (handled by the SearchPage); Ctrl+Q quits.
    //
    // `Shortcut` (not `Controls.Action`) is the correct primitive for
    // window-scoped keybindings — top-level `Action`s only fire when
    // they're attached to a Menu / ToolButton, which Kirigami's
    // `ApplicationWindow` doesn't expose for us at the root level.
    Shortcut {
        sequence: "F1"
        context: Qt.ApplicationShortcut
        onActivated: app.goTo("about")
    }
    Shortcut {
        sequences: [StandardKey.Preferences, "Ctrl+3"]
        context: Qt.ApplicationShortcut
        onActivated: app.goTo("settings")
    }
    Shortcut {
        sequence: "Ctrl+1"
        context: Qt.ApplicationShortcut
        onActivated: app.goTo("search")
    }
    Shortcut {
        sequence: "Ctrl+2"
        context: Qt.ApplicationShortcut
        onActivated: app.goTo("regex")
    }
    Shortcut {
        sequence: "Ctrl+4"
        context: Qt.ApplicationShortcut
        onActivated: app.goTo("about")
    }
    Shortcut {
        sequence: StandardKey.Cancel
        context: Qt.ApplicationShortcut
        onActivated: {
            if (app.searchController.busy) {
                app.searchController.cancel()
            }
        }
    }
    Shortcut {
        sequence: StandardKey.Quit
        context: Qt.ApplicationShortcut
        onActivated: Qt.quit()
    }
    // Ctrl+T / Ctrl+W are scoped to the application but only act
    // when the Search page is mounted — `currentItem` is the live
    // page instance. The functions below probe for the tab-API
    // surface that SearchPage exposes; missing on other pages
    // (Settings, Regex Builder, About).
    Shortcut {
        sequence: "Ctrl+T"
        context: Qt.ApplicationShortcut
        onActivated: {
            const p = app.pageStack.currentItem
            if (p && p.openNewTab) {
                app.goTo("search")
                p.openNewTab()
            } else {
                app.goTo("search")
            }
        }
    }
    Shortcut {
        sequence: "Ctrl+W"
        context: Qt.ApplicationShortcut
        onActivated: {
            const p = app.pageStack.currentItem
            if (p && p.closeTab && p.activeTab !== undefined) {
                p.closeTab(p.activeTab)
            }
        }
    }

    function goTo(key) {
        currentPageKey = key
        switch (key) {
            case "search":   app.pageStack.replace(searchPage); break
            case "regex":    app.pageStack.replace(regexPage); break
            case "history":  app.pageStack.replace(historyPage); break
            case "profiles": app.pageStack.replace(profilesPage); break
            case "settings": app.pageStack.replace(settingsPage); break
            case "about":    app.pageStack.replace(aboutPage); break
        }
    }

    // ---- Sidebar -------------------------------------------------
    // Mailspring-class chrome panel: distinct surface tint (cooler
    // than canvas), generous padding, ALL-CAPS micro-section labels,
    // and full-width pill rows for nav items. A hairline at the
    // right edge keeps the boundary clean without a hard divider.
    globalDrawer: Kirigami.OverlayDrawer {
        id: drawer
        edge: Qt.LeftEdge
        modal: false
        drawerOpen: true
        width: Kirigami.Units.gridUnit * 14
        handleVisible: false

        background: Rectangle {
            color: tokens.surfaceSidebar
            Rectangle {
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: 1
                color: tokens.separator
            }
        }

        contentItem: ColumnLayout {
            spacing: 0

            // -- App header — icon tile + wordmark + tagline.
            // The icon sits inside a soft rounded tile so it reads
            // as the app brand "lozenge" the way Mailspring's mark
            // sits in the top-left chrome.
            RowLayout {
                Layout.fillWidth: true
                Layout.preferredHeight: 64
                Layout.topMargin: tokens.spaceL
                Layout.leftMargin: tokens.spaceL
                Layout.rightMargin: tokens.spaceL
                Layout.bottomMargin: tokens.spaceL
                spacing: tokens.spaceM

                Rectangle {
                    Layout.preferredWidth: 40
                    Layout.preferredHeight: 40
                    radius: tokens.radiusAvatar
                    color: "transparent"
                    Image {
                        anchors.fill: parent
                        source: "qrc:/qt/qml/com/visorcraft/Grexa/resources/grexa.png"
                        sourceSize.width: 80
                        sourceSize.height: 80
                        smooth: true
                        mipmap: true
                    }
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 1
                    Controls.Label {
                        text: "Grexa"
                        font.pixelSize: tokens.textSubheading + 1
                        font.weight: tokens.weightBold
                        font.family: tokens.sansFamily
                        font.letterSpacing: -0.2
                    }
                    Controls.Label {
                        text: qsTr("Fast file search")
                        font.pixelSize: tokens.textCaption
                        opacity: 0.55
                    }
                }
            }

            // -- Workspace section
            Controls.Label {
                Layout.fillWidth: true
                Layout.leftMargin: tokens.spaceL
                Layout.rightMargin: tokens.spaceL
                Layout.topMargin: tokens.spaceS
                Layout.bottomMargin: tokens.spaceS
                text: qsTr("WORKSPACE")
                font.pixelSize: 10
                font.weight: tokens.weightSemibold
                font.letterSpacing: 1.6
                opacity: 0.5
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("Search")
                iconName: "edit-find-symbolic"
                active: app.currentPageKey === "search"
                onTriggered: app.goTo("search")
            }

            // -- Tools section
            Controls.Label {
                Layout.fillWidth: true
                Layout.leftMargin: tokens.spaceL
                Layout.rightMargin: tokens.spaceL
                Layout.topMargin: tokens.spaceXL
                Layout.bottomMargin: tokens.spaceS
                text: qsTr("TOOLS")
                font.pixelSize: 10
                font.weight: tokens.weightSemibold
                font.letterSpacing: 1.6
                opacity: 0.5
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("Regex Builder")
                iconName: "code-context-symbolic"
                active: app.currentPageKey === "regex"
                onTriggered: app.goTo("regex")
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("History")
                iconName: "history-symbolic"
                active: app.currentPageKey === "history"
                onTriggered: app.goTo("history")
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("Profiles")
                iconName: "document-save-symbolic"
                active: app.currentPageKey === "profiles"
                onTriggered: app.goTo("profiles")
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("Settings")
                iconName: "settings-configure-symbolic"
                active: app.currentPageKey === "settings"
                onTriggered: app.goTo("settings")
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("About")
                iconName: "help-about-symbolic"
                active: app.currentPageKey === "about"
                onTriggered: app.goTo("about")
            }

            Item { Layout.fillHeight: true; Layout.fillWidth: true }

            // -- Footer — version pill on the left, license badge
            // on the right. No hard divider; the breathing room
            // sells the separation.
            RowLayout {
                Layout.fillWidth: true
                Layout.leftMargin: tokens.spaceL
                Layout.rightMargin: tokens.spaceL
                Layout.bottomMargin: tokens.spaceL
                Layout.topMargin: tokens.spaceM
                spacing: tokens.spaceS

                Rectangle {
                    radius: tokens.radiusPill
                    color: tokens.surface1
                    border.color: tokens.separator
                    border.width: 1
                    implicitHeight: 22
                    implicitWidth: versionLabel.implicitWidth + tokens.spaceM * 2
                    Controls.Label {
                        id: versionLabel
                        anchors.centerIn: parent
                        text: "v" + Qt.application.version
                        font.pixelSize: tokens.textCaption
                        font.family: tokens.monoFamily
                        opacity: 0.7
                    }
                }
                Item { Layout.fillWidth: true }
                Controls.Label {
                    text: qsTr("GPL v3")
                    font.pixelSize: tokens.textCaption
                    font.family: tokens.sansFamily
                    opacity: 0.4
                }
            }
        }
    }

    Component { id: searchPage;   SearchPage {} }
    Component { id: regexPage;    RegexBuilderPage {} }
    Component { id: historyPage;  HistoryPage {} }
    Component { id: profilesPage; ProfilesPage {} }
    Component { id: settingsPage; SettingsPage {} }
    Component { id: aboutPage;    AboutPage {} }

    // Residual replace-journal recovery dialog. Shown at startup
    // when a previous run was killed mid-replace and the journal
    // wasn't cleaned up.
    Controls.Dialog {
        id: residualJournal
        modal: true
        title: qsTr("Interrupted replace from a previous run")
        standardButtons: Controls.Dialog.Discard | Controls.Dialog.Close
        property var entry: ({})
        width: Math.min(app.width * 0.6, 520)
        onDiscarded: {
            searchController.clearResidualJournal()
            close()
        }

        contentItem: ColumnLayout {
            spacing: tokens.spaceM
            Controls.Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: qsTr("Grexa found a residual replace journal at $XDG_STATE_HOME/grexa/replace-journal.json. The previous run rewrote some files before being interrupted.")
            }
            Controls.Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                font.family: tokens.monoFamily
                font.pixelSize: tokens.textCaption + 1
                opacity: 0.85
                text: {
                    if (!residualJournal.entry || !residualJournal.entry.root) return ""
                    const modified = (residualJournal.entry.modified_files || []).length
                    const failed = (residualJournal.entry.failed_files || []).length
                    return qsTr("root: %1\nterm: %2 → %3\nfiles modified: %4\nfailures: %5")
                        .arg(residualJournal.entry.root || "")
                        .arg(residualJournal.entry.search_term || "")
                        .arg(residualJournal.entry.replacement || "")
                        .arg(modified)
                        .arg(failed)
                }
            }
            Controls.Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                font.pixelSize: tokens.textCaption + 1
                opacity: 0.7
                text: qsTr("Click Discard to remove the journal, or Close to keep it for forensic review. The file is a JSON document you can inspect by hand.")
            }
        }
    }
}
