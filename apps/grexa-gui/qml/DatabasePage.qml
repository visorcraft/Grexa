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
    property var schemaFields: []
    property var views: []
    property var activeFilters: []

    DbController {
        id: db
        onRecord_paths_ready: {
            var raw = recordPathsResult;
            page.records = raw.split("\n").filter(r => r.length > 0);
        }
        onValidate_ready: {
            validateResult.text = db.validateResult;
            validateResult.visible = true;
        }
        onQuery_ready: {
            var raw = queryResult;
            page.records = raw.split("\n").filter(r => r.length > 0);
        }
    }

    function refreshViews() {
        var raw = db.listView();
        page.views = raw.split("\n").filter(v => v.length > 0);
    }

    function openDb(path) {
        if (db.openDb(path)) {
            var names = db.collectionNames().split("\n").filter(n => n.length > 0);
            page.collections = names;
            page.records = [];
            page.selectedCollection = "";
            page.schemaFields = [];
            page.refreshViews();
        }
    }

    function selectCollection(name) {
        page.selectedCollection = name;
        page.records = [];
        var schemaRaw = db.schemaJson(name);
        page.schemaFields = JSON.parse(schemaRaw);
        db.recordPaths(name);
    }

    function applyFilters() {
        if (page.activeFilters.length === 0) {
            db.recordPaths(page.selectedCollection);
            return;
        }
        db.queryRecords(page.selectedCollection, JSON.stringify(page.activeFilters));
    }

    ColumnLayout {
        width: page.width
        spacing: Kirigami.Units.largeSpacing

        // ── Open database ───────────────────────────────────────
        RowLayout {
            Layout.fillWidth: true
            Controls.TextField {
                id: pathInput
                Layout.fillWidth: true
                placeholderText: "~/my-notes-db"
            }
            Controls.Button {
                text: i18n("Open")
                icon.name: "folder-open"
                onClicked: page.openDb(pathInput.text)
            }
        }

        // ── Collections ─────────────────────────────────────────
        Kirigami.Heading {
            text: i18n("Collections")
            level: 2
            visible: db.isOpen
        }
        Repeater {
            model: page.collections
            delegate: Controls.Button {
                Layout.fillWidth: true
                text: modelData
                flat: true
                highlighted: page.selectedCollection === modelData
                onClicked: page.selectCollection(modelData)
            }
        }

        // ── Schema browser ──────────────────────────────────────
        Kirigami.Heading {
            text: i18n("Schema")
            level: 3
            visible: page.schemaFields.length > 0
        }
        Repeater {
            model: page.schemaFields
            visible: page.schemaFields.length > 0
            delegate: RowLayout {
                Layout.fillWidth: true
                Controls.Label {
                    text: modelData.name
                    font.bold: true
                    Layout.preferredWidth: 120
                }
                Controls.Label {
                    text: modelData.type
                    color: Kirigami.Theme.disabledTextColor
                    Layout.preferredWidth: 100
                }
                Controls.Label {
                    text: modelData.required ? i18n("required") : i18n("optional")
                    color: modelData.required ? Kirigami.Theme.negativeTextColor
                                              : Kirigami.Theme.disabledTextColor
                }
            }
        }

        // ── Structured filter builder ──────────────────────────
        Kirigami.Heading {
            text: i18n("Filters")
            level: 3
            visible: page.schemaFields.length > 0
        }
        ColumnLayout {
            Layout.fillWidth: true
            visible: page.schemaFields.length > 0
            spacing: Kirigami.Units.smallSpacing

            Repeater {
                model: page.activeFilters
                delegate: RowLayout {
                    Layout.fillWidth: true
                    Controls.Label {
                        text: modelData.field + " " + modelData.op + " " + modelData.value
                        Layout.fillWidth: true
                    }
                    Controls.Button {
                        icon.name: "list-remove"
                        flat: true
                        onClicked: {
                            var idx = model.index;
                            var copy = page.activeFilters.slice();
                            copy.splice(idx, 1);
                            page.activeFilters = copy;
                            page.applyFilters();
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Controls.ComboBox {
                    id: filterField
                    Layout.preferredWidth: 120
                    model: page.schemaFields.map(f => f.name)
                }
                Controls.ComboBox {
                    id: filterOp
                    Layout.preferredWidth: 80
                    model: ["eq", "ne", "lt", "le", "gt", "ge", "contains"]
                }
                Controls.TextField {
                    id: filterValue
                    Layout.fillWidth: true
                    placeholderText: i18n("value")
                    onAccepted: addFilterBtn.clicked()
                }
                Controls.Button {
                    id: addFilterBtn
                    icon.name: "list-add"
                    text: i18n("Add")
                    onClicked: {
                        if (filterValue.text.length > 0) {
                            page.activeFilters = page.activeFilters.concat([{
                                field: filterField.currentText,
                                op: filterOp.currentText,
                                value: filterValue.text
                            }]);
                            filterValue.text = "";
                            page.applyFilters();
                        }
                    }
                }
            }
        }

        // ── Record cards ────────────────────────────────────────
        Kirigami.Heading {
            text: i18n("Records (%1)").arg(page.records.length)
            level: 3
            visible: page.selectedCollection !== ""
        }
        Repeater {
            model: page.records
            visible: page.selectedCollection !== ""
            delegate: RecordCard {
                Layout.fillWidth: true
                collection: page.selectedCollection
                recordPath: modelData
                controller: db
            }
        }

        Kirigami.Separator {
            Layout.fillWidth: true
            visible: page.selectedCollection !== ""
        }

        // ── Actions ─────────────────────────────────────────────
        RowLayout {
            Layout.fillWidth: true
            visible: page.selectedCollection !== ""
            Controls.Button {
                text: i18n("Validate")
                icon.name: "document-edit-verify"
                onClicked: db.validate(page.selectedCollection)
            }
        }

        Controls.TextArea {
            id: validateResult
            Layout.fillWidth: true
            Layout.preferredHeight: 150
            visible: false
            readOnly: true
            wrapMode: TextEdit.Wrap
        }

        // ── Materialize view ────────────────────────────────────
        Kirigami.Heading {
            text: i18n("Materialize View")
            level: 3
            visible: page.selectedCollection !== ""
        }
        RowLayout {
            Layout.fillWidth: true
            visible: page.selectedCollection !== ""
            Controls.TextField {
                id: viewNameField
                Layout.fillWidth: true
                placeholderText: i18n("View name")
            }
            Controls.TextField {
                id: groupByField
                Layout.fillWidth: true
                placeholderText: i18n("Group by (e.g. tags)")
            }
            Controls.Button {
                text: i18n("Create")
                icon.name: "view-file-columns"
                onClicked: {
                    var name = viewNameField.text || (page.selectedCollection + "-view");
                    db.materializeView(page.selectedCollection, name, groupByField.text);
                    page.refreshViews();
                }
            }
        }

        // ── Saved views navigator ───────────────────────────────
        Kirigami.Heading {
            text: i18n("Saved Views")
            level: 3
            visible: page.views.length > 0
        }
        Repeater {
            model: page.views
            visible: page.views.length > 0
            delegate: RowLayout {
                Layout.fillWidth: true
                Controls.Label {
                    text: modelData
                    Layout.fillWidth: true
                }
                Controls.Button {
                    icon.name: "edit-delete"
                    flat: true
                    onClicked: {
                        db.deleteView(modelData);
                        page.refreshViews();
                    }
                }
            }
        }

        // ── Status ──────────────────────────────────────────────
        Controls.Label {
            text: db.statusMessage
            Layout.fillWidth: true
            color: Kirigami.Theme.disabledTextColor
            visible: db.statusMessage.length > 0
        }
        Controls.BusyIndicator {
            visible: db.busy
            Layout.alignment: Qt.AlignHCenter
        }
    }
}
