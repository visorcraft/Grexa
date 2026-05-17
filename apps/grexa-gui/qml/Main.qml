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

    SearchController { id: searchController }
    SettingsController {
        id: settingsController
        Component.onCompleted: reload()
    }
    RegexBuilderController { id: regexController }
    AiController {
        id: aiController
        Component.onCompleted: refreshKeyState()
    }

    pageStack.initialPage: searchPage
    pageStack.globalToolBar.style: Kirigami.ApplicationHeaderStyle.None

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
    globalDrawer: Kirigami.OverlayDrawer {
        id: drawer
        edge: Qt.LeftEdge
        modal: false
        drawerOpen: true
        width: Kirigami.Units.gridUnit * 13
        handleVisible: false

        background: Rectangle {
            // Subtle vertical gradient — top tint, bottom matches the
            // page background so it doesn't visually divide too hard.
            gradient: Gradient {
                GradientStop { position: 0.0; color: tokens.surface2 }
                GradientStop { position: 1.0; color: tokens.surface1 }
            }
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

            // -- App header
            RowLayout {
                Layout.fillWidth: true
                Layout.preferredHeight: 56
                Layout.topMargin: tokens.spaceL
                Layout.leftMargin: tokens.spaceL
                Layout.rightMargin: tokens.spaceL
                Layout.bottomMargin: tokens.spaceM
                spacing: tokens.spaceM

                Image {
                    source: "qrc:/qt/qml/com/visorcraft/Grexa/resources/grexa.svg"
                    sourceSize.width: 34
                    sourceSize.height: 34
                    Layout.preferredWidth: 34
                    Layout.preferredHeight: 34
                    smooth: true
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 0
                    Controls.Label {
                        text: "Grexa"
                        font.pixelSize: tokens.textSubheading
                        font.weight: tokens.weightBold
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
                Layout.bottomMargin: tokens.spaceXS
                text: qsTr("WORKSPACE")
                font.pixelSize: 10
                font.weight: tokens.weightMedium
                font.letterSpacing: 1.4
                opacity: 0.45
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
                Layout.topMargin: tokens.spaceL
                Layout.bottomMargin: tokens.spaceXS
                text: qsTr("TOOLS")
                font.pixelSize: 10
                font.weight: tokens.weightMedium
                font.letterSpacing: 1.4
                opacity: 0.45
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

            // -- Footer separator + meta
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: tokens.spaceL
                Layout.rightMargin: tokens.spaceL
                Layout.bottomMargin: tokens.spaceS
                height: 1
                color: tokens.separator
            }
            RowLayout {
                Layout.fillWidth: true
                Layout.leftMargin: tokens.spaceL
                Layout.rightMargin: tokens.spaceL
                Layout.bottomMargin: tokens.spaceM
                spacing: tokens.spaceS

                Controls.Label {
                    text: "v" + Qt.application.version
                    font.pixelSize: tokens.textCaption
                    font.family: tokens.monoFamily
                    opacity: 0.55
                }
                Item { Layout.fillWidth: true }
                Controls.Label {
                    text: "GPL-3.0"
                    font.pixelSize: tokens.textCaption
                    font.family: tokens.monoFamily
                    opacity: 0.45
                }
            }
        }
    }

    Component { id: searchPage;   SearchPage {} }
    Component { id: regexPage;    RegexBuilderPage {} }
    Component { id: settingsPage; SettingsPage {} }
    Component { id: aboutPage;    AboutPage {} }
}
