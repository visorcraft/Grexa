// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// About page — large brand hero with version + license, a row of
// feature highlight pills, the lineage and credits cards, and a
// closing attribution line. Designed to feel substantive rather
// than the "logo floating on a mostly-empty page" of v1.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    id: page
    padding: 0
    titleDelegate: Item {}
    globalToolBarStyle: Kirigami.ApplicationHeaderStyle.None

    readonly property var features: [
        { icon: "edit-find-symbolic",        title: qsTr("Fast content search"),
          body: qsTr("Streams matches as files are scanned — no waiting for the whole tree.") },
        { icon: "code-context-symbolic",      title: qsTr("Regex builder"),
          body: qsTr("Test patterns against a sample with the same engine the search uses.") },
        { icon: "view-list-symbolic",         title: qsTr("Smart filters"),
          body: qsTr(".gitignore-aware, with per-extension include + per-directory exclude globs.") },
        { icon: "tools-symbolic",             title: qsTr("Optional AI assist"),
          body: qsTr("Plug in any OpenAI-compatible endpoint. Keys live in Secret Service.") }
    ]

    ColumnLayout {
        width: page.width
        spacing: 0

        // -- Page header -------------------------------------------
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 76
            color: app.tokens.surface0
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
                spacing: 1
                Layout.alignment: Qt.AlignVCenter
                Controls.Label {
                    text: qsTr("About")
                    font.pixelSize: app.tokens.textHeading
                    font.weight: app.tokens.weightBold
                    font.family: app.tokens.sansFamily
                    font.letterSpacing: -0.3
                }
                Controls.Label {
                    text: qsTr("Built on Rust + Qt 6 / Kirigami via cxx-qt.")
                    font.pixelSize: app.tokens.textCaption + 1
                    font.family: app.tokens.sansFamily
                    opacity: 0.6
                }
            }
        }

        // -- Body --------------------------------------------------
        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: app.tokens.spaceXL
            Layout.rightMargin: app.tokens.spaceXL
            Layout.topMargin: app.tokens.spaceXL
            spacing: app.tokens.spaceL

            // -- Brand hero card. The icon sits inside an accent-tinted
            // halo, with the wordmark and version + license pills
            // next to it. Reads like a Mailspring product card.
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 168
                radius: app.tokens.radiusCard
                color: app.tokens.surface1
                border.color: app.tokens.separator
                border.width: 1

                // Soft accent halo behind the icon to give the hero
                // a subtle product-card sheen.
                Rectangle {
                    anchors.left: parent.left
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    width: 240
                    radius: parent.radius
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.0; color: app.tokens.accentMute }
                        GradientStop { position: 1.0; color: "transparent" }
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: app.tokens.spaceXL
                    anchors.rightMargin: app.tokens.spaceXL
                    spacing: app.tokens.spaceXL

                    Image {
                        source: "qrc:/qt/qml/com/visorcraft/Grexa/resources/grexa.png"
                        sourceSize.width: 224
                        sourceSize.height: 224
                        Layout.preferredWidth: 112
                        Layout.preferredHeight: 112
                        smooth: true
                        mipmap: true
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: app.tokens.spaceXS
                        Controls.Label {
                            text: "Grexa"
                            font.pixelSize: app.tokens.textDisplay
                            font.weight: app.tokens.weightBold
                            font.family: app.tokens.sansFamily
                            font.letterSpacing: -0.5
                        }
                        Controls.Label {
                            text: qsTr("Fast Linux file content search — built on the ripgrep core.")
                            font.pixelSize: app.tokens.textBody + 1
                            font.family: app.tokens.sansFamily
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
                                implicitHeight: 26
                                implicitWidth: versionLabel.implicitWidth + app.tokens.spaceL * 2
                                Controls.Label {
                                    id: versionLabel
                                    anchors.centerIn: parent
                                    text: qsTr("v%1").arg(Qt.application.version)
                                    font.pixelSize: app.tokens.textCaption + 1
                                    font.weight: app.tokens.weightSemibold
                                    font.family: app.tokens.monoFamily
                                    color: app.tokens.accent
                                }
                            }
                            Rectangle {
                                radius: app.tokens.radiusPill
                                color: app.tokens.surface2
                                border.color: app.tokens.separatorStrong
                                border.width: 1
                                implicitHeight: 26
                                implicitWidth: gplLabel.implicitWidth + app.tokens.spaceL * 2
                                Controls.Label {
                                    id: gplLabel
                                    anchors.centerIn: parent
                                    text: qsTr("GPL v3")
                                    font.pixelSize: app.tokens.textCaption + 1
                                    font.family: app.tokens.sansFamily
                                    opacity: 0.85
                                }
                            }
                            Rectangle {
                                radius: app.tokens.radiusPill
                                color: app.tokens.surface2
                                border.color: app.tokens.separatorStrong
                                border.width: 1
                                implicitHeight: 26
                                implicitWidth: platformLabel.implicitWidth + app.tokens.spaceL * 2
                                Controls.Label {
                                    id: platformLabel
                                    anchors.centerIn: parent
                                    // Qt has no QML-exposed runtime
                                    // version string that's safe to
                                    // print here (`Qt.application.version`
                                    // returns OUR app version). The
                                    // major version is part of the
                                    // build contract.
                                    text: qsTr("Linux · Qt 6")
                                    font.pixelSize: app.tokens.textCaption + 1
                                    font.family: app.tokens.monoFamily
                                    opacity: 0.85
                                }
                            }
                        }
                    }
                }
            }

            // -- Feature highlights ---------------------------------
            // A 2×2 grid of icon-led capability cards. Each card
            // has its own subtle border and an icon avatar.
            Controls.Label {
                text: qsTr("WHAT'S INSIDE")
                font.pixelSize: 10
                font.weight: app.tokens.weightSemibold
                font.letterSpacing: 1.6
                opacity: 0.5
                Layout.topMargin: app.tokens.spaceM
            }

            GridLayout {
                Layout.fillWidth: true
                columns: 2
                columnSpacing: app.tokens.spaceL
                rowSpacing: app.tokens.spaceL

                Repeater {
                    model: page.features
                    delegate: Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 84
                        radius: app.tokens.radiusCard
                        color: app.tokens.surface1
                        border.color: app.tokens.separator
                        border.width: 1
                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: app.tokens.spaceL
                            spacing: app.tokens.spaceL
                            Rectangle {
                                Layout.preferredWidth: 44
                                Layout.preferredHeight: 44
                                Layout.alignment: Qt.AlignVCenter
                                radius: app.tokens.radiusAvatar
                                color: app.tokens.accentMute
                                border.color: Qt.rgba(app.tokens.accent.r,
                                                      app.tokens.accent.g,
                                                      app.tokens.accent.b, 0.35)
                                border.width: 1
                                Kirigami.Icon {
                                    anchors.centerIn: parent
                                    source: modelData.icon
                                    implicitWidth: 22
                                    implicitHeight: 22
                                    color: app.tokens.accent
                                    isMask: true
                                }
                            }
                            ColumnLayout {
                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignVCenter
                                spacing: 2
                                Controls.Label {
                                    text: modelData.title
                                    font.pixelSize: app.tokens.textBodyEmphasis
                                    font.weight: app.tokens.weightSemibold
                                    font.family: app.tokens.sansFamily
                                }
                                Controls.Label {
                                    Layout.fillWidth: true
                                    text: modelData.body
                                    font.pixelSize: app.tokens.textCaption + 1
                                    font.family: app.tokens.sansFamily
                                    opacity: 0.65
                                    wrapMode: Text.WordWrap
                                    elide: Text.ElideRight
                                    maximumLineCount: 2
                                }
                            }
                        }
                    }
                }
            }

            // -- Sister-project card --------------------------------
            // Compact card mirroring the brand hero pattern: the green
            // Grex (Windows / male gecko) mark sits in an accent halo
            // on the left, with a short note that Grexa is the
            // Linux/female counterpart and a direct link to the Grex
            // repository on the right.
            Rectangle {
                Layout.fillWidth: true
                Layout.topMargin: app.tokens.spaceM
                Layout.preferredHeight: 96
                radius: app.tokens.radiusCard
                color: app.tokens.surface1
                border.color: app.tokens.separator
                border.width: 1

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: app.tokens.spaceL
                    anchors.rightMargin: app.tokens.spaceL
                    spacing: app.tokens.spaceL

                    Rectangle {
                        Layout.preferredWidth: 56
                        Layout.preferredHeight: 56
                        Layout.alignment: Qt.AlignVCenter
                        radius: app.tokens.radiusAvatar
                        color: app.tokens.surface2
                        border.color: app.tokens.separatorStrong
                        border.width: 1
                        Image {
                            anchors.fill: parent
                            anchors.margins: 4
                            source: "qrc:/qt/qml/com/visorcraft/Grexa/resources/grex-mark.png"
                            sourceSize.width: 96
                            sourceSize.height: 96
                            smooth: true
                            mipmap: true
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter
                        spacing: 2
                        Controls.Label {
                            text: qsTr("Official Linux port of our Grex tool for Windows.")
                            font.pixelSize: app.tokens.textBodyEmphasis
                            font.weight: app.tokens.weightSemibold
                            font.family: app.tokens.sansFamily
                        }
                        Controls.Label {
                            textFormat: Text.RichText
                            text: qsTr("<a href='https://github.com/visorcraft/grex'>github.com/visorcraft/grex</a>")
                            onLinkActivated: link => Qt.openUrlExternally(link)
                            linkColor: app.tokens.accent
                            font.pixelSize: app.tokens.textBody
                            font.family: app.tokens.sansFamily
                        }
                    }

                    Controls.Button {
                        Layout.alignment: Qt.AlignVCenter
                        flat: true
                        icon.name: "go-next-symbolic"
                        text: qsTr("Visit Grex")
                        display: Controls.AbstractButton.TextBesideIcon
                        onClicked: Qt.openUrlExternally("https://github.com/visorcraft/grex")
                    }
                }
            }

            // -- Third-party credits card --------------------------
            Card {
                Layout.fillWidth: true
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

            // -- Footer attribution --------------------------------
            Controls.Label {
                Layout.alignment: Qt.AlignHCenter
                Layout.topMargin: app.tokens.spaceL
                Layout.bottomMargin: app.tokens.spaceXL
                textFormat: Text.RichText
                text: qsTr("Built by <b>VisorCraft</b>") + "  ·  " + qsTr("Powered by Rust, Qt 6, Kirigami, and cxx-qt")
                font.pixelSize: app.tokens.textCaption + 1
                font.family: app.tokens.sansFamily
                opacity: 0.55
            }
        }
    }
}
