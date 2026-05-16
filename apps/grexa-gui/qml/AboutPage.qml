// About page — Phase 4 destination.
//
// Displays: app version, license (GPL-3.0-only), upstream URL, third-
// party credits (encoding_rs, ignore, ureq, keyring, fluent, …),
// release notes for the current build.

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
            text: i18n("Version 0.1.0-alpha")
            Layout.alignment: Qt.AlignHCenter
            font.pointSize: Kirigami.Theme.smallFont.pointSize
        }

        Label {
            text: i18n("Licensed under GPL-3.0-only.")
            Layout.alignment: Qt.AlignHCenter
        }
    }
}
