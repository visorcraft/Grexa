// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Controls.ItemDelegate {
    id: card
    property string collection: ""
    property string recordPath: ""
    property var controller: null

    property var fields: ({})
    property bool loaded: false

    onClicked: {
        if (!loaded && controller) {
            var raw = controller.recordFrontmatter(collection, recordPath);
            try {
                fields = JSON.parse(raw);
            } catch (e) {
                fields = {};
            }
            loaded = true;
        }
    }

    background: Rectangle {
        color: card.down ? Kirigami.Theme.alternateBackgroundColor
                         : Kirigami.Theme.backgroundColor
        radius: Kirigami.Units.cornerRadius
        border.width: 1
        border.color: Qt.alpha(Kirigami.Theme.textColor, 0.15)
    }

    contentItem: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            Layout.fillWidth: true

            Controls.Label {
                text: card.recordPath
                font.bold: true
                elide: Text.ElideRight
                Layout.fillWidth: true
            }

            Kirigami.Icon {
                source: card.loaded ? "go-down" : "go-right"
                implicitWidth: Kirigami.Units.iconSizes.small
                implicitHeight: Kirigami.Units.iconSizes.small
                opacity: 0.6
            }
        }

        // Frontmatter fields (visible after first click)
        Repeater {
            model: {
                if (!card.loaded) return [];
                return Object.keys(card.fields)
                    .filter(k => card.fields[k] !== null && card.fields[k] !== undefined)
                    .map(k => ({key: k, value: String(card.fields[k])}));
            }

            delegate: RowLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Kirigami.Units.largeSpacing

                Controls.Label {
                    text: modelData.key + ":"
                    color: Kirigami.Theme.disabledTextColor
                    Layout.preferredWidth: 100
                }
                Controls.Label {
                    text: modelData.value
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                }
            }
        }
    }
}
