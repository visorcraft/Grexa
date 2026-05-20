// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Shared GUI workspace stores.
//!
//! Owns the persistent stores and Fluent bundle that multiple cxx-qt
//! QObjects need. The QObjects borrow this via a thread-local pointer.

#[cfg(test)]
use std::path::Path;
use std::rc::Rc;

use grexa_core::{
    AppPaths, RecentPathStore, SearchHistoryStore, SearchProfileStore, SettingsStore,
};
use grexa_i18n::{Bundle, Locale};

/// Workspace state — one per running Grexa process.
pub struct Workspace {
    pub recent_paths: RecentPathStore,
    pub history: SearchHistoryStore,
    pub profiles: SearchProfileStore,
    pub settings: SettingsStore,
    /// Fluent localization bundle, locale-resolved from the persisted
    /// `ui_language` setting. Used by status / notification formatters
    /// to compose plural-aware count fragments without hardcoding
    /// English inflection.
    pub bundle: Rc<Bundle>,
}

impl Workspace {
    pub fn new() -> Self {
        Self::from_paths(AppPaths::from_env())
    }

    /// Use a custom XDG root — required by tests so they never write to
    /// the user's real settings.
    #[cfg(test)]
    pub fn under(base: &Path) -> Self {
        Self::from_paths(AppPaths::under(base))
    }

    fn from_paths(paths: AppPaths) -> Self {
        let settings = SettingsStore::new(&paths);
        let bundle = build_bundle(&settings);
        Self {
            recent_paths: RecentPathStore::new(&paths),
            history: SearchHistoryStore::new(&paths),
            profiles: SearchProfileStore::new(&paths),
            settings,
            bundle,
        }
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve the user's `ui_language` setting into a Fluent bundle.
/// Falls back to English if either the persisted language or the
/// catalog itself can't be loaded — so the GUI never panics on a
/// corrupt settings.json.
fn build_bundle(settings: &SettingsStore) -> Rc<Bundle> {
    let lang = settings
        .load()
        .ok()
        .map(|s| s.ui_language)
        .unwrap_or_default();
    let locale = if lang.trim().is_empty() {
        Locale::English
    } else {
        Locale::from_tag(&lang)
    };
    Rc::new(
        Bundle::for_locale(locale)
            .or_else(|_| Bundle::for_locale(Locale::English))
            .expect("English fallback bundle always loads"),
    )
}
