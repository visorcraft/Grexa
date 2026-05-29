// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts

Controls.Dialog {
    id: dialog

    property string bodyText: ""
    property string detailText: ""
    property string titleText: qsTr("License Text")

    function openLicenseText() {
        dialog.openDocument(
            qsTr("GNU General Public License v3"),
            qsTr("GPL-3.0-only license text bundled with Grexa."),
            app.settingsController.gplLicenseText()
        )
    }

    function openDocument(title, detail, text) {
        dialog.titleText = title
        dialog.detailText = detail
        dialog.bodyText = text && text.length > 0
            ? text
            : qsTr("No bundled license text is available.")
        dialog.open()
    }

    modal: true
    title: dialog.titleText
    standardButtons: Controls.Dialog.Close
    closePolicy: Controls.Popup.CloseOnEscape | Controls.Popup.CloseOnPressOutside
    width: Math.min(app.width - app.tokens.spaceXXL * 2, 920)
    height: Math.min(app.height - app.tokens.spaceXXL * 2, 680)
    x: Math.max(app.tokens.spaceXL, (app.width - width) / 2)
    y: Math.max(app.tokens.spaceXL, (app.height - height) / 2)

    palette.window:          app.tokens.surface1
    palette.windowText:      app.tokens.textPrimary
    palette.base:            app.tokens.surface0
    palette.text:            app.tokens.textPrimary
    palette.button:          app.tokens.surface2
    palette.buttonText:      app.tokens.textPrimary
    palette.highlight:       app.tokens.accent
    palette.highlightedText: app.tokens.accentText

    background: Rectangle {
        color: app.tokens.surface1
        radius: app.tokens.radiusCard
        border.color: app.tokens.separatorStrong
        border.width: 1
    }

    contentItem: ColumnLayout {
        spacing: app.tokens.spaceM

        Controls.Label {
            Layout.fillWidth: true
            text: dialog.detailText
            wrapMode: Text.WordWrap
            font.pixelSize: app.tokens.textBody
            font.family: app.tokens.sansFamily
            opacity: 0.7
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: app.tokens.radiusInput
            color: app.tokens.surface0
            border.color: app.tokens.separator
            border.width: 1
            clip: true

            Controls.ScrollView {
                anchors.fill: parent
                anchors.margins: app.tokens.spaceM
                clip: true

                Controls.TextArea {
                    text: dialog.bodyText
                    readOnly: true
                    selectByMouse: true
                    wrapMode: TextEdit.Wrap
                    color: app.tokens.textPrimary
                    selectedTextColor: app.tokens.accentText
                    selectionColor: app.tokens.accent
                    font.pixelSize: app.tokens.textCaption + 1
                    font.family: app.tokens.monoFamily
                    background: Rectangle { color: "transparent" }
                }
            }
        }
    }
}
