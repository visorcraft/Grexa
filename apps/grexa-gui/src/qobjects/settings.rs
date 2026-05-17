// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! `SettingsController` — bridges [`grexa_core::DefaultSettings`] to QML.
//!
//! Load semantics: `reload()` reads from disk, populates qproperties.
//! Save semantics: `apply()` writes the current qproperty values back
//! to `settings.json`. Per-property auto-save is left for v0.3 when
//! the toggle UX is firm.

use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use grexa_core::{DefaultSettings, ThemePreference};

use super::workspace_handle::with_workspace;

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, regex)]
        #[qproperty(bool, case_sensitive)]
        #[qproperty(bool, respect_gitignore)]
        #[qproperty(bool, include_hidden)]
        #[qproperty(bool, include_binary)]
        #[qproperty(bool, include_subfolders)]
        #[qproperty(bool, files_search_mode)]
        #[qproperty(bool, enable_container_search)]
        #[qproperty(bool, ai_search_enabled)]
        #[qproperty(QString, ai_endpoint)]
        #[qproperty(QString, ai_model)]
        #[qproperty(QString, default_match_files)]
        #[qproperty(QString, default_exclude_dirs)]
        #[qproperty(i32, theme)]
        #[qproperty(i32, context_lines_before)]
        #[qproperty(i32, context_lines_after)]
        #[qproperty(QString, last_save_status)]
        type SettingsController = super::SettingsControllerRust;

        /// Reload settings from `settings.json`.
        #[qinvokable]
        fn reload(self: Pin<&mut SettingsController>);

        /// Persist the current property values to `settings.json`.
        #[qinvokable]
        fn apply(self: Pin<&mut SettingsController>);
    }
}

#[derive(Default)]
pub struct SettingsControllerRust {
    regex: bool,
    case_sensitive: bool,
    respect_gitignore: bool,
    include_hidden: bool,
    include_binary: bool,
    include_subfolders: bool,
    files_search_mode: bool,
    enable_container_search: bool,
    ai_search_enabled: bool,
    ai_endpoint: QString,
    ai_model: QString,
    default_match_files: QString,
    default_exclude_dirs: QString,
    theme: i32,
    context_lines_before: i32,
    context_lines_after: i32,
    last_save_status: QString,
}

impl SettingsControllerRust {
    pub fn load_from(&mut self, s: &DefaultSettings) {
        self.regex = s.regex_search;
        self.case_sensitive = s.search_case_sensitive;
        self.respect_gitignore = s.respect_gitignore;
        self.include_hidden = s.include_hidden_items;
        self.include_binary = s.include_binary_files;
        self.include_subfolders = s.include_subfolders;
        self.files_search_mode = s.files_search;
        self.enable_container_search = s.enable_container_search;
        self.ai_search_enabled = s.ai_search_enabled;
        self.ai_endpoint = QString::from(&s.ai_search_endpoint);
        self.ai_model = QString::from(&s.ai_search_model);
        self.default_match_files = QString::from(&s.default_match_files);
        self.default_exclude_dirs = QString::from(&s.default_exclude_dirs);
        self.theme = theme_to_i32(s.theme_preference);
        self.context_lines_before = s.context_preview_lines_before as i32;
        self.context_lines_after = s.context_preview_lines_after as i32;
    }

    pub fn write_into(&self, s: &mut DefaultSettings) {
        s.regex_search = self.regex;
        s.search_case_sensitive = self.case_sensitive;
        s.respect_gitignore = self.respect_gitignore;
        s.include_hidden_items = self.include_hidden;
        s.include_binary_files = self.include_binary;
        s.include_subfolders = self.include_subfolders;
        s.files_search = self.files_search_mode;
        s.enable_container_search = self.enable_container_search;
        s.ai_search_enabled = self.ai_search_enabled;
        s.ai_search_endpoint = self.ai_endpoint.to_string();
        s.ai_search_model = self.ai_model.to_string();
        s.default_match_files = self.default_match_files.to_string();
        s.default_exclude_dirs = self.default_exclude_dirs.to_string();
        s.theme_preference = theme_from_i32(self.theme);
        s.context_preview_lines_before = self.context_lines_before.clamp(0, 50) as u8;
        s.context_preview_lines_after = self.context_lines_after.clamp(0, 50) as u8;
    }
}

fn theme_to_i32(t: ThemePreference) -> i32 {
    match t {
        ThemePreference::System => 0,
        ThemePreference::Light => 1,
        ThemePreference::Dark => 2,
        ThemePreference::GentleGecko => 3,
        ThemePreference::BlackKnight => 4,
        ThemePreference::Diamond => 5,
        ThemePreference::Dreams => 6,
        ThemePreference::Paranoid => 7,
        ThemePreference::RedVelvet => 8,
        ThemePreference::Subspace => 9,
        ThemePreference::Tiefling => 10,
        ThemePreference::Vibes => 11,
    }
}

fn theme_from_i32(v: i32) -> ThemePreference {
    match v {
        1 => ThemePreference::Light,
        2 => ThemePreference::Dark,
        3 => ThemePreference::GentleGecko,
        4 => ThemePreference::BlackKnight,
        5 => ThemePreference::Diamond,
        6 => ThemePreference::Dreams,
        7 => ThemePreference::Paranoid,
        8 => ThemePreference::RedVelvet,
        9 => ThemePreference::Subspace,
        10 => ThemePreference::Tiefling,
        11 => ThemePreference::Vibes,
        _ => ThemePreference::System,
    }
}

impl ffi::SettingsController {
    fn reload(mut self: Pin<&mut Self>) {
        let settings = with_workspace(|w| w.settings.load().unwrap_or_default());
        // Stage all the new values on the Rust struct in one pass, then
        // re-emit each property via its setter so QML sees a clean batch
        // of change signals.
        self.as_mut().rust_mut().load_from(&settings);
        // Read each field via a short-lived borrow so we can issue
        // setters (which take `Pin<&mut Self>`) immediately after.
        let regex = self.as_ref().rust().regex;
        let case_sensitive = self.as_ref().rust().case_sensitive;
        let respect_gitignore = self.as_ref().rust().respect_gitignore;
        let include_hidden = self.as_ref().rust().include_hidden;
        let include_binary = self.as_ref().rust().include_binary;
        let include_subfolders = self.as_ref().rust().include_subfolders;
        let files_search_mode = self.as_ref().rust().files_search_mode;
        let enable_container_search = self.as_ref().rust().enable_container_search;
        let ai_search_enabled = self.as_ref().rust().ai_search_enabled;
        let ai_endpoint = self.as_ref().rust().ai_endpoint.clone();
        let ai_model = self.as_ref().rust().ai_model.clone();
        let default_match_files = self.as_ref().rust().default_match_files.clone();
        let default_exclude_dirs = self.as_ref().rust().default_exclude_dirs.clone();
        let theme = self.as_ref().rust().theme;
        let context_lines_before = self.as_ref().rust().context_lines_before;
        let context_lines_after = self.as_ref().rust().context_lines_after;

        self.as_mut().set_regex(regex);
        self.as_mut().set_case_sensitive(case_sensitive);
        self.as_mut().set_respect_gitignore(respect_gitignore);
        self.as_mut().set_include_hidden(include_hidden);
        self.as_mut().set_include_binary(include_binary);
        self.as_mut().set_include_subfolders(include_subfolders);
        self.as_mut().set_files_search_mode(files_search_mode);
        self.as_mut()
            .set_enable_container_search(enable_container_search);
        self.as_mut().set_ai_search_enabled(ai_search_enabled);
        self.as_mut().set_ai_endpoint(ai_endpoint);
        self.as_mut().set_ai_model(ai_model);
        self.as_mut().set_default_match_files(default_match_files);
        self.as_mut().set_default_exclude_dirs(default_exclude_dirs);
        self.as_mut().set_theme(theme);
        self.as_mut().set_context_lines_before(context_lines_before);
        self.as_mut().set_context_lines_after(context_lines_after);
        self.as_mut().set_last_save_status(QString::from("Loaded"));
    }

    fn apply(mut self: Pin<&mut Self>) {
        let mut settings = with_workspace(|w| w.settings.load().unwrap_or_default());
        self.as_ref().rust().write_into(&mut settings);
        let outcome = with_workspace(|w| w.settings.save(&settings));
        let msg = match outcome {
            Ok(()) => "Saved",
            Err(_) => "Save failed",
        };
        self.as_mut().set_last_save_status(QString::from(msg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_via_default_settings() {
        let mut state = SettingsControllerRust::default();
        let mut s = DefaultSettings {
            regex_search: true,
            search_case_sensitive: true,
            respect_gitignore: false,
            theme_preference: ThemePreference::Dark,
            ..Default::default()
        };
        state.load_from(&s);
        assert!(state.regex);
        assert!(state.case_sensitive);
        assert!(!state.respect_gitignore);
        assert_eq!(state.theme, 2);

        state.theme = 1;
        state.respect_gitignore = true;
        state.write_into(&mut s);
        assert!(matches!(s.theme_preference, ThemePreference::Light));
        assert!(s.respect_gitignore);
    }

    #[test]
    fn theme_round_trip_covers_all_variants() {
        for v in 0..=11 {
            let t = theme_from_i32(v);
            assert_eq!(theme_to_i32(t), v, "variant {v} did not round-trip");
        }
    }
}
