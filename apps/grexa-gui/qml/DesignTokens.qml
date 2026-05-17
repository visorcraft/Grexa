// Grexa design tokens. Single source of truth for spacing, radius,
// colors, typography, animation duration, row density, and table
// metrics. Every QML page imports this as a property to ensure visual
// consistency across pages.
//
// Tokens are intentionally numeric / color literals — they do NOT bind
// to Kirigami.Theme because we want a stable visual rhythm even when
// the user picks a high-contrast variant. Each Grexa theme override
// (Phase 18 follow-up) supplies its own token set.

pragma Singleton
import QtQuick

QtObject {
    // -- Spacing rhythm --------------------------------------------------
    readonly property int spacingTiny:   2
    readonly property int spacingSmall:  4
    readonly property int spacingNormal: 8
    readonly property int spacingLarge:  12
    readonly property int spacingXL:     20

    // -- Radii ----------------------------------------------------------
    readonly property int radiusButton: 4
    readonly property int radiusCard:   6
    readonly property int radiusInput:  4

    // -- Typography -----------------------------------------------------
    readonly property string monoFamily: "monospace"
    readonly property int sizeSmall:  11
    readonly property int sizeNormal: 13
    readonly property int sizeHeader: 16

    // -- Animation ------------------------------------------------------
    readonly property int durationFast:    120
    readonly property int durationNormal:  200
    readonly property int durationSlow:    360

    // -- Row density ----------------------------------------------------
    readonly property int rowHeightCompact:  22
    readonly property int rowHeightNormal:   28
    readonly property int rowHeightSpacious: 36

    // -- Table metrics --------------------------------------------------
    readonly property int gutterWidth:        50
    readonly property int matchIndicatorWidth: 4
    readonly property color matchIndicator:   "#3DAEE9"
    readonly property color matchRowOverlay:  "#283D4F"

    // -- Status colors --------------------------------------------------
    readonly property color statusOk:    "#27AE60"
    readonly property color statusWarn:  "#E67E22"
    readonly property color statusError: "#C0392B"

    // -- Theme tokens ---------------------------------------------------
    // The actual colors come from Kirigami.Theme at runtime; these are
    // semantic accessors so Phase 18 work has somewhere to centralize.
    readonly property color surfaceQuiet: Qt.rgba(0, 0, 0, 0.04)
    readonly property color separator:    Qt.rgba(0, 0, 0, 0.10)
}
