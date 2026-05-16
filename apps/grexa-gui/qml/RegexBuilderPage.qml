// Regex Builder — Phase 9 destination.
//
// Two-pane layout:
//   - Top pane: sample text + pattern input + toggle row
//     (case-insensitive, multiline, global).
//   - Bottom pane: live match list with capture-group highlights and
//     a "Send to Search" button.
//
// Backed by `grexa_core::pattern::PatternEngine` so the two-engine
// cascade is honored — patterns that need `fancy-regex` automatically
// fall through.

import QtQuick
import QtQuick.Controls
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    title: i18n("Regex Builder")

    Kirigami.PlaceholderMessage {
        anchors.centerIn: parent
        width: parent.width * 0.8
        text: i18n("Regex Builder placeholder.")
        explanation: i18n(
            "Lands in Phase 9. The Rust pattern engine, fixtures, and " +
            "fancy-regex fallback already work — only the QML pane is pending.")
    }
}
