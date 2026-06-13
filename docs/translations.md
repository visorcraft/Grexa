# Translating Grexa

Grexa uses [Fluent] (`.ftl`) catalogs for runtime strings. This doc
walks translators through adding or updating a locale.

[Fluent]: https://projectfluent.org/

## Where the catalogs live

```
crates/grexa-i18n/locales/
├── en/grexa.ftl       (canonical, source of truth)
├── de/grexa.ftl       (German)
└── ja/grexa.ftl       (Japanese)
```

Each locale's catalog must define **exactly** the same set of message
ids as English. The sync gate enforces this two ways:

1. `python3 scripts/check_locale_sync.py` from the repo root.
2. `cargo test -p grexa-i18n every_locale_has_same_key_set_as_english`.

Both fire in CI.

## Adding a new locale

1. Pick the BCP-47 tag (e.g. `fr` for French). Avoid region tags
   unless the language genuinely needs them (`zh-CN` vs `zh-TW`,
   `pt-BR` vs `pt-PT`).
2. Create `crates/grexa-i18n/locales/<tag>/grexa.ftl` by copying
   the English file:

   ```bash
   mkdir -p crates/grexa-i18n/locales/fr
   cp crates/grexa-i18n/locales/en/grexa.ftl crates/grexa-i18n/locales/fr/grexa.ftl
   ```

3. Translate every value. Keep every key id intact.
4. Add the locale to `crates/grexa-i18n/src/lib.rs::Locale`:

   ```rust
   pub enum Locale {
       English,
       German,
       Japanese,
       French,    // <- new
   }

   impl Locale {
       pub fn lang_id(self) -> LanguageIdentifier {
           match self {
               // …
               Locale::French => langid!("fr"),
           }
       }
       pub fn ftl_source(self) -> &'static str {
           match self {
               // …
               Locale::French => include_str!("../locales/fr/grexa.ftl"),
           }
       }
       pub fn from_tag(tag: &str) -> Locale {
           match primary.as_str() {
               // …
               "fr" => Locale::French,
           }
       }
   }
   ```

5. Add the locale to the `every_locale_has_same_key_set_as_english`
   test:

   ```rust
   for &locale in &[Locale::German, Locale::Japanese, Locale::French] {
   ```

6. Run `cargo test -p grexa-i18n` and `python3 scripts/check_locale_sync.py`.
   Both must pass.

## Fluent syntax cheat-sheet

Every message has an id and a value:

```ftl
search-status-ready = Ready
```

Argument interpolation:

```ftl
search-status-error = Error: {$message}
```

Plural selector (`one` for ≈1, `*[other]` for the default):

```ftl
search-status-found = {$matches ->
    [one] Found 1 match
   *[other] Found {$matches} matches
}
```

Use plural categories that match the target language. Russian / Polish /
Welsh / Arabic / Lithuanian need `zero` / `few` / `many` selectors;
Chinese / Japanese / Korean / Thai have a single form (no selector).

Comments use `#` (one line) or `##` (file section) or `###` (group):

```ftl
## App chrome
app-name = Grexa
```

## What changes from Grex

- **Placeholders**: Grex `.resw` files used `string.Format`'s
  positional `{0}` / `{1}` syntax. Fluent uses named placeholders
  (`{$matches}`) so translators can reorder them per-language.
- **Plurals**: Grex baked the English `s` into the resource string,
  which mis-renders other languages. Fluent's selector solves this.
- **Key scope**: Grex resource keys were per-XAML-control
  (`SearchButtonContent.Text`). Fluent keys are per-concept
  (`search-status-ready`). The migration matrix at
  [grex-strings-migration-matrix.md](grex-strings-migration-matrix.md)
  records the mapping for every Grex key.

## Using strings from QML

QML accesses the Fluent bundle through helper functions on the root
`ApplicationWindow`. Call `app.i18n("key")` for simple messages and
`app.i18nPlural("key", count)` for plural selectors:

```qml
import org.kde.kirigami as Kirigami

Kirigami.ApplicationWindow {
    // exposed by Main.qml
    function i18n(key) { return searchController.i18n(key); }
    function i18nPlural(key, n) { return searchController.i18n_plural(key, n); }
}
```

In any QML file:

```qml
Controls.Label {
    text: app.i18n("ui-search-term")
}

Controls.Label {
    text: app.i18nPlural("count-matches", matchModel.count)
}
```

Do **not** use `qsTr()` for new strings. The migration of existing
`qsTr()` calls to Fluent keys is complete, and `scripts/check_locale_sync.py`
now expects zero `qsTr()` calls in shipped QML.

## Adding new keys

1. Add the message to `crates/grexa-i18n/locales/en/grexa.ftl`.
2. Add the same key with a translated value to every other locale.
3. Run `just ci`. The sync test will fail if you missed any locale.

The CI gate ensures Grexa never ships a missing translation in any
language.

## Reviewing translations

A few rules that catch most defects:

- Plural selectors mirror the source. If English has `[one] … *[other] …`,
  the target should reuse the same branch names, even when the language
  needs more (e.g. Russian's `[few]` arm gets *added*).
- Placeholders match the source. If the source uses `{$matches}` and
  `{$files}`, the target must also reference both.
- Trailing whitespace is preserved exactly. Fluent strips one space
  on either side of `=`; everything else is significant.
- File ends with a trailing newline. The sync test treats the last
  line specially.

## Distro packaging

Translation catalogs are embedded into the binary via `include_str!`,
so packagers don't need to install separate `.mo` / `.qm` files.
Installing Grexa installs every shipped locale.

## Sync check from CI

```yaml
- name: locale sync
  run: python3 scripts/check_locale_sync.py
```

This is part of `.github/workflows/ci.yml`.
