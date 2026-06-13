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
                        text: app.i18n("ui-settings")
                        font.pixelSize: app.tokens.textHeading
                        font.weight: app.tokens.weightBold
                    }
                    Controls.Label {
                        text: app.i18n("ui-autosaved-to-configgrexasettingsjson")
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
                            text: saveStatusPill.failed ? app.i18n("ui-save-failed") : app.i18n("ui-saved")
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
                    text: app.i18n("ui-reload")
                    display: Controls.AbstractButton.TextBesideIcon
                    Controls.ToolTip.text: app.i18n("ui-reread-settingsjson-from-disk-useful-after-a0edd0")
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
                title: app.i18n("ui-appearance")
                subtitle: app.i18n("ui-theme-variant-the-gtkplasma-host-palette-6a5274")
                RowLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceM
                    Controls.Label { text: app.i18n("ui-theme") }
                    AppComboBox {
                        Layout.fillWidth: true
                        Accessible.name: app.i18n("ui-theme")
                        model: [
                            app.i18n("ui-follow-system"), app.i18n("ui-light"), app.i18n("ui-dark"),
                            "OLED Black",
                            "Gentle Gecko", "Black Knight",
                            "Diamond", "Dreams", "Paranoid",
                            "Red Velvet", "Subspace",
                            "Tiefling", "Vibes",
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
                title: app.i18n("ui-search-defaults")
                subtitle: app.i18n("ui-applied-to-every-new-tab-you-5b6703")
                GridLayout {
                    columns: 2
                    columnSpacing: app.tokens.spaceL
                    rowSpacing: app.tokens.spaceS
                    Layout.fillWidth: true
                    AppCheckBox {
                        text: app.i18n("ui-regex-by-default")
                        checked: page.settings.regex
                        onToggled: { page.settings.regex = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: app.i18n("ui-filesmode-by-default")
                        checked: page.settings.filesSearchMode
                        onToggled: { page.settings.filesSearchMode = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: app.i18n("ui-respect-gitignore")
                        checked: page.settings.respectGitignore
                        onToggled: { page.settings.respectGitignore = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: app.i18n("ui-case-sensitive")
                        checked: page.settings.caseSensitive
                        onToggled: { page.settings.caseSensitive = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: app.i18n("ui-include-subfolders")
                        checked: page.settings.includeSubfolders
                        onToggled: { page.settings.includeSubfolders = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: app.i18n("ui-include-hidden")
                        checked: page.settings.includeHidden
                        onToggled: { page.settings.includeHidden = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: app.i18n("ui-include-binarydocs")
                        checked: page.settings.includeBinary
                        onToggled: { page.settings.includeBinary = checked; page.commit() }
                    }
                }
            }

            // -- Filter defaults
            Card {
                Layout.fillWidth: true
                title: app.i18n("ui-filter-defaults")
                subtitle: app.i18n("ui-glob-patterns-and-directory-excludes-that-0e7194")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceS
                    Controls.Label {
                        text: app.i18n("ui-match-files")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.65
                    }
                    AppTextField {
                        Layout.fillWidth: true
                        Accessible.name: app.i18n("ui-match-files")
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
                        text: app.i18n("ui-exclude-dirs")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.65
                        Layout.topMargin: app.tokens.spaceS
                    }
                    AppTextField {
                        Layout.fillWidth: true
                        Accessible.name: app.i18n("ui-exclude-dirs")
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
                title: app.i18n("ui-context-preview")
                subtitle: app.i18n("ui-how-many-lines-surround-a-match-92f6cc")
                GridLayout {
                    columns: 4
                    columnSpacing: app.tokens.spaceL
                    Layout.fillWidth: true
                    Controls.Label { text: app.i18n("ui-lines-before") }
                    AppSpinBox {
                        from: 1; to: 20
                        Accessible.name: app.i18n("ui-lines-before")
                        value: page.settings.contextLinesBefore
                        onValueModified: { page.settings.contextLinesBefore = value; page.commit() }
                    }
                    Controls.Label { text: app.i18n("ui-lines-after") }
                    AppSpinBox {
                        from: 1; to: 20
                        Accessible.name: app.i18n("ui-lines-after")
                        value: page.settings.contextLinesAfter
                        onValueModified: { page.settings.contextLinesAfter = value; page.commit() }
                    }
                }
            }

            // -- Containers
            Card {
                Layout.fillWidth: true
                title: app.i18n("ui-containers")
                subtitle: app.i18n("ui-allow-grexa-to-search-inside-running-eb34b5")
                AppCheckBox {
                    text: app.i18n("ui-enable-container-search")
                    checked: page.settings.enableContainerSearch
                    onToggled: { page.settings.enableContainerSearch = checked; page.commit() }
                }
            }

            // -- AI Search
            Card {
                Layout.fillWidth: true
                title: app.i18n("ui-ai-search")
                subtitle: app.i18n("ui-openaicompatible-chat-endpoint-api-key-is-676397")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceM

                    AppCheckBox {
                        text: app.i18n("ui-enable-ai-chat-panel-on-the")
                        checked: page.settings.aiSearchEnabled
                        onToggled: { page.settings.aiSearchEnabled = checked; page.commit() }
                    }

                    GridLayout {
                        columns: 2
                        columnSpacing: app.tokens.spaceL
                        rowSpacing: app.tokens.spaceS
                        Layout.fillWidth: true

                        Controls.Label { text: app.i18n("ui-endpoint") }
                        AppTextField {
                            Layout.fillWidth: true
                            Accessible.name: app.i18n("ui-endpoint")
                            placeholderText: "https://api.openai.com/v1"
                            text: page.settings.aiEndpoint
                            onTextEdited: {
                                page.settings.aiEndpoint = text
                                page.ai.endpoint = text
                            }
                            onEditingFinished: page.commit()
                        }
                        Controls.Label { text: app.i18n("ui-model") }
                        AppTextField {
                            Layout.fillWidth: true
                            Accessible.name: app.i18n("ui-model")
                            placeholderText: "gpt-4o-mini"
                            text: page.settings.aiModel
                            onTextEdited: {
                                page.settings.aiModel = text
                                page.ai.model = text
                            }
                            onEditingFinished: page.commit()
                        }
                        Controls.Label { text: app.i18n("ui-api-key") }
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: app.tokens.spaceS
                            AppTextField {
                                id: keyField
                                Layout.fillWidth: true
                                echoMode: TextInput.Password
                                Accessible.name: app.i18n("ui-api-key")
                                placeholderText: page.ai.hasApiKey ? app.i18n("ui-api-key-stored") : app.i18n("ui-paste-a-key")
                            }
                            AppFlatButton {
                                icon.name: "kt-password-stored"
                                icon.color: app.tokens.textPrimary
                                text: app.i18n("ui-save")
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
                                text: app.i18n("ui-clear")
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
                                ? app.i18n("ui-key-stored")
                                : app.i18n("ui-no-key-stored")
                            font.pixelSize: app.tokens.textCaption
                            opacity: 0.7
                            Layout.fillWidth: true
                        }
                        AppFlatButton {
                            icon.name: "network-connect"
                            icon.color: app.tokens.textPrimary
                            text: app.i18n("ui-test-endpoint")
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
                title: app.i18n("ui-editor")
                subtitle: app.i18n("ui-which-editor-opens-when-you-choose-b1c23a")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceS
                    RowLayout {
                        Layout.fillWidth: true
                        Controls.Label { text: app.i18n("ui-preset") }
                        AppComboBox {
                            Layout.fillWidth: true
                            Accessible.name: app.i18n("ui-editor-preset")
                            model: [
                                "Kate", "KWrite", "VS Code",
                                "VSCodium", "Sublime Text",
                                app.i18n("ui-jetbrains-ide"), "GNOME Text Editor",
                                app.i18n("ui-neovim-terminal"), app.i18n("ui-system-default-xdgopen"),
                            ]
                            currentIndex: page.settings.editorPreset
                            onActivated: { page.settings.editorPreset = currentIndex; page.commit() }
                        }
                    }
                    Controls.Label {
                        text: app.i18n("ui-custom-command-overrides-preset-supports-path-65d401")
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.6
                        Layout.topMargin: app.tokens.spaceS
                    }
                    AppTextField {
                        Layout.fillWidth: true
                        Accessible.name: app.i18n("ui-custom-command")
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
                title: app.i18n("ui-replace-2")
                subtitle: app.i18n("ui-safety-recovery-options-for-the-irreversible")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceS
                    AppCheckBox {
                        text: app.i18n("ui-confirm-before-replacing")
                        checked: page.settings.replaceConfirm
                        onToggled: { page.settings.replaceConfirm = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: app.i18n("ui-surface-residual-journal-on-startup")
                        checked: page.settings.replaceShowJournalOnStartup
                        onToggled: { page.settings.replaceShowJournalOnStartup = checked; page.commit() }
                    }
                }
            }

            // -- Accessibility
            Card {
                Layout.fillWidth: true
                title: app.i18n("ui-accessibility")
                subtitle: app.i18n("ui-reduced-motion-disables-resultrow-transitions-and-bfd4cf")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceS
                    AppCheckBox {
                        text: app.i18n("ui-reduce-motion")
                        checked: page.settings.accessibilityReducedMotion
                        onToggled: { page.settings.accessibilityReducedMotion = checked; page.commit() }
                    }
                    AppCheckBox {
                        text: app.i18n("ui-high-contrast")
                        checked: page.settings.accessibilityHighContrast
                        onToggled: { page.settings.accessibilityHighContrast = checked; page.commit() }
                    }
                }
            }

            // -- Privacy
            Card {
                Layout.fillWidth: true
                title: app.i18n("ui-privacy")
                subtitle: app.i18n("ui-redact-filesystem-paths-from-grexaguilog-and-eb384e")
                AppCheckBox {
                    text: app.i18n("ui-redact-paths-in-diagnostics")
                    checked: page.settings.privacyRedactPaths
                    onToggled: { page.settings.privacyRedactPaths = checked; page.commit() }
                }
            }

            // -- Diagnostics
            Card {
                Layout.fillWidth: true
                title: app.i18n("ui-diagnostics")
                subtitle: app.i18n("ui-where-grexa-writes-its-logs-and")
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceXS
                    Controls.Label {
                        text: app.i18n("ui-log-xdgstatehomegrexagrexaguilog")
                        font.family: app.tokens.monoFamily
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.7
                    }
                    Controls.Label {
                        text: app.i18n("ui-filter-grexaloginfogrexacoredebug")
                        font.family: app.tokens.monoFamily
                        font.pixelSize: app.tokens.textCaption
                        opacity: 0.7
                    }
                }
            }
        }
    }
}
