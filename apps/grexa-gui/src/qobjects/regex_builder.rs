// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! `RegexBuilderController` — drives the Regex Builder page.
//!
//! Wraps [`grexa_core::PatternEngine`]. Recomputes match-count + error
//! state whenever the pattern or sample text changes. All work is
//! synchronous and cheap (single regex compile + scan over the sample
//! string), so no threading.

use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use grexa_core::PatternEngine;

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
    }
}

#[derive(Default)]
pub struct RegexBuilderControllerRust {
    pattern: QString,
    sample: QString,
    case_insensitive: bool,
    match_count: i32,
    error: QString,
}

impl RegexBuilderControllerRust {
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
}

impl ffi::RegexBuilderController {
    fn evaluate(mut self: Pin<&mut Self>) {
        let pattern = self.as_ref().rust().pattern.to_string();
        let sample = self.as_ref().rust().sample.to_string();
        let ci = self.as_ref().rust().case_insensitive;
        let (count, err) = RegexBuilderControllerRust::evaluate_strings(&pattern, &sample, ci);
        self.as_mut().set_match_count(count);
        self.as_mut().set_error(QString::from(&err));
    }
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
}
