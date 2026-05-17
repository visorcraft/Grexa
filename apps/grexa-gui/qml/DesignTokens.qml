// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Grexa design tokens — single source of truth for spacing, type
// scale, radii, palette accents, motion, and density.
//
// Instantiated once in Main.qml as `id: tokens`; pages reach the
// values via `app.tokens.spaceM`, `app.tokens.accent`, etc. We do
// NOT use `pragma Singleton` because registering a singleton through
// cxx-qt-build's QML module pipeline adds complexity without payoff
// at this scale.

import QtQuick
import org.kde.kirigami as Kirigami

QtObject {
    // ---- Spacing scale (4px rhythm) ---------------------------------
    readonly property int spaceXS:  4
    readonly property int spaceS:   8
    readonly property int spaceM:  12
    readonly property int spaceL:  16
    readonly property int spaceXL: 24
    readonly property int spaceXXL: 32

    // ---- Radii ------------------------------------------------------
    readonly property int radiusButton: 4
    readonly property int radiusCard:   8
    readonly property int radiusPill:   12
    readonly property int radiusInput:  4

    // ---- Type scale -------------------------------------------------
    readonly property int textCaption:      11   // metadata, gutter
    readonly property int textBody:         13   // body text default
    readonly property int textBodyEmphasis: 14   // primary text in dense rows
    readonly property int textSubheading:   16   // section labels
    readonly property int textHeading:      20   // page titles
    readonly property int textDisplay:      28   // about page title

    readonly property int weightNormal: Font.Normal
    readonly property int weightMedium: Font.Medium
    readonly property int weightBold:   Font.DemiBold

    readonly property string monoFamily: "Fira Code, JetBrains Mono, monospace"

    // ---- Accent palette --------------------------------------------
    // Action blue — the only saturated color in the UI. Used for the
    // primary action button, focus rings, selection highlights, and
    // the active-nav-item accent bar.
    readonly property color accent:       "#3D8FE5"
    readonly property color accentHover:  "#4F9EEF"
    readonly property color accentPressed: "#2F7BCC"
    readonly property color accentMute:   Qt.rgba(0.24, 0.56, 0.90, 0.18)
    readonly property color accentRing:   Qt.rgba(0.24, 0.56, 0.90, 0.35)

    readonly property color success:      "#27AE60"
    readonly property color warning:      "#E67E22"
    readonly property color error:        "#C0392B"

    // Match-highlight tint (yellow @ ~30%) for inline match spans
    // and the context-preview match line.
    readonly property color matchTint:    Qt.rgba(1.0, 0.84, 0.31, 0.28)
    readonly property color matchTintStrong: Qt.rgba(1.0, 0.84, 0.31, 0.55)

    // ---- Surface elevation -----------------------------------------
    // Subtle overlays that work on both light and dark Kirigami
    // themes. The base color is always
    // Kirigami.Theme.backgroundColor; these add a faint lift.
    readonly property color surface0:     Kirigami.Theme.backgroundColor
    readonly property color surface1:     Qt.tint(Kirigami.Theme.backgroundColor,
                                                  Qt.rgba(1, 1, 1, 0.03))
    readonly property color surface2:     Qt.tint(Kirigami.Theme.backgroundColor,
                                                  Qt.rgba(1, 1, 1, 0.06))
    readonly property color separator:    Qt.rgba(Kirigami.Theme.textColor.r,
                                                  Kirigami.Theme.textColor.g,
                                                  Kirigami.Theme.textColor.b,
                                                  0.10)
    readonly property color separatorStrong: Qt.rgba(Kirigami.Theme.textColor.r,
                                                     Kirigami.Theme.textColor.g,
                                                     Kirigami.Theme.textColor.b,
                                                     0.18)

    // ---- Motion -----------------------------------------------------
    readonly property int durationSnap:        120   // toggles, hover
    readonly property int durationNormal:      200   // page transitions, fades
    readonly property int durationSlow:        320   // result-list populate
    readonly property int durationDecorative:  480   // empty-state breathing

    readonly property int easing: Easing.OutCubic

    // ---- Density ----------------------------------------------------
    readonly property int rowCompact:  28
    readonly property int rowNormal:   40
    readonly property int rowSpacious: 56

    // ---- File-type icon resolver -----------------------------------
    // Maps a relative-path extension to the freedesktop icon name
    // Kirigami.Icon will look up. Falls back to `text-x-generic`.
    function iconForPath(path) {
        if (!path) return "text-x-generic"
        const lower = path.toLowerCase()
        const dot = lower.lastIndexOf(".")
        const ext = dot >= 0 ? lower.substring(dot + 1) : ""
        switch (ext) {
            case "rs": return "text-rust"
            case "py": return "text-x-python"
            case "go": return "text-x-go"
            case "c": case "h": return "text-x-csrc"
            case "cpp": case "cc": case "cxx": case "hpp": return "text-x-c++src"
            case "js": case "mjs": case "cjs": return "application-javascript"
            case "ts": case "tsx": return "application-typescript"
            case "jsx": return "application-javascript"
            case "html": case "htm": return "text-html"
            case "css": case "scss": case "less": return "text-css"
            case "json": return "application-json"
            case "toml": case "yaml": case "yml": case "ini":
            case "conf": case "cfg": return "text-x-script"
            case "md": case "markdown": case "rst": return "text-markdown"
            case "sh": case "bash": case "zsh": case "fish":
                return "text-x-script"
            case "qml": return "application-x-qml"
            case "java": case "kt": case "scala": return "text-x-java"
            case "rb": return "application-x-ruby"
            case "php": return "application-x-php"
            case "xml": case "svg": return "text-xml"
            case "ftl": case "po": case "pot": return "text-x-script"
            case "log": case "txt": return "text-plain"
            case "lock": return "application-x-trash"
            default: return "text-x-generic"
        }
    }
}
