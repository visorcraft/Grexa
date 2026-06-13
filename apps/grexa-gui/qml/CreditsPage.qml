// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.Page {
    id: page
    padding: 0
    titleDelegate: Item {}
    globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

    property string filterText: ""
    property var crates: []

    readonly property int rowHeight: 36
    readonly property int nameColumnWidth: Math.max(210, Math.min(300, page.width * 0.25))
    readonly property int versionColumnWidth: 124
    readonly property int linkColumnWidth: 44
    property var runtimeComponents: []
    readonly property var filteredCrates: {
        const needle = page.filterText.trim().toLowerCase()
        if (needle.length === 0)
            return page.crates
        return page.crates.filter(row =>
            String(row.name).toLowerCase().indexOf(needle) !== -1
                || String(row.version).toLowerCase().indexOf(needle) !== -1
                || String(row.license).toLowerCase().indexOf(needle) !== -1)
    }

    Kirigami.Theme.inherit: false
    Kirigami.Theme.colorSet: Kirigami.Theme.View
    Kirigami.Theme.backgroundColor: app.tokens.surface0
    Kirigami.Theme.textColor: app.tokens.textPrimary
    Kirigami.Theme.highlightColor: app.tokens.accent
    Kirigami.Theme.highlightedTextColor: app.tokens.accentText

    palette.window:          app.tokens.surface0
    palette.windowText:      app.tokens.textPrimary
    palette.base:            app.tokens.surface1
    palette.alternateBase:   app.tokens.surface2
    palette.text:            app.tokens.textPrimary
    palette.button:          app.tokens.surface1
    palette.buttonText:      app.tokens.textPrimary
    palette.brightText:      app.tokens.accentText
    palette.highlight:       app.tokens.accent
    palette.highlightedText: app.tokens.accentText
    palette.toolTipBase:     app.tokens.surface2
    palette.toolTipText:     app.tokens.textPrimary
    palette.mid:             app.tokens.separator
    palette.midlight:        app.tokens.surface1
    palette.light:           app.tokens.surface2
    palette.dark:            app.tokens.surface0
    palette.shadow:          app.tokens.shadowFar
    palette.placeholderText: Qt.rgba(app.tokens.textPrimary.r,
                                     app.tokens.textPrimary.g,
                                     app.tokens.textPrimary.b, 0.55)

    background: Rectangle { color: app.tokens.surface0 }

    Component.onCompleted: page.loadCredits()

    function loadCredits() {
        try {
            page.crates = JSON.parse(app.settingsController.thirdPartyCreditsJson())
        } catch (e) {
            page.crates = []
        }
        try {
            page.runtimeComponents = JSON.parse(app.settingsController.runtimeComponentsJson())
        } catch (e) {
            page.runtimeComponents = []
        }
    }

    function openUrl(url) {
        if (url && String(url).length > 0)
            Qt.openUrlExternally(url)
    }

    function openComponentLicense(comp) {
        const ids = comp.spdx || []
        const sections = []
        for (let i = 0; i < ids.length; ++i) {
            const id = ids[i]
            sections.push("===== " + id + " =====\n\n"
                + app.settingsController.runtimeLicenseText(id))
        }
        licenseDialog.openDocument(comp.name, comp.licenses, sections.join("\n\n\n"))
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 96
            color: app.tokens.surface1

            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: 1
                color: app.tokens.separator
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.leftMargin: app.tokens.spaceXL
                anchors.rightMargin: app.tokens.spaceXL
                spacing: app.tokens.spaceXS

                Item { Layout.fillHeight: true }

                Controls.Label {
                    Layout.fillWidth: true
                    text: app.i18n("ui-credits")
                    color: app.tokens.textPrimary
                    font.pixelSize: 26
                    font.weight: app.tokens.weightBold
                    font.family: app.tokens.sansFamily
                    font.letterSpacing: 0
                }

                Controls.Label {
                    Layout.fillWidth: true
                    text: app.i18n("ui-1-cargo-crates-2-runtime-components-4ce163")
                        .arg(page.crates.length)
                        .arg(page.runtimeComponents.length)
                    color: app.tokens.textPrimary
                    font.pixelSize: app.tokens.textCaption + 1
                    font.family: app.tokens.sansFamily
                    opacity: 0.62
                    elide: Text.ElideRight
                }

                Item { Layout.fillHeight: true }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceL
            Layout.bottomMargin: app.tokens.spaceL
            spacing: app.tokens.spaceM

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: runtimeContent.implicitHeight + app.tokens.spaceL * 2
                radius: app.tokens.radiusCard
                color: app.tokens.surface1
                border.color: app.tokens.separator
                border.width: 1

                ColumnLayout {
                    id: runtimeContent
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: app.tokens.spaceL
                    spacing: app.tokens.spaceS

                    Controls.Label {
                        Layout.fillWidth: true
                        text: app.i18n("ui-runtime-components")
                        color: app.tokens.textPrimary
                        font.pixelSize: app.tokens.textBodyEmphasis
                        font.weight: app.tokens.weightBold
                        font.family: app.tokens.sansFamily
                    }

                    Controls.Label {
                        Layout.fillWidth: true
                        text: app.i18n("ui-system-libraries-grexa-links-against-at-0646c4")
                        color: app.tokens.textPrimary
                        font.pixelSize: app.tokens.textBody
                        font.family: app.tokens.sansFamily
                        opacity: 0.62
                        wrapMode: Text.WordWrap
                    }

                    Repeater {
                        model: page.runtimeComponents
                        delegate: RowLayout {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            spacing: app.tokens.spaceM

                            Controls.Label {
                                Layout.preferredWidth: page.nameColumnWidth + 90
                                Layout.maximumWidth: page.nameColumnWidth + 150
                                text: modelData.name
                                color: app.tokens.textPrimary
                                font.pixelSize: app.tokens.textBody
                                font.weight: app.tokens.weightSemibold
                                font.family: app.tokens.sansFamily
                                elide: Text.ElideRight
                            }

                            Controls.Label {
                                Layout.fillWidth: true
                                text: modelData.licenses
                                color: app.tokens.textPrimary
                                font.pixelSize: app.tokens.textCaption + 1
                                font.family: app.tokens.monoFamily
                                opacity: 0.86
                                elide: Text.ElideRight
                            }

                            AppFlatButton {
                                Layout.preferredWidth: 34
                                Layout.preferredHeight: 28
                                icon.name: "document-preview-symbolic"
                                display: Controls.AbstractButton.IconOnly
                                onClicked: page.openComponentLicense(modelData)
                                Controls.ToolTip.text: app.i18n("ui-view-license-text")
                                Controls.ToolTip.visible: hovered
                            }

                            AppFlatButton {
                                Layout.preferredWidth: 34
                                Layout.preferredHeight: 28
                                icon.name: "internet-services-symbolic"
                                display: Controls.AbstractButton.IconOnly
                                onClicked: page.openUrl(modelData.url)
                                Controls.ToolTip.text: app.i18n("ui-open-project-website")
                                Controls.ToolTip.visible: hovered
                            }
                        }
                    }
                }
            }

            Controls.Label {
                Layout.fillWidth: true
                Layout.topMargin: app.tokens.spaceS
                text: app.i18n("ui-cargo-crates")
                color: app.tokens.textPrimary
                font.pixelSize: 10
                font.weight: app.tokens.weightSemibold
                font.family: app.tokens.sansFamily
                font.letterSpacing: 0
                opacity: 0.5
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: app.tokens.spaceM

                AppTextField {
                    id: filterField
                    Layout.fillWidth: true
                    placeholderText: app.i18n("ui-filter-by-crate-name-or-license")
                    onTextChanged: page.filterText = text
                    Accessible.name: app.i18n("ui-filter-thirdparty-credits")
                }

                Controls.Label {
                    Layout.preferredWidth: 68
                    text: app.i18n("ui-1-2-d4b2ac").arg(page.filteredCrates.length).arg(page.crates.length)
                    color: app.tokens.textPrimary
                    font.pixelSize: app.tokens.textCaption + 1
                    font.family: app.tokens.monoFamily
                    opacity: 0.62
                    horizontalAlignment: Text.AlignRight
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 160
                radius: app.tokens.radiusCard
                color: app.tokens.surface1
                border.color: app.tokens.separator
                border.width: 1
                clip: true

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 38
                        color: app.tokens.surface2

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: app.tokens.spaceL
                            anchors.rightMargin: app.tokens.spaceL
                            spacing: app.tokens.spaceM

                            Controls.Label {
                                Layout.preferredWidth: page.nameColumnWidth
                                text: app.i18n("ui-crate")
                                color: app.tokens.textPrimary
                                font.pixelSize: app.tokens.textCaption + 1
                                font.weight: app.tokens.weightSemibold
                                font.family: app.tokens.sansFamily
                                opacity: 0.72
                            }

                            Controls.Label {
                                Layout.preferredWidth: page.versionColumnWidth
                                text: app.i18n("ui-version")
                                color: app.tokens.textPrimary
                                font.pixelSize: app.tokens.textCaption + 1
                                font.weight: app.tokens.weightSemibold
                                font.family: app.tokens.sansFamily
                                opacity: 0.72
                            }

                            Controls.Label {
                                Layout.fillWidth: true
                                text: app.i18n("ui-license-expression")
                                color: app.tokens.textPrimary
                                font.pixelSize: app.tokens.textCaption + 1
                                font.weight: app.tokens.weightSemibold
                                font.family: app.tokens.sansFamily
                                opacity: 0.72
                            }

                            Item { Layout.preferredWidth: page.linkColumnWidth }
                        }
                    }

                    ListView {
                        id: crateList
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        boundsBehavior: Flickable.StopAtBounds
                        model: page.filteredCrates

                        delegate: Rectangle {
                            width: crateList.width
                            height: page.rowHeight
                            color: index % 2 === 1 ? Qt.rgba(app.tokens.surface2.r,
                                                             app.tokens.surface2.g,
                                                             app.tokens.surface2.b, 0.34)
                                                    : "transparent"

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: app.tokens.spaceL
                                anchors.rightMargin: app.tokens.spaceL
                                spacing: app.tokens.spaceM

                                Controls.Label {
                                    Layout.preferredWidth: page.nameColumnWidth
                                    text: modelData.name
                                    color: app.tokens.textPrimary
                                    font.pixelSize: app.tokens.textCaption + 1
                                    font.family: app.tokens.monoFamily
                                    elide: Text.ElideRight
                                }

                                Controls.Label {
                                    Layout.preferredWidth: page.versionColumnWidth
                                    text: modelData.version
                                    color: app.tokens.textPrimary
                                    font.pixelSize: app.tokens.textCaption + 1
                                    font.family: app.tokens.monoFamily
                                    opacity: 0.74
                                    elide: Text.ElideRight
                                }

                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 24
                                    Layout.alignment: Qt.AlignVCenter
                                    radius: app.tokens.radiusPill
                                    color: app.tokens.successMute

                                    Controls.Label {
                                        anchors.fill: parent
                                        anchors.leftMargin: app.tokens.spaceS
                                        anchors.rightMargin: app.tokens.spaceS
                                        verticalAlignment: Text.AlignVCenter
                                        text: modelData.license
                                        color: app.tokens.textPrimary
                                        font.pixelSize: app.tokens.textCaption
                                        font.family: app.tokens.monoFamily
                                        elide: Text.ElideRight
                                        opacity: 0.9
                                    }
                                }

                                AppFlatButton {
                                    Layout.preferredWidth: page.linkColumnWidth
                                    Layout.preferredHeight: page.rowHeight
                                    icon.name: "internet-services-symbolic"
                                    display: Controls.AbstractButton.IconOnly
                                    onClicked: page.openUrl(modelData.url)
                                    Controls.ToolTip.text: app.i18n("ui-open-crate-project")
                                    Controls.ToolTip.visible: hovered
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    GplLicenseDialog { id: licenseDialog }
}
