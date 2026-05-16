//! Multi-tab workspace controller.
//!
//! Owns the `Vec<TabState>`, the active tab, the recent-paths /
//! history / profiles stores, and the cross-tab "Open in editor" /
//! "Reveal in file manager" plumbing. This is the type a future cxx-qt
//! `QObject` wraps; the in-memory contract is identical regardless of
//! whether the GUI is a real cxx-qt binding or the `qml6`-spawn
//! fallback.

use std::path::Path;

use anyhow::Result;
use grexa_core::{
    AppPaths, CancelToken, EditorPreset, RecentPathStore, RecentSearch, ReplaceOptions,
    ReplaceSummary, SearchHistoryStore, SearchOptions, SearchProfile, SearchProfileStore,
    SettingsStore, open_in_editor_command, replace_with, reveal_with_xdg_open, search_with,
};

use crate::tab::{ResultMode, TabId, TabState, TabStatus};

/// Workspace state — one per running Grexa process. Methods are
/// exposed for the future QML bindings; many are unused on the bare
/// host today, hence the `dead_code` allow on the type.
#[allow(dead_code)]
pub struct Workspace {
    pub paths: AppPaths,
    pub tabs: Vec<TabState>,
    pub active: Option<usize>,
    next_tab: u64,
    pub recent_paths: RecentPathStore,
    pub history: SearchHistoryStore,
    pub profiles: SearchProfileStore,
    pub settings: SettingsStore,
}

#[allow(dead_code)]
impl Workspace {
    pub fn new() -> Self {
        let paths = AppPaths::from_env();
        Self {
            recent_paths: RecentPathStore::new(&paths),
            history: SearchHistoryStore::new(&paths),
            profiles: SearchProfileStore::new(&paths),
            settings: SettingsStore::new(&paths),
            paths,
            tabs: Vec::new(),
            active: None,
            next_tab: 1,
        }
    }

    /// Use a custom XDG root — required by tests so they never write to
    /// the user's real settings.
    pub fn under(base: &Path) -> Self {
        let paths = AppPaths::under(base);
        Self {
            recent_paths: RecentPathStore::new(&paths),
            history: SearchHistoryStore::new(&paths),
            profiles: SearchProfileStore::new(&paths),
            settings: SettingsStore::new(&paths),
            paths,
            tabs: Vec::new(),
            active: None,
            next_tab: 1,
        }
    }

    /// Create a fresh tab and activate it.
    pub fn open_tab(&mut self, options: SearchOptions) -> TabId {
        let id = TabId(self.next_tab);
        self.next_tab += 1;
        self.tabs.push(TabState::new(id, options));
        self.active = Some(self.tabs.len() - 1);
        id
    }

    /// Close a tab; activates the next/previous tab.
    pub fn close_tab(&mut self, id: TabId) {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(idx);
            self.active = if self.tabs.is_empty() {
                None
            } else {
                Some(idx.min(self.tabs.len() - 1))
            };
        }
    }

    pub fn active_tab(&self) -> Option<&TabState> {
        self.active.and_then(|idx| self.tabs.get(idx))
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut TabState> {
        self.active.and_then(|idx| self.tabs.get_mut(idx))
    }

    pub fn tab(&self, id: TabId) -> Option<&TabState> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut TabState> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn set_active(&mut self, id: TabId) {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            self.active = Some(idx);
        }
    }

    /// Synchronous search drive for tests. Production routes searches
    /// through a worker thread + `ProgressEvent` stream; this helper is
    /// the minimum to verify the tab transitions end-to-end without
    /// touching threads.
    pub fn run_search_blocking(&mut self, id: TabId) -> Result<()> {
        let options = {
            let tab = self
                .tab(id)
                .ok_or_else(|| anyhow::anyhow!("tab {id:?} not found"))?;
            tab.options.clone()
        };
        let cancel = CancelToken::new();
        if let Some(tab) = self.tab_mut(id) {
            tab.status = TabStatus::Searching;
            tab.cancel = cancel.clone();
        }
        let summary = search_with(&options, &cancel, None)?;
        // Record the path and dedupe by Grex's seven-field key.
        let _ = self.recent_paths.add(options.path.clone());
        let _ = self.history.add(RecentSearch::from_options(
            &options,
            matches!(
                self.tab(id).map(|t| t.result_mode),
                Some(ResultMode::Files)
            ),
            summary.matches,
        ));
        if let Some(tab) = self.tab_mut(id) {
            tab.install_summary(summary);
        }
        Ok(())
    }

    /// Run the replace flow on `id`. The Grex audit calls for the result
    /// mode to flip to Files when the replace completes (the user is
    /// expected to scan files-changed counts, not individual lines).
    /// The journal in `$XDG_STATE_HOME/grexa/replace-journal.json` is
    /// owned by `replace_with`; the GUI doesn't need to touch it.
    pub fn run_replace_blocking(
        &mut self,
        id: TabId,
        replacement: impl Into<String>,
    ) -> Result<ReplaceSummary> {
        let (options, cancel) = {
            let tab = self
                .tab(id)
                .ok_or_else(|| anyhow::anyhow!("tab {id:?} not found"))?;
            let cancel = tab.cancel.clone();
            (tab.options.clone(), cancel)
        };
        if let Some(tab) = self.tab_mut(id) {
            tab.status = crate::tab::TabStatus::Replacing;
            tab.replacement = replacement.into();
        }
        let replacement_string = self
            .tab(id)
            .map(|t| t.replacement.clone())
            .unwrap_or_default();
        let summary = replace_with(
            &ReplaceOptions {
                search: options,
                replacement: replacement_string,
            },
            &cancel,
            None,
        )?;
        if let Some(tab) = self.tab_mut(id) {
            // Flip to Files mode so the user sees the per-file counts.
            tab.result_mode = ResultMode::Files;
            tab.status = if summary.cancelled {
                crate::tab::TabStatus::Cancelled
            } else {
                crate::tab::TabStatus::Completed
            };
        }
        Ok(summary)
    }

    /// Stop the search on `id`. Cancellation is cooperative; the next
    /// poll in `search_with` returns.
    pub fn cancel_search(&mut self, id: TabId) {
        if let Some(tab) = self.tab(id) {
            tab.cancel.cancel();
        }
    }

    /// Build the editor argv that opens a search result at its match
    /// line. Editor preset is read from settings (eventually); for now
    /// the helper takes the preset explicitly.
    pub fn open_result_command(
        &self,
        preset: EditorPreset,
        path: &Path,
        line: Option<usize>,
    ) -> Vec<std::ffi::OsString> {
        open_in_editor_command(preset, path, line)
    }

    /// Build the argv to reveal the result in the file manager.
    pub fn reveal_command(&self, path: &Path) -> Vec<std::ffi::OsString> {
        reveal_with_xdg_open(path)
    }

    /// Save the active tab's current options as a named profile.
    pub fn save_profile(&self, name: &str) -> Result<Vec<SearchProfile>> {
        let tab = self
            .active_tab()
            .ok_or_else(|| anyhow::anyhow!("no active tab"))?;
        let files_mode = matches!(tab.result_mode, ResultMode::Files);
        let profile = SearchProfile::new(name, tab.options.clone(), files_mode);
        Ok(self.profiles.upsert(profile)?)
    }

    /// Replay a saved profile into a new tab.
    pub fn open_profile(&mut self, profile: &SearchProfile) -> TabId {
        let id = self.open_tab(profile.search_options.clone());
        if let Some(tab) = self.tab_mut(id) {
            tab.result_mode = if profile.files_search {
                ResultMode::Files
            } else {
                ResultMode::Content
            };
        }
        id
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    fn workspace_under(base: &Path) -> Workspace {
        Workspace::under(base)
    }

    #[test]
    fn opens_and_closes_tabs() {
        let dir = tempdir().unwrap();
        let mut ws = workspace_under(dir.path());
        let a = ws.open_tab(SearchOptions::new("/tmp/a", "TODO"));
        let b = ws.open_tab(SearchOptions::new("/tmp/b", "FIXME"));
        assert_eq!(ws.tabs.len(), 2);
        assert_eq!(ws.active, Some(1));

        ws.close_tab(a);
        assert_eq!(ws.tabs.len(), 1);
        assert_eq!(ws.tab(b).unwrap().options.search_term, "FIXME");
    }

    #[test]
    fn run_search_blocking_populates_summary_and_history() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("proj");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("alpha.txt"), "TODO write\n").unwrap();

        let xdg = dir.path().join("xdg");
        let mut ws = workspace_under(&xdg);
        let id = ws.open_tab(SearchOptions::new(&project, "TODO"));
        ws.run_search_blocking(id).unwrap();

        let tab = ws.tab(id).unwrap();
        assert_eq!(tab.status, TabStatus::Completed);
        assert_eq!(tab.summary.as_ref().unwrap().matches, 1);

        // Recent paths + history both have one entry.
        assert_eq!(ws.recent_paths.load().unwrap().len(), 1);
        assert_eq!(ws.history.load().unwrap().len(), 1);
    }

    #[test]
    fn save_and_open_profile_round_trips() {
        let dir = tempdir().unwrap();
        let xdg = dir.path().join("xdg");
        let mut ws = workspace_under(&xdg);
        let mut options = SearchOptions::new("/tmp/proj", "TODO");
        options.regex = true;
        ws.open_tab(options);
        ws.save_profile("my-profile").unwrap();

        let profiles = ws.profiles.load().unwrap();
        assert_eq!(profiles.len(), 1);
        let profile = &profiles[0];
        assert_eq!(profile.name, "my-profile");
        assert!(profile.search_options.regex);
    }

    #[test]
    fn run_replace_blocking_flips_to_files_mode() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("proj");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("a.txt"), "TODO ship it\n").unwrap();

        let xdg = dir.path().join("xdg");
        // Redirect the journal so we don't pollute the user's state dir.
        let journal = dir.path().join("journal.json");
        grexa_core::set_journal_path_override(Some(journal));

        let mut ws = workspace_under(&xdg);
        let id = ws.open_tab(SearchOptions::new(&project, "TODO"));
        let summary = ws.run_replace_blocking(id, "DONE").unwrap();
        assert_eq!(summary.files_modified, 1);

        let tab = ws.tab(id).unwrap();
        assert_eq!(tab.result_mode, ResultMode::Files);
        assert_eq!(tab.status, crate::tab::TabStatus::Completed);

        // File was actually rewritten.
        let body = fs::read_to_string(project.join("a.txt")).unwrap();
        assert_eq!(body, "DONE ship it\n");

        grexa_core::set_journal_path_override(None);
    }

    #[test]
    fn cancel_token_propagates() {
        let dir = tempdir().unwrap();
        let xdg = dir.path().join("xdg");
        let mut ws = workspace_under(&xdg);
        let id = ws.open_tab(SearchOptions::new("/tmp", "TODO"));
        ws.cancel_search(id);
        let tab = ws.tab(id).unwrap();
        assert!(tab.cancel.is_cancelled());
    }
}
