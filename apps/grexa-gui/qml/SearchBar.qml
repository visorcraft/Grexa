// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// The unified search bar — one horizontal stripe combining scope
// (path) selector, term input, flag chips, and the primary action.
// Modeled after a browser address bar / Raycast command palette so
// search reads as a single intention rather than a stacked form.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Rectangle {
    id: root

    property alias pathText: pathField.editText
    property alias termText: termField.text
    property alias recentPathsModel: pathField.model
    property bool regexEnabled: false
    property bool caseSensitive: false
    property bool busy: false
    signal submitted()
    signal browse()

    implicitHeight: 52
    radius: app.tokens.radiusInput
    color: app.tokens.surface1
    border.color: pathField.activeFocus || termField.activeFocus
        ? app.tokens.accent : app.tokens.separator
    border.width: 1

    // Focus ring (outer glow)
    Rectangle {
        anchors.fill: parent
        anchors.margins: -3
        radius: parent.radius + 3
        color: "transparent"
        border.color: app.tokens.accentRing
        border.width: 2
        opacity: pathField.activeFocus || termField.activeFocus ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: app.tokens.durationSnap } }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: app.tokens.spaceM
        anchors.rightMargin: app.tokens.spaceS
        spacing: 0

        Kirigami.Icon {
            source: "folder-symbolic"
            implicitWidth: 16
            implicitHeight: 16
            color: Kirigami.Theme.textColor
            opacity: 0.55
            isMask: true
        }

        Controls.ComboBox {
            id: pathField
            editable: true
            textRole: "pathText"
            Layout.preferredWidth: 280
            Layout.fillHeight: true
            Layout.leftMargin: app.tokens.spaceS
            font.pixelSize: app.tokens.textBody
            background: null
            Keys.onReturnPressed: root.submitted()
        }

        Controls.Button {
            flat: true
            icon.name: "folder-open-symbolic"
            display: Controls.AbstractButton.IconOnly
            Layout.preferredWidth: 28
            Layout.preferredHeight: 28
            Controls.ToolTip.text: qsTr("Browse for a folder")
            Controls.ToolTip.visible: hovered
            onClicked: root.browse()
        }

        // Divider between scope and term
        Rectangle {
            Layout.preferredWidth: 1
            Layout.fillHeight: true
            Layout.topMargin: app.tokens.spaceS
            Layout.bottomMargin: app.tokens.spaceS
            Layout.leftMargin: app.tokens.spaceM
            Layout.rightMargin: app.tokens.spaceM
            color: app.tokens.separator
        }

        Kirigami.Icon {
            source: "edit-find-symbolic"
            implicitWidth: 16
            implicitHeight: 16
            color: Kirigami.Theme.textColor
            opacity: 0.55
            isMask: true
        }

        Controls.TextField {
            id: termField
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceS
            placeholderText: qsTr("Search code, configs, anything…")
            font.pixelSize: app.tokens.textBody
            background: null
            Keys.onReturnPressed: root.submitted()
        }

        // Flag chips
        FlagChip {
            id: regexChip
            label: ".*"
            tooltip: qsTr("Regex")
            checked: root.regexEnabled
            onToggled: root.regexEnabled = checked
        }
        FlagChip {
            id: caseChip
            label: "Aa"
            tooltip: qsTr("Case-sensitive")
            checked: root.caseSensitive
            onToggled: root.caseSensitive = checked
        }

        // Vertical divider before primary action
        Rectangle {
            Layout.preferredWidth: 1
            Layout.fillHeight: true
            Layout.topMargin: app.tokens.spaceS
            Layout.bottomMargin: app.tokens.spaceS
            Layout.leftMargin: app.tokens.spaceS
            Layout.rightMargin: app.tokens.spaceS
            color: app.tokens.separator
        }

        // Primary action — embedded in the bar like browser "Go"
        Controls.Button {
            Layout.preferredHeight: 36
            Layout.alignment: Qt.AlignVCenter
            leftPadding: app.tokens.spaceL
            rightPadding: app.tokens.spaceL
            enabled: !root.busy && pathField.editText.length > 0 && termField.text.length > 0
            onClicked: root.submitted()
            contentItem: Row {
                spacing: app.tokens.spaceS
                Kirigami.Icon {
                    source: root.busy ? "view-refresh-symbolic" : "edit-find-symbolic"
                    implicitWidth: 14
                    implicitHeight: 14
                    color: "white"
                    isMask: true
                    anchors.verticalCenter: parent.verticalCenter
                    RotationAnimator on rotation {
                        running: root.busy
                        from: 0; to: 360; duration: 900
                        loops: Animation.Infinite
                    }
                }
                Controls.Label {
                    text: root.busy ? qsTr("Searching") : qsTr("Search")
                    color: "white"
                    font.weight: app.tokens.weightMedium
                    font.pixelSize: app.tokens.textBody
                    anchors.verticalCenter: parent.verticalCenter
                }
                Controls.Label {
                    text: "↵"
                    color: "white"
                    opacity: 0.7
                    font.pixelSize: app.tokens.textCaption
                    font.family: app.tokens.monoFamily
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            background: Rectangle {
                radius: app.tokens.radiusButton
                color: !parent.enabled ? Qt.darker(app.tokens.accent, 1.6)
                    : parent.pressed ? app.tokens.accentPressed
                    : parent.hovered ? app.tokens.accentHover
                    : app.tokens.accent
                opacity: parent.enabled ? 1.0 : 0.45
                Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
            }
        }
    }
}
