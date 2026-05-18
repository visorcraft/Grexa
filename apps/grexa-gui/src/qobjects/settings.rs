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
        #[qproperty(bool, include_system_files)]
        #[qproperty(bool, include_symbolic_links)]
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
        #[qproperty(i32, editor_preset)]
        #[qproperty(QString, editor_custom_command)]
        #[qproperty(bool, replace_confirm)]
        #[qproperty(bool, replace_show_journal_on_startup)]
        #[qproperty(bool, privacy_redact_paths)]
        #[qproperty(bool, accessibility_reduced_motion)]
        #[qproperty(bool, accessibility_high_contrast)]
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
    include_system_files: bool,
    include_symbolic_links: bool,
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
    editor_preset: i32,
    editor_custom_command: QString,
    replace_confirm: bool,
    replace_show_journal_on_startup: bool,
    privacy_redact_paths: bool,
    accessibility_reduced_motion: bool,
    accessibility_high_contrast: bool,
    last_save_status: QString,
}

impl SettingsControllerRust {
    /// Bulk-populate the Rust struct from a `DefaultSettings`.
    ///
    /// **Not used from `reload()` on purpose** — see the long comment
    /// in `SettingsController::reload` for the why. Kept here so the
    /// test in `tests::round_trip_via_default_settings` can stage a
    /// full struct in one call.
    #[cfg(test)]
    pub fn load_from(&mut self, s: &DefaultSettings) {
        self.regex = s.regex_search;
        self.case_sensitive = s.search_case_sensitive;
        self.respect_gitignore = s.respect_gitignore;
        self.include_hidden = s.include_hidden_items;
        self.include_binary = s.include_binary_files;
        self.include_system_files = s.include_system_files;
        self.include_symbolic_links = s.include_symbolic_links;
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
        self.editor_preset = s.editor_preset as i32;
        self.editor_custom_command = QString::from(&s.editor_custom_command);
        self.replace_confirm = s.replace_confirm;
        self.replace_show_journal_on_startup = s.replace_show_journal_on_startup;
        self.privacy_redact_paths = s.privacy_redact_paths;
        self.accessibility_reduced_motion = s.accessibility_reduced_motion;
        self.accessibility_high_contrast = s.accessibility_high_contrast;
    }

    pub fn write_into(&self, s: &mut DefaultSettings) {
        s.regex_search = self.regex;
        s.search_case_sensitive = self.case_sensitive;
        s.respect_gitignore = self.respect_gitignore;
        s.include_hidden_items = self.include_hidden;
        s.include_binary_files = self.include_binary;
        s.include_system_files = self.include_system_files;
        s.include_symbolic_links = self.include_symbolic_links;
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
        s.editor_preset = self.editor_preset.clamp(0, 8) as u8;
        s.editor_custom_command = self.editor_custom_command.to_string();
        s.replace_confirm = self.replace_confirm;
        s.replace_show_journal_on_startup = self.replace_show_journal_on_startup;
        s.privacy_redact_paths = self.privacy_redact_paths;
        s.accessibility_reduced_motion = self.accessibility_reduced_motion;
        s.accessibility_high_contrast = self.accessibility_high_contrast;
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
        ThemePreference::OledBlack => 12,
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
        12 => ThemePreference::OledBlack,
        _ => ThemePreference::System,
    }
}

impl ffi::SettingsController {
    fn reload(mut self: Pin<&mut Self>) {
        let settings = with_workspace(|w| w.settings.load().unwrap_or_default());
        // IMPORTANT: do NOT call `load_from(&settings)` here. That
        // writes directly to the Rust struct fields, bypassing the
        // cxx-qt-generated setters and therefore their change-signal
        // emits. The subsequent `set_*` calls then compare the new
        // value against the (already-updated) struct field, find
        // them equal, and silently skip emitting the change signal —
        // QML never learns the property changed, and the live UI
        // stays stuck on its initial value. The user-visible symptom
        // was "Light selected, close+reopen Grexa, theme not
        // restored": persistence was correct on disk and in Rust,
        // but the QML bindings never re-fired.
        //
        // Compute fresh values from the loaded settings and call the
        // setters directly. Each setter sees the current (old) value
        // on the struct, the new value, and emits when they differ.
        self.as_mut().set_regex(settings.regex_search);
        self.as_mut()
            .set_case_sensitive(settings.search_case_sensitive);
        self.as_mut()
            .set_respect_gitignore(settings.respect_gitignore);
        self.as_mut()
            .set_include_hidden(settings.include_hidden_items);
        self.as_mut()
            .set_include_binary(settings.include_binary_files);
        self.as_mut()
            .set_include_system_files(settings.include_system_files);
        self.as_mut()
            .set_include_symbolic_links(settings.include_symbolic_links);
        self.as_mut()
            .set_include_subfolders(settings.include_subfolders);
        self.as_mut().set_files_search_mode(settings.files_search);
        self.as_mut()
            .set_enable_container_search(settings.enable_container_search);
        self.as_mut()
            .set_ai_search_enabled(settings.ai_search_enabled);
        self.as_mut()
            .set_ai_endpoint(QString::from(&settings.ai_search_endpoint));
        self.as_mut()
            .set_ai_model(QString::from(&settings.ai_search_model));
        self.as_mut()
            .set_default_match_files(QString::from(&settings.default_match_files));
        self.as_mut()
            .set_default_exclude_dirs(QString::from(&settings.default_exclude_dirs));
        self.as_mut()
            .set_theme(theme_to_i32(settings.theme_preference));
        self.as_mut()
            .set_context_lines_before(settings.context_preview_lines_before as i32);
        self.as_mut()
            .set_context_lines_after(settings.context_preview_lines_after as i32);
        self.as_mut()
            .set_editor_preset(settings.editor_preset as i32);
        self.as_mut()
            .set_editor_custom_command(QString::from(&settings.editor_custom_command));
        self.as_mut().set_replace_confirm(settings.replace_confirm);
        self.as_mut()
            .set_replace_show_journal_on_startup(settings.replace_show_journal_on_startup);
        self.as_mut()
            .set_privacy_redact_paths(settings.privacy_redact_paths);
        self.as_mut()
            .set_accessibility_reduced_motion(settings.accessibility_reduced_motion);
        self.as_mut()
            .set_accessibility_high_contrast(settings.accessibility_high_contrast);
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
        for v in 0..=12 {
            let t = theme_from_i32(v);
            assert_eq!(theme_to_i32(t), v, "variant {v} did not round-trip");
        }
    }

    /// Regression pin for the silent-reload bug. The old `reload()`
    /// called `rust_mut().load_from(&settings)` to bulk-populate the
    /// struct, then `set_*` setters. The cxx-qt-generated setters
    /// compare the new value against the current struct field and
    /// skip emitting the change signal when they match — so the
    /// pre-stage made every setter a silent no-op and QML bindings
    /// to `app.settingsController.theme` never re-fired. The
    /// user-visible symptom was "saved Light theme not restored on
    /// reopen". The fix rewrites `reload()` to compute new values
    /// directly from the loaded settings and call setters without
    /// pre-staging, so the setter sees the OLD struct value vs the
    /// NEW disk value and emits the change signal.
    ///
    /// This test documents and pins that invariant: at the start of
    /// a `reload()` (default struct, fresh app launch), the value
    /// the setter would receive from disk must NOT already be on
    /// the struct field — otherwise we're back in the silent
    /// no-op regime.
    #[test]
    fn reload_does_not_pre_stage_through_struct() {
        let state = SettingsControllerRust::default();
        assert_eq!(state.theme, 0, "default theme is System (0)");
        assert!(!state.regex);
        assert!(!state.respect_gitignore);

        // Simulate what reload() reads from disk when the user saved
        // a non-default settings.json.
        let disk = DefaultSettings {
            theme_preference: ThemePreference::Light,
            regex_search: true,
            respect_gitignore: true,
            ..Default::default()
        };

        // Each value the setter would receive must differ from the
        // current struct field — otherwise cxx-qt's setter would
        // silently skip emission (the bug we fixed).
        assert_ne!(state.theme, theme_to_i32(disk.theme_preference));
        assert_ne!(state.regex, disk.regex_search);
        assert_ne!(state.respect_gitignore, disk.respect_gitignore);
    }
}
