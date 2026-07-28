# Translating Grexa

Grexa uses [Fluent](https://projectfluent.org/) catalogs embedded into the
binary. English is canonical. German and Japanese currently ship beside it.

```text
crates/grexa-i18n/locales/
├── en/grexa.ftl
├── de/grexa.ftl
└── ja/grexa.ftl
```

Every shipped locale must define exactly the same message IDs as English.

## Update an existing translation

1. Find the English key in
   `crates/grexa-i18n/locales/en/grexa.ftl`.
2. Update the same key in the target catalog.
3. Preserve all variables and selector branches.
4. Run:

   ```bash
   python3 scripts/check_locale_sync.py
   cargo test -p grexa-i18n
   ```

5. Launch Grexa with the target UI language and check the affected page at
   normal and narrow window widths.

Do not rename an ID only in one locale.

## Add a user-facing string

1. Add a concept-based ID to English.
2. Add the same ID to German and Japanese.
3. Use the key from Rust or QML.
4. Run the sync checks.

QML:

```qml
Controls.Label {
    text: app.i18n("ui-search-term")
}
```

Plural QML:

```qml
Controls.Label {
    text: app.i18nPlural("count-matches", matchCount)
}
```

Rust:

```rust
let text = bundle.t("search-status-ready")?;
let count = bundle.plural_count("count-files", files as i64)?;
```

New QML text must not use `qsTr()`. All shipped QML strings have moved to
Fluent.

## Add a locale

Use a primary BCP-47 language tag unless regional catalogs have materially
different translations.

1. Copy the English catalog:

   ```bash
   mkdir -p crates/grexa-i18n/locales/fr
   cp crates/grexa-i18n/locales/en/grexa.ftl \
     crates/grexa-i18n/locales/fr/grexa.ftl
   ```

2. Translate every value without changing IDs.
3. Add a `Locale` variant in `crates/grexa-i18n/src/lib.rs`.
4. Add that variant to:
   - `Locale::lang_id`;
   - `Locale::ftl_source`;
   - `Locale::from_tag`;
   - `every_locale_has_same_key_set_as_english`.
5. Add locale-specific plural tests when the language has meaningful plural
   categories.
6. Add the locale to the shipped-language lists in
   [README](../README.md), [Features](features.md), and
   [Reference](reference.md).
7. Run:

   ```bash
   python3 scripts/check_locale_sync.py
   cargo test -p grexa-i18n
   just ci
   ```

8. Manually check Search, History, Profiles, Settings, dialogs, empty states,
   status text, About, Credits, and Licenses.

Catalogs are compiled with `include_str!`; packages do not install separate
`.mo`, `.qm`, or `.ftl` files.

## Fluent syntax

Simple message:

```ftl
search-status-ready = Ready
```

Variable:

```ftl
search-status-error = Error: { $message }
```

Plural selector:

```ftl
count-files =
    { $count ->
        [one] { $count } file
       *[other] { $count } files
    }
```

Keep the variable names used by callers. Translators may reorder text around
variables.

Use the plural categories required by the target language. The `other` branch
must be marked as the default with `*`.

Comments:

```ftl
# Translator note for one message
## Section heading
### Subsection heading
```

## Locale resolution

`Locale::from_tag` accepts BCP-47 and common POSIX forms:

```text
de
de-DE
de_DE.UTF-8
ja-JP
```

It selects by primary language. Unknown languages fall back to English.

`Bundle::for_locale` also carries an English fallback bundle. A missing target
message therefore falls back at runtime, but the key-parity tests prevent
shipping that state.

## What the checks enforce

`cargo test -p grexa-i18n` verifies:

- catalogs parse;
- locale resolution;
- formatting and plural helpers;
- every non-English catalog has exactly the English ID set.

`python3 scripts/check_locale_sync.py` additionally verifies:

- every QML file under `apps/grexa-gui/qml/` is listed in
  `apps/grexa-gui/build.rs`;
- no empty `qsTr()` call exists;
- reports that all `qsTr()` calls have been migrated.

`just ci` runs the Rust parity test. Run the Python checker directly whenever
QML files or localized strings change.

The checks do not prove translation quality, placeholder meaning, layout fit,
or correct grammar. Review those manually.

## Review checklist

- Meaning matches the English source and UI context.
- Variables are neither removed nor renamed.
- Plural categories fit the target language.
- Punctuation and capitalization are natural for that locale.
- Keyboard labels and button text remain concise.
- Long text wraps without clipping.
- Placeholders such as `%1` that are intentionally consumed by QML `.arg()`
  remain present.
- File ends with a newline.
- No new English text is hardcoded in QML or user-facing Rust paths.

## Relationship to Grex

Grex used `.resw` resources and positional `string.Format` placeholders.
Grexa uses concept IDs, named Fluent variables, and plural selectors.

The source mapping is preserved in
[grex-strings-migration-matrix.md](grex-strings-migration-matrix.md). It is a
reference for intent, not an input format that Grexa loads at runtime.
