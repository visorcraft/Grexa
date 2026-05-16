// Grexa GUI shell — Phase 1/4 placeholder.
//
// This file is loaded by the Rust host (apps/grexa-gui/src/main.rs)
// via `qml6 Main.qml`. The structure here is the contract the future
// cxx-qt iteration will inhabit: Kirigami ApplicationWindow with a
// compact navigation rail, tab strip, command strip, two-pane search
// layout, and per-tab result tables.
//
// Today every page is a static stub that lists what data it expects
// from the Rust side. The expected wiring is:
//
//   - SearchPage receives: SearchOptions + ProgressEvent stream +
//     SearchSummary
//   - RegexBuilderPage receives: a writable regex pattern + sample
//     text + live match list
//   - SettingsPage receives: DefaultSettings (round-trippable JSON)
//   - AboutPage receives: app-version + commit-sha + locale list
//
// All strings flow through the Fluent bundle exposed by Rust as the
// `tr(key)` JS function. For the placeholder we inline English.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ApplicationWindow {
    id: app
    title: i18n("Grexa")
    width: 1100
    height: 700
    minimumWidth: 760
    minimumHeight: 480

    function tr(key) {
        return key
    }

    pageStack.initialPage: searchPage

    globalDrawer: Kirigami.GlobalDrawer {
        title: i18n("Grexa")
        titleIcon: "io.visorcraft.Grexa"
        isMenu: false
        modal: false
        actions: [
            Kirigami.Action {
                text: i18n("Search")
                icon.name: "edit-find"
                onTriggered: app.pageStack.replace(searchPage)
            },
            Kirigami.Action {
                text: i18n("Regex Builder")
                icon.name: "code-context"
                onTriggered: app.pageStack.replace(regexPage)
            },
            Kirigami.Action {
                text: i18n("Settings")
                icon.name: "settings-configure"
                onTriggered: app.pageStack.replace(settingsPage)
            },
            Kirigami.Action {
                text: i18n("About")
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
