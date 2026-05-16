// AI Search Chat — owned by an individual search tab. Hides the result
// list while active; flows messages through `grexa_ai::AiSearchClient`.
//
// States:
//   - disabled (DefaultSettings.ai_search_enabled = false): banner + Settings link
//   - empty (no conversation yet): "Click AI to start"
//   - loading (request in flight): spinner + cancel
//   - error: typed error from AiSearchResponse.error_message
//   - active: turn-by-turn message list + input + send button

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

ColumnLayout {
    id: panel
    spacing: Kirigami.Units.smallSpacing

    property bool aiEnabled: false
    property bool busy: false
    property string errorMessage: ""

    // Disabled banner
    Kirigami.InlineMessage {
        Layout.fillWidth: true
        visible: !panel.aiEnabled
        type: Kirigami.MessageType.Information
        text: i18n("AI search is off. Enable it in Settings → AI Search.")
        actions: Kirigami.Action {
            text: i18n("Open Settings")
            icon.name: "settings-configure"
        }
    }

    // Error banner
    Kirigami.InlineMessage {
        Layout.fillWidth: true
        visible: panel.errorMessage.length > 0
        type: Kirigami.MessageType.Error
        text: panel.errorMessage
    }

    // Conversation list
    Frame {
        Layout.fillWidth: true
        Layout.fillHeight: true
        enabled: panel.aiEnabled

        ListView {
            id: messages
            anchors.fill: parent
            clip: true
            model: ListModel { id: messageModel }

            delegate: Kirigami.SubtitleDelegate {
                text: model.role === "user" ? i18n("You") : i18n("Assistant")
                subtitle: model.content
            }

            Kirigami.PlaceholderMessage {
                anchors.centerIn: parent
                visible: messageModel.count === 0 && !panel.busy && panel.aiEnabled
                text: i18n("Click AI to start an AI-assisted search discussion.")
            }

            BusyIndicator {
                anchors.centerIn: parent
                running: panel.busy
                visible: panel.busy
            }
        }
    }

    // Input row
    RowLayout {
        Layout.fillWidth: true
        enabled: panel.aiEnabled && !panel.busy
        spacing: Kirigami.Units.smallSpacing

        TextArea {
            id: input
            Layout.fillWidth: true
            placeholderText: i18n("Ask the AI…")
            wrapMode: TextEdit.Wrap
        }
        Button {
            text: i18n("Send")
            icon.name: "document-send"
            enabled: input.text.trim().length > 0
            onClicked: {
                // Rust hook: emit `sendChatTurn(input.text)`.
                input.text = ""
            }
        }
        Button {
            visible: panel.busy
            text: i18n("Cancel")
            icon.name: "process-stop"
        }
    }
}
