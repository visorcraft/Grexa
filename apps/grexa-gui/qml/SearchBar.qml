// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// The unified search bar — one horizontal stripe combining scope
// (path) selector, term input, flag chips, and the primary action.
// Modeled after a browser address bar / Raycast command palette so
// search reads as a single intention rather than a stacked form.
//
// Visual treatment is Mailspring-class: soft elevation via stacked
// faint shadow rectangles, generous interior padding, refined
// focus ring that lifts on input focus.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Item {
    id: root

    property alias pathText: pathField.editText
    property alias termText: termField.text
    property alias recentPathsModel: pathField.model
    property bool regexEnabled: false
    property bool caseSensitive: false
    property bool wholeWordEnabled: false
    property bool busy: false
    signal submitted()
    signal browse()

    function focusTermField() { termField.forceActiveFocus() }

    implicitHeight: 56

    // -- Soft elevation stack (drop-shadow fake) -----------------
    Rectangle {
        anchors.fill: bar
        anchors.topMargin: 3
        anchors.bottomMargin: -3
        radius: bar.radius + 1
        color: app.tokens.shadowFar
        z: -2
    }
    Rectangle {
        anchors.fill: bar
        anchors.topMargin: 1
        anchors.bottomMargin: -1
        radius: bar.radius
        color: app.tokens.shadowNear
        z: -1
    }

    Rectangle {
        id: bar
        anchors.fill: parent
        radius: app.tokens.radiusInput + 2
        color: app.tokens.surface2
        border.color: pathField.activeFocus || termField.activeFocus
            ? app.tokens.accent : app.tokens.separatorStrong
        border.width: 1.2
        Behavior on border.color { ColorAnimation { duration: app.tokens.durationSnap } }

        // Focus ring (outer glow) — lifts on focus.
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
            anchors.leftMargin: app.tokens.spaceL
            anchors.rightMargin: app.tokens.spaceS
            spacing: 0

            Kirigami.Icon {
                source: "folder-symbolic"
                implicitWidth: 16
                implicitHeight: 16
                color: pathField.activeFocus ? app.tokens.accent : Kirigami.Theme.textColor
                opacity: pathField.activeFocus ? 1.0 : 0.55
                isMask: true
                Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
            }

            AppComboBox {
                id: pathField
                editable: true
                textRole: "pathText"
                // Preferred 300 but allow shrinking to a usable
                // ~140 so a narrow window still shows the term
                // input and primary action without clipping.
                Layout.preferredWidth: 300
                Layout.minimumWidth: 140
                Layout.fillHeight: true
                Layout.leftMargin: app.tokens.spaceS
                font.pixelSize: app.tokens.textBody
                font.family: app.tokens.sansFamily
                background: null
                Keys.onReturnPressed: root.submitted()

                // Each dropdown row gets a small × that removes the
                // path from the recent-paths store without picking it.
                // Hover-only so the row reads cleanly at rest.
                delegate: Controls.ItemDelegate {
                    id: pathRow
                    width: pathField.width
                    hoverEnabled: true
                    contentItem: RowLayout {
                        spacing: app.tokens.spaceS
                        Controls.Label {
                            Layout.fillWidth: true
                            text: pathText
                            elide: Text.ElideMiddle
                            font.pixelSize: app.tokens.textBody
                        }
                        AppFlatButton {
                            icon.name: "edit-delete-symbolic"
                            icon.color: app.tokens.textPrimary
                            display: Controls.AbstractButton.IconOnly
                            // Hover-only "forget" affordance. Bind to
                            // the delegate's own `hovered` via the
                            // explicit `pathRow` id — `parent.parent`
                            // is brittle across Qt minor versions
                            // because of how `contentItem` is
                            // reparented.
                            visible: pathRow.hovered
                            Controls.ToolTip.text: qsTr("Forget this path")
                            Controls.ToolTip.visible: hovered
                            onClicked: {
                                app.searchController.removeRecentPath(pathText)
                                pathField.popup.close()
                            }
                        }
                    }
                    onClicked: {
                        pathField.currentIndex = index
                        pathField.editText = pathText
                        pathField.popup.close()
                    }
                }
            }

            AppFlatButton {
                icon.name: "folder-open-symbolic"
                icon.color: app.tokens.textPrimary
                display: Controls.AbstractButton.IconOnly
                Layout.preferredWidth: 32
                Layout.preferredHeight: 32
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
                color: termField.activeFocus ? app.tokens.accent : Kirigami.Theme.textColor
                opacity: termField.activeFocus ? 1.0 : 0.55
                isMask: true
                Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
            }

            AppTextField {
                id: termField
                Layout.fillWidth: true
                Layout.leftMargin: app.tokens.spaceS
                placeholderText: qsTr("Search code, configs, anything…")
                font.pixelSize: app.tokens.textBody
                font.family: app.tokens.sansFamily
                background: null
                Keys.onReturnPressed: root.submitted()
                Accessible.name: qsTr("Search term")
            }

            // Flag chips
            FlagChip {
                id: regexChip
                label: ".*"
                tooltip: qsTr("Regex")
                active: root.regexEnabled
                onToggled: root.regexEnabled = !root.regexEnabled
            }
            FlagChip {
                id: caseChip
                label: "Aa"
                tooltip: qsTr("Case-sensitive")
                active: root.caseSensitive
                onToggled: root.caseSensitive = !root.caseSensitive
            }
            FlagChip {
                id: wholeWordChip
                label: qsTr("W")
                tooltip: qsTr("Whole word")
                active: root.wholeWordEnabled
                onToggled: root.wholeWordEnabled = !root.wholeWordEnabled
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

            // Primary action — embedded in the bar like browser "Go".
            // Uses a subtle vertical gradient so it reads as a
            // raised, premium element.
            Controls.Button {
                id: primaryAction
                Layout.preferredHeight: 38
                Layout.alignment: Qt.AlignVCenter
                Accessible.name: qsTr("Search")
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
                        font.weight: app.tokens.weightSemibold
                        font.pixelSize: app.tokens.textBody
                        font.family: app.tokens.sansFamily
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Controls.Label {
                        text: "↵"
                        color: "white"
                        opacity: 0.65
                        font.pixelSize: app.tokens.textCaption
                        font.family: app.tokens.monoFamily
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
                background: Rectangle {
                    radius: app.tokens.radiusButton
                    gradient: Gradient {
                        GradientStop {
                            position: 0.0
                            color: !primaryAction.enabled ? Qt.darker(app.tokens.accent, 1.6)
                                : primaryAction.pressed ? app.tokens.accentPressed
                                : primaryAction.hovered ? app.tokens.accentHover
                                : app.tokens.accent
                        }
                        GradientStop {
                            position: 1.0
                            color: !primaryAction.enabled ? Qt.darker(app.tokens.accent, 1.8)
                                : primaryAction.pressed ? app.tokens.accentDeep
                                : primaryAction.hovered ? app.tokens.accent
                                : app.tokens.accentPressed
                        }
                    }
                    opacity: primaryAction.enabled ? 1.0 : 0.45
                    // Inner highlight — Mailspring/iOS-class soft sheen
                    Rectangle {
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: 1
                        height: parent.height * 0.5
                        radius: parent.radius - 1
                        gradient: Gradient {
                            GradientStop { position: 0.0; color: Qt.rgba(1, 1, 1, 0.18) }
                            GradientStop { position: 1.0; color: Qt.rgba(1, 1, 1, 0.0) }
                        }
                    }
                }
            }
        }
    }
}
