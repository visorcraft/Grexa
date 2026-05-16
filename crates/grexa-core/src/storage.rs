use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::{SearchOptions, SizeUnit, StringComparisonMode, UnicodeNormalizationMode};

const APP_DIR: &str = "grexa";
const RECENT_PATH_LIMIT: usize = 20;
const RECENT_SEARCH_LIMIT: usize = 20;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        }
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

    pub fn load(&self) -> Result<DefaultSettings, JsonStoreError> {
        load_json_or_default(&self.path)
    }

    pub fn save(&self, settings: &DefaultSettings) -> Result<(), JsonStoreError> {
        save_json(&self.path, settings)
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
        let mut paths = self.load()?;
        paths.retain(|path| path != &recent_path);
        paths.insert(0, recent_path);
        paths.truncate(self.limit);
        save_json(&self.path, &paths)?;
        Ok(paths)
    }

    pub fn remove(&self, recent_path: &Path) -> Result<Vec<PathBuf>, JsonStoreError> {
        let mut paths = self.load()?;
        paths.retain(|path| path != recent_path);
        save_json(&self.path, &paths)?;
        Ok(paths)
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
        let mut searches = self.load()?;
        searches.retain(|existing| {
            existing.search_path != search.search_path || existing.search_term != search.search_term
        });
        searches.insert(0, search);
        searches.truncate(self.limit);
        save_json(&self.path, &searches)?;
        Ok(searches)
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

    pub fn upsert(&self, mut profile: SearchProfile) -> Result<Vec<SearchProfile>, JsonStoreError> {
        let mut profiles = self.load()?;
        if let Some(existing) = profiles
            .iter_mut()
            .find(|existing| existing.name == profile.name)
        {
            profile.created_unix = existing.created_unix;
            profile.updated_unix = unix_now();
            *existing = profile;
        } else {
            profiles.push(profile);
        }

        profiles.sort_by_key(|profile| profile.name.to_lowercase());
        save_json(&self.path, &profiles)?;
        Ok(profiles)
    }

    pub fn remove(&self, name: &str) -> Result<Vec<SearchProfile>, JsonStoreError> {
        let mut profiles = self.load()?;
        profiles.retain(|profile| profile.name != name);
        save_json(&self.path, &profiles)?;
        Ok(profiles)
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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn settings_round_trip() {
        let dir = tempdir().unwrap();
        let paths = AppPaths::under(dir.path());
        let store = SettingsStore::new(&paths);

        let mut settings = store.load().unwrap();
        settings.default_match_files = "*.rs".to_string();
        store.save(&settings).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.default_match_files, "*.rs");
    }

    #[test]
    fn recent_paths_dedupe_and_remove() {
        let dir = tempdir().unwrap();
        let paths = AppPaths::under(dir.path());
        let store = RecentPathStore::new(&paths);

        store.add("/tmp/a").unwrap();
        store.add("/tmp/b").unwrap();
        let paths = store.add("/tmp/a").unwrap();

        assert_eq!(
            paths,
            vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")]
        );

        let paths = store.remove(Path::new("/tmp/a")).unwrap();
        assert_eq!(paths, vec![PathBuf::from("/tmp/b")]);
    }

    #[test]
    fn search_history_dedupes_by_path_and_term() {
        let dir = tempdir().unwrap();
        let paths = AppPaths::under(dir.path());
        let store = SearchHistoryStore::new(&paths);

        let options = SearchOptions::new("/tmp/project", "TODO");
        store
            .add(RecentSearch::from_options(&options, false, 1))
            .unwrap();
        let searches = store
            .add(RecentSearch::from_options(&options, false, 2))
            .unwrap();

        assert_eq!(searches.len(), 1);
        assert_eq!(searches[0].result_count, 2);
    }

    #[test]
    fn profiles_upsert_and_remove() {
        let dir = tempdir().unwrap();
        let paths = AppPaths::under(dir.path());
        let store = SearchProfileStore::new(&paths);

        let profile = SearchProfile::new("main", SearchOptions::new("/tmp/project", "TODO"), false);
        store.upsert(profile).unwrap();
        let updated =
            SearchProfile::new("main", SearchOptions::new("/tmp/project", "FIXME"), false);
        let profiles = store.upsert(updated).unwrap();

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].search_options.search_term, "FIXME");

        let profiles = store.remove("main").unwrap();
        assert!(profiles.is_empty());
    }
}
