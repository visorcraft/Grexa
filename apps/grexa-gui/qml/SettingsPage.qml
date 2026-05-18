// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Settings — bound to `app.settingsController`. Sections live in
// Card surfaces with a description so each toggle's intent is
// obvious. Apply/Reload pinned to the page header.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    id: page
    padding: 0
    titleDelegate: Item {}
    globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

    property var settings: app.settingsController
    property var ai: app.aiController

    // Latched copy of `settings.lastSaveStatus`. The pill reads
    // from this so it can stay coloured while it fades out, even
    // if the controller writes a different status between the
    // commit and the fade.
    property string lastSaveResult: ""

    Component.onCompleted: settings.reload()

    // Every settings input calls `commit()` on change so the user
    // never has to remember to hit Apply. Saves are atomic on the
    // Rust side (write to settings.json then rename), so this is
    // cheap and safe to call on every keystroke / toggle. The AI
    // controller also re-reads so the chat panel sees endpoint /
    // model changes without restarting.
    function commit() {
        page.settings.apply()
        page.ai.reloadFromSettings()
    }

    ColumnLayout {
        width: page.width
        spacing: 0

        // -- Page header
        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 64
            Rectangle {
                anchors.fill: parent
                color: app.tokens.surface0
                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 1
                    color: app.tokens.separator
                }
            }
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: app.tokens.spaceXL
                anchors.rightMargin: app.tokens.spaceXL
                spacing: app.tokens.spaceM

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 0
                    Controls.Label {
                        text: qsTr("Settings")
                        font.pixelSize: app.tokens.textHeading
                        font.weight: app.tokens.weightBold
                    }
                    Controls.Label {
                        text: qsTr("Auto-saved to ~/.config/grexa/settings.json")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.6
                    }
                }
                // Save-status indicator. Green/accent pill when a
                // commit succeeded; red/negative pill when the disk
                // write failed (the user sees *some* feedback either
                // way — silent failure is a footgun). Both states
                // fade in via the timer; the failure pill stays
                // visible longer so the user can actually read it.
                Rectangle {
                    id: saveStatusPill
                    readonly property bool failed: lastSaveResult === "Save failed"
                    Layout.preferredHeight: 26
                    Layout.preferredWidth: saveStatusLabel.implicitWidth + app.tokens.spaceL * 2 + 8
                    radius: app.tokens.radiusPill
                    color: failed ? app.tokens.errorMute : app.tokens.accentMute
                    border.color: failed ? app.tokens.error : app.tokens.accent
                    border.width: 1
                    opacity: savedTimer.running ? 1.0 : 0.0
                    Behavior on opacity { NumberAnimation { duration: app.tokens.durationNormal } }
                    RowLayout {
                        anchors.centerIn: parent
                        spacing: app.tokens.spaceXS
                        Kirigami.Icon {
                            source: saveStatusPill.failed
                                ? "dialog-error-symbolic"
                                : "dialog-ok-symbolic"
                            implicitWidth: 12
                            implicitHeight: 12
                            color: saveStatusPill.failed
                                ? app.tokens.error
                                : app.tokens.accent
                            isMask: true
                        }
                        Controls.Label {
                            id: saveStatusLabel
                            text: saveStatusPill.failed ? qsTr("Save failed") : qsTr("Saved")
                            font.pixelSize: app.tokens.textCaption
                            font.weight: app.tokens.weightSemibold
                            color: saveStatusPill.failed
                                ? app.tokens.error
                                : app.tokens.accent
                        }
                    }
                    Controls.ToolTip.text: page.lastSaveResult
                    Controls.ToolTip.visible: failed
                        && saveStatusMouse.containsMouse
                    MouseArea {
                        id: saveStatusMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.NoButton
                    }
                }
                Timer {
                    id: savedTimer
                    // Success fades quickly; failures stay visible
                    // a few seconds so the user can read them.
                    interval: page.lastSaveResult === "Save failed" ? 4500 : 1400
                    repeat: false
                }
                Connections {
                    target: page.settings
                    function onLastSaveStatusChanged() {
                        // Latch the latest status into a local property
                        // so the pill keeps the right colour while
                        // fading out (the controller could clear or
                        // overwrite `lastSaveStatus` between the
                        // commit and the fade).
                        page.lastSaveResult = page.settings.lastSaveStatus
                        if (page.lastSaveResult === "Saved"
                            || page.lastSaveResult === "Save failed") {
                            savedTimer.restart()
                        }
                    }
                }
                Controls.Button {
                    flat: true
                    icon.name: "view-refresh"
                    text: qsTr("Reload")
                    display: Controls.AbstractButton.TextBesideIcon
                    Controls.ToolTip.text: qsTr("Re-read settings.json from disk (useful after editing the file by hand).")
                    Controls.ToolTip.visible: hovered
                    onClicked: page.settings.reload()
                }
            }
        }

        // -- Cards
        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceL
            Layout.bottomMargin: app.tokens.spaceL
            spacing: app.tokens.spaceL

            // -- Appearance
            Card {
                Layout.fillWidth: true
                title: qsTr("Appearance")
                subtitle: qsTr("Theme variant — the GTK/Plasma host palette still drives the chrome; this picks the in-app accent.")
                RowLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceM
                    Controls.Label { text: qsTr("Theme") }
                    Controls.ComboBox {
                        Layout.fillWidth: true
                        model: [
                            qsTr("Follow system"), qsTr("Light"), qsTr("Dark"),
                            qsTr("Gentle Gecko"), qsTr("Black Knight"),
                            qsTr("Diamond"), qsTr("Dreams"), qsTr("Paranoid"),
                            qsTr("Red Velvet"), qsTr("Subspace"),
                            qsTr("Tiefling"), qsTr("Vibes"),
                        ]
                        currentIndex: page.settings.theme
                        onActivated: { page.settings.theme = currentIndex; page.commit() }
                    }
                }
            }

            // -- Search defaults
            Card {
                Layout.fillWidth: true
                title: qsTr("Search defaults")
                subtitle: qsTr("Applied to every new tab. You can still toggle these per-search in the Search page.")
                GridLayout {
                    columns: 2
                    columnSpacing: app.tokens.spaceL
                    rowSpacing: app.tokens.spaceS
                    Layout.fillWidth: true
                    Controls.CheckBox {
                        text: qsTr("Regex by default")
                        checked: page.settings.regex
                        onToggled: { page.settings.regex = checked; page.commit() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Files-mode by default")
                        checked: page.settings.filesSearchMode
                        onToggled: { page.settings.filesSearchMode = checked; page.commit() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Respect .gitignore")
                        checked: page.settings.respectGitignore
                        onToggled: { page.settings.respectGitignore = checked; page.commit() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Case sensitive")
                        checked: page.settings.caseSensitive
                        onToggled: { page.settings.caseSensitive = checked; page.commit() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Include subfolders")
                        checked: page.settings.includeSubfolders
                        onToggled: { page.settings.includeSubfolders = checked; page.commit() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Include hidden")
                        checked: page.settings.includeHidden
                        onToggled: { page.settings.includeHidden = checked; page.commit() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Include binary/docs")
                        checked: page.settings.includeBinary
                        onToggled: { page.settings.includeBinary = checked; page.commit() }
                    }
                }
            }

            // -- Filter defaults
            Card {
                Layout.fillWidth: true
                title: qsTr("Filter defaults")
                subtitle: qsTr("Glob patterns and directory excludes that pre-populate every new search.")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceS
                    Controls.Label {
                        text: qsTr("Match files")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.65
                    }
                    Controls.TextField {
                        Layout.fillWidth: true
                        placeholderText: "*.rs|*.toml|-target*"
                        text: page.settings.defaultMatchFiles
                        // Text fields commit on editing-finished (focus
                        // loss / Enter) rather than every keystroke so
                        // we don't thrash settings.json while the user
                        // is mid-edit. Keystroke updates still flow
                        // through the qproperty for live preview.
                        onTextEdited: page.settings.defaultMatchFiles = text
                        onEditingFinished: page.commit()
                    }
                    Controls.Label {
                        text: qsTr("Exclude dirs")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.65
                        Layout.topMargin: app.tokens.spaceS
                    }
                    Controls.TextField {
                        Layout.fillWidth: true
                        placeholderText: "node_modules, target, .venv"
                        text: page.settings.defaultExcludeDirs
                        onTextEdited: page.settings.defaultExcludeDirs = text
                        onEditingFinished: page.commit()
                    }
                }
            }

            // -- Context preview
            Card {
                Layout.fillWidth: true
                title: qsTr("Context preview")
                subtitle: qsTr("How many lines surround a match when you open the preview dialog.")
                GridLayout {
                    columns: 4
                    columnSpacing: app.tokens.spaceL
                    Layout.fillWidth: true
                    Controls.Label { text: qsTr("Lines before") }
                    Controls.SpinBox {
                        from: 0; to: 50
                        value: page.settings.contextLinesBefore
                        onValueModified: { page.settings.contextLinesBefore = value; page.commit() }
                    }
                    Controls.Label { text: qsTr("Lines after") }
                    Controls.SpinBox {
                        from: 0; to: 50
                        value: page.settings.contextLinesAfter
                        onValueModified: { page.settings.contextLinesAfter = value; page.commit() }
                    }
                }
            }

            // -- Containers
            Card {
                Layout.fillWidth: true
                title: qsTr("Containers")
                subtitle: qsTr("Allow Grexa to search inside running Docker and Podman containers.")
                Controls.CheckBox {
                    text: qsTr("Enable container search")
                    checked: page.settings.enableContainerSearch
                    onToggled: { page.settings.enableContainerSearch = checked; page.commit() }
                }
            }

            // -- AI Search
            Card {
                Layout.fillWidth: true
                title: qsTr("AI Search")
                subtitle: qsTr("OpenAI-compatible chat endpoint. API key is stored in Secret Service (KWallet / GNOME Keyring) and never round-trips through QML.")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceM

                    Controls.CheckBox {
                        text: qsTr("Enable AI chat panel on the Search page")
                        checked: page.settings.aiSearchEnabled
                        onToggled: { page.settings.aiSearchEnabled = checked; page.commit() }
                    }

                    GridLayout {
                        columns: 2
                        columnSpacing: app.tokens.spaceL
                        rowSpacing: app.tokens.spaceS
                        Layout.fillWidth: true

                        Controls.Label { text: qsTr("Endpoint") }
                        Controls.TextField {
                            Layout.fillWidth: true
                            placeholderText: "https://api.openai.com/v1"
                            text: page.settings.aiEndpoint
                            onTextEdited: {
                                page.settings.aiEndpoint = text
                                page.ai.endpoint = text
                            }
                            onEditingFinished: page.commit()
                        }
                        Controls.Label { text: qsTr("Model") }
                        Controls.TextField {
                            Layout.fillWidth: true
                            placeholderText: "gpt-4o-mini"
                            text: page.settings.aiModel
                            onTextEdited: {
                                page.settings.aiModel = text
                                page.ai.model = text
                            }
                            onEditingFinished: page.commit()
                        }
                        Controls.Label { text: qsTr("API key") }
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: app.tokens.spaceS
                            Controls.TextField {
                                id: keyField
                                Layout.fillWidth: true
                                echoMode: TextInput.Password
                                placeholderText: page.ai.hasApiKey ? qsTr("•••••• (stored)") : qsTr("paste a key…")
                            }
                            Controls.Button {
                                flat: true
                                icon.name: "kt-password-stored"
                                text: qsTr("Save")
                                display: Controls.AbstractButton.TextBesideIcon
                                enabled: keyField.text.length > 0
                                onClicked: {
                                    if (page.ai.setApiKey(keyField.text)) {
                                        keyField.text = ""
                                    }
                                }
                            }
                            Controls.Button {
                                flat: true
                                icon.name: "edit-delete"
                                text: qsTr("Clear")
                                display: Controls.AbstractButton.TextBesideIcon
                                enabled: page.ai.hasApiKey
                                onClicked: page.ai.clearApiKey()
                            }
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        Controls.Label {
                            text: page.ai.hasApiKey
                                ? qsTr("Key stored.")
                                : qsTr("No key stored.")
                            font.pixelSize: app.tokens.textCaption
                            opacity: 0.7
                            Layout.fillWidth: true
                        }
                        Controls.Button {
                            flat: true
                            icon.name: "network-connect"
                            text: qsTr("Test endpoint")
                            display: Controls.AbstractButton.TextBesideIcon
                            enabled: !page.ai.busy && page.settings.aiEndpoint.length > 0
                            onClicked: {
                                page.ai.endpoint = page.settings.aiEndpoint
                                page.ai.testEndpoint()
                            }
                        }
                    }
                }
            }

            // -- Editor
            Card {
                Layout.fillWidth: true
                title: qsTr("Editor")
                subtitle: qsTr("Which editor opens when you choose “Open in editor” from a result row.")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceS
                    RowLayout {
                        Layout.fillWidth: true
                        Controls.Label { text: qsTr("Preset") }
                        Controls.ComboBox {
                            Layout.fillWidth: true
                            model: [
                                qsTr("Kate"), qsTr("KWrite"), qsTr("VS Code"),
                                qsTr("VSCodium"), qsTr("Sublime Text"),
                                qsTr("JetBrains IDE"), qsTr("GNOME Text Editor"),
                                qsTr("Neovim (terminal)"), qsTr("System default (xdg-open)"),
                            ]
                            currentIndex: page.settings.editorPreset
                            onActivated: { page.settings.editorPreset = currentIndex; page.commit() }
                        }
                    }
                    Controls.Label {
                        text: qsTr("Custom command (overrides preset; supports {path} and {line})")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.6
                        Layout.topMargin: app.tokens.spaceS
                    }
                    Controls.TextField {
                        Layout.fillWidth: true
                        placeholderText: "kate --line {line} {path}"
                        text: page.settings.editorCustomCommand
                        onTextEdited: page.settings.editorCustomCommand = text
                        onEditingFinished: page.commit()
                    }
                }
            }

            // -- Replace
            Card {
                Layout.fillWidth: true
                title: qsTr("Replace")
                subtitle: qsTr("Safety + recovery options for the irreversible rewrite flow.")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceS
                    Controls.CheckBox {
                        text: qsTr("Confirm before replacing")
                        checked: page.settings.replaceConfirm
                        onToggled: { page.settings.replaceConfirm = checked; page.commit() }
                    }
                    Controls.CheckBox {
                        text: qsTr("Surface residual journal on startup")
                        checked: page.settings.replaceShowJournalOnStartup
                        onToggled: { page.settings.replaceShowJournalOnStartup = checked; page.commit() }
                    }
                }
            }

            // -- Accessibility
            Card {
                Layout.fillWidth: true
                title: qsTr("Accessibility")
                subtitle: qsTr("Reduced motion disables result-row transitions and busy spinners. High contrast nudges the palette toward higher legibility.")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceS
                    Controls.CheckBox {
                        text: qsTr("Reduce motion")
                        checked: page.settings.accessibilityReducedMotion
                        onToggled: { page.settings.accessibilityReducedMotion = checked; page.commit() }
                    }
                    Controls.CheckBox {
                        text: qsTr("High contrast")
                        checked: page.settings.accessibilityHighContrast
                        onToggled: { page.settings.accessibilityHighContrast = checked; page.commit() }
                    }
                }
            }

            // -- Privacy
            Card {
                Layout.fillWidth: true
                title: qsTr("Privacy")
                subtitle: qsTr("Redact filesystem paths from grexa-gui.log and any crash diagnostics generated locally.")
                Controls.CheckBox {
                    text: qsTr("Redact paths in diagnostics")
                    checked: page.settings.privacyRedactPaths
                    onToggled: { page.settings.privacyRedactPaths = checked; page.commit() }
                }
            }

            // -- Diagnostics
            Card {
                Layout.fillWidth: true
                title: qsTr("Diagnostics")
                subtitle: qsTr("Where Grexa writes its logs and how to control verbosity.")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceXS
                    Controls.Label {
                        text: qsTr("Log: $XDG_STATE_HOME/grexa/grexa-gui.log")
                        font.family: app.tokens.monoFamily
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.7
                    }
                    Controls.Label {
                        text: qsTr("Filter: GREXA_LOG=info,grexa_core=debug")
                        font.family: app.tokens.monoFamily
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.7
                    }
                }
            }
        }
    }
}
