// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Settings — bound to `app.settingsController` (which wraps
// `grexa_core::SettingsStore`). Reload happens on load; Apply
// persists. The AI section additionally uses `app.aiController` for
// the keyring operations.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    id: page
    title: i18n("Settings")
    padding: Kirigami.Units.smallSpacing

    property var settings: app.settingsController
    property var ai: app.aiController

    Component.onCompleted: settings.reload()

    actions: [
        Kirigami.Action {
            text: i18n("Apply")
            icon.name: "dialog-ok-apply"
            onTriggered: settings.apply()
        },
        Kirigami.Action {
            text: i18n("Reload")
            icon.name: "view-refresh"
            onTriggered: settings.reload()
        }
    ]

    ColumnLayout {
        spacing: Kirigami.Units.largeSpacing
        width: parent.width

        Kirigami.FormLayout {
            Layout.fillWidth: true

            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Appearance")
            }

            ComboBox {
                Kirigami.FormData.label: i18n("Theme:")
                model: [
                    i18n("Follow system"),
                    i18n("Light"),
                    i18n("Dark"),
                    i18n("Gentle Gecko"),
                    i18n("Black Knight"),
                    i18n("Diamond"),
                    i18n("Dreams"),
                    i18n("Paranoid"),
                    i18n("Red Velvet"),
                    i18n("Subspace"),
                    i18n("Tiefling"),
                    i18n("Vibes"),
                ]
                currentIndex: settings.theme
                onActivated: settings.theme = currentIndex
            }

            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Search defaults")
            }
            CheckBox {
                Kirigami.FormData.label: i18n("Regex search by default")
                checked: settings.regex
                onToggled: settings.regex = checked
            }
            CheckBox {
                Kirigami.FormData.label: i18n("Files-mode by default")
                checked: settings.filesSearchMode
                onToggled: settings.filesSearchMode = checked
            }
            CheckBox {
                Kirigami.FormData.label: i18n("Respect .gitignore")
                checked: settings.respectGitignore
                onToggled: settings.respectGitignore = checked
            }
            CheckBox {
                Kirigami.FormData.label: i18n("Case sensitive")
                checked: settings.caseSensitive
                onToggled: settings.caseSensitive = checked
            }
            CheckBox {
                Kirigami.FormData.label: i18n("Include subfolders")
                checked: settings.includeSubfolders
                onToggled: settings.includeSubfolders = checked
            }
            CheckBox {
                Kirigami.FormData.label: i18n("Include hidden")
                checked: settings.includeHidden
                onToggled: settings.includeHidden = checked
            }
            CheckBox {
                Kirigami.FormData.label: i18n("Include binary/docs")
                checked: settings.includeBinary
                onToggled: settings.includeBinary = checked
            }

            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Filter defaults")
            }
            TextField {
                Kirigami.FormData.label: i18n("Match files:")
                placeholderText: "*.rs|*.toml"
                text: settings.defaultMatchFiles
                onEditingFinished: settings.defaultMatchFiles = text
            }
            TextField {
                Kirigami.FormData.label: i18n("Exclude dirs:")
                placeholderText: "node_modules,target"
                text: settings.defaultExcludeDirs
                onEditingFinished: settings.defaultExcludeDirs = text
            }

            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Context preview")
            }
            SpinBox {
                Kirigami.FormData.label: i18n("Lines before:")
                from: 0; to: 50
                value: settings.contextLinesBefore
                onValueModified: settings.contextLinesBefore = value
            }
            SpinBox {
                Kirigami.FormData.label: i18n("Lines after:")
                from: 0; to: 50
                value: settings.contextLinesAfter
                onValueModified: settings.contextLinesAfter = value
            }

            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Containers")
            }
            CheckBox {
                Kirigami.FormData.label: i18n("Enable container search")
                checked: settings.enableContainerSearch
                onToggled: settings.enableContainerSearch = checked
            }

            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("AI Search")
            }
            CheckBox {
                Kirigami.FormData.label: i18n("Enable AI chat")
                checked: settings.aiSearchEnabled
                onToggled: settings.aiSearchEnabled = checked
            }
            TextField {
                Kirigami.FormData.label: i18n("Endpoint:")
                placeholderText: "https://api.openai.com/v1"
                text: settings.aiEndpoint
                onEditingFinished: {
                    settings.aiEndpoint = text
                    ai.endpoint = text
                }
            }
            TextField {
                Kirigami.FormData.label: i18n("Model:")
                placeholderText: "gpt-4o-mini"
                text: settings.aiModel
                onEditingFinished: {
                    settings.aiModel = text
                    ai.model = text
                }
            }
            RowLayout {
                Kirigami.FormData.label: i18n("API key:")
                TextField {
                    id: keyField
                    Layout.fillWidth: true
                    echoMode: TextInput.Password
                    placeholderText: ai.hasApiKey ? i18n("(stored)") : i18n("Enter API key")
                }
                Button {
                    text: i18n("Save")
                    icon.name: "kt-password-stored"
                    enabled: keyField.text.length > 0
                    onClicked: {
                        if (ai.setApiKey(keyField.text)) {
                            keyField.text = ""
                        }
                    }
                }
                Button {
                    text: i18n("Delete")
                    icon.name: "edit-delete"
                    enabled: ai.hasApiKey
                    onClicked: ai.clearApiKey()
                }
            }
            Label {
                text: ai.hasApiKey
                    ? i18n("Key stored in Secret Service.")
                    : i18n("No key stored. Keys are saved via org.freedesktop.secrets (KWallet / GNOME Keyring).")
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                opacity: 0.7
            }
            Button {
                text: i18n("Test endpoint")
                icon.name: "network-connect"
                enabled: !ai.busy && settings.aiEndpoint.length > 0
                onClicked: {
                    ai.endpoint = settings.aiEndpoint
                    ai.testEndpoint()
                }
            }
        }

        Kirigami.InlineMessage {
            Layout.fillWidth: true
            visible: settings.lastSaveStatus.length > 0
            type: settings.lastSaveStatus === "Saved" ? Kirigami.MessageType.Positive
                : settings.lastSaveStatus === "Save failed" ? Kirigami.MessageType.Error
                : Kirigami.MessageType.Information
            text: settings.lastSaveStatus
        }

        Label {
            text: i18n("Log file: $XDG_STATE_HOME/grexa/grexa-gui.log\nOverride verbosity with the GREXA_LOG environment variable.")
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
            opacity: 0.7
        }
    }
}
