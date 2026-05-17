// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;

use crate::models::{SearchOptions, SizeUnit, StringComparisonMode, UnicodeNormalizationMode};

const APP_DIR: &str = "grexa";
const RECENT_PATH_LIMIT: usize = 20;
const RECENT_SEARCH_LIMIT: usize = 20;
const CONTEXT_PREVIEW_MIN: u8 = 1;
const CONTEXT_PREVIEW_MAX: u8 = 20;
const MIN_IMPORTED_WINDOW_DIM: u32 = 400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub state_dir: PathBuf,
}

impl AppPaths {
    pub fn from_env() -> Self {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let config_home = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        let data_home = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"));
        let cache_home = env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".cache"));
        let state_home = env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/state"));

        Self {
            config_dir: config_home.join(APP_DIR),
            data_dir: data_home.join(APP_DIR),
            cache_dir: cache_home.join(APP_DIR),
            state_dir: state_home.join(APP_DIR),
        }
    }

    pub fn under(base: impl AsRef<Path>) -> Self {
        let base = base.as_ref();
        Self {
            config_dir: base.join("config").join(APP_DIR),
            data_dir: base.join("data").join(APP_DIR),
            cache_dir: base.join("cache").join(APP_DIR),
            state_dir: base.join("state").join(APP_DIR),
        }
    }

    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn recent_paths_file(&self) -> PathBuf {
        self.data_dir.join("recent_paths.json")
    }

    pub fn search_history_file(&self) -> PathBuf {
        self.data_dir.join("search_history.json")
    }

    pub fn search_profiles_file(&self) -> PathBuf {
        self.data_dir.join("search_profiles.json")
    }
}

#[derive(Debug, Error)]
pub enum JsonStoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("invalid settings file format")]
    NullDocument,
    #[error("invalid JSON format: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("save error: {0}")]
    Save(#[from] JsonStoreError),
}

impl ImportError {
    pub fn user_message(&self) -> String {
        match self {
            ImportError::NullDocument => "Invalid settings file format.".to_string(),
            ImportError::Parse(err) => format!("Invalid JSON format: {err}"),
            ImportError::Io(err) => format!("Error importing settings: {err}"),
            ImportError::Save(err) => format!("Error importing settings: {err}"),
        }
    }
}

/// Theme identifier. Serialized as the integer values Grex uses on Windows so
/// settings.json files round-trip across Grex and Grexa.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ThemePreference {
    #[default]
    System = 0,
    Light = 1,
    Dark = 2,
    GentleGecko = 3,
    BlackKnight = 4,
    Diamond = 5,
    Dreams = 6,
    Paranoid = 7,
    RedVelvet = 8,
    Subspace = 9,
    Tiefling = 10,
    Vibes = 11,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DefaultSettings {
    pub regex_search: bool,
    pub files_search: bool,
    pub respect_gitignore: bool,
    pub search_case_sensitive: bool,
    pub include_system_files: bool,
    pub include_subfolders: bool,
    pub include_hidden_items: bool,
    pub include_binary_files: bool,
    pub include_symbolic_links: bool,
    pub use_file_index: bool,
    pub enable_container_search: bool,
    pub size_unit: SizeUnit,
    pub theme_preference: ThemePreference,
    pub ui_language: String,
    pub string_comparison_mode: StringComparisonMode,
    pub unicode_normalization_mode: UnicodeNormalizationMode,
    pub diacritic_sensitive: bool,
    pub culture: String,
    pub default_match_files: String,
    pub default_exclude_dirs: String,
    pub content_line_column_visible: bool,
    pub content_column_column_visible: bool,
    pub content_path_column_visible: bool,
    pub files_size_column_visible: bool,
    pub files_matches_column_visible: bool,
    pub files_path_column_visible: bool,
    pub files_ext_column_visible: bool,
    pub files_encoding_column_visible: bool,
    pub files_date_modified_column_visible: bool,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub context_preview_lines_before: u8,
    pub context_preview_lines_after: u8,
    pub ai_search_endpoint: String,
    pub ai_search_model: String,
    /// AI is **opt-in**. The chat panel can be enabled even when an API key
    /// is stored, but no request is ever sent until the user toggles this on
    /// in Settings. The audit (`docs/grex-ai-search-service-audit.md`) and
    /// PLAN.md phase 8 require this explicit gate; secret storage alone is
    /// not enough.
    pub ai_search_enabled: bool,
}

impl Default for DefaultSettings {
    fn default() -> Self {
        Self {
            regex_search: false,
            files_search: false,
            respect_gitignore: false,
            search_case_sensitive: false,
            include_system_files: false,
            include_subfolders: true,
            include_hidden_items: false,
            include_binary_files: false,
            include_symbolic_links: false,
            use_file_index: false,
            enable_container_search: false,
            size_unit: SizeUnit::KB,
            theme_preference: ThemePreference::System,
            ui_language: "en-US".to_string(),
            string_comparison_mode: StringComparisonMode::Ordinal,
            unicode_normalization_mode: UnicodeNormalizationMode::None,
            diacritic_sensitive: true,
            culture: "en-US".to_string(),
            default_match_files: String::new(),
            default_exclude_dirs: String::new(),
            content_line_column_visible: true,
            content_column_column_visible: true,
            content_path_column_visible: true,
            files_size_column_visible: true,
            files_matches_column_visible: true,
            files_path_column_visible: true,
            files_ext_column_visible: true,
            files_encoding_column_visible: true,
            files_date_modified_column_visible: true,
            window_width: Some(1100),
            window_height: Some(700),
            context_preview_lines_before: 5,
            context_preview_lines_after: 5,
            ai_search_endpoint: "https://api.openai.com/v1".to_string(),
            ai_search_model: "gpt-4o-mini".to_string(),
            ai_search_enabled: false,
        }
    }
}

impl DefaultSettings {
    pub fn clamp_context_preview(&mut self) {
        self.context_preview_lines_before = self
            .context_preview_lines_before
            .clamp(CONTEXT_PREVIEW_MIN, CONTEXT_PREVIEW_MAX);
        self.context_preview_lines_after = self
            .context_preview_lines_after
            .clamp(CONTEXT_PREVIEW_MIN, CONTEXT_PREVIEW_MAX);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentSearch {
    pub search_term: String,
    pub search_path: PathBuf,
    pub match_file_names: String,
    pub exclude_dirs: String,
    pub regex_search: bool,
    pub files_search: bool,
    pub search_case_sensitive: bool,
    pub respect_gitignore: bool,
    pub include_subfolders: bool,
    pub include_hidden_items: bool,
    pub include_binary_files: bool,
    pub timestamp_unix: u64,
    pub result_count: usize,
}

impl RecentSearch {
    pub fn from_options(options: &SearchOptions, files_search: bool, result_count: usize) -> Self {
        Self {
            search_term: options.search_term.clone(),
            search_path: options.path.clone(),
            match_file_names: options.match_file_names.clone(),
            exclude_dirs: options.exclude_dirs.clone(),
            regex_search: options.regex,
            files_search,
            search_case_sensitive: options.case_sensitive,
            respect_gitignore: options.respect_gitignore,
            include_subfolders: options.include_subfolders,
            include_hidden_items: options.include_hidden,
            include_binary_files: options.include_binary,
            timestamp_unix: unix_now(),
            result_count,
        }
    }

    /// Dedupe identity. Matches Grex `RecentSearch.GetKey()` byte for byte so
    /// `search_history.json` imported from a Grex backup keeps the same row
    /// identity. The seven fields and the `True`/`False` casing come directly
    /// from `Boolean.ToString()` in C#.
    pub fn key(&self) -> String {
        format!(
            "{term}|{path}|{regex}|{files}|{case}|{match_files}|{exclude}",
            term = self.search_term,
            path = self.search_path.to_string_lossy(),
            regex = csharp_bool(self.regex_search),
            files = csharp_bool(self.files_search),
            case = csharp_bool(self.search_case_sensitive),
            match_files = self.match_file_names,
            exclude = self.exclude_dirs,
        )
    }
}

fn csharp_bool(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchProfile {
    pub name: String,
    pub search_options: SearchOptions,
    pub files_search: bool,
    pub created_unix: u64,
    pub updated_unix: u64,
}

impl SearchProfile {
    pub fn new(name: impl Into<String>, search_options: SearchOptions, files_search: bool) -> Self {
        let now = unix_now();
        Self {
            name: name.into(),
            search_options,
            files_search,
            created_unix: now,
            updated_unix: now,
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(paths: &AppPaths) -> Self {
        Self {
            path: paths.settings_file(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<DefaultSettings, JsonStoreError> {
        let mut settings: DefaultSettings = load_json_or_default(&self.path)?;
        settings.clamp_context_preview();
        Ok(settings)
    }

    pub fn save(&self, settings: &DefaultSettings) -> Result<(), JsonStoreError> {
        save_json(&self.path, settings)
    }

    /// Restore defaults by removing the settings file. Missing file is treated
    /// as success, matching Grex `SettingsService.DeleteSettingsFile`.
    pub fn delete(&self) -> Result<(), JsonStoreError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    /// Export the current on-disk settings as pretty-printed JSON, matching
    /// Grex `ExportSettingsAsJson`.
    pub fn export_json(&self) -> Result<String, JsonStoreError> {
        let settings = self.load()?;
        Ok(serde_json::to_string_pretty(&settings)?)
    }

    /// Import settings from a JSON string and merge them with the current
    /// settings, matching the rules documented in
    /// `docs/grex-storage-services-audit.md` §Import Semantics.
    ///
    /// - `null` JSON literal → [`ImportError::NullDocument`].
    /// - Unknown properties are ignored (serde `default` + missing fields fall
    ///   back to the loaded value).
    /// - `ui_language` and `culture` are kept when the imported value is empty.
    /// - Window position fields from Grex backups are dropped; window
    ///   dimensions are accepted only when sensible (≥ 400px each).
    /// - `ai_search_endpoint` and `ai_search_model` are trimmed.
    pub fn import_json(&self, json: &str) -> Result<DefaultSettings, ImportError> {
        let parsed: serde_json::Value = serde_json::from_str(json)?;
        if parsed.is_null() {
            return Err(ImportError::NullDocument);
        }

        let imported: DefaultSettings = serde_json::from_value(parsed)?;
        let mut merged = self.load().unwrap_or_default();

        merged.regex_search = imported.regex_search;
        merged.files_search = imported.files_search;
        merged.respect_gitignore = imported.respect_gitignore;
        merged.search_case_sensitive = imported.search_case_sensitive;
        merged.include_system_files = imported.include_system_files;
        merged.include_subfolders = imported.include_subfolders;
        merged.include_hidden_items = imported.include_hidden_items;
        merged.include_binary_files = imported.include_binary_files;
        merged.include_symbolic_links = imported.include_symbolic_links;
        merged.use_file_index = imported.use_file_index;
        merged.enable_container_search = imported.enable_container_search;
        merged.size_unit = imported.size_unit;
        merged.theme_preference = imported.theme_preference;

        if !imported.ui_language.is_empty() {
            merged.ui_language = imported.ui_language;
        }

        merged.string_comparison_mode = imported.string_comparison_mode;
        merged.unicode_normalization_mode = imported.unicode_normalization_mode;
        merged.diacritic_sensitive = imported.diacritic_sensitive;

        if !imported.culture.is_empty() {
            merged.culture = imported.culture;
        }

        merged.default_match_files = imported.default_match_files;
        merged.default_exclude_dirs = imported.default_exclude_dirs;

        merged.content_line_column_visible = imported.content_line_column_visible;
        merged.content_column_column_visible = imported.content_column_column_visible;
        merged.content_path_column_visible = imported.content_path_column_visible;
        merged.files_size_column_visible = imported.files_size_column_visible;
        merged.files_matches_column_visible = imported.files_matches_column_visible;
        merged.files_path_column_visible = imported.files_path_column_visible;
        merged.files_ext_column_visible = imported.files_ext_column_visible;
        merged.files_encoding_column_visible = imported.files_encoding_column_visible;
        merged.files_date_modified_column_visible = imported.files_date_modified_column_visible;

        // Window position is intentionally not imported. Dimensions are only
        // accepted when they would not produce a tiny or zero-sized window.
        if matches!(imported.window_width, Some(w) if w >= MIN_IMPORTED_WINDOW_DIM) {
            merged.window_width = imported.window_width;
        }
        if matches!(imported.window_height, Some(h) if h >= MIN_IMPORTED_WINDOW_DIM) {
            merged.window_height = imported.window_height;
        }

        merged.context_preview_lines_before = imported.context_preview_lines_before;
        merged.context_preview_lines_after = imported.context_preview_lines_after;
        merged.clamp_context_preview();

        merged.ai_search_endpoint = imported.ai_search_endpoint.trim().to_string();
        merged.ai_search_model = imported.ai_search_model.trim().to_string();
        merged.ai_search_enabled = imported.ai_search_enabled;

        self.save(&merged)?;
        Ok(merged)
    }
}

pub struct RecentPathStore {
    path: PathBuf,
    limit: usize,
}

impl RecentPathStore {
    pub fn new(paths: &AppPaths) -> Self {
        Self {
            path: paths.recent_paths_file(),
            limit: RECENT_PATH_LIMIT,
        }
    }

    pub fn load(&self) -> Result<Vec<PathBuf>, JsonStoreError> {
        load_json_or_default(&self.path)
    }

    pub fn add(&self, recent_path: impl Into<PathBuf>) -> Result<Vec<PathBuf>, JsonStoreError> {
        let recent_path = recent_path.into();
        if path_is_blank(&recent_path) {
            return self.load();
        }

        let mut paths = self.load()?;
        paths.retain(|path| path != &recent_path);
        paths.insert(0, recent_path);
        paths.truncate(self.limit);
        save_json(&self.path, &paths)?;
        Ok(paths)
    }

    pub fn remove(&self, recent_path: &Path) -> Result<Vec<PathBuf>, JsonStoreError> {
        if path_is_blank(recent_path) {
            return self.load();
        }

        let mut paths = self.load()?;
        paths.retain(|path| path != recent_path);
        save_json(&self.path, &paths)?;
        Ok(paths)
    }

    /// Case-insensitive substring filter over the path's string form. Matches
    /// Grex `FilterPaths`: empty/whitespace query returns the full list.
    pub fn filter(&self, query: &str) -> Result<Vec<PathBuf>, JsonStoreError> {
        let paths = self.load()?;
        if query.trim().is_empty() {
            return Ok(paths);
        }

        let needle = query.to_lowercase();
        Ok(paths
            .into_iter()
            .filter(|path| path.to_string_lossy().to_lowercase().contains(&needle))
            .collect())
    }
}

pub struct SearchHistoryStore {
    path: PathBuf,
    limit: usize,
}

impl SearchHistoryStore {
    pub fn new(paths: &AppPaths) -> Self {
        Self {
            path: paths.search_history_file(),
            limit: RECENT_SEARCH_LIMIT,
        }
    }

    pub fn load(&self) -> Result<Vec<RecentSearch>, JsonStoreError> {
        load_json_or_default(&self.path)
    }

    pub fn add(&self, search: RecentSearch) -> Result<Vec<RecentSearch>, JsonStoreError> {
        if search.search_term.trim().is_empty() {
            return self.load();
        }

        let key = search.key();
        let mut searches = self.load()?;
        searches.retain(|existing| existing.key() != key);
        searches.insert(0, search);
        searches.truncate(self.limit);
        save_json(&self.path, &searches)?;
        Ok(searches)
    }

    pub fn remove_by_key(&self, key: &str) -> Result<Vec<RecentSearch>, JsonStoreError> {
        let mut searches = self.load()?;
        searches.retain(|existing| existing.key() != key);
        save_json(&self.path, &searches)?;
        Ok(searches)
    }

    pub fn remove(&self, search: &RecentSearch) -> Result<Vec<RecentSearch>, JsonStoreError> {
        self.remove_by_key(&search.key())
    }

    pub fn clear(&self) -> Result<(), JsonStoreError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    /// Case-insensitive substring filter over `search_term` or `search_path`,
    /// matching Grex `FilterSearches`. Empty/whitespace query returns the full
    /// list.
    pub fn filter(&self, query: &str) -> Result<Vec<RecentSearch>, JsonStoreError> {
        let searches = self.load()?;
        if query.trim().is_empty() {
            return Ok(searches);
        }

        let needle = query.to_lowercase();
        Ok(searches
            .into_iter()
            .filter(|search| {
                search.search_term.to_lowercase().contains(&needle)
                    || search
                        .search_path
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&needle)
            })
            .collect())
    }
}

pub struct SearchProfileStore {
    path: PathBuf,
}

impl SearchProfileStore {
    pub fn new(paths: &AppPaths) -> Self {
        Self {
            path: paths.search_profiles_file(),
        }
    }

    pub fn load(&self) -> Result<Vec<SearchProfile>, JsonStoreError> {
        load_json_or_default(&self.path)
    }

    /// Case-insensitive name lookup matching Grex `Exists`.
    pub fn exists(&self, name: &str) -> Result<bool, JsonStoreError> {
        if name.trim().is_empty() {
            return Ok(false);
        }
        Ok(self
            .load()?
            .iter()
            .any(|profile| profile.name.eq_ignore_ascii_case(name)))
    }

    /// Insert a new profile or update an existing one (case-insensitive name
    /// match). The touched profile is moved to index 0 to match Grex
    /// `SearchProfilesService.AddOrUpdateProfile` ordering.
    pub fn upsert(&self, mut profile: SearchProfile) -> Result<Vec<SearchProfile>, JsonStoreError> {
        if profile.name.trim().is_empty() {
            return self.load();
        }

        let mut profiles = self.load()?;
        let now = unix_now();

        let existing_index = profiles
            .iter()
            .position(|existing| existing.name.eq_ignore_ascii_case(&profile.name));

        match existing_index {
            Some(index) => {
                let existing = profiles.remove(index);
                profile.created_unix = if existing.created_unix == 0 {
                    now
                } else {
                    existing.created_unix
                };
                profile.updated_unix = now;
            }
            None => {
                if profile.created_unix == 0 {
                    profile.created_unix = now;
                }
                profile.updated_unix = now;
            }
        }

        profiles.insert(0, profile);
        save_json(&self.path, &profiles)?;
        Ok(profiles)
    }

    pub fn remove(&self, name: &str) -> Result<Vec<SearchProfile>, JsonStoreError> {
        if name.trim().is_empty() {
            return self.load();
        }

        let mut profiles = self.load()?;
        profiles.retain(|profile| !profile.name.eq_ignore_ascii_case(name));
        save_json(&self.path, &profiles)?;
        Ok(profiles)
    }

    pub fn clear(&self) -> Result<(), JsonStoreError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

fn load_json_or_default<T>(path: &Path) -> Result<T, JsonStoreError>
where
    T: DeserializeOwned + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }

    let bytes = fs::read(path)?;
    if bytes.iter().all(|b| b.is_ascii_whitespace()) {
        return Ok(T::default());
    }
    Ok(serde_json::from_slice(&bytes)?)
}

fn save_json<T>(path: &Path, value: &T) -> Result<(), JsonStoreError>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_vec_pretty(value)?;
    fs::write(path, json)?;
    Ok(())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn path_is_blank(path: &Path) -> bool {
    path.as_os_str().is_empty() || path.to_str().map(|s| s.trim().is_empty()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;

    use super::*;

    fn make_paths() -> (tempfile::TempDir, AppPaths) {
        let dir = tempdir().unwrap();
        let paths = AppPaths::under(dir.path());
        (dir, paths)
    }

    #[test]
    fn settings_round_trip() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        let mut settings = store.load().unwrap();
        settings.default_match_files = "*.rs".to_string();
        store.save(&settings).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.default_match_files, "*.rs");
    }

    #[test]
    fn settings_load_with_no_file_returns_defaults() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);
        let loaded = store.load().unwrap();
        assert_eq!(loaded, DefaultSettings::default());
    }

    #[test]
    fn settings_delete_tolerates_missing_file() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);
        store.delete().unwrap();
    }

    #[test]
    fn settings_delete_resets_defaults() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        let mut settings = store.load().unwrap();
        settings.regex_search = true;
        store.save(&settings).unwrap();
        assert!(store.load().unwrap().regex_search);

        store.delete().unwrap();
        assert!(!store.load().unwrap().regex_search);
    }

    #[test]
    fn settings_export_emits_pretty_json() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);
        let mut settings = store.load().unwrap();
        settings.theme_preference = ThemePreference::Dark;
        settings.ui_language = "de-DE".to_string();
        store.save(&settings).unwrap();

        let json = store.export_json().unwrap();
        assert!(json.contains('\n'));
        assert!(json.contains("\"theme_preference\": 2"));
        assert!(json.contains("\"ui_language\": \"de-DE\""));
    }

    #[test]
    fn settings_export_uses_grex_theme_integer_values() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        for (value, name) in [
            (3, ThemePreference::GentleGecko),
            (4, ThemePreference::BlackKnight),
            (5, ThemePreference::Diamond),
            (6, ThemePreference::Dreams),
            (7, ThemePreference::Paranoid),
            (8, ThemePreference::RedVelvet),
            (9, ThemePreference::Subspace),
            (10, ThemePreference::Tiefling),
            (11, ThemePreference::Vibes),
        ] {
            let mut settings = store.load().unwrap();
            settings.theme_preference = name;
            store.save(&settings).unwrap();
            let json = store.export_json().unwrap();
            assert!(
                json.contains(&format!("\"theme_preference\": {value}")),
                "theme {name:?} should serialize as {value}"
            );
        }
    }

    #[test]
    fn settings_import_updates_provided_fields() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        let json = r#"{
            "regex_search": true,
            "files_search": true,
            "respect_gitignore": true,
            "search_case_sensitive": true,
            "include_subfolders": false,
            "include_hidden_items": true,
            "theme_preference": 1,
            "ui_language": "es-ES"
        }"#;

        let imported = store.import_json(json).unwrap();
        assert!(imported.regex_search);
        assert!(imported.files_search);
        assert!(imported.respect_gitignore);
        assert!(imported.search_case_sensitive);
        assert!(!imported.include_subfolders);
        assert!(imported.include_hidden_items);
        assert_eq!(imported.theme_preference, ThemePreference::Light);
        assert_eq!(imported.ui_language, "es-ES");
    }

    #[test]
    fn settings_import_drops_window_position_and_clamps_small_dims() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        let mut settings = store.load().unwrap();
        settings.window_width = Some(1100);
        settings.window_height = Some(700);
        store.save(&settings).unwrap();

        let json = r#"{
            "window_width": 50,
            "window_height": 50
        }"#;
        let imported = store.import_json(json).unwrap();
        assert_eq!(imported.window_width, Some(1100));
        assert_eq!(imported.window_height, Some(700));
    }

    #[test]
    fn settings_import_keeps_language_and_culture_when_explicitly_blank() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        let mut settings = store.load().unwrap();
        settings.ui_language = "ja-JP".to_string();
        settings.culture = "ja-JP".to_string();
        store.save(&settings).unwrap();

        // Matches Grex `ImportSettingsFromJson`: an empty string in the
        // imported JSON does NOT overwrite the live value. A missing field is
        // a different case — JSON `{}` carries the default-constructed value,
        // which Grex treats as non-empty.
        let json = r#"{"ui_language": "", "culture": ""}"#;
        let imported = store.import_json(json).unwrap();
        assert_eq!(imported.ui_language, "ja-JP");
        assert_eq!(imported.culture, "ja-JP");
    }

    #[test]
    fn settings_import_trims_ai_endpoint_and_model() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        let json = r#"{
            "ai_search_endpoint": "  https://api.custom.local/v1  ",
            "ai_search_model": "  gpt-4o-mini  "
        }"#;
        let imported = store.import_json(json).unwrap();
        assert_eq!(imported.ai_search_endpoint, "https://api.custom.local/v1");
        assert_eq!(imported.ai_search_model, "gpt-4o-mini");
    }

    #[test]
    fn settings_import_clamps_context_preview() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        let json = r#"{
            "context_preview_lines_before": 99,
            "context_preview_lines_after": 0
        }"#;
        let imported = store.import_json(json).unwrap();
        assert_eq!(imported.context_preview_lines_before, 20);
        assert_eq!(imported.context_preview_lines_after, 1);
    }

    #[test]
    fn settings_import_rejects_null_document() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);
        let err = store.import_json("null").unwrap_err();
        assert!(matches!(err, ImportError::NullDocument));
    }

    #[test]
    fn settings_import_rejects_invalid_json() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);
        let err = store.import_json("{ not json }").unwrap_err();
        assert!(matches!(err, ImportError::Parse(_)));
    }

    #[test]
    fn settings_import_ignores_unknown_properties() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        let json = r#"{
            "regex_search": true,
            "UnknownProperty": "ignored"
        }"#;
        let imported = store.import_json(json).unwrap();
        assert!(imported.regex_search);
    }

    #[test]
    fn settings_export_import_round_trip_high_contrast_themes() {
        let (_dir, paths) = make_paths();
        let store = SettingsStore::new(&paths);

        for theme in [
            ThemePreference::GentleGecko,
            ThemePreference::BlackKnight,
            ThemePreference::Diamond,
            ThemePreference::Dreams,
            ThemePreference::Paranoid,
            ThemePreference::RedVelvet,
            ThemePreference::Subspace,
            ThemePreference::Tiefling,
            ThemePreference::Vibes,
        ] {
            store.delete().unwrap();
            let mut settings = store.load().unwrap();
            settings.theme_preference = theme;
            store.save(&settings).unwrap();

            let exported = store.export_json().unwrap();
            store.delete().unwrap();
            let imported = store.import_json(&exported).unwrap();
            assert_eq!(imported.theme_preference, theme);
        }
    }

    // ------------------------------------------------------------------
    // RecentPathStore
    // ------------------------------------------------------------------

    #[test]
    fn recent_paths_empty_when_file_missing() {
        let (_dir, paths) = make_paths();
        let store = RecentPathStore::new(&paths);
        assert!(store.load().unwrap().is_empty());
    }

    #[test]
    fn recent_paths_dedupe_and_remove() {
        let (_dir, paths) = make_paths();
        let store = RecentPathStore::new(&paths);

        store.add("/tmp/a").unwrap();
        store.add("/tmp/b").unwrap();
        let after_add = store.add("/tmp/a").unwrap();
        assert_eq!(after_add, vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")]);

        let after_remove = store.remove(Path::new("/tmp/a")).unwrap();
        assert_eq!(after_remove, vec![PathBuf::from("/tmp/b")]);
    }

    #[test]
    fn recent_paths_skips_blank_input() {
        let (_dir, paths) = make_paths();
        let store = RecentPathStore::new(&paths);

        store.add("").unwrap();
        store.add("   ").unwrap();
        assert!(store.load().unwrap().is_empty());

        store.remove(Path::new("")).unwrap();
        store.remove(Path::new("   ")).unwrap();
    }

    #[test]
    fn recent_paths_enforces_limit_of_20() {
        let (_dir, paths) = make_paths();
        let store = RecentPathStore::new(&paths);
        for i in 0..25 {
            store.add(format!("/tmp/p{i}")).unwrap();
        }
        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 20);
        assert_eq!(loaded[0], PathBuf::from("/tmp/p24"));
    }

    #[test]
    fn recent_paths_filter_returns_all_when_query_blank() {
        let (_dir, paths) = make_paths();
        let store = RecentPathStore::new(&paths);
        store.add("/tmp/alpha").unwrap();
        store.add("/tmp/beta").unwrap();

        assert_eq!(store.filter("").unwrap().len(), 2);
        assert_eq!(store.filter("   ").unwrap().len(), 2);
    }

    #[test]
    fn recent_paths_filter_case_insensitive_substring() {
        let (_dir, paths) = make_paths();
        let store = RecentPathStore::new(&paths);
        store.add("/home/me/Projects/code").unwrap();
        store.add("/tmp/other").unwrap();

        let hits = store.filter("PROJECTS").unwrap();
        assert_eq!(hits, vec![PathBuf::from("/home/me/Projects/code")]);
    }

    #[test]
    fn recent_paths_filter_unicode() {
        let (_dir, paths) = make_paths();
        let store = RecentPathStore::new(&paths);
        store.add("/home/me/测试").unwrap();
        store.add("/home/me/other").unwrap();

        let hits = store.filter("测试").unwrap();
        assert_eq!(hits, vec![PathBuf::from("/home/me/测试")]);
    }

    // ------------------------------------------------------------------
    // SearchHistoryStore
    // ------------------------------------------------------------------

    fn make_search(term: &str, path: &str) -> RecentSearch {
        let mut options = SearchOptions::new(path, term);
        options.match_file_names = "*.cs".to_string();
        options.exclude_dirs = "bin,obj".to_string();
        options.respect_gitignore = true;
        RecentSearch::from_options(&options, false, 42)
    }

    #[test]
    fn search_history_dedupes_by_full_grex_key() {
        let (_dir, paths) = make_paths();
        let store = SearchHistoryStore::new(&paths);

        let first = make_search("first", "/tmp/first");
        let second = make_search("second", "/tmp/second");
        let mut first_again = make_search("first", "/tmp/first");
        first_again.result_count = 100;

        store.add(first).unwrap();
        store.add(second).unwrap();
        let after = store.add(first_again).unwrap();

        assert_eq!(after.len(), 2);
        assert_eq!(after[0].search_term, "first");
        assert_eq!(after[0].result_count, 100);
        assert_eq!(after[1].search_term, "second");
    }

    #[test]
    fn search_history_key_differs_when_regex_flag_differs() {
        let plain = make_search("query", "/tmp");
        let mut regex = make_search("query", "/tmp");
        regex.regex_search = true;
        assert_ne!(plain.key(), regex.key());
    }

    #[test]
    fn search_history_key_uses_csharp_boolean_casing() {
        let mut search = make_search("term", "/tmp");
        search.match_file_names = "*.rs".to_string();
        search.exclude_dirs = "bin".to_string();
        search.regex_search = true;
        search.files_search = false;
        search.search_case_sensitive = true;
        assert_eq!(search.key(), "term|/tmp|True|False|True|*.rs|bin");
    }

    #[test]
    fn search_history_skips_blank_search_term() {
        let (_dir, paths) = make_paths();
        let store = SearchHistoryStore::new(&paths);

        let mut empty = make_search("", "/tmp");
        empty.search_term = "   ".to_string();

        store.add(empty).unwrap();
        assert!(store.load().unwrap().is_empty());
    }

    #[test]
    fn search_history_remove_and_clear() {
        let (_dir, paths) = make_paths();
        let store = SearchHistoryStore::new(&paths);

        let first = make_search("alpha", "/tmp");
        let second = make_search("beta", "/tmp");
        store.add(first.clone()).unwrap();
        store.add(second.clone()).unwrap();

        let after = store.remove(&first).unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].search_term, "beta");

        store.clear().unwrap();
        assert!(store.load().unwrap().is_empty());
        // clear on missing file is a no-op
        store.clear().unwrap();
    }

    #[test]
    fn search_history_enforces_limit_of_20() {
        let (_dir, paths) = make_paths();
        let store = SearchHistoryStore::new(&paths);

        for i in 0..25 {
            store.add(make_search(&format!("term{i}"), "/tmp")).unwrap();
        }
        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 20);
        assert_eq!(loaded[0].search_term, "term24");
    }

    #[test]
    fn search_history_filter_matches_term_or_path() {
        let (_dir, paths) = make_paths();
        let store = SearchHistoryStore::new(&paths);

        store
            .add(make_search("search for foo", "/projects/app"))
            .unwrap();
        store.add(make_search("bar query", "/other")).unwrap();
        store.add(make_search("another foo", "/other")).unwrap();
        store.add(make_search("xyz", "/projects/code")).unwrap();

        assert_eq!(store.filter("foo").unwrap().len(), 2);
        assert_eq!(store.filter("projects").unwrap().len(), 2);
        assert_eq!(store.filter("").unwrap().len(), 4);
    }

    // ------------------------------------------------------------------
    // SearchProfileStore
    // ------------------------------------------------------------------

    fn make_profile(name: &str, path: &str, term: &str) -> SearchProfile {
        SearchProfile::new(name, SearchOptions::new(path, term), false)
    }

    #[test]
    fn profile_store_returns_empty_when_missing() {
        let (_dir, paths) = make_paths();
        let store = SearchProfileStore::new(&paths);
        assert!(store.load().unwrap().is_empty());
    }

    #[test]
    fn profile_upsert_inserts_at_top() {
        let (_dir, paths) = make_paths();
        let store = SearchProfileStore::new(&paths);
        store
            .upsert(make_profile("alpha", "/tmp/a", "TODO"))
            .unwrap();
        store
            .upsert(make_profile("beta", "/tmp/b", "FIXME"))
            .unwrap();
        let profiles = store.load().unwrap();
        assert_eq!(profiles[0].name, "beta");
        assert_eq!(profiles[1].name, "alpha");
    }

    #[test]
    fn profile_upsert_updates_existing_case_insensitive_and_moves_to_top() {
        let (_dir, paths) = make_paths();
        let store = SearchProfileStore::new(&paths);
        store
            .upsert(make_profile("Profile", "/tmp/first", "alpha"))
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        store
            .upsert(make_profile("OTHER", "/tmp/other", "needle"))
            .unwrap();
        store
            .upsert(make_profile("profile", "/tmp/second", "beta"))
            .unwrap();

        let profiles = store.load().unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].name, "profile");
        assert_eq!(profiles[0].search_options.path, PathBuf::from("/tmp/second"));
        assert_eq!(profiles[1].name, "OTHER");
    }

    #[test]
    fn profile_upsert_preserves_created_unix_on_update() {
        let (_dir, paths) = make_paths();
        let store = SearchProfileStore::new(&paths);
        store.upsert(make_profile("Keep", "/tmp", "alpha")).unwrap();
        let created = store.load().unwrap()[0].created_unix;

        std::thread::sleep(std::time::Duration::from_millis(1100));
        store.upsert(make_profile("keep", "/tmp", "beta")).unwrap();
        let profiles = store.load().unwrap();
        assert_eq!(profiles[0].created_unix, created);
        assert!(profiles[0].updated_unix >= created);
    }

    #[test]
    fn profile_exists_is_case_insensitive() {
        let (_dir, paths) = make_paths();
        let store = SearchProfileStore::new(&paths);
        store.upsert(make_profile("CaseTest", "/tmp", "x")).unwrap();
        assert!(store.exists("casetest").unwrap());
        assert!(store.exists("CASETEST").unwrap());
        assert!(!store.exists("other").unwrap());
        assert!(!store.exists("").unwrap());
        assert!(!store.exists("   ").unwrap());
    }

    #[test]
    fn profile_remove_is_case_insensitive() {
        let (_dir, paths) = make_paths();
        let store = SearchProfileStore::new(&paths);
        store.upsert(make_profile("ToDelete", "/tmp", "x")).unwrap();
        store.remove("todelete").unwrap();
        assert!(store.load().unwrap().is_empty());
    }

    #[test]
    fn profile_upsert_rejects_blank_name() {
        let (_dir, paths) = make_paths();
        let store = SearchProfileStore::new(&paths);

        let blank = SearchProfile::new("", SearchOptions::new("/tmp", "x"), false);
        store.upsert(blank).unwrap();
        let whitespace = SearchProfile::new("   ", SearchOptions::new("/tmp", "x"), false);
        store.upsert(whitespace).unwrap();

        assert!(store.load().unwrap().is_empty());
    }

    #[test]
    fn profile_clear_removes_all() {
        let (_dir, paths) = make_paths();
        let store = SearchProfileStore::new(&paths);
        store.upsert(make_profile("a", "/tmp", "x")).unwrap();
        store.clear().unwrap();
        assert!(store.load().unwrap().is_empty());
        // missing file → no-op
        store.clear().unwrap();
    }
}
