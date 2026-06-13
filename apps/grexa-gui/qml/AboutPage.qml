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

    signal navigateRequested(string pageKey)

    // See SettingsPage.qml — Pages render under the View colorSet.
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

    readonly property var features: [
        { icon: "edit-find-symbolic",        title: app.i18n("ui-fast-content-search"),
          body: app.i18n("ui-streams-matches-as-files-are-scanned-a42cad") },
        { icon: "code-context-symbolic",      title: app.i18n("ui-regex-builder"),
          body: app.i18n("ui-test-patterns-against-a-sample-with-ac6cae") },
        { icon: "view-list-symbolic",         title: app.i18n("ui-smart-filters"),
          body: app.i18n("ui-gitignoreaware-with-perextension-include-perdirectory-exclude-0d4b24") },
        { icon: "tools-symbolic",             title: app.i18n("ui-optional-ai-assist"),
          body: app.i18n("ui-plug-in-any-openaicompatible-endpoint-keys-931293") }
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
                    text: app.i18n("ui-about")
                    font.pixelSize: app.tokens.textHeading
                    font.weight: app.tokens.weightBold
                    font.family: app.tokens.sansFamily
                    font.letterSpacing: 0
                }
                Controls.Label {
                    text: app.i18n("ui-built-on-rust-qt-6-kirigami")
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
                            text: app.i18n("app-name")
                            font.pixelSize: app.tokens.textDisplay
                            font.weight: app.tokens.weightBold
                            font.family: app.tokens.sansFamily
                            font.letterSpacing: 0
                        }
                        Controls.Label {
                            text: app.i18n("ui-fast-linux-file-content-search-built")
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
                                    text: app.i18n("ui-version-prefix").arg(Qt.application.version)
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
                                    text: app.i18n("ui-gpl-v3")
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
                                    text: app.i18n("ui-linux-qt-6")
                                    font.pixelSize: app.tokens.textCaption + 1
                                    font.family: app.tokens.monoFamily
                                    opacity: 0.85
                                }
                            }
                            Controls.Label {
                                visible: {
                                    const sha = app.settingsController.commitSha
                                    return sha !== "unknown" && sha.length > 0
                                }
                                text: {
                                    const sha = app.settingsController.commitSha
                                    return sha.length >= 8 ? sha.substring(0, 8) : sha
                                }
                                font.pixelSize: app.tokens.textCaption
                                font.family: app.tokens.monoFamily
                                opacity: 0.45
                                Layout.leftMargin: app.tokens.spaceXS
                            }
                        }
                    }
                }
            }

            // -- Feature highlights ---------------------------------
            // A 2×2 grid of icon-led capability cards. Each card
            // has its own subtle border and an icon avatar.
            Controls.Label {
                text: app.i18n("ui-whats-inside")
                font.pixelSize: 10
                font.weight: app.tokens.weightSemibold
                font.letterSpacing: 0
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

            // -- Grexa project card ---------------------------------
            Rectangle {
                Layout.fillWidth: true
                Layout.topMargin: app.tokens.spaceM
                Layout.preferredHeight: 96
                radius: app.tokens.radiusCard
                color: app.tokens.surface1
                border.color: app.tokens.separator
                border.width: 1

                Rectangle {
                    id: grexaProjectIcon
                    anchors.leftMargin: app.tokens.spaceL
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    width: 56
                    height: 56
                    radius: app.tokens.radiusAvatar
                    color: app.tokens.surface2
                    border.color: app.tokens.separatorStrong
                    border.width: 1
                    Image {
                        anchors.fill: parent
                        anchors.margins: 4
                        source: "qrc:/qt/qml/com/visorcraft/Grexa/resources/grexa.png"
                        sourceSize.width: 96
                        sourceSize.height: 96
                        smooth: true
                        mipmap: true
                    }
                }

                ColumnLayout {
                    anchors.left: grexaProjectIcon.right
                    anchors.leftMargin: app.tokens.spaceXL
                    anchors.right: grexaProjectButton.left
                    anchors.rightMargin: app.tokens.spaceL
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 2
                    Controls.Label {
                        Layout.fillWidth: true
                        text: app.i18n("ui-native-linux-search-app-built-with-0ff6f8")
                        font.pixelSize: app.tokens.textBodyEmphasis
                        font.weight: app.tokens.weightSemibold
                        font.family: app.tokens.sansFamily
                        elide: Text.ElideRight
                    }
                    Controls.Label {
                        Layout.fillWidth: true
                        textFormat: Text.RichText
                        text: app.i18n("ui-a-hrefhttpsgithubcomvisorcraftgrexagithubcomvisorcraftgrexaa-90eda6")
                        onLinkActivated: link => Qt.openUrlExternally(link)
                        linkColor: app.tokens.accent
                        font.pixelSize: app.tokens.textBody
                        font.family: app.tokens.sansFamily
                        elide: Text.ElideRight
                    }
                }

                AppFlatButton {
                    id: grexaProjectButton
                    anchors.right: parent.right
                    anchors.rightMargin: app.tokens.spaceL
                    anchors.verticalCenter: parent.verticalCenter
                    icon.name: "go-next-symbolic"
                    icon.color: app.tokens.textPrimary
                    text: app.i18n("ui-visit-grexa")
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: Qt.openUrlExternally("https://github.com/visorcraft/grexa")
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

                Rectangle {
                    id: grexProjectIcon
                    anchors.leftMargin: app.tokens.spaceL
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    width: 56
                    height: 56
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
                    anchors.left: grexProjectIcon.right
                    anchors.leftMargin: app.tokens.spaceXL
                    anchors.right: grexProjectButton.left
                    anchors.rightMargin: app.tokens.spaceL
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 2
                    Controls.Label {
                        Layout.fillWidth: true
                        text: app.i18n("ui-official-linux-port-of-our-grex")
                        font.pixelSize: app.tokens.textBodyEmphasis
                        font.weight: app.tokens.weightSemibold
                        font.family: app.tokens.sansFamily
                        elide: Text.ElideRight
                    }
                    Controls.Label {
                        Layout.fillWidth: true
                        textFormat: Text.RichText
                        text: app.i18n("ui-a-hrefhttpsgithubcomvisorcraftgrexgithubcomvisorcraftgrexa-0e491d")
                        onLinkActivated: link => Qt.openUrlExternally(link)
                        linkColor: app.tokens.accent
                        font.pixelSize: app.tokens.textBody
                        font.family: app.tokens.sansFamily
                        elide: Text.ElideRight
                    }
                }

                AppFlatButton {
                    id: grexProjectButton
                    anchors.right: parent.right
                    anchors.rightMargin: app.tokens.spaceL
                    anchors.verticalCenter: parent.verticalCenter
                    icon.name: "go-next-symbolic"
                    icon.color: app.tokens.textPrimary
                    text: app.i18n("ui-visit-grex")
                    display: Controls.AbstractButton.TextBesideIcon
                    onClicked: Qt.openUrlExternally("https://github.com/visorcraft/grex")
                }
            }

            // -- Licenses & Credits card --------------------------
            Card {
                Layout.fillWidth: true
                title: app.i18n("ui-licenses-credits")
                subtitle: app.i18n("ui-every-direct-transitive-crate-acknowledgments-and-2f6d3e")
                RowLayout {
                    Layout.fillWidth: true
                    spacing: app.tokens.spaceM
                    AppFlatButton {
                        icon.name: "view-list-text"
                        icon.color: app.tokens.textPrimary
                        text: app.i18n("ui-licenses")
                        display: Controls.AbstractButton.TextBesideIcon
                        onClicked: page.navigateRequested("licenses")
                    }
                    AppFlatButton {
                        icon.name: "help-about-symbolic"
                        icon.color: app.tokens.textPrimary
                        text: app.i18n("ui-credits")
                        display: Controls.AbstractButton.TextBesideIcon
                        onClicked: page.navigateRequested("credits")
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
                text: app.i18n("ui-built-by-bvisorcraftb") + "  ·  " + app.i18n("ui-powered-by-rust-qt-6-kirigami")
                font.pixelSize: app.tokens.textCaption + 1
                font.family: app.tokens.sansFamily
                opacity: 0.55
            }
        }
    }
}
