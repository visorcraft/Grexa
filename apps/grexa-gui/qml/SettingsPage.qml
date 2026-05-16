// Settings page — Phase 10 destination.
//
// Sections (in display order):
//   1. Appearance — theme preference (System / Light / Dark / 9 high-contrast variants)
//   2. Language — UI locale dropdown (en / de / ja / …; populated from grexa-i18n)
//   3. Search defaults — every SearchOptions field with a default toggle
//   4. Filter defaults — match-files, exclude-dirs, size unit
//   5. Context preview — before / after line counts (clamped 1..=20)
//   6. Containers — enable Docker / Podman target dropdown
//   7. AI Search — endpoint, model, opt-in toggle, test button, keyring status
//   8. Backup / Restore — export JSON, import JSON, restore defaults
//   9. Diagnostics — log path, GREXA_LOG override
//   10. About — passthrough to the About page
//
// Every section binds to a single Rust controller; saves are instant.

import QtQuick
import QtQuick.Controls
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    title: i18n("Settings")

    Kirigami.PlaceholderMessage {
        anchors.centerIn: parent
        width: parent.width * 0.8
        text: i18n("Settings page placeholder.")
        explanation: i18n(
            "Storage, import/export, and field semantics are all done in " +
            "`grexa-core::storage`. The QML form lands in Phase 10.")
    }
}
