# Accessibility

Grexa supports keyboard-driven search and uses Qt Quick accessibility metadata
for its primary controls, result list, and AI conversation. Accessibility is
an active release criterion, not a claim of certification.

## Current support

### Keyboard

The desktop application provides:

- global navigation shortcuts;
- search, stop, tab, focus, and quit shortcuts;
- keyboard traversal through standard Qt controls;
- Space for result context preview;
- Enter for open-in-editor;
- Escape for search cancellation and dialog/drawer close behavior.

The complete mapping is in
[Reference: desktop keyboard and pointer actions](reference.md#desktop-keyboard-and-pointer-actions).

### Assistive technology metadata

Shared controls declare Qt `Accessible` roles and names:

- buttons and navigation items;
- text fields;
- check boxes;
- combo boxes;
- spin boxes and sliders;
- result rows and result list;
- AI message list;
- static empty states.

Result rows announce `relative-path:line` as their name and the matched preview
as their description. Decorative icons in the search and result surfaces are
marked ignored where they would add noise.

Raw Qt Quick Controls retain their native roles. New custom interaction
surfaces must set an explicit role, name, and description when the native
control cannot infer them.

### Visual settings

**Settings → Accessibility** provides:

- **Reduce motion**: token animation durations become zero and busy animations
  stop where wired;
- **High contrast**: selected theme tokens use stronger border and text
  contrast.

The appearance selector also includes system, light, dark, OLED black, and
named Grex-derived palettes.

### CLI

The CLI:

- emits one logical match per line;
- has stable text, JSON, and CSV shapes;
- uses documented grep-like exit codes;
- supports `--quiet`, `--count`, and `--files-only`;
- does not require color to understand status;
- replaces terminal control characters only when writing directly to a TTY.

## Known limitations

- CI builds and launches the GUI offscreen, but does not run an automated
  AT-SPI or screen-reader assertion suite.
- Orca behavior must be checked manually before releases that materially
  change navigation, dialogs, result rows, or custom controls.
- Grexa currently ships no right-to-left locale, so RTL layout has not received
  a full translated end-to-end release check.
- Reduced motion is an application setting; Grexa does not automatically read
  every desktop environment's animation preference.
- High contrast changes Grexa's token palette. It does not integrate with
  every desktop high-contrast protocol or guarantee a formal WCAG conformance
  level.
- Database-page controls mostly use native Qt roles and have less custom
  descriptive metadata than the Search and Settings surfaces.

Report accessibility bugs through
[GitHub Issues](https://github.com/visorcraft/grexa/issues). Include the Grexa
version, desktop, Qt version, assistive technology, input method, and exact
control or workflow.

## Manual verification

### Keyboard-only

1. Start with a fresh process.
2. Reach every sidebar item without a pointer.
3. Enter a path and term, toggle Regex/Case/Whole Word, and run search.
4. Open/close Filters and AI drawers with keyboard input.
5. Traverse result headers and rows.
6. Open preview with Space and editor with Enter.
7. Open and cancel Replace without accidental activation.
8. Create, switch, and close tabs.
9. Visit History, Profiles, Regex Builder, Database, Settings, About,
   Credits, and Licenses.
10. Confirm visible focus does not disappear behind drawers or dialogs.

### Orca / AT-SPI

Run a normal desktop build with accessibility forced on if the environment
does not enable it automatically:

```bash
QT_ACCESSIBILITY=1 target/debug/grexa
```

Verify:

- controls announce a useful name and role;
- result rows include path, line, and match context;
- changing pages moves focus predictably;
- busy and completion state is discoverable;
- disabled controls are announced as disabled;
- icon-only controls have names;
- decorative images are silent;
- dialogs announce title, primary action, and cancel path.

### Display and motion

Check:

- system, light, dark, and OLED black;
- application high contrast on light and dark bases;
- application reduced motion;
- 100%, 125%, 150%, and 200% scale;
- narrow and default window sizes;
- long German labels;
- keyboard focus against every palette.

## Contributor requirements

For every new or changed interactive QML item:

1. Prefer an existing `App*.qml` or native Qt control.
2. Provide visible text or `Accessible.name`.
3. Add `Accessible.description` when the action is not clear from the name.
4. Use a correct role.
5. Mark decorative images/icons ignored.
6. Keep a keyboard activation path.
7. Preserve visible focus.
8. Route text through Fluent.
9. Honor `app.tokens.reducedMotion` for nonessential animation.
10. Check high-contrast colors and scaling.

Do not set `Accessible.focusOnPress` blindly on every control. Preserve the
native focus behavior unless a custom interaction requires an override.

## CI coverage

Current automated coverage provides:

- Rust tests for status strings, settings round-trip, result model roles, and
  reduced-motion/high-contrast settings;
- Fluent key parity;
- QML source checks through `scripts/check_locale_sync.py`;
- cxx-qt/QML compilation;
- offscreen root-window launch;
- desktop-entry and AppStream validation.

These checks catch broken wiring and missing text, but do not replace manual
screen-reader verification.
