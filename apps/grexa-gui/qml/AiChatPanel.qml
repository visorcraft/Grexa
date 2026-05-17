// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// AI Search Chat — bound to `app.aiController`. Conversation rendered
// as Mailspring-class chat bubbles: user messages right-aligned in
// an accent-tinted bubble; assistant messages left-aligned in a
// surface bubble with a soft tail. The composer at the bottom is
// a multi-line input with a circular accent send button.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

ColumnLayout {
    id: panel
    spacing: app.tokens.spaceM

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

    // -- Conversation pane -----------------------------------------
    Rectangle {
        Layout.fillWidth: true
        Layout.fillHeight: true
        color: "transparent"
        enabled: panel.aiEnabled

        ListView {
            id: messages
            anchors.fill: parent
            anchors.margins: app.tokens.spaceXS
            clip: true
            spacing: app.tokens.spaceS
            model: ListModel { id: messageModel }
            verticalLayoutDirection: ListView.TopToBottom
            onCountChanged: Qt.callLater(() => positionViewAtEnd())

            // -- Bubble delegate
            delegate: Item {
                width: messages.width
                implicitHeight: bubble.implicitHeight + app.tokens.spaceXS

                readonly property bool isUser: model.role === "user"

                Row {
                    id: bubble
                    spacing: app.tokens.spaceS
                    anchors.right: parent.isUser ? parent.right : undefined
                    anchors.left: parent.isUser ? undefined : parent.left
                    anchors.rightMargin: parent.isUser ? app.tokens.spaceS : 0
                    anchors.leftMargin: parent.isUser ? 0 : app.tokens.spaceS
                    width: Math.min(implicitWidth, parent.width * 0.82)

                    // Assistant avatar — only on assistant side
                    Rectangle {
                        visible: !parent.parent.isUser
                        width: 28; height: 28
                        radius: 14
                        color: app.tokens.accentMute
                        border.color: Qt.rgba(app.tokens.accent.r, app.tokens.accent.g, app.tokens.accent.b, 0.4)
                        border.width: 1
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: 2
                        Kirigami.Icon {
                            anchors.centerIn: parent
                            source: "tools-symbolic"
                            implicitWidth: 16
                            implicitHeight: 16
                            color: app.tokens.accent
                            isMask: true
                        }
                    }

                    Rectangle {
                        radius: app.tokens.radiusCard
                        // Subtle tail-cutting corner — flatten the bottom
                        // corner closest to the speaker.
                        color: parent.parent.isUser ? app.tokens.accent : app.tokens.surface2
                        border.color: parent.parent.isUser ? "transparent" : app.tokens.separator
                        border.width: parent.parent.isUser ? 0 : 1
                        implicitWidth: Math.min(messageLabel.implicitWidth + app.tokens.spaceL * 2,
                                                messages.width * 0.78)
                        implicitHeight: messageLabel.implicitHeight + app.tokens.spaceM * 2

                        // Soft inner highlight on the user bubble
                        Rectangle {
                            visible: parent.parent.parent.isUser
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.margins: 1
                            height: parent.height * 0.5
                            radius: parent.radius - 1
                            gradient: Gradient {
                                GradientStop { position: 0.0; color: Qt.rgba(1, 1, 1, 0.15) }
                                GradientStop { position: 1.0; color: Qt.rgba(1, 1, 1, 0.0) }
                            }
                        }

                        Controls.Label {
                            id: messageLabel
                            anchors.fill: parent
                            anchors.leftMargin: app.tokens.spaceL
                            anchors.rightMargin: app.tokens.spaceL
                            anchors.topMargin: app.tokens.spaceM
                            anchors.bottomMargin: app.tokens.spaceM
                            text: model.content
                            wrapMode: Text.Wrap
                            font.family: app.tokens.sansFamily
                            font.pixelSize: app.tokens.textBody
                            color: parent.parent.parent.isUser ? "white" : Kirigami.Theme.textColor
                        }
                    }
                }
            }

            Kirigami.PlaceholderMessage {
                anchors.centerIn: parent
                width: parent.width - 2 * app.tokens.spaceXL
                visible: messageModel.count === 0 && !controller.busy && panel.aiEnabled
                icon.name: "tools-symbolic"
                text: qsTr("Ask AI for help shaping a search")
                explanation: qsTr("Describe what you're looking for in plain English. The model will suggest a path, term, and flags.")
            }

            Controls.BusyIndicator {
                anchors.centerIn: parent
                running: controller.busy
                visible: controller.busy
            }
        }
    }

    // -- Composer ---------------------------------------------------
    Rectangle {
        Layout.fillWidth: true
        Layout.minimumHeight: 56
        Layout.preferredHeight: composer.implicitHeight + app.tokens.spaceM * 2
        radius: app.tokens.radiusInput
        color: app.tokens.surface1
        border.color: input.activeFocus ? app.tokens.accent : app.tokens.separatorStrong
        border.width: 1
        enabled: panel.aiEnabled && !controller.busy
        Behavior on border.color { ColorAnimation { duration: app.tokens.durationSnap } }

        RowLayout {
            id: composer
            anchors.fill: parent
            anchors.leftMargin: app.tokens.spaceM
            anchors.rightMargin: app.tokens.spaceXS
            anchors.topMargin: app.tokens.spaceXS
            anchors.bottomMargin: app.tokens.spaceXS
            spacing: app.tokens.spaceS

            Controls.ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Controls.TextArea {
                    id: input
                    placeholderText: qsTr("Ask the AI…")
                    wrapMode: TextEdit.Wrap
                    font.family: app.tokens.sansFamily
                    font.pixelSize: app.tokens.textBody
                    background: null
                    Keys.onReturnPressed: (event) => {
                        if (event.modifiers & Qt.ShiftModifier) {
                            event.accepted = false   // let newline through
                        } else {
                            sendButton.send()
                            event.accepted = true
                        }
                    }
                }
            }

            // Circular accent send button
            Rectangle {
                id: sendButton
                Layout.preferredWidth: 40
                Layout.preferredHeight: 40
                Layout.alignment: Qt.AlignBottom
                Layout.bottomMargin: 1
                radius: 20
                enabled: input.text.trim().length > 0 && !controller.busy
                color: enabled
                    ? (sendMouse.containsPress ? app.tokens.accentPressed
                        : sendMouse.containsMouse ? app.tokens.accentHover
                        : app.tokens.accent)
                    : Qt.darker(app.tokens.accent, 1.7)
                opacity: enabled ? 1.0 : 0.45
                Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }

                function send() {
                    const prompt = input.text.trim()
                    if (prompt.length === 0) return
                    messageModel.append({ role: "user", content: prompt })
                    controller.sendMessage(prompt)
                    input.text = ""
                }

                Kirigami.Icon {
                    anchors.centerIn: parent
                    source: "document-send"
                    implicitWidth: 18
                    implicitHeight: 18
                    color: "white"
                    isMask: true
                }

                MouseArea {
                    id: sendMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: parent.enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: parent.send()
                }
            }
        }
    }
}
