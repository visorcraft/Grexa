// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// AI Search Chat — bound to `app.aiController`. Holds a local
// conversation list; each Send pushes the user message into the
// model, fires the controller, and on `responseReady` appends the
// assistant reply.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

ColumnLayout {
    id: panel
    spacing: Kirigami.Units.smallSpacing

    property var controller: app.aiController
    property bool aiEnabled: app.settingsController.aiSearchEnabled

    Connections {
        target: controller
        function onResponseReady() {
            const text = controller.lastResponse
            if (text.length > 0) {
                messageModel.append({ role: "assistant", content: text })
            }
        }
    }

    Kirigami.InlineMessage {
        Layout.fillWidth: true
        visible: !panel.aiEnabled
        type: Kirigami.MessageType.Information
        text: qsTr("AI search is off. Enable it in Settings → AI Search.")
    }

    Kirigami.InlineMessage {
        Layout.fillWidth: true
        visible: controller.lastError.length > 0
        type: Kirigami.MessageType.Error
        text: controller.lastError
    }

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
                text: model.role === "user" ? qsTr("You") : qsTr("Assistant")
                subtitle: model.content
            }

            Kirigami.PlaceholderMessage {
                anchors.centerIn: parent
                visible: messageModel.count === 0 && !controller.busy && panel.aiEnabled
                text: qsTr("Ask the AI for help shaping a search.")
            }

            BusyIndicator {
                anchors.centerIn: parent
                running: controller.busy
                visible: controller.busy
            }
        }
    }

    RowLayout {
        Layout.fillWidth: true
        enabled: panel.aiEnabled && !controller.busy
        spacing: Kirigami.Units.smallSpacing

        TextArea {
            id: input
            Layout.fillWidth: true
            placeholderText: qsTr("Ask the AI…")
            wrapMode: TextEdit.Wrap
        }
        Button {
            text: qsTr("Send")
            icon.name: "document-send"
            enabled: input.text.trim().length > 0
            onClicked: {
                const prompt = input.text
                messageModel.append({ role: "user", content: prompt })
                controller.sendMessage(prompt)
                input.text = ""
            }
        }
    }
}
