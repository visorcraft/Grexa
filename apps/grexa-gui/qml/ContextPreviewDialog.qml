// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Context-preview dialog. Renders ±5 lines around a search match by
// calling `searchController.previewAt(path, line)`. The result is a
// pre-formatted monospace block with `>` marking the match line.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Dialog {
    id: dialog
    modal: true
    title: dialog.path.length > 0 ? i18n("%1 — line %2", baseName(dialog.path), dialog.lineNumber) : i18n("Preview")
    standardButtons: Dialog.Close
    width: Math.min(parent.width * 0.8, 900)
    height: Math.min(parent.height * 0.7, 500)

    property string path: ""
    property int lineNumber: 0

    function baseName(p) {
        if (!p) return ""
        const idx = p.lastIndexOf("/")
        return idx >= 0 ? p.substring(idx + 1) : p
    }

    onOpened: {
        if (dialog.path.length > 0 && dialog.lineNumber > 0) {
            previewText.text = app.searchController.previewAt(dialog.path, dialog.lineNumber)
        } else {
            previewText.text = ""
        }
    }

    contentItem: ScrollView {
        clip: true
        TextArea {
            id: previewText
            readOnly: true
            font.family: "monospace"
            wrapMode: TextEdit.NoWrap
            text: ""
            selectByMouse: true
        }
    }
}
