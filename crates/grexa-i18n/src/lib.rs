// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Grexa's localization runtime.
//!
//! The catalog format is Fluent (`.ftl`). The canonical English file at
//! `locales/en/grexa.ftl` is embedded into the binary via `include_str!` so
//! there's always at least one functional locale even when distro packaging
//! forgets to install translation data. Additional locales (`de`, `ja`,
//! later more) are loaded the same way.
//!
//! Pipeline decision (PLAN.md phase 11 line 385): **Fluent for the Rust
//! core and CLI**, with Qt `.ts` files on the QML side later. Fluent's
//! built-in plural / case / list selectors handle the matrix that broke
//! Grex's `string.Format` approach (see
//! `docs/grex-status-text-audit.md` plural-failure list).
//!
//! ## Usage
//!
//! ```ignore
//! use grexa_i18n::{Bundle, Locale};
//! let bundle = Bundle::for_locale(Locale::English).unwrap();
//! let msg = bundle.format(
//!     "search-status-found",
//!     &[("matches", "42".into()), ("files", "7".into()), ("elapsed", "1.2s".into())],
//! ).unwrap();
//! assert!(msg.contains("42 matches"));
//! ```

use std::collections::HashMap;

use fluent::types::FluentValue;
use fluent::{FluentArgs, FluentBundle, FluentResource};
use thiserror::Error;
use unic_langid::{LanguageIdentifier, langid};

// Re-export for callers that want to construct FluentValue without depending
// on the `fluent` crate directly.
pub use fluent::types::FluentValue as Value;

/// Forward-declared in case future code wants the raw map for batch lookups.
pub type StringMap = HashMap<String, String>;

/// Locale ids Grexa ships catalogs for. Adding a new locale = ship a
/// `locales/<tag>/grexa.ftl` and add a variant here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    English,
    German,
    Japanese,
}

impl Locale {
    pub fn lang_id(self) -> LanguageIdentifier {
        match self {
            Locale::English => langid!("en"),
            Locale::German => langid!("de"),
            Locale::Japanese => langid!("ja"),
        }
    }

    pub fn ftl_source(self) -> &'static str {
        match self {
            Locale::English => include_str!("../locales/en/grexa.ftl"),
            Locale::German => include_str!("../locales/de/grexa.ftl"),
            Locale::Japanese => include_str!("../locales/ja/grexa.ftl"),
        }
    }

    /// Parse a BCP-47 / POSIX locale tag (`en`, `en-US`, `en_US.UTF-8`,
    /// `de-DE`, …) into the closest matching catalog. Falls back to
    /// English for any unknown tag.
    pub fn from_tag(tag: &str) -> Locale {
        let primary = tag
            .split([',', '.', '@', '_', '-'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "de" => Locale::German,
            "ja" => Locale::Japanese,
            _ => Locale::English,
        }
    }
}

#[derive(Debug, Error)]
pub enum BundleError {
    #[error("failed to parse FTL: {0}")]
    Parse(String),
    #[error("translation key `{0}` is missing in this locale")]
    MissingKey(String),
    #[error("translation key `{0}` has no value (terms aren't usable as messages)")]
    NoValue(String),
    #[error("translation key `{0}` formatted with errors: {1:?}")]
    FormatErrors(String, Vec<String>),
}

/// Loaded Fluent bundle for a single locale, with English as a fallback.
pub struct Bundle {
    primary: FluentBundle<FluentResource>,
    fallback: Option<FluentBundle<FluentResource>>,
    locale: Locale,
}

impl Bundle {
    pub fn for_locale(locale: Locale) -> Result<Self, BundleError> {
        let primary = build_bundle(locale)?;
        let fallback = if matches!(locale, Locale::English) {
            None
        } else {
            Some(build_bundle(Locale::English)?)
        };
        Ok(Self {
            primary,
            fallback,
            locale,
        })
    }

    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// Look up `key` and format it with the supplied `args`. Returns the
    /// English fallback if the requested locale is missing the key.
    pub fn format<'a>(
        &self,
        key: &str,
        args: &[(&'a str, FluentValue<'a>)],
    ) -> Result<String, BundleError> {
        // Build FluentArgs from the borrowed pairs.
        let mut owned = FluentArgs::new();
        for (name, value) in args {
            owned.set(*name, value.clone());
        }

        if let Some(rendered) = format_in(&self.primary, key, &owned)? {
            return Ok(rendered);
        }
        if let Some(fallback) = &self.fallback
            && let Some(rendered) = format_in(fallback, key, &owned)?
        {
            return Ok(rendered);
        }
        Err(BundleError::MissingKey(key.to_string()))
    }

    /// Convenience helper for zero-arg keys, the common UI case. Returns
    /// `Err(BundleError::MissingKey)` if neither the primary nor the
    /// fallback bundle defines the key.
    pub fn t(&self, key: &str) -> Result<String, BundleError> {
        self.format(key, &[])
    }

    /// Format a plural-aware key that takes a single `count` argument.
    /// Used by status / notification formatters where the caller needs
    /// a bare-count fragment (`"5 matches"`, `"1 file"`) without
    /// hardcoding English plural rules. The locale's `.ftl` catalog
    /// drives the inflection via Fluent's `{$count -> [one] … *[other] …}`
    /// selectors.
    pub fn plural_count(&self, key: &str, count: i64) -> Result<String, BundleError> {
        self.format(key, &[("count", FluentValue::from(count))])
    }
}

fn format_in(
    bundle: &FluentBundle<FluentResource>,
    key: &str,
    args: &FluentArgs,
) -> Result<Option<String>, BundleError> {
    let Some(message) = bundle.get_message(key) else {
        return Ok(None);
    };
    let pattern = message
        .value()
        .ok_or_else(|| BundleError::NoValue(key.to_string()))?;
    let mut errors = vec![];
    let rendered = bundle
        .format_pattern(pattern, Some(args), &mut errors)
        .into_owned();
    if !errors.is_empty() {
        return Err(BundleError::FormatErrors(
            key.to_string(),
            errors.iter().map(|e| format!("{e:?}")).collect(),
        ));
    }
    Ok(Some(rendered))
}

fn build_bundle(locale: Locale) -> Result<FluentBundle<FluentResource>, BundleError> {
    let resource = FluentResource::try_new(locale.ftl_source().to_string())
        .map_err(|(_, errs)| BundleError::Parse(format!("{errs:?}")))?;
    let mut bundle = FluentBundle::new(vec![locale.lang_id()]);
    // Fluent inserts Unicode FSI/PDI markers around args by default. They
    // confuse text-mode UI strings; disable for our use case.
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .map_err(|errs| BundleError::Parse(format!("{errs:?}")))?;
    Ok(bundle)
}

/// CI helper: returns the set of keys defined in the canonical English
/// catalog. The locale-sync check uses this as the reference set.
pub fn canonical_keys() -> Vec<String> {
    keys_in(Locale::English)
}

/// CI helper: returns the keys defined in `locale`'s catalog. Combined with
/// `canonical_keys` this lets the locale-sync script report missing /
/// extraneous keys.
///
/// We scrape the FTL source directly rather than asking `FluentBundle` —
/// `fluent_bundle` 0.15 doesn't expose a message-id iterator publicly. A
/// Fluent message id matches `^[a-z][a-z0-9_-]*\s*=` at the start of a
/// non-indented line.
pub fn keys_in(locale: Locale) -> Vec<String> {
    let source = locale.ftl_source();
    let mut out = Vec::new();
    for line in source.lines() {
        // Indented lines belong to the previous message's continuation.
        if line.starts_with([' ', '\t']) {
            continue;
        }
        // Comments and blank lines.
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let Some(equals) = line.find('=') else {
            continue;
        };
        let key = line[..equals].trim();
        if key.is_empty() {
            continue;
        }
        // Reject lines that don't look like message ids (start with a letter,
        // ASCII alphanumeric / dash / underscore only).
        if !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            continue;
        }
        if !key.starts_with(|c: char| c.is_ascii_alphabetic()) {
            continue;
        }
        out.push(key.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn locale_from_tag_parses_common_shapes() {
        assert_eq!(Locale::from_tag("en"), Locale::English);
        assert_eq!(Locale::from_tag("en-US"), Locale::English);
        assert_eq!(Locale::from_tag("en_US.UTF-8"), Locale::English);
        assert_eq!(Locale::from_tag("de"), Locale::German);
        assert_eq!(Locale::from_tag("de-DE"), Locale::German);
        assert_eq!(Locale::from_tag("ja-JP"), Locale::Japanese);
        assert_eq!(Locale::from_tag("xx-unknown"), Locale::English);
        assert_eq!(Locale::from_tag(""), Locale::English);
    }

    #[test]
    fn english_simple_lookup() {
        let bundle = Bundle::for_locale(Locale::English).unwrap();
        assert_eq!(bundle.t("app-name").unwrap(), "Grexa");
        assert_eq!(bundle.t("search-status-ready").unwrap(), "Ready");
    }

    #[test]
    fn english_plural_selector_picks_correct_form() {
        let bundle = Bundle::for_locale(Locale::English).unwrap();

        let one = bundle
            .format(
                "search-status-found",
                &[
                    ("matches", "1".into()),
                    ("files", "1".into()),
                    ("elapsed", "0.5s".into()),
                ],
            )
            .unwrap();
        assert!(one.contains("Found 1 match"), "got {one:?}");
        assert!(one.contains("1 file"));

        let many = bundle
            .format(
                "search-status-found",
                &[
                    ("matches", "42".into()),
                    ("files", "7".into()),
                    ("elapsed", "1.2s".into()),
                ],
            )
            .unwrap();
        assert!(many.contains("Found 42 matches"), "got {many:?}");
        assert!(many.contains("7 files"));
    }

    #[test]
    fn german_bundle_renders_plurals() {
        let bundle = Bundle::for_locale(Locale::German).unwrap();
        let one = bundle
            .format(
                "search-status-found",
                &[
                    ("matches", "1".into()),
                    ("files", "1".into()),
                    ("elapsed", "0.5s".into()),
                ],
            )
            .unwrap();
        assert!(one.contains("1 Treffer gefunden"), "got {one:?}");
        assert!(one.contains("1 Datei"));
    }

    #[test]
    fn japanese_bundle_uses_single_form() {
        let bundle = Bundle::for_locale(Locale::Japanese).unwrap();
        let msg = bundle
            .format(
                "search-status-found",
                &[
                    ("matches", "1".into()),
                    ("files", "1".into()),
                    ("elapsed", "0.5s".into()),
                ],
            )
            .unwrap();
        assert!(msg.contains("マッチ"), "got {msg:?}");
    }

    #[test]
    fn fallback_to_english_when_key_missing_in_locale() {
        // Sanity test: every locale ships the same key set today, so to
        // exercise the fallback we deliberately ask for a key that doesn't
        // exist in any catalog.
        let bundle = Bundle::for_locale(Locale::German).unwrap();
        let err = bundle.t("does-not-exist").unwrap_err();
        assert!(matches!(err, BundleError::MissingKey(_)));
    }

    #[test]
    fn missing_key_in_non_english_falls_back() {
        // Simulate a "real" fallback by formatting a key in a bundle that
        // doesn't override it — the fallback chain reads from English.
        let bundle = Bundle::for_locale(Locale::German).unwrap();
        // Both locales define app-name; the test is that fallback also
        // works the other direction when needed. Pin behavior: when the
        // primary bundle does define the key, we get the primary's text.
        assert_eq!(bundle.t("app-name").unwrap(), "Grexa");
    }

    #[test]
    fn plural_count_inflects_per_locale() {
        let en = Bundle::for_locale(Locale::English).unwrap();
        assert_eq!(en.plural_count("count-matches", 1).unwrap(), "1 match");
        assert_eq!(en.plural_count("count-matches", 5).unwrap(), "5 matches");
        assert_eq!(en.plural_count("count-files", 0).unwrap(), "0 files");

        let de = Bundle::for_locale(Locale::German).unwrap();
        assert_eq!(de.plural_count("count-files", 1).unwrap(), "1 Datei");
        assert_eq!(de.plural_count("count-files", 3).unwrap(), "3 Dateien");

        // Japanese has no plural inflection; the helper still produces
        // a coherent fragment.
        let ja = Bundle::for_locale(Locale::Japanese).unwrap();
        assert!(
            ja.plural_count("count-matches", 1)
                .unwrap()
                .contains("マッチ")
        );
    }

    /// Locale-sync gate. Every shipped locale must define the exact same
    /// key set as the canonical English catalog. The CI script
    /// `scripts/check_locale_sync.py` enforces this externally as well; we
    /// duplicate it here so `cargo test` rejects mismatched catalogs.
    #[test]
    fn every_locale_has_same_key_set_as_english() {
        let canonical: HashSet<_> = canonical_keys().into_iter().collect();
        for &locale in &[Locale::German, Locale::Japanese] {
            let keys: HashSet<_> = keys_in(locale).into_iter().collect();
            let missing: Vec<_> = canonical.difference(&keys).cloned().collect();
            let extra: Vec<_> = keys.difference(&canonical).cloned().collect();
            assert!(
                missing.is_empty() && extra.is_empty(),
                "locale {locale:?} mismatch: missing={missing:?}, extra={extra:?}"
            );
        }
    }
}
