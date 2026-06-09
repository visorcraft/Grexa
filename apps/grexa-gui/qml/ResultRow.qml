// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// A single search-result row, styled like a Mailspring message row.
//
// 68px tall · round file-type "avatar" tinted by extension · two-line
// content (path as headline, monospace preview with the match
// highlighted as supporting text) · line:column pill on the right.
// Hover lifts the row with a subtle surface fill and a chevron;
// selection paints a soft accent wash plus an accent-edge stripe.

import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Controls.ItemDelegate {
    id: root
    property string relativePath: ""
    property int line: 0
    property int column: 0
    property string previewBefore: ""
    property string previewMatch: ""
    property string previewAfter: ""
    /// Full filesystem path — needed for the context-menu actions
    /// that route through `SearchController` (open in editor,
    /// reveal in file manager, copy path). Set by the SearchPage
    /// delegate factory from `searchController.rowFullPath(index)`.
    property string fullPath: ""
    signal openPreview()

    height: 68
    padding: 0
    hoverEnabled: true

    // Right-click → context menu. The MouseArea only accepts the
    // right button so left-clicks fall through to the ItemDelegate's
    // own click handler (which opens the preview). Reachable via
    // Space-key on a focused row (handled in SearchPage's ListView
    // keyNavigation).
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.RightButton
        onPressed: function(mouse) { contextMenu.popup() }
    }

    function fullLine() {
        return root.previewBefore + root.previewMatch + root.previewAfter
    }

    Controls.Menu {
        id: contextMenu
        Controls.MenuItem {
            text: qsTr("Preview")
            icon.name: "document-preview-symbolic"
            onTriggered: root.openPreview()
        }
        Controls.MenuItem {
            text: qsTr("Open in editor")
            icon.name: "document-edit"
            onTriggered: app.searchController.openInEditor(root.fullPath, root.line)
        }
        Controls.MenuItem {
            text: qsTr("Reveal in file manager")
            icon.name: "system-file-manager"
            onTriggered: app.searchController.revealInFileManager(root.fullPath)
        }
        Controls.MenuItem {
            text: qsTr("Move to Trash")
            icon.name: "edit-delete-symbolic"
            onTriggered: {
                var err = app.searchController.moveToTrash(root.fullPath)
                if (err.length > 0) {
                    console.warn("Move to trash failed:", err)
                }
            }
        }
        Controls.MenuSeparator {}
        Controls.MenuItem {
            text: qsTr("Copy full path")
            icon.name: "edit-copy-symbolic"
            onTriggered: app.searchController.copyToClipboard(root.fullPath)
        }
        Controls.MenuItem {
            text: qsTr("Copy file name")
            icon.name: "edit-copy-symbolic"
            onTriggered: app.searchController.copyToClipboard(root.fileName(root.relativePath))
        }
        Controls.MenuItem {
            text: qsTr("Copy relative path")
            icon.name: "edit-copy-symbolic"
            onTriggered: app.searchController.copyToClipboard(root.relativePath)
        }
        Controls.MenuItem {
            text: qsTr("Copy line content")
            icon.name: "edit-copy-symbolic"
            onTriggered: app.searchController.copyToClipboard(root.fullLine())
        }
        Controls.MenuItem {
            text: qsTr("Copy %1:%2").arg(root.fullPath).arg(root.line)
            icon.name: "edit-copy-symbolic"
            onTriggered: app.searchController.copyToClipboard(root.fullPath + ":" + root.line)
        }
    }

    function escapeHtml(s) {
        return String(s)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
    }

    function fileName(p) {
        if (!p) return ""
        const idx = p.lastIndexOf("/")
        return idx >= 0 ? p.substring(idx + 1) : p
    }

    function dirName(p) {
        if (!p) return ""
        const idx = p.lastIndexOf("/")
        return idx >= 0 ? p.substring(0, idx) : ""
    }

    background: Rectangle {
        color: root.pressed ? app.tokens.surface2
            : root.hovered ? app.tokens.surface1
            : "transparent"
        Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }

        // Accent edge stripe on the left, fades in on hover. Subtle —
        // it sits behind the avatar and gives the row a "next-action"
        // affordance without screaming.
        Rectangle {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: 3
            color: app.tokens.accent
            opacity: root.hovered || root.pressed ? 0.9 : 0
            Behavior on opacity { NumberAnimation { duration: app.tokens.durationSnap } }
        }

        // Bottom hairline — indents past the avatar so the list reads
        // as a grouped sequence rather than a hard table.
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: app.tokens.spaceXL + 44 + app.tokens.spaceL
            anchors.rightMargin: app.tokens.spaceXL
            height: 1
            color: app.tokens.separator
        }
    }

    contentItem: RowLayout {
        anchors.fill: parent
        anchors.leftMargin: app.tokens.spaceXL
        anchors.rightMargin: app.tokens.spaceXL
        spacing: app.tokens.spaceL

        // -- File-type "avatar" -------------------------------------
        // Round-rect tinted by file extension. Mailspring-style:
        // identity-first glance, with the freedesktop icon centred.
        Rectangle {
            Layout.preferredWidth: 44
            Layout.preferredHeight: 44
            Layout.alignment: Qt.AlignVCenter
            radius: app.tokens.radiusAvatar
            color: app.tokens.tintForPath(root.relativePath)
            border.color: Qt.rgba(0, 0, 0, 0.04)
            border.width: 1

            Kirigami.Icon {
                anchors.centerIn: parent
                source: app.tokens.iconForPath(root.relativePath)
                implicitWidth: 22
                implicitHeight: 22
                isMask: false
            }
        }

        // -- Headline + preview -------------------------------------
        ColumnLayout {
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter
            spacing: 2

            // Headline: file name in semibold, dim parent dir trail
            // before it. Reads at a glance like Mailspring's bold
            // sender + lighter context.
            Controls.Label {
                Layout.fillWidth: true
                font.pixelSize: app.tokens.textBodyEmphasis
                font.weight: app.tokens.weightSemibold
                font.family: app.tokens.sansFamily
                elide: Text.ElideMiddle
                textFormat: Text.RichText
                text: {
                    const dir = root.escapeHtml(root.dirName(root.relativePath))
                    const file = root.escapeHtml(root.fileName(root.relativePath))
                    if (dir.length === 0) return file
                    return "<span style='opacity:0.55; font-weight:400;'>"
                        + dir + "/</span>" + file
                }
            }

            // Preview line — single line of mono, lighter so the eye
            // lands on the match tint. The tint is rendered as an
            // rgba() background (CSS-safe via app.rgbaCss).
            Controls.Label {
                Layout.fillWidth: true
                font.family: app.tokens.monoFamily
                font.pixelSize: app.tokens.textCaption + 1
                textFormat: Text.RichText
                wrapMode: Text.NoWrap
                elide: Text.ElideRight
                opacity: 0.88
                text: {
                    const before = root.escapeHtml(root.previewBefore)
                    const match = root.escapeHtml(root.previewMatch)
                    const after = root.escapeHtml(root.previewAfter)
                    const bg = app.rgbaCss(app.tokens.matchTint)
                    return "<span style='opacity:0.62'>" + before + "</span>"
                        + "<span style='background-color:" + bg
                        + "; padding:0 3px; border-radius:3px;'>"
                        + match + "</span>"
                        + "<span style='opacity:0.62'>" + after + "</span>"
                }
            }
        }

        // -- Line:column pill --------------------------------------
        Rectangle {
            Layout.alignment: Qt.AlignVCenter
            radius: app.tokens.radiusPill
            color: root.hovered ? app.tokens.accentMute : app.tokens.surface1
            border.color: root.hovered ? app.tokens.accent : app.tokens.separator
            border.width: 1
            implicitHeight: 24
            implicitWidth: lineLabel.implicitWidth + app.tokens.spaceL * 2
            Behavior on color { ColorAnimation { duration: app.tokens.durationSnap } }
            Behavior on border.color { ColorAnimation { duration: app.tokens.durationSnap } }

            Controls.Label {
                id: lineLabel
                anchors.centerIn: parent
                text: root.line + ":" + root.column
                font.family: app.tokens.monoFamily
                font.pixelSize: app.tokens.textCaption
                font.weight: app.tokens.weightMedium
                color: root.hovered ? app.tokens.accent : Kirigami.Theme.textColor
                opacity: root.hovered ? 1.0 : 0.72
            }
        }

        // -- Chevron (hover affordance) ----------------------------
        Kirigami.Icon {
            Layout.alignment: Qt.AlignVCenter
            source: "go-next-symbolic"
            implicitWidth: 14
            implicitHeight: 14
            color: app.tokens.accent
            isMask: true
            opacity: root.hovered ? 0.9 : 0
            x: root.hovered ? 0 : -4
            Behavior on opacity { NumberAnimation { duration: app.tokens.durationSnap } }
            Behavior on x { NumberAnimation { duration: app.tokens.durationSnap; easing.type: Easing.OutCubic } }
        }
    }

    onClicked: root.openPreview()
}
