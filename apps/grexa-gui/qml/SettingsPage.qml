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

    // Pages render with the View colorSet, separate from the
    // Window colorSet we override on ApplicationWindow. Without
    // this block, the page canvas keeps the host theme's View bg
    // (visible as a stale dark stripe behind cards on Light).
    Kirigami.Theme.inherit: false
    Kirigami.Theme.colorSet: Kirigami.Theme.View
    Kirigami.Theme.backgroundColor: app.tokens.surface0
    Kirigami.Theme.textColor: app.tokens.textPrimary
    Kirigami.Theme.highlightColor: app.tokens.accent
    Kirigami.Theme.highlightedTextColor: app.tokens.accentText

    // Re-apply the Qt palette at the page level too. The window-
    // level palette set in Main.qml does cascade to most children,
    // but Kirigami's page chrome resets palette inheritance at the
    // page boundary — leaving flat Controls.Buttons (e.g. Reload)
    // painted with the host's `windowText` (white on dark) instead
    // of our themed `tokens.textPrimary`, which is invisible on
    // light surfaces.
    palette.window:          app.tokens.surface0
    palette.windowText:      app.tokens.textPrimary
    palette.base:            app.tokens.surface1
    palette.alternateBase:   app.tokens.surface2
    palette.text:            app.tokens.textPrimary
    palette.button:          app.tokens.surface1
    palette.buttonText:      app.tokens.textPrimary
    palette.brightText:      app.tokens.accentText
    palette.highlight:       app.tokens.accent
    palette.highlightedText: app.tokens.accentText
    palette.toolTipBase:     app.tokens.surface2
    palette.toolTipText:     app.tokens.textPrimary
    palette.mid:             app.tokens.separator
    palette.midlight:        app.tokens.surface1
    palette.light:           app.tokens.surface2
    palette.dark:            app.tokens.surface0
    palette.shadow:          app.tokens.shadowFar
    palette.placeholderText: Qt.rgba(app.tokens.textPrimary.r,
                                     app.tokens.textPrimary.g,
                                     app.tokens.textPrimary.b, 0.55)

    property var settings: app.settingsController
    readonly property var themeValues: [0, 1, 2, 12, 3, 4, 5, 6, 7, 8, 9, 10, 11]
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
                AppFlatButton {
                    icon.name: "view-refresh"
                    icon.color: app.tokens.textPrimary
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
                    AppComboBox {
                        Layout.fillWidth: true
                        model: [
                            qsTr("Follow system"), qsTr("Light"), qsTr("Dark"),
                            qsTr("OLED Black"),
                            qsTr("Gentle Gecko"), qsTr("Black Knight"),
                            qsTr("Diamond"), qsTr("Dreams"), qsTr("Paranoid"),
                            qsTr("Red Velvet"), qsTr("Subspace"),
                            qsTr("Tiefling"), qsTr("Vibes"),
                        ]
                        currentIndex: page.themeValues.indexOf(page.settings.theme)
                        onActivated: {
                            page.settings.theme = page.themeValues[currentIndex]
                            page.commit()
                        }
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
                    AppCheckBox {
                        text: qsTr("Regex by default")
                        checked: page.settings.regex
                        onToggled: { page.settings.regex = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: qsTr("Files-mode by default")
                        checked: page.settings.filesSearchMode
                        onToggled: { page.settings.filesSearchMode = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: qsTr("Respect .gitignore")
                        checked: page.settings.respectGitignore
                        onToggled: { page.settings.respectGitignore = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: qsTr("Case sensitive")
                        checked: page.settings.caseSensitive
                        onToggled: { page.settings.caseSensitive = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: qsTr("Include subfolders")
                        checked: page.settings.includeSubfolders
                        onToggled: { page.settings.includeSubfolders = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: qsTr("Include hidden")
                        checked: page.settings.includeHidden
                        onToggled: { page.settings.includeHidden = checked; page.commit() }
                    }
                    AppCheckBox {
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
                    AppTextField {
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
                    AppTextField {
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
                    AppSpinBox {
                        from: 1; to: 20
                        value: page.settings.contextLinesBefore
                        onValueModified: { page.settings.contextLinesBefore = value; page.commit() }
                    }
                    Controls.Label { text: qsTr("Lines after") }
                    AppSpinBox {
                        from: 1; to: 20
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
                AppCheckBox {
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

                    AppCheckBox {
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
                        AppTextField {
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
                        AppTextField {
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
                            AppTextField {
                                id: keyField
                                Layout.fillWidth: true
                                echoMode: TextInput.Password
                                placeholderText: page.ai.hasApiKey ? qsTr("•••••• (stored)") : qsTr("paste a key…")
                            }
                            AppFlatButton {
                                icon.name: "kt-password-stored"
                                icon.color: app.tokens.textPrimary
                                text: qsTr("Save")
                                display: Controls.AbstractButton.TextBesideIcon
                                enabled: keyField.text.length > 0
                                onClicked: {
                                    if (page.ai.setApiKey(keyField.text)) {
                                        keyField.text = ""
                                    }
                                }
                            }
                            AppFlatButton {
                                icon.name: "edit-delete"
                                icon.color: app.tokens.textPrimary
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
                        AppFlatButton {
                            icon.name: "network-connect"
                            icon.color: app.tokens.textPrimary
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
                        AppComboBox {
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
                    AppTextField {
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
                    AppCheckBox {
                        text: qsTr("Confirm before replacing")
                        checked: page.settings.replaceConfirm
                        onToggled: { page.settings.replaceConfirm = checked; page.commit() }
                    }
                    AppCheckBox {
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
                    AppCheckBox {
                        text: qsTr("Reduce motion")
                        checked: page.settings.accessibilityReducedMotion
                        onToggled: { page.settings.accessibilityReducedMotion = checked; page.commit() }
                    }
                    AppCheckBox {
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
                AppCheckBox {
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
