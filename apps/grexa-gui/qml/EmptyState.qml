// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Centered empty-state. Compact 96px illustration, semibold title,
// 13px helper, and a row of monospace-labeled suggestion chips with
// a trailing arrow.

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
        width: Math.min(420, parent.width * 0.66)
        spacing: app.tokens.spaceL

        Image {
            source: root.illustration
            sourceSize.width: 96
            sourceSize.height: 96
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: 96
            Layout.preferredHeight: 96
            opacity: 0.78
            smooth: true
        }

        Controls.Label {
            text: root.title
            font.pixelSize: 18
            font.weight: app.tokens.weightBold
            horizontalAlignment: Text.AlignHCenter
            Layout.alignment: Qt.AlignHCenter
        }

        Controls.Label {
            text: root.explanation
            font.pixelSize: app.tokens.textBody
            opacity: 0.55
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        Flow {
            visible: chipRepeater.count > 0
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
            Layout.topMargin: app.tokens.spaceS
            spacing: app.tokens.spaceS

            Repeater {
                id: chipRepeater
                delegate: Rectangle {
                    radius: app.tokens.radiusPill
                    color: chipMouse.containsPress ? app.tokens.surface2
                        : chipMouse.containsMouse ? app.tokens.surface1
                        : Qt.rgba(0, 0, 0, 0)
                    border.color: chipMouse.containsMouse ? app.tokens.separatorStrong : app.tokens.separator
                    border.width: 1
                    implicitHeight: 26
                    implicitWidth: chipRow.implicitWidth + app.tokens.spaceL * 2
                    Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }

                    Row {
                        id: chipRow
                        anchors.centerIn: parent
                        spacing: app.tokens.spaceS
                        Controls.Label {
                            text: modelData.label
                            font.pixelSize: app.tokens.textCaption
                            font.family: app.tokens.monoFamily
                            opacity: 0.85
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        Kirigami.Icon {
                            source: "go-next-symbolic"
                            implicitWidth: 12
                            implicitHeight: 12
                            opacity: 0.45
                            color: Kirigami.Theme.textColor
                            isMask: true
                            anchors.verticalCenter: parent.verticalCenter
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
