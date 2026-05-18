// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

// Grexa design tokens — single source of truth for spacing, type
// scale, radii, palette accents, motion, density, and elevation.
//
// Visual language: clean, breathable, Mailspring-class. Subtle
// surface tinting differentiates sidebar from canvas; cards and
// rows lift via faint shadow stacks instead of hard borders.
//
// Instantiated once in Main.qml as `id: tokens`; pages reach the
// values via `app.tokens.spaceM`, `app.tokens.accent`, etc.

import QtQuick
import org.kde.kirigami as Kirigami

QtObject {
    // ---- Spacing scale (4px rhythm) ---------------------------------
    readonly property int spaceXS:   4
    readonly property int spaceS:    8
    readonly property int spaceM:   12
    readonly property int spaceL:   16
    readonly property int spaceXL:  24
    readonly property int spaceXXL: 32
    readonly property int spaceXXXL: 48

    // ---- Radii ------------------------------------------------------
    readonly property int radiusButton: 6
    readonly property int radiusCard:   10
    readonly property int radiusPill:   999
    readonly property int radiusInput:  8
    readonly property int radiusAvatar: 10

    // ---- Type scale -------------------------------------------------
    readonly property int textCaption:      11   // metadata, gutter
    readonly property int textBody:         13   // body text default
    readonly property int textBodyEmphasis: 14   // primary text in dense rows
    readonly property int textSubheading:   16   // section labels
    readonly property int textHeading:      22   // page titles
    readonly property int textDisplay:      30   // about page title

    readonly property int weightNormal: Font.Normal
    readonly property int weightMedium: Font.Medium
    readonly property int weightSemibold: Font.DemiBold
    readonly property int weightBold:   Font.Bold

    // Monospace family — resolved at startup via Qt.fontFamilies()
    // probe. `Controls.Label` doesn't expose `font.families`, so we
    // commit to a single string. The probe gives us the prettiest
    // installed mono first, then falls back to generic "monospace"
    // (which Qt routes to whatever fontconfig considers default).
    readonly property string monoFamily: {
        const preferred = ["JetBrains Mono", "Fira Code", "Iosevka",
                           "Cascadia Mono", "Source Code Pro", "Hack",
                           "DejaVu Sans Mono"]
        const installed = Qt.fontFamilies()
        for (let i = 0; i < preferred.length; ++i) {
            if (installed.indexOf(preferred[i]) !== -1) return preferred[i]
        }
        return "monospace"
    }

    // Preferred UI sans family — same idea. Mailspring uses Nunito;
    // we pick whichever clean humanist sans is installed and fall
    // back to the platform default. Empty string lets Qt pick.
    readonly property string sansFamily: {
        const preferred = ["Inter", "Inter Display", "Nunito Sans",
                           "Nunito", "IBM Plex Sans", "Source Sans 3",
                           "Source Sans Pro", "Noto Sans",
                           "Cantarell"]
        const installed = Qt.fontFamilies()
        for (let i = 0; i < preferred.length; ++i) {
            if (installed.indexOf(preferred[i]) !== -1) return preferred[i]
        }
        return ""   // empty → Qt picks the platform default
    }

    // ---- Accent palette --------------------------------------------
    // Cobalt-leaning blue — slightly more saturated than the
    // previous tone so the primary action and selection feel
    // confident without becoming neon. Used for the primary button,
    // focus rings, selection highlights, and the active-nav pill.
    readonly property color accent:        "#2D7FF9"
    readonly property color accentHover:   "#4892FB"
    readonly property color accentPressed: "#1F6BDB"
    readonly property color accentDeep:    "#1656B8"
    readonly property color accentMute:    Qt.rgba(0.18, 0.50, 0.98, 0.14)
    readonly property color accentMuteStrong: Qt.rgba(0.18, 0.50, 0.98, 0.22)
    readonly property color accentRing:    Qt.rgba(0.18, 0.50, 0.98, 0.32)
    readonly property color accentText:    "#FFFFFF"

    // Secondary accent (warm amber) — reserved for match-highlight
    // tints, where a complementary warm tone reads as "found this"
    // without competing with the cooler primary actions.
    readonly property color matchTint:        Qt.rgba(1.0, 0.78, 0.18, 0.32)
    readonly property color matchTintStrong:  Qt.rgba(1.0, 0.78, 0.18, 0.58)
    readonly property color matchUnderline:   "#E5A100"

    readonly property color success:       "#1FA862"
    readonly property color successMute:   Qt.rgba(0.12, 0.66, 0.38, 0.16)
    readonly property color warning:       "#E08319"
    readonly property color error:         "#D93B3B"
    readonly property color errorMute:     Qt.rgba(0.85, 0.23, 0.23, 0.14)

    // ---- Surface elevation -----------------------------------------
    // A four-step elevation stack. `surface0` is the canvas;
    // `surfaceSidebar` is the chrome panel (slightly cooler than
    // canvas to anchor it visually); `surface1` is a soft lift used
    // by cards and rows; `surface2` is the highest-lift state
    // (hover/press, raised pills). Derived from Kirigami so the
    // overall feel still follows the host palette in light / dark.
    readonly property color surface0:     Kirigami.Theme.backgroundColor
    // The sidebar uses a noticeably darker tint on dark themes and a
    // cool warm tint on light themes — enough contrast to read as
    // a distinct chrome panel without becoming a banded slab.
    readonly property color surfaceSidebar: Qt.tint(Kirigami.Theme.backgroundColor,
                                                    isDark
                                                        ? Qt.rgba(0.0, 0.0, 0.0, 0.22)
                                                        : Qt.rgba(0.20, 0.30, 0.50, 0.06))
    // surface1 is the "elevated row / bar" colour — clearly above
    // canvas on either theme. surface2 is the highest lift, used
    // for hover/press states and headlines.
    readonly property color surface1:     Qt.tint(Kirigami.Theme.backgroundColor,
                                                  isDark
                                                      ? Qt.rgba(1, 1, 1, 0.07)
                                                      : Qt.rgba(0, 0, 0, 0.04))
    readonly property color surface2:     Qt.tint(Kirigami.Theme.backgroundColor,
                                                  isDark
                                                      ? Qt.rgba(1, 1, 1, 0.12)
                                                      : Qt.rgba(0, 0, 0, 0.07))
    readonly property color surfaceCard:  surface1
    // High-contrast biases the separator alpha up so card edges
    // remain visible against very light or very dark wallpapers.
    readonly property color separator:    Qt.rgba(Kirigami.Theme.textColor.r,
                                                  Kirigami.Theme.textColor.g,
                                                  Kirigami.Theme.textColor.b,
                                                  highContrast ? (isDark ? 0.32 : 0.22)
                                                               : (isDark ? 0.12 : 0.09))
    readonly property color separatorStrong: Qt.rgba(Kirigami.Theme.textColor.r,
                                                     Kirigami.Theme.textColor.g,
                                                     Kirigami.Theme.textColor.b,
                                                     highContrast ? (isDark ? 0.50 : 0.38)
                                                                  : (isDark ? 0.22 : 0.16))
    readonly property color selection:    accentMute
    readonly property color selectionEdge: accent

    // Drop-shadow tints. We layer two faint Rectangles below a card
    // to fake a soft elevation. These are alpha values designed to
    // work on both light and dark host themes.
    readonly property color shadowNear: Qt.rgba(0, 0, 0, isDark ? 0.45 : 0.10)
    readonly property color shadowFar:  Qt.rgba(0, 0, 0, isDark ? 0.30 : 0.05)

    // Quick check for whether the host theme is dark — we use this
    // to lift our subtle surface tints the right direction.
    //
    // The user's Settings → Appearance toggle is consulted first:
    //   0 (System): luminance of Kirigami.Theme.backgroundColor
    //   1 (Light) : forced false
    //   2 (Dark)  : forced true
    //   3..11     : dark-leaning custom palettes — forced true so the
    //               surface tints / separators bias the right
    //               direction until each variant defines its own
    //               color stops.
    //
    // Effect: a real "apply now" theme toggle without requiring a
    // KColorSchemeManager binding through cxx-qt-lib.
    readonly property bool isDark: {
        const pref = app.settingsController ? app.settingsController.theme : 0
        if (pref === 1) return false
        if (pref === 2) return true
        if (pref >= 3 && pref <= 11) return true
        const c = Kirigami.Theme.backgroundColor
        return (c.r + c.g + c.b) / 3.0 < 0.5
    }

    // ---- Motion -----------------------------------------------------
    // When the user has asked for reduced motion in Settings →
    // Accessibility, every duration collapses to zero. Animations
    // still execute (they're declarative bindings on `Behavior` and
    // `Transition`), they just complete instantly.
    readonly property bool reducedMotion: app.settingsController
        ? app.settingsController.accessibilityReducedMotion : false

    readonly property int durationSnap:        reducedMotion ? 0 : 110   // toggles, hover
    readonly property int durationNormal:      reducedMotion ? 0 : 180   // page transitions, fades
    readonly property int durationSlow:        reducedMotion ? 0 : 280   // result-list populate
    readonly property int durationDecorative:  reducedMotion ? 0 : 420   // empty-state breathing

    readonly property int easing: Easing.OutCubic

    // High-contrast bias: when the toggle is on, push every text
    // color closer to pure black/white and every separator closer
    // to the active foreground. Cheap to compute; reads as a meta-
    // theme over whichever Kirigami palette is current.
    readonly property bool highContrast: app.settingsController
        ? app.settingsController.accessibilityHighContrast : false

    // ---- Density ----------------------------------------------------
    readonly property int rowCompact:  30
    readonly property int rowNormal:   44
    readonly property int rowSpacious: 60
    readonly property int navRowHeight: 36

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

    // Stable accent-tinted color from the file extension — used to
    // colour the round icon "avatar" on ResultRow so a directory of
    // results visually clusters by file type even at a glance.
    function tintForPath(path) {
        if (!path) return Qt.rgba(0.55, 0.60, 0.70, 0.22)
        const lower = path.toLowerCase()
        const dot = lower.lastIndexOf(".")
        const ext = dot >= 0 ? lower.substring(dot + 1) : ""
        switch (ext) {
            case "rs":                                return Qt.rgba(0.85, 0.42, 0.18, 0.22)
            case "py":                                return Qt.rgba(0.20, 0.55, 0.90, 0.22)
            case "go":                                return Qt.rgba(0.20, 0.78, 0.85, 0.22)
            case "c": case "h":                       return Qt.rgba(0.42, 0.55, 0.85, 0.22)
            case "cpp": case "cc": case "cxx": case "hpp": return Qt.rgba(0.30, 0.45, 0.85, 0.22)
            case "js": case "mjs": case "cjs":        return Qt.rgba(0.95, 0.82, 0.25, 0.24)
            case "ts": case "tsx":                    return Qt.rgba(0.20, 0.45, 0.85, 0.22)
            case "html": case "htm":                  return Qt.rgba(0.90, 0.35, 0.20, 0.22)
            case "css": case "scss": case "less":     return Qt.rgba(0.35, 0.45, 0.95, 0.22)
            case "json":                              return Qt.rgba(0.65, 0.55, 0.30, 0.24)
            case "toml": case "yaml": case "yml":
            case "ini": case "conf": case "cfg":      return Qt.rgba(0.50, 0.60, 0.55, 0.22)
            case "md": case "markdown": case "rst":   return Qt.rgba(0.40, 0.55, 0.95, 0.22)
            case "sh": case "bash": case "zsh": case "fish":
                                                      return Qt.rgba(0.30, 0.65, 0.40, 0.22)
            case "qml":                               return Qt.rgba(0.45, 0.30, 0.85, 0.22)
            case "java": case "kt": case "scala":     return Qt.rgba(0.85, 0.45, 0.20, 0.22)
            case "rb":                                return Qt.rgba(0.85, 0.20, 0.30, 0.22)
            case "php":                               return Qt.rgba(0.45, 0.40, 0.75, 0.22)
            case "xml": case "svg":                   return Qt.rgba(0.70, 0.55, 0.30, 0.22)
            case "lock":                              return Qt.rgba(0.55, 0.55, 0.55, 0.22)
            default:                                  return Qt.rgba(0.55, 0.60, 0.70, 0.22)
        }
    }
}
