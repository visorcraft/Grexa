// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Grexa GUI shell.
//
// Kirigami `ApplicationWindow` with a global drawer and a page stack.
// The Rust-side controllers (`SearchController`, `SettingsController`,
// `RegexBuilderController`, `AiController`) are instantiated here so
// every page can reference them through the `app.*` ids — keeps each
// controller a singleton without having to register a QML singleton
// type.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.visorcraft.Grexa 1.0

Kirigami.ApplicationWindow {
    id: app
    title: qsTr("Grexa")
    width: 1100
    height: 700
    minimumWidth: 760
    minimumHeight: 480
    visible: true

    // Cross-page controllers. Each holds Rust-side state; QML reads
    // them via `app.searchController.busy` etc.
    property alias searchController: searchController
    property alias settingsController: settingsController
    property alias regexController: regexController
    property alias aiController: aiController

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


    globalDrawer: Kirigami.GlobalDrawer {
        title: qsTr("Grexa")
        titleIcon: "io.visorcraft.Grexa"
        isMenu: false
        modal: false
        collapsible: false
        width: Kirigami.Units.gridUnit * 14
        actions: [
            Kirigami.Action {
                text: qsTr("Search")
                icon.name: "edit-find"
                onTriggered: app.pageStack.replace(searchPage)
            },
            Kirigami.Action {
                text: qsTr("Regex Builder")
                icon.name: "code-context"
                onTriggered: app.pageStack.replace(regexPage)
            },
            Kirigami.Action {
                text: qsTr("Settings")
                icon.name: "settings-configure"
                onTriggered: app.pageStack.replace(settingsPage)
            },
            Kirigami.Action {
                text: qsTr("About")
                icon.name: "help-about"
                onTriggered: app.pageStack.replace(aboutPage)
            }
        ]
    }

    Component {
        id: searchPage
        SearchPage {}
    }
    Component {
        id: regexPage
        RegexBuilderPage {}
    }
    Component {
        id: settingsPage
        SettingsPage {}
    }
    Component {
        id: aboutPage
        AboutPage {}
    }
}
