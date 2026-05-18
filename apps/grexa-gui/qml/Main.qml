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
    width: 1180
    height: 760
    minimumWidth: 920
    minimumHeight: 600
    visible: true

    // Cascade our per-theme palette through Kirigami's attached
    // Theme so Pages, Cards, Labels, and Controls pick up the
    // canvas / text / highlight colors. With `inherit: false` we
    // override the host palette; descendants then inherit OURS
    // unless they re-override locally. Themes 0/1/2 (System / Light
    // / Dark) skip the bg + text overrides so the host Kirigami
    // theme keeps owning the chrome.
    Kirigami.Theme.inherit: false
    Kirigami.Theme.colorSet: Kirigami.Theme.Window
    Kirigami.Theme.backgroundColor: tokens.surface0
    Kirigami.Theme.textColor: tokens.textPrimary
    Kirigami.Theme.highlightColor: tokens.accent
    Kirigami.Theme.highlightedTextColor: tokens.accentText
    color: tokens.surface0

    // QtQuick.Controls (TextField, ComboBox, CheckBox, SpinBox …)
    // paint their backgrounds from Qt's `palette`, NOT from
    // Kirigami.Theme. Without these overrides, inputs keep the host
    // theme's dark base color even on our Light surface, leaving
    // unreadable dark bars on a light canvas.
    palette.window:          tokens.surface0
    palette.windowText:      tokens.textPrimary
    palette.base:            tokens.surface1
    palette.alternateBase:   tokens.surface2
    palette.text:            tokens.textPrimary
    palette.button:          tokens.surface1
    palette.buttonText:      tokens.textPrimary
    palette.brightText:      tokens.accentText
    palette.highlight:       tokens.accent
    palette.highlightedText: tokens.accentText
    palette.toolTipBase:     tokens.surface2
    palette.toolTipText:     tokens.textPrimary
    palette.mid:             tokens.separator
    palette.midlight:        tokens.surface1
    palette.light:           tokens.surface2
    palette.dark:            tokens.surface0
    palette.shadow:          tokens.shadowFar
    palette.placeholderText: Qt.rgba(tokens.textPrimary.r,
                                     tokens.textPrimary.g,
                                     tokens.textPrimary.b, 0.55)

    Component.onCompleted: {
        app.raise()
        app.requestActivate()
        // GREXA_INITIAL_PAGE env var lets dev/QA jump to a specific
        // page on launch — used for screenshot validation across the
        // theme palette.
        const initial = Qt.application.arguments && Qt.application.arguments.length > 1
            ? Qt.application.arguments[1] : ""
        if (initial && ["search","regex","history","profiles","settings","about"].indexOf(initial) !== -1) {
            app.goTo(initial)
        }
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
    property alias hostTheme: hostTheme
    property alias searchController: searchController
    property alias settingsController: settingsController
    property alias regexController: regexController
    property alias aiController: aiController
    property string currentPageKey: "search"

    // Snapshot of the host Kirigami palette captured *before* our
    // overrides cascade through the window. DesignTokens reads its
    // System-theme fallback from here, not from Kirigami.Theme on
    // the window — otherwise the fallback chains back into our own
    // override and Qt severs the binding loop, leaving every
    // surface stuck on its initial value (the exact symptom the
    // user hits: "Light saved but reopen doesn't apply").
    // Snapshot of the host palette captured *before* our QML
    // overrides cascade. `Kirigami.Theme.inherit: false` would zero
    // every color (Theme has no values without inheritance), so we
    // read from Qt's application palette instead — that holds the
    // platform/KDE colors regardless of our window-level overrides.
    QtObject {
        id: hostTheme
        readonly property color background: Qt.application.palette
            ? Qt.application.palette.window : "#1A1A1A"
        readonly property color textColor: Qt.application.palette
            ? Qt.application.palette.windowText : "#F5F5F5"
        readonly property color highlight: Qt.application.palette
            ? Qt.application.palette.highlight : "#2D7FF9"
    }

    title: qsTr("Grexa")

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
    // Esc only intercepts when a search is in flight — otherwise it
    // falls through to Qt's default popup/dialog/drawer close
    // handling. Without this gate, an Esc with a drawer open does
    // nothing (the Shortcut consumes the event, finds `busy` is
    // false, and exits silently — making the drawer's Esc-to-close
    // contract feel broken).
    //
    // StandardKey.Cancel / .Quit each alias multiple platform
    // keystrokes (Esc on most platforms; Ctrl+Q vs Ctrl+W vs Cmd+Q
    // depending on the host). `sequences: [ ... ]` registers every
    // binding — using the singular `sequence:` only registers the
    // first and Qt logs a warning at startup.
    Shortcut {
        sequences: [StandardKey.Cancel]
        context: Qt.ApplicationShortcut
        enabled: app.searchController.busy
        onActivated: app.searchController.cancel()
    }
    Shortcut {
        sequences: [StandardKey.Quit]
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
        // Short-circuit: re-clicking the active nav item would
        // otherwise tear down + rebuild the current page and lose
        // form state (e.g. typed search term, scroll position).
        if (key === currentPageKey) return
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
    //
    // `collapsible: true` gives the built-in handle Kirigami renders
    // at the bottom edge — clicking it toggles the drawer between
    // its full width and an icon-only strip ("Open Sidebar" /
    // "Close Sidebar"). Section labels and the wordmark hide in the
    // collapsed state so the strip stays clean.
    globalDrawer: Kirigami.GlobalDrawer {
        id: drawer
        edge: Qt.LeftEdge
        modal: false
        drawerOpen: true
        collapsible: true
        collapsed: false
        // Explicit width binding — Kirigami's default
        // `width = collapsed ? collapsedSize : implicitWidth` is
        // broken by setting `width:` at all, so we drive both states
        // ourselves. Collapsed value matches LinSync's icon strip;
        // expanded uses our previous 14-gridUnit chrome width.
        width: drawer.isCollapsed ? Kirigami.Units.gridUnit * 3
                                  : Kirigami.Units.gridUnit * 14
        Behavior on width { NumberAnimation { duration: tokens.durationSnap; easing.type: Easing.OutCubic } }
        handleVisible: false

        readonly property bool isCollapsed: drawer.collapsible && drawer.collapsed

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
                Layout.leftMargin: drawer.isCollapsed ? 0 : tokens.spaceL
                Layout.rightMargin: drawer.isCollapsed ? 0 : tokens.spaceL
                Layout.bottomMargin: tokens.spaceL
                spacing: tokens.spaceM

                // Hamburger toggle — collapses / expands the sidebar.
                // `globalToolBar.style: None` suppresses Kirigami's
                // own header hamburger, so we host one here instead.
                // Sits at the leftmost edge of the header so it stays
                // anchored in the same spot when the sidebar collapses;
                // centers in the narrow strip via the row's margins.
                Controls.ToolButton {
                    Layout.alignment: drawer.isCollapsed
                                          ? Qt.AlignHCenter | Qt.AlignVCenter
                                          : Qt.AlignVCenter
                    Layout.fillWidth: drawer.isCollapsed
                    icon.name: "application-menu-symbolic"
                    icon.color: tokens.textPrimary
                    display: Controls.AbstractButton.IconOnly
                    Controls.ToolTip.text: drawer.isCollapsed ? qsTr("Open Sidebar")
                                                              : qsTr("Close Sidebar")
                    Controls.ToolTip.visible: hovered
                    Controls.ToolTip.delay: 400
                    Accessible.name: Controls.ToolTip.text
                    onClicked: drawer.collapsed = !drawer.collapsed
                }
                Rectangle {
                    Layout.preferredWidth: 40
                    Layout.preferredHeight: 40
                    radius: tokens.radiusAvatar
                    color: "transparent"
                    visible: !drawer.isCollapsed
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
                    visible: !drawer.isCollapsed
                    Controls.Label {
                        text: "Grexa"
                        font.pixelSize: tokens.textSubheading + 1
                        font.weight: tokens.weightBold
                        font.family: tokens.sansFamily
                        font.letterSpacing: -0.2
                        color: tokens.textPrimary
                    }
                    Controls.Label {
                        text: qsTr("Fast file search")
                        font.pixelSize: tokens.textCaption
                        opacity: 0.55
                        color: tokens.textPrimary
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
                visible: !drawer.isCollapsed
                color: tokens.textPrimary
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("Search")
                iconName: "edit-find-symbolic"
                active: app.currentPageKey === "search"
                compact: drawer.isCollapsed
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
                visible: !drawer.isCollapsed
                color: tokens.textPrimary
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("Regex Builder")
                iconName: "code-context-symbolic"
                active: app.currentPageKey === "regex"
                compact: drawer.isCollapsed
                onTriggered: app.goTo("regex")
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("History")
                iconName: "history-symbolic"
                active: app.currentPageKey === "history"
                compact: drawer.isCollapsed
                onTriggered: app.goTo("history")
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("Profiles")
                iconName: "document-save-symbolic"
                active: app.currentPageKey === "profiles"
                compact: drawer.isCollapsed
                onTriggered: app.goTo("profiles")
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("Settings")
                iconName: "settings-configure-symbolic"
                active: app.currentPageKey === "settings"
                compact: drawer.isCollapsed
                onTriggered: app.goTo("settings")
            }
            NavItem {
                Layout.fillWidth: true
                label: qsTr("About")
                iconName: "help-about-symbolic"
                active: app.currentPageKey === "about"
                compact: drawer.isCollapsed
                onTriggered: app.goTo("about")
            }

            Item { Layout.fillHeight: true; Layout.fillWidth: true }

            // -- Footer — version pill on the left, license badge
            // on the right. Hidden when the sidebar is collapsed
            // so the icon strip stays clean.
            RowLayout {
                Layout.fillWidth: true
                Layout.leftMargin: tokens.spaceL
                Layout.rightMargin: tokens.spaceL
                Layout.bottomMargin: tokens.spaceL
                Layout.topMargin: tokens.spaceM
                spacing: tokens.spaceS
                visible: !drawer.isCollapsed

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
                        color: tokens.textPrimary
                    }
                }
                Item { Layout.fillWidth: true }
                Controls.Label {
                    text: qsTr("GPL v3")
                    font.pixelSize: tokens.textCaption
                    font.family: tokens.sansFamily
                    opacity: 0.4
                    color: tokens.textPrimary
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
