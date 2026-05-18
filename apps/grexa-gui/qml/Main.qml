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
    Controls.Action {
        shortcut: "F1"
        onTriggered: app.goTo("about")
    }
    Controls.Action {
        shortcut: StandardKey.Preferences
        onTriggered: app.goTo("settings")
    }
    Controls.Action {
        shortcut: "Ctrl+1"
        onTriggered: app.goTo("search")
    }
    Controls.Action {
        shortcut: "Ctrl+2"
        onTriggered: app.goTo("regex")
    }
    Controls.Action {
        shortcut: "Ctrl+3"
        onTriggered: app.goTo("settings")
    }
    Controls.Action {
        shortcut: "Ctrl+4"
        onTriggered: app.goTo("about")
    }
    Controls.Action {
        shortcut: StandardKey.Cancel
        onTriggered: {
            if (app.searchController.busy) {
                app.searchController.cancel()
            }
        }
    }
    Controls.Action {
        shortcut: StandardKey.Quit
        onTriggered: Qt.quit()
    }

    function goTo(key) {
        currentPageKey = key
        switch (key) {
            case "search":   app.pageStack.replace(searchPage); break
            case "regex":    app.pageStack.replace(regexPage); break
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
    Component { id: settingsPage; SettingsPage {} }
    Component { id: aboutPage;    AboutPage {} }
}
