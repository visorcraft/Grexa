// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// About page — large app icon, version + license badge, attribution,
// and links to the canonical docs.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    id: page
    padding: 0
    titleDelegate: Item {}
    globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

    ColumnLayout {
        width: page.width
        spacing: 0

        // -- Header
        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 64
            Rectangle {
                anchors.fill: parent
                color: app.tokens.surface0
                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 1
                    color: app.tokens.separator
                }
            }
            ColumnLayout {
                anchors.fill: parent
                anchors.leftMargin: app.tokens.spaceXL
                anchors.rightMargin: app.tokens.spaceXL
                spacing: 0
                Controls.Label {
                    text: qsTr("About")
                    font.pixelSize: app.tokens.textHeading
                    font.weight: app.tokens.weightBold
                }
                Controls.Label {
                    text: qsTr("Built on Rust + Qt 6 / Kirigami via cxx-qt.")
                    font.pixelSize: app.tokens.textCaption
                    opacity: 0.6
                }
            }
        }

        // -- Body
        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceXL
            spacing: app.tokens.spaceL

            // Icon + name + version
            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: app.tokens.spaceL
                Image {
                    source: "qrc:/qt/qml/com/visorcraft/Grexa/resources/grexa.svg"
                    sourceSize.width: 128
                    sourceSize.height: 128
                    Layout.preferredWidth: 128
                    Layout.preferredHeight: 128
                    smooth: true
                }
                ColumnLayout {
                    spacing: app.tokens.spaceXS
                    Controls.Label {
                        text: "Grexa"
                        font.pixelSize: app.tokens.textDisplay
                        font.weight: app.tokens.weightBold
                    }
                    Controls.Label {
                        text: qsTr("Fast Linux file content search.")
                        font.pixelSize: app.tokens.textBody
                        opacity: 0.7
                    }
                    RowLayout {
                        spacing: app.tokens.spaceS
                        Layout.topMargin: app.tokens.spaceS
                        Rectangle {
                            radius: app.tokens.radiusPill
                            color: app.tokens.accentMute
                            border.color: app.tokens.accent
                            border.width: 1
                            implicitHeight: 24
                            implicitWidth: versionLabel.implicitWidth + app.tokens.spaceM * 2
                            Controls.Label {
                                id: versionLabel
                                anchors.centerIn: parent
                                text: qsTr("v%1").arg(Qt.application.version)
                                font.pixelSize: app.tokens.textCaption
                                font.weight: app.tokens.weightMedium
                                color: app.tokens.accent
                            }
                        }
                        Rectangle {
                            radius: app.tokens.radiusPill
                            color: app.tokens.surface1
                            border.color: app.tokens.separatorStrong
                            border.width: 1
                            implicitHeight: 24
                            implicitWidth: gplLabel.implicitWidth + app.tokens.spaceM * 2
                            Controls.Label {
                                id: gplLabel
                                anchors.centerIn: parent
                                text: "GPL-3.0-only"
                                font.pixelSize: app.tokens.textCaption
                                font.family: app.tokens.monoFamily
                                opacity: 0.8
                            }
                        }
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: app.tokens.separator
                Layout.topMargin: app.tokens.spaceL
                Layout.bottomMargin: app.tokens.spaceM
            }

            // Lineage card
            Card {
                title: qsTr("Lineage")
                subtitle: qsTr("Grexa inherits its behavior contract from VisorCraft's upstream Windows tool — every divergence is recorded in docs/linux-decisions.md.")
                RowLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceM
                    Controls.Label {
                        textFormat: Text.RichText
                        text: qsTr("Upstream: <a href='https://github.com/visorcraft/grex'>github.com/visorcraft/grex</a>")
                        onLinkActivated: link => Qt.openUrlExternally(link)
                        font.pixelSize: app.tokens.textBody
                        Layout.fillWidth: true
                    }
                }
            }

            // Attribution card
            Card {
                title: qsTr("Third-party credits")
                subtitle: qsTr("Every direct + transitive crate, with full license text, is auto-generated into docs/credits-third-party.md.")
                RowLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceM
                    Controls.Button {
                        flat: true
                        icon.name: "document-edit"
                        text: qsTr("CREDITS.md")
                        display: Controls.AbstractButton.TextBesideIcon
                        onClicked: Qt.openUrlExternally("https://github.com/visorcraft/grexa/blob/main/CREDITS.md")
                    }
                    Controls.Button {
                        flat: true
                        icon.name: "view-list-text"
                        text: qsTr("Transitive list")
                        display: Controls.AbstractButton.TextBesideIcon
                        onClicked: Qt.openUrlExternally("https://github.com/visorcraft/grexa/blob/main/docs/credits-third-party.md")
                    }
                    Item { Layout.fillWidth: true }
                }
            }

            // Footer
            Controls.Label {
                Layout.alignment: Qt.AlignHCenter
                Layout.topMargin: app.tokens.spaceL
                textFormat: Text.RichText
                text: qsTr("Built by <b>VisorCraft</b>") + " · " + qsTr("Powered by Rust, Qt 6, Kirigami, and cxx-qt")
                font.pixelSize: app.tokens.textCaption
                opacity: 0.6
            }
        }
    }
}
