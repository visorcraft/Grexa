// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Settings — bound to `grexa_core::SettingsStore`. Instant-save: every
// toggle invokes `store.save(...)` synchronously. The keyring portion
// for the AI section calls into `grexa_ai::store_api_key` /
// `delete_api_key`.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    title: i18n("Settings")
    padding: Kirigami.Units.smallSpacing

    ColumnLayout {
        spacing: Kirigami.Units.largeSpacing
        width: parent.width

        // -- Appearance ---------------------------------------------------
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
            }

            // -- Language -----------------------------------------------
            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Language")
            }
            ComboBox {
                Kirigami.FormData.label: i18n("UI language:")
                model: [i18n("English"), i18n("Deutsch"), i18n("日本語")]
            }

            // -- Search defaults ----------------------------------------
            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Search defaults")
            }
            CheckBox { Kirigami.FormData.label: i18n("Regex search by default"); text: "" }
            CheckBox { Kirigami.FormData.label: i18n("Files-mode by default"); text: "" }
            CheckBox { Kirigami.FormData.label: i18n("Respect .gitignore"); text: "" }
            CheckBox { Kirigami.FormData.label: i18n("Case sensitive"); text: "" }
            CheckBox { Kirigami.FormData.label: i18n("Include subfolders"); text: ""; checked: true }
            CheckBox { Kirigami.FormData.label: i18n("Include hidden"); text: "" }
            CheckBox { Kirigami.FormData.label: i18n("Include binary/docs"); text: "" }
            CheckBox { Kirigami.FormData.label: i18n("Follow symlinks"); text: "" }
            CheckBox { Kirigami.FormData.label: i18n("Use Linux file index (Baloo)"); text: "" }

            // -- Filter defaults ----------------------------------------
            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Filter defaults")
            }
            TextField { Kirigami.FormData.label: i18n("Match files:"); placeholderText: "*.rs|*.toml" }
            TextField { Kirigami.FormData.label: i18n("Exclude dirs:"); placeholderText: "node_modules,target" }
            ComboBox  { Kirigami.FormData.label: i18n("Size unit:"); model: ["KB", "MB", "GB"] }

            // -- String comparison --------------------------------------
            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("String comparison")
            }
            ComboBox {
                Kirigami.FormData.label: i18n("Mode:")
                model: [i18n("Ordinal"), i18n("Current culture"), i18n("Invariant culture")]
            }
            ComboBox {
                Kirigami.FormData.label: i18n("Unicode normalization:")
                model: ["None", "Form C", "Form D", "Form KC", "Form KD"]
            }
            CheckBox { Kirigami.FormData.label: i18n("Diacritic-sensitive"); checked: true; text: "" }
            TextField { Kirigami.FormData.label: i18n("Culture override:"); placeholderText: "tr-TR" }

            // -- Context preview ----------------------------------------
            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Context preview")
            }
            SpinBox { Kirigami.FormData.label: i18n("Lines before:"); from: 1; to: 20; value: 5 }
            SpinBox { Kirigami.FormData.label: i18n("Lines after:"); from: 1; to: 20; value: 5 }

            // -- Containers --------------------------------------------
            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("Containers")
            }
            CheckBox { Kirigami.FormData.label: i18n("Show container target in search bar"); text: "" }

            // -- AI Search ---------------------------------------------
            Kirigami.Heading {
                Kirigami.FormData.isSection: true
                text: i18n("AI Search")
            }
            CheckBox { Kirigami.FormData.label: i18n("Enable AI chat"); text: "" }
            TextField { Kirigami.FormData.label: i18n("Endpoint:"); placeholderText: "https://api.openai.com/v1" }
            TextField { Kirigami.FormData.label: i18n("Model:"); placeholderText: "gpt-4o-mini" }
            RowLayout {
                Kirigami.FormData.label: i18n("API key:")
                TextField { id: keyField; Layout.fillWidth: true; echoMode: TextInput.Password }
                Button { text: i18n("Save to keyring"); icon.name: "kt-password-stored" }
                Button { text: i18n("Delete"); icon.name: "edit-delete" }
            }
            Label {
                text: i18n("Keys are stored via org.freedesktop.secrets (KWallet / GNOME Keyring).")
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                opacity: 0.7
            }
            Button { text: i18n("Test endpoint"); icon.name: "network-connect" }
        }

        // -- Backup / Restore -----------------------------------------------
        Kirigami.Heading {
            Layout.alignment: Qt.AlignLeft
            level: 3
            text: i18n("Backup and restore")
        }
        RowLayout {
            Layout.fillWidth: true
            Button { text: i18n("Export settings…"); icon.name: "document-export" }
            Button { text: i18n("Import settings…"); icon.name: "document-import" }
            Button { text: i18n("Restore defaults"); icon.name: "edit-clear" }
        }

        // -- Diagnostics ---------------------------------------------------
        Kirigami.Heading {
            Layout.alignment: Qt.AlignLeft
            level: 3
            text: i18n("Diagnostics")
        }
        Label {
            text: i18n("Log file: $XDG_STATE_HOME/grexa/grexa.log\nOverride verbosity with the GREXA_LOG environment variable.")
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }
    }
}
