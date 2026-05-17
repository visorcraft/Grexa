// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Two-engine regex strategy.
//!
//! Grex was written against .NET's regex engine, which supports lookaround,
//! backreferences, conditional groups, and possessive quantifiers. The
//! pure-Rust [`regex`] crate is dramatically faster but does not implement
//! those constructs. Grexa keeps speed for the common case and falls back to
//! [`fancy_regex`] only for patterns the fast engine cannot compile.
//!
//! Compilation cascade for `build(pattern, case_insensitive)`:
//!
//! 1. Try [`regex::RegexBuilder`] with the case flag toggled. If it returns
//!    `Ok`, wrap it as [`PatternEngine::Fast`] and stop.
//! 2. Otherwise, ask [`fancy_regex`] by prepending `(?i)` for case-insensitive
//!    matching. If it returns `Ok`, wrap as [`PatternEngine::Extended`].
//! 3. If both engines reject the pattern, surface the original `regex` crate
//!    error verbatim because its diagnostics are the most actionable.
//!
//! See `docs/grex-culture-comparison-audit.md` for the cases that need the
//! extended engine.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PatternError {
    #[error("invalid regex pattern: {0}")]
    Invalid(String),
}

/// Compiled regex that hides which engine actually owns the work.
#[derive(Debug)]
pub enum PatternEngine {
    /// Fast path — `regex` crate.
    Fast(regex::Regex),
    /// Extended path — `fancy-regex` crate. Used for lookaround,
    /// backreferences, conditional groups, etc.
    Extended(fancy_regex::Regex),
}

impl PatternEngine {
    /// Build the cheapest engine that can compile `pattern`. Returns
    /// [`PatternError::Invalid`] only when *both* engines reject the input.
    pub fn build(pattern: &str, case_insensitive: bool) -> Result<Self, PatternError> {
        match regex::RegexBuilder::new(pattern)
            .case_insensitive(case_insensitive)
            .build()
        {
            Ok(re) => Ok(PatternEngine::Fast(re)),
            Err(fast_err) => {
                let amended = if case_insensitive {
                    format!("(?i){pattern}")
                } else {
                    pattern.to_string()
                };
                match fancy_regex::Regex::new(&amended) {
                    Ok(re) => Ok(PatternEngine::Extended(re)),
                    Err(_) => Err(PatternError::Invalid(fast_err.to_string())),
                }
            }
        }
    }

    /// `true` when this pattern compiled through `fancy-regex`. The CLI uses
    /// this to print a one-time stderr notice telling the user they're on the
    /// slower path.
    pub fn is_extended(&self) -> bool {
        matches!(self, PatternEngine::Extended(_))
    }

    /// Collect every match in `haystack` as `(start_byte, end_byte)` ranges.
    /// Matches that error out on the extended engine are skipped — this can
    /// only happen for pathological patterns that recurse past
    /// `fancy_regex`'s safety limits, in which case dropping the offending
    /// match is the right thing for a UI that wants to keep going.
    pub fn find_iter(&self, haystack: &str) -> Vec<(usize, usize)> {
        match self {
            PatternEngine::Fast(re) => re
                .find_iter(haystack)
                .map(|mat| (mat.start(), mat.end()))
                .collect(),
            PatternEngine::Extended(re) => re
                .find_iter(haystack)
                .filter_map(Result::ok)
                .map(|mat| (mat.start(), mat.end()))
                .collect(),
        }
    }

    /// Replace every match with `replacement`. Capture references (`$1`,
    /// `$name`) are honored by both engines.
    pub fn replace_all(&self, haystack: &str, replacement: &str) -> String {
        match self {
            PatternEngine::Fast(re) => re.replace_all(haystack, replacement).into_owned(),
            PatternEngine::Extended(re) => re.replace_all(haystack, replacement).into_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_pattern_uses_fast_engine() {
        let engine = PatternEngine::build(r"\d+", false).unwrap();
        assert!(!engine.is_extended());
        assert_eq!(engine.find_iter("abc 123 def 456"), vec![(4, 7), (12, 15)]);
    }

    #[test]
    fn case_insensitive_fast_engine() {
        let engine = PatternEngine::build(r"hello", true).unwrap();
        assert!(!engine.is_extended());
        assert_eq!(engine.find_iter("HELLO world"), vec![(0, 5)]);
    }

    #[test]
    fn lookahead_falls_through_to_extended_engine() {
        // `(?=...)` is not supported by the `regex` crate.
        let engine = PatternEngine::build(r"foo(?=bar)", false).unwrap();
        assert!(engine.is_extended(), "expected fancy-regex");
        assert_eq!(engine.find_iter("foobar foobaz"), vec![(0, 3)]);
    }

    #[test]
    fn lookbehind_falls_through_to_extended_engine() {
        let engine = PatternEngine::build(r"(?<=foo)bar", false).unwrap();
        assert!(engine.is_extended());
        assert_eq!(engine.find_iter("foobar zzbar"), vec![(3, 6)]);
    }

    #[test]
    fn backreference_falls_through_to_extended_engine() {
        let engine = PatternEngine::build(r"(\w+) \1", false).unwrap();
        assert!(engine.is_extended());
        assert_eq!(engine.find_iter("hello hello world"), vec![(0, 11)]);
    }

    #[test]
    fn case_insensitive_extended_engine() {
        // Mixed feature: lookbehind plus case insensitivity.
        let engine = PatternEngine::build(r"(?<=foo)bar", true).unwrap();
        assert!(engine.is_extended());
        assert_eq!(engine.find_iter("FOObar foobar").len(), 2);
    }

    #[test]
    fn invalid_in_both_engines_returns_error() {
        let err = PatternEngine::build(r"(?P<bad", false).unwrap_err();
        match err {
            PatternError::Invalid(msg) => assert!(!msg.is_empty()),
        }
    }

    #[test]
    fn replace_all_through_fast_engine_with_captures() {
        let engine = PatternEngine::build(r"(\w+) (\w+)", false).unwrap();
        let out = engine.replace_all("foo bar baz qux", "$2 $1");
        assert_eq!(out, "bar foo qux baz");
    }

    #[test]
    fn replace_all_through_extended_engine_with_captures() {
        let engine = PatternEngine::build(r"(?<word>\w+) \k<word>", false).unwrap();
        assert!(engine.is_extended());
        let out = engine.replace_all("hi hi there", "(dup:$word)");
        assert_eq!(out, "(dup:hi) there");
    }
}
