//! Property-based tests for the parts of `grexa-core` that have well-defined
//! algebraic shape — globs, exclude-dir filters, size-limit math, snippet
//! boundaries.
//!
//! These are intentionally cheap (small input sizes, ~64 cases per property)
//! so they fit inside `cargo test`'s default budget. The audit doc
//! `docs/grex-gitignore-audit.md` enumerates the specific properties that
//! must hold; the cases here exist to surface regressions when those
//! properties get re-broken.

use std::fs;

use grexa_core::{SearchOptions, search};
use proptest::prelude::*;
use tempfile::tempdir;

proptest! {
    /// Globs of the form `*.<ext>` always match a file with that extension
    /// and never match a file with a different extension. The randomized
    /// portion is the filename root + extension; we deliberately keep the
    /// alphabet narrow so test paths stay predictable.
    #[test]
    fn glob_extension_match_or_miss(
        name_root in "[a-z]{1,8}",
        ext in "[a-z]{1,5}",
        sibling_ext in "[a-z]{1,5}",
    ) {
        prop_assume!(ext != sibling_ext);

        let dir = tempdir().unwrap();
        let match_file = dir.path().join(format!("{name_root}.{ext}"));
        let miss_file = dir.path().join(format!("{name_root}.{sibling_ext}"));
        fs::write(&match_file, "TODO\n").unwrap();
        fs::write(&miss_file, "TODO\n").unwrap();

        let mut options = SearchOptions::new(dir.path(), "TODO");
        options.match_file_names = format!("*.{ext}");
        let summary = search(&options).unwrap();
        let names: Vec<_> = summary
            .results
            .iter()
            .map(|r| r.file_name.clone())
            .collect();
        let match_name = format!("{name_root}.{ext}");
        let miss_name = format!("{name_root}.{sibling_ext}");
        prop_assert!(names.contains(&match_name));
        prop_assert!(!names.contains(&miss_name));
    }
}

proptest! {
    /// Exclude-dir filters with comma-separated names match any path component
    /// that equals one of the names. The filter is case-insensitive (Grex
    /// pinned this in the original test suite).
    #[test]
    fn exclude_dirs_skip_matching_directory_component(
        excluded in "[a-z]{2,12}",
        keeper in "[a-z]{2,12}",
    ) {
        prop_assume!(excluded != keeper);
        prop_assume!(!excluded.starts_with('.'));
        prop_assume!(!keeper.starts_with('.'));

        let dir = tempdir().unwrap();
        let excluded_sub = dir.path().join(&excluded);
        let keeper_sub = dir.path().join(&keeper);
        fs::create_dir_all(&excluded_sub).unwrap();
        fs::create_dir_all(&keeper_sub).unwrap();
        fs::write(excluded_sub.join("a.txt"), "TODO\n").unwrap();
        fs::write(keeper_sub.join("a.txt"), "TODO\n").unwrap();

        let mut options = SearchOptions::new(dir.path(), "TODO");
        options.exclude_dirs = excluded.clone();
        let summary = search(&options).unwrap();
        let paths: Vec<_> = summary
            .results
            .iter()
            .map(|r| r.full_path.to_string_lossy().to_string())
            .collect();
        let keeper_needle = keeper.clone();
        let excluded_needle = format!("/{excluded}/");
        prop_assert!(paths.iter().any(|p| p.contains(&keeper_needle)));
        prop_assert!(!paths.iter().any(|p| p.contains(&excluded_needle)));
    }
}

proptest! {
    /// Search results are deterministic for the same input: running the
    /// engine twice on the same tree must produce the same match count.
    #[test]
    fn search_is_deterministic(
        term in "[a-zA-Z]{3,8}",
        file_count in 1usize..6,
    ) {
        let dir = tempdir().unwrap();
        for i in 0..file_count {
            let body = format!("hello\n{term} line\nfooter\n");
            fs::write(dir.path().join(format!("f{i}.txt")), body).unwrap();
        }

        let options = SearchOptions::new(dir.path(), &term);
        let first = search(&options).unwrap();
        let second = search(&options).unwrap();
        prop_assert_eq!(first.matches, second.matches);
        prop_assert_eq!(first.files_matched, second.files_matched);
    }
}

proptest! {
    /// Snippet preview widths cap at the documented MATCH_PREVIEW_MAX_CHARS
    /// (400 chars). Even when the line is 10kb long, the line_content the
    /// engine returns must not exceed the cap (the byte count can exceed
    /// the char count for non-ASCII).
    #[test]
    fn snippet_preview_caps_line_length(
        line_byte_len in 100usize..10_000,
    ) {
        let dir = tempdir().unwrap();
        let mut body = String::new();
        body.push_str("preceding\n");
        body.push_str(&"x".repeat(line_byte_len));
        body.push_str(" TODO trailing\n");
        fs::write(dir.path().join("a.txt"), &body).unwrap();

        let options = SearchOptions::new(dir.path(), "TODO");
        let summary = search(&options).unwrap();
        prop_assert!(summary.matches >= 1);
        for r in &summary.results {
            prop_assert!(r.line_content.chars().count() <= 400);
        }
    }
}
