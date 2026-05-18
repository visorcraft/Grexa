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

    // ---- Per-theme palette -----------------------------------------
    // Mirrors upstream Grex's MainWindow.xaml.cs theme stops so a
    // Grex user can pick the same name and get the same look. Each
    // entry exposes five stops:
    //   bg        — canvas / window background
    //   secondary — sidebar chrome, card surface
    //   tertiary  — hover / pressed lift, accent-adjacent fills
    //   text      — primary text color (must read on bg)
    //   accent    — active row, primary button, focus ring, selection
    //
    // Themes 0/1/2 (System/Light/Dark) intentionally return null for
    // bg/secondary/tertiary so the host Kirigami palette stays in
    // charge of the chrome — only the accent is forced for those.
    //
    // Index map (matches SettingsPage.qml's ComboBox and the
    // ThemePreference enum on the Rust side):
    //   0 Follow system  3 Gentle Gecko  6 Dreams      9 Subspace
    //   1 Light          4 Black Knight  7 Paranoid   10 Tiefling
    //   2 Dark           5 Diamond       8 Red Velvet 11 Vibes
    readonly property int themeIdx: app.settingsController
        ? app.settingsController.theme : 0

    function themePalette(idx) {
        switch (idx) {
            case 3:  return { bg: "#000000", secondary: "#003322", tertiary: "#00593D", text: "#FFFFFF", accent: "#00B86B" } // Gentle Gecko
            case 4:  return { bg: "#000000", secondary: "#003366", tertiary: "#00478F", text: "#FFFFFF", accent: "#0078D4" } // Black Knight
            case 5:  return { bg: "#2D5B67", secondary: "#4F7F8C", tertiary: "#7CA2B1", text: "#B9DAE9", accent: "#A5C5D5" } // Diamond
            case 6:  return { bg: "#210B4B", secondary: "#3F1C6D", tertiary: "#6A2A98", text: "#FF3D94", accent: "#B5307E" } // Dreams
            case 7:  return { bg: "#1D1D4E", secondary: "#3F3F88", tertiary: "#5F5FBF", text: "#D2D2F4", accent: "#9A9AE0" } // Paranoid
            case 8:  return { bg: "#1A0F0F", secondary: "#3C1414", tertiary: "#8B2323", text: "#FFDCDC", accent: "#DC3C3C" } // Red Velvet
            case 9:  return { bg: "#2E1A47", secondary: "#4A2A6A", tertiary: "#794B8B", text: "#E2C7E6", accent: "#B77BB4" } // Subspace
            case 10: return { bg: "#3A0A4D", secondary: "#711D9A", tertiary: "#A42DB4", text: "#F9C54E", accent: "#FF5C8A" } // Tiefling
            case 11: return { bg: "#0F0F1E", secondary: "#1E1E3C", tertiary: "#CC00FF", text: "#00FFCC", accent: "#FFCC00" } // Vibes
            case 1:  return { bg: "#F5F5F5", secondary: null,      tertiary: null,      text: "#1A1A1A", accent: "#2D7FF9" } // Light
            case 2:  return { bg: "#181818", secondary: null,      tertiary: null,      text: "#F5F5F5", accent: "#2D7FF9" } // Dark
            default: return { bg: null,      secondary: null,      tertiary: null,      text: null,      accent: "#2D7FF9" } // System
        }
    }
    readonly property var palette: themePalette(themeIdx)
    readonly property bool customPalette: themeIdx >= 3

    // ---- Accent ----------------------------------------------------
    // Derived: hover / pressed are tonal shifts; mute is the alpha
    // wash used for the active nav row fill and selection tint.
    readonly property color accent:        palette.accent
    readonly property color accentHover:   Qt.lighter(accent, 1.15)
    readonly property color accentPressed: Qt.darker(accent, 1.15)
    readonly property color accentDeep:    Qt.darker(accent, 1.55)
    readonly property color accentMute:    Qt.rgba(accent.r, accent.g, accent.b, 0.18)
    readonly property color accentMuteStrong: Qt.rgba(accent.r, accent.g, accent.b, 0.28)
    readonly property color accentRing:    Qt.rgba(accent.r, accent.g, accent.b, 0.40)
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
    // surface0  — canvas / page background
    // surfaceSidebar — chrome panel; uses the palette's secondary
    //   when defined, otherwise a luminance tint over the host bg
    // surface1  — elevated row / card surface (above canvas)
    // surface2  — highest lift, hover / press states, headlines
    //
    // For named themes (idx >= 3) the surfaces come straight from
    // Grex's per-theme color stops. For System/Light/Dark, surfaces
    // are derived from the host Kirigami background with luminance
    // tints — preserving the original Mailspring-style chrome.
    readonly property color surface0: customPalette
        ? palette.bg
        : (palette.bg !== null ? palette.bg : Kirigami.Theme.backgroundColor)
    readonly property color surfaceSidebar: {
        if (customPalette) return palette.secondary
        return Qt.tint(surface0,
                       isDark
                           ? Qt.rgba(0.0, 0.0, 0.0, 0.22)
                           : Qt.rgba(0.20, 0.30, 0.50, 0.06))
    }
    readonly property color surface1: {
        if (customPalette) return palette.secondary
        return Qt.tint(surface0,
                       isDark ? Qt.rgba(1, 1, 1, 0.07)
                              : Qt.rgba(0, 0, 0, 0.04))
    }
    readonly property color surface2: {
        if (customPalette) return palette.tertiary
        return Qt.tint(surface0,
                       isDark ? Qt.rgba(1, 1, 1, 0.12)
                              : Qt.rgba(0, 0, 0, 0.07))
    }
    // Primary text color — overridden for named themes so high-
    // chroma palettes (Tiefling's gold text on plum, Dreams's pink
    // on violet) match Grex.
    readonly property color textPrimary: customPalette
        ? palette.text
        : Kirigami.Theme.textColor
    readonly property color surfaceCard:  surface1
    // High-contrast biases the separator alpha up so card edges
    // remain visible against very light or very dark wallpapers.
    readonly property color separator:    Qt.rgba(textPrimary.r,
                                                  textPrimary.g,
                                                  textPrimary.b,
                                                  highContrast ? (isDark ? 0.32 : 0.22)
                                                               : (isDark ? 0.12 : 0.09))
    readonly property color separatorStrong: Qt.rgba(textPrimary.r,
                                                     textPrimary.g,
                                                     textPrimary.b,
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
