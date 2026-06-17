// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! `RegexBuilderController` — drives the Regex Builder page.
//!
//! Wraps [`grexa_core::PatternEngine`]. Recomputes match-count + error
//! state whenever the pattern or sample text changes. Compilation and
//! scanning run on a worker thread so the UI stays responsive; results
//! are queued back to the Qt thread via [`cxx_qt::Threading`].

use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use grexa_core::PatternEngine;
use serde_json;

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
        #[qproperty(QString, pattern)]
        #[qproperty(QString, sample)]
        #[qproperty(bool, case_insensitive)]
        #[qproperty(i32, match_count)]
        #[qproperty(QString, error)]
        type RegexBuilderController = super::RegexBuilderControllerRust;

        /// Recompile the pattern and re-scan the sample text. Updates
        /// `match_count` and `error`.
        #[qinvokable]
        fn evaluate(self: Pin<&mut RegexBuilderController>);

        /// Return the byte offsets of every match in `sample` against
        /// the current `pattern` / `case_insensitive` as a JSON array
        /// of `[start, end]` pairs. QML uses these to draw a highlight
        /// overlay that is guaranteed to agree with `match_count` —
        /// no JS regex engine is involved.
        #[qinvokable]
        fn match_ranges_json(self: &RegexBuilderController) -> QString;
    }

    impl cxx_qt::Threading for RegexBuilderController {}
}

#[derive(Default)]
pub struct RegexBuilderControllerRust {
    pattern: QString,
    sample: QString,
    case_insensitive: bool,
    match_count: i32,
    error: QString,
    /// Monotonic generation counter so late worker results from a previous
    /// keystroke don't overwrite the current state.
    evaluate_generation: u64,
}

/// Maximum number of match ranges the regex builder will serialize for the
/// highlight overlay. Past this cap the badge still shows the true count, but
/// the side-panel list and highlight are truncated to keep the GUI responsive.
const MAX_REGEX_RANGES: usize = 10_000;

impl RegexBuilderControllerRust {
    /// Bump the evaluate generation and return the new value. Each keystroke
    /// starts a fresh generation so a slow worker's result can be matched
    /// against the latest request and discarded if it has been superseded.
    fn bump_generation(&mut self) -> u64 {
        self.evaluate_generation = self.evaluate_generation.wrapping_add(1);
        self.evaluate_generation
    }

    /// True when `generation` is still the latest one issued — i.e. a worker
    /// result tagged with it has not been superseded and may be applied.
    fn is_current_generation(&self, generation: u64) -> bool {
        self.evaluate_generation == generation
    }

    /// Pure Rust evaluation. Returns `(match_count, error_text)`.
    pub fn evaluate_strings(pattern: &str, sample: &str, case_insensitive: bool) -> (i32, String) {
        if pattern.is_empty() {
            return (0, String::new());
        }
        match PatternEngine::build(pattern, case_insensitive) {
            Ok(engine) => {
                let count = engine.find_iter(sample).len() as i32;
                (count, String::new())
            }
            Err(err) => (0, err.to_string()),
        }
    }

    /// JSON-encoded `[[start, end], ...]` pairs of byte offsets in
    /// `sample`. Returns `"[]"` when the pattern is empty or invalid.
    /// Truncates to [`MAX_REGEX_RANGES`] so a pathological regex against a
    /// huge sample doesn't serialize megabytes of JSON.
    pub fn match_ranges_json_str(pattern: &str, sample: &str, case_insensitive: bool) -> String {
        if pattern.is_empty() {
            return "[]".into();
        }
        match PatternEngine::build(pattern, case_insensitive) {
            Ok(engine) => {
                let ranges = engine.find_iter(sample);
                let ranges: Vec<_> = ranges.into_iter().take(MAX_REGEX_RANGES).collect();
                serde_json::to_string(&ranges).unwrap_or_else(|_| "[]".into())
            }
            Err(_) => "[]".into(),
        }
    }
}

impl ffi::RegexBuilderController {
    fn evaluate(mut self: Pin<&mut Self>) {
        let pattern = self.as_ref().rust().pattern.to_string();
        let sample = self.as_ref().rust().sample.to_string();
        let ci = self.as_ref().rust().case_insensitive;
        let generation = self.as_mut().rust_mut().bump_generation();

        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let (count, err) = RegexBuilderControllerRust::evaluate_strings(&pattern, &sample, ci);
            let _ = thread.queue(move |pin| {
                finish_evaluate(pin, generation, count, err);
            });
        });
    }

    fn match_ranges_json(&self) -> QString {
        let r = self.rust();
        QString::from(&RegexBuilderControllerRust::match_ranges_json_str(
            &r.pattern.to_string(),
            &r.sample.to_string(),
            r.case_insensitive,
        ))
    }
}

fn finish_evaluate(
    mut pin: Pin<&mut ffi::RegexBuilderController>,
    generation: u64,
    count: i32,
    err: String,
) {
    if !pin.as_ref().rust().is_current_generation(generation) {
        return;
    }
    pin.as_mut().set_match_count(count);
    pin.as_mut().set_error(QString::from(&err));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_pattern_counts_occurrences() {
        let (count, err) =
            RegexBuilderControllerRust::evaluate_strings("TODO", "TODO 1\nTODO 2\nplain", false);
        assert_eq!(count, 2);
        assert_eq!(err, "");
    }

    #[test]
    fn invalid_regex_yields_error() {
        let (count, err) = RegexBuilderControllerRust::evaluate_strings("(", "irrelevant", false);
        assert_eq!(count, 0);
        assert!(!err.is_empty(), "expected error text, got empty");
    }

    #[test]
    fn empty_pattern_returns_zero_no_error() {
        let (count, err) = RegexBuilderControllerRust::evaluate_strings("", "any", false);
        assert_eq!(count, 0);
        assert_eq!(err, "");
    }

    #[test]
    fn match_ranges_json_emits_pairs() {
        let json = RegexBuilderControllerRust::match_ranges_json_str(
            "TODO",
            "TODO 1\nTODO 2\nplain",
            false,
        );
        assert_eq!(json, "[[0,4],[7,11]]");
    }

    #[test]
    fn match_ranges_json_returns_empty_array_on_error() {
        let json = RegexBuilderControllerRust::match_ranges_json_str("(", "irrelevant", false);
        assert_eq!(json, "[]");
    }

    #[test]
    fn match_ranges_json_caps_ranges() {
        let sample = "a ".repeat(MAX_REGEX_RANGES + 100);
        let json = RegexBuilderControllerRust::match_ranges_json_str("a", &sample, false);
        let ranges: Vec<(usize, usize)> = serde_json::from_str(&json).unwrap();
        assert_eq!(ranges.len(), MAX_REGEX_RANGES);
    }

    #[test]
    fn generation_gating_drops_stale_worker_results() {
        // Each keystroke bumps the generation; a worker result is applied only
        // if its generation is still current. This is what stops a slow
        // evaluation from a previous keystroke from clobbering newer state.
        let mut state = RegexBuilderControllerRust::default();
        let first = state.bump_generation();
        assert!(state.is_current_generation(first));

        // A newer keystroke supersedes the first request.
        let second = state.bump_generation();
        assert_ne!(first, second);
        assert!(
            !state.is_current_generation(first),
            "a result from the superseded generation must be dropped"
        );
        assert!(
            state.is_current_generation(second),
            "the latest generation's result must still apply"
        );
    }

    #[test]
    fn generation_counter_wraps_without_panicking() {
        let mut state = RegexBuilderControllerRust {
            evaluate_generation: u64::MAX,
            ..Default::default()
        };
        assert_eq!(state.bump_generation(), 0, "generation must wrap, not panic");
        assert!(state.is_current_generation(0));
    }
}
