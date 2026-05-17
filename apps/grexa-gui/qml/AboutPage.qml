// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// About page.
//
// Shows the app icon, name, tagline, version, license, Grex
// upstream-link, attribution-link to CREDITS.md, and the
// "Created by VisorCraft" line.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    title: i18n("About Grexa")

    ColumnLayout {
        spacing: Kirigami.Units.largeSpacing
        width: parent.width

        Kirigami.Icon {
            source: "io.visorcraft.Grexa"
            Layout.alignment: Qt.AlignHCenter
            implicitWidth: 128
            implicitHeight: 128
        }

        Kirigami.Heading {
            level: 1
            text: i18n("Grexa")
            Layout.alignment: Qt.AlignHCenter
        }

        Label {
            text: i18n("Fast Linux file content search")
            Layout.alignment: Qt.AlignHCenter
            opacity: 0.7
        }

        Label {
            text: i18n("Version %1", Qt.application.version)
            Layout.alignment: Qt.AlignHCenter
            font.pointSize: Kirigami.Theme.smallFont.pointSize
        }

        Label {
            text: i18n("Licensed under GPL-3.0-only.")
            Layout.alignment: Qt.AlignHCenter
        }

        Label {
            text: i18n("A Linux/Qt port of <a href=\"https://github.com/visorcraft/grex\">Grex</a> by VisorCraft.")
            Layout.alignment: Qt.AlignHCenter
            textFormat: Text.RichText
            onLinkActivated: link => Qt.openUrlExternally(link)
        }

        Label {
            text: i18n("Third-party attribution lives in <a href=\"https://github.com/visorcraft/grexa/blob/main/CREDITS.md\">CREDITS.md</a>.")
            Layout.alignment: Qt.AlignHCenter
            textFormat: Text.RichText
            onLinkActivated: link => Qt.openUrlExternally(link)
        }

        Label {
            text: i18n("Created by <strong>VisorCraft</strong>")
            Layout.alignment: Qt.AlignHCenter
            textFormat: Text.RichText
            opacity: 0.7
        }
    }
}
