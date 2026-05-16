// Search workspace — the primary tab. Will host:
//
//   - Path picker (with recent-path AutoSuggest + browse-on-Enter)
//   - Search-term input (with Enter ↔ trigger)
//   - Replace input (hidden by default; shown when toggle is on)
//   - Mode selectors: Text / Regex, Content / Files
//   - Target selector: Local / Docker / Podman
//   - Command strip: Search • Stop • AI • Replace • Reset • Filter Options
//                    • Profiles • History • Export
//   - Filter pane (collapsible): respect-gitignore, case, hidden,
//     subfolders, binary, symlinks, match-files glob, exclude-dirs,
//     size limit
//   - Results: virtualized ListView with sticky headers + sortable
//     columns; Content + Files modes share the model
//
// The QML side is a placeholder until the Rust controllers expose
// signals; today this is a static description that points the user
// at the CLI.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    title: i18n("Search")

    ColumnLayout {
        spacing: Kirigami.Units.largeSpacing
        width: parent.width

        Kirigami.Heading {
            level: 1
            text: i18n("Grexa")
            Layout.alignment: Qt.AlignHCenter
        }

        Kirigami.Heading {
            level: 4
            text: i18n("Fast Linux file content search")
            Layout.alignment: Qt.AlignHCenter
            opacity: 0.7
        }

        Kirigami.PlaceholderMessage {
            Layout.fillWidth: true
            text: i18n("GUI shell is in active development.")
            explanation: i18n(
                "The Rust core, replace pipeline, container adapter, AI client, " +
                "and CLI are all working today. Run `grexa-cli --help` for the " +
                "scriptable surface; the QML search page lands in Phase 4 of " +
                "PLAN.md.")
            helpfulAction: Kirigami.Action {
                text: i18n("Open the CLI usage docs")
                icon.name: "help-contents"
            }
        }
    }
}
