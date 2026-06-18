// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.visorcraft.Grexa 1.0

Kirigami.ScrollablePage {
    id: page
    title: i18n("Database")

    property string selectedCollection: ""
    property var collections: []
    property var records: []

    DbController {
        id: db
    }

    ColumnLayout {
        width: page.width
        spacing: Kirigami.Units.largeSpacing

        // --- Open database ---
        Kirigami.FormLayout {
            Layout.fillWidth: true

            Controls.TextField {
                Kirigami.FormData.label: i18n("Database path:")
                Layout.fillWidth: true
                placeholderText: "~/my-notes"
                text: db.dbPath
                id: pathInput
            }

            Controls.Button {
                text: i18n("Open")
                Layout.fillWidth: true
                onClicked: {
                    if (db.openDb(pathInput.text)) {
                        var names = db.collectionNames().split("\n").filter(n => n.length > 0);
                        page.collections = names;
                        page.records = [];
                        page.selectedCollection = "";
                    }
                }
            }
        }

        Kirigami.Separator {
            Layout.fillWidth: true
        }

        // --- Collections ---
        Kirigami.Heading {
            text: i18n("Collections")
            level: 2
            visible: db.isOpen
        }

        Repeater {
            model: page.collections
            visible: db.isOpen

            delegate: Controls.Button {
                text: modelData
                Layout.fillWidth: true
                flat: true
                highlighted: page.selectedCollection === modelData
                onClicked: {
                    page.selectedCollection = modelData;
                    var raw = db.recordPaths(modelData);
                    page.records = raw.split("\n").filter(r => r.length > 0);
                }
            }
        }

        Kirigami.Separator {
            Layout.fillWidth: true
            visible: db.isOpen
        }

        // --- Records ---
        Kirigami.Heading {
            text: i18n("Records (%1)").arg(page.records.length)
            level: 2
            visible: page.selectedCollection !== ""
        }

        Repeater {
            model: page.records
            visible: page.selectedCollection !== ""

            delegate: Controls.ItemDelegate {
                Layout.fillWidth: true
                text: modelData
                icon.name: "document"
            }
        }

        Kirigami.Separator {
            Layout.fillWidth: true
            visible: page.selectedCollection !== ""
        }

        // --- Actions ---
        RowLayout {
            Layout.fillWidth: true
            visible: page.selectedCollection !== ""
            spacing: Kirigami.Units.spacing

            Controls.Button {
                text: i18n("Validate")
                icon.name: "document-edit-verify"
                onClicked: {
                    var report = db.validate(page.selectedCollection);
                    validateResult.text = report;
                    validateResult.visible = true;
                }
            }

            Controls.Button {
                text: i18n("Materialize View")
                icon.name: "view-file-columns"
                onClicked: {
                    var name = viewNameField.text || (page.selectedCollection + "-view");
                    var group = groupByField.text;
                    db.materializeView(page.selectedCollection, name, group);
                }
            }
        }

        Controls.TextField {
            id: viewNameField
            Layout.fillWidth: true
            visible: page.selectedCollection !== ""
            placeholderText: i18n("View name (optional)")
        }

        Controls.TextField {
            id: groupByField
            Layout.fillWidth: true
            visible: page.selectedCollection !== ""
            placeholderText: i18n("Group by field (optional, e.g. tags)")
        }

        Controls.TextArea {
            id: validateResult
            Layout.fillWidth: true
            Layout.preferredHeight: 200
            visible: false
            readOnly: true
            wrapMode: TextEdit.Wrap
        }

        // --- Status ---
        Controls.Label {
            text: db.statusMessage
            Layout.fillWidth: true
            color: Kirigami.Theme.disabledTextColor
            visible: db.statusMessage.length > 0
        }
    }
}
