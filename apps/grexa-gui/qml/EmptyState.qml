// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Centered empty-state — Mailspring-class composition.
// Generous 144px illustration, semibold display title, 13px helper
// in muted text, and a Flow of monospace suggestion chips with a
// trailing arrow. The vertical rhythm leans on the spacing scale
// so it never feels cramped.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Item {
    id: root
    property string illustration: "qrc:/qt/qml/com/visorcraft/Grexa/resources/empty-search.svg"
    property string title: ""
    property string explanation: ""
    property alias chipsModel: chipRepeater.model
    signal chipClicked(int index, var data)

    ColumnLayout {
        anchors.centerIn: parent
        width: Math.min(480, parent.width * 0.7)
        spacing: app.tokens.spaceXL

        Image {
            source: root.illustration
            sourceSize.width: 168
            sourceSize.height: 168
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: 168
            Layout.preferredHeight: 168
            smooth: true
        }

        ColumnLayout {
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
            spacing: app.tokens.spaceS

            Controls.Label {
                text: root.title
                font.pixelSize: 22
                font.weight: app.tokens.weightSemibold
                font.family: app.tokens.sansFamily
                horizontalAlignment: Text.AlignHCenter
                Layout.alignment: Qt.AlignHCenter
            }

            Controls.Label {
                text: root.explanation
                font.pixelSize: app.tokens.textBody + 1
                font.family: app.tokens.sansFamily
                opacity: 0.58
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        // -- Suggestion chips
        ColumnLayout {
            visible: chipRepeater.count > 0
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
            Layout.topMargin: app.tokens.spaceM
            spacing: app.tokens.spaceS

            Controls.Label {
                text: qsTr("TRY ONE OF THESE")
                font.pixelSize: 10
                font.weight: app.tokens.weightSemibold
                font.letterSpacing: 0
                opacity: 0.4
                horizontalAlignment: Text.AlignHCenter
                Layout.alignment: Qt.AlignHCenter
            }

            Flow {
                Layout.alignment: Qt.AlignHCenter
                Layout.fillWidth: true
                spacing: app.tokens.spaceS

                Repeater {
                    id: chipRepeater
                    delegate: Rectangle {
                        radius: app.tokens.radiusPill
                        color: chipMouse.containsPress ? app.tokens.surface2
                            : chipMouse.containsMouse ? app.tokens.accentMute
                            : app.tokens.surface1
                        border.color: chipMouse.containsMouse ? app.tokens.accent : app.tokens.separatorStrong
                        border.width: 1
                        implicitHeight: 32
                        implicitWidth: chipRow.implicitWidth + app.tokens.spaceL * 2
                        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
                        Behavior on border.color { ColorAnimation { duration: app.tokens.durationSnap } }

                        Row {
                            id: chipRow
                            anchors.centerIn: parent
                            spacing: app.tokens.spaceS
                            Controls.Label {
                                text: modelData.label
                                font.pixelSize: app.tokens.textCaption + 1
                                font.family: app.tokens.monoFamily
                                color: chipMouse.containsMouse ? app.tokens.accent : Kirigami.Theme.textColor
                                opacity: chipMouse.containsMouse ? 1.0 : 0.85
                                anchors.verticalCenter: parent.verticalCenter
                                Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
                            }
                            Kirigami.Icon {
                                source: "go-next-symbolic"
                                implicitWidth: 12
                                implicitHeight: 12
                                opacity: chipMouse.containsMouse ? 0.9 : 0.4
                                color: chipMouse.containsMouse ? app.tokens.accent : Kirigami.Theme.textColor
                                isMask: true
                                anchors.verticalCenter: parent.verticalCenter
                                Behavior on opacity { NumberAnimation { duration: app.tokens.durationSnap } }
                            }
                        }

                        MouseArea {
                            id: chipMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.chipClicked(index, modelData)
                        }
                    }
                }
            }
        }
    }
}
