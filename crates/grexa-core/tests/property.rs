// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Property-based tests for the parts of `grexa-core` that have well-defined
//! algebraic shape — globs, exclude-dir filters, size-limit math, snippet
//! boundaries, replace semantics, diacritic stripping, and max-results caps.
//!
//! These are intentionally cheap (small input sizes, ~64 cases per property)
//! so they fit inside `cargo test`'s default budget.

use std::fs;

use grexa_core::{CancelToken, ReplaceOptions, SearchOptions, replace_with, search};
use proptest::prelude::*;
use tempfile::tempdir;

const BINARY_EXTS: &[&str] = &[
    "exe", "dll", "obj", "bin", "zip", "tar", "gz", "7z", "rar", "png", "jpg", "jpeg", "gif",
    "bmp", "ico", "svg", "webp", "mp3", "mp4", "avi", "mkv", "wav", "flac", "ogg", "pdf", "doc",
    "docx", "xls", "xlsx", "ppt", "pptx", "pdb", "cache", "lock", "pack", "idx", "rtf", "odt",
    "ods", "odp",
];

proptest! {
    #[test]
    fn glob_extension_match_or_miss(
        name_root in "[a-z]{1,8}",
        ext in "[a-z]{1,5}",
        sibling_ext in "[a-z]{1,5}",
    ) {
        prop_assume!(ext != sibling_ext);
        prop_assume!(!BINARY_EXTS.contains(&ext.as_str()));
        prop_assume!(!BINARY_EXTS.contains(&sibling_ext.as_str()));

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

proptest! {
    #[test]
    fn diacritic_insensitive_matches_stripped_form(
        base in "[a-z]{2,6}",
        accent_variant in 0usize..8,
    ) {
        let accented = match accent_variant {
            0 => format!("{base}\u{00E9}xyz"),
            1 => format!("{base}\u{00F1}xyz"),
            2 => format!("{base}\u{00FC}xyz"),
            3 => format!("{base}\u{00E4}xyz"),
            4 => format!("{base}\u{00F6}xyz"),
            5 => format!("{base}\u{00E0}xyz"),
            6 => format!("{base}\u{00E8}xyz"),
            _ => format!("{base}\u{00EB}xyz"),
        };
        let bare = match accent_variant {
            0 => format!("{base}exyz"),
            1 => format!("{base}nxyz"),
            2 => format!("{base}uxyz"),
            3 => format!("{base}axyz"),
            4 => format!("{base}oxyz"),
            5 => format!("{base}axyz"),
            6 => format!("{base}exyz"),
            _ => format!("{base}exyz"),
        };

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), format!("{accented}\n")).unwrap();

        let mut options = SearchOptions::new(dir.path(), &bare);
        options.diacritic_sensitive = false;
        let summary = search(&options).unwrap();
        prop_assert!(
            summary.matches >= 1,
            "bare '{bare}' should match accented '{accented}'"
        );
    }
}

proptest! {
    #[test]
    fn replace_preserves_non_matching_lines(
        needle in "[a-z]{3,6}",
        filler in "[a-z]{3,6}",
        replacement in "[a-z]{2,4}",
    ) {
        prop_assume!(needle != filler);

        let dir = tempdir().unwrap();
        let filler_clone = filler.clone();
        let body = format!("{filler}\n{needle}\n{filler_clone}\n");
        fs::write(dir.path().join("a.txt"), &body).unwrap();

        let options = ReplaceOptions {
            search: SearchOptions::new(dir.path(), &needle),
            replacement: replacement.clone(),
        };
        let cancel = CancelToken::new();
        let summary = replace_with(&options, &cancel, None).unwrap();
        prop_assert_eq!(summary.files_modified, 1);
        prop_assert_eq!(summary.matches_replaced, 1);

        let new_content = fs::read_to_string(dir.path().join("a.txt")).unwrap();
        let lines: Vec<&str> = new_content.lines().collect();
        prop_assert_eq!(lines.len(), 3);
        prop_assert_eq!(lines[0], filler);
        prop_assert_eq!(lines[1], replacement);
        prop_assert_eq!(lines[2], &filler_clone);
    }
}

proptest! {
    #[test]
    fn replace_all_occurrences_in_file(
        needle in "[a-z]{3,5}",
        count in 1usize..6,
        replacement in "[a-z]{2,4}",
    ) {
        let dir = tempdir().unwrap();
        let body = (0..count)
            .map(|i| format!("{needle}_{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(dir.path().join("a.txt"), format!("{body}\n")).unwrap();

        let options = ReplaceOptions {
            search: SearchOptions::new(dir.path(), &needle),
            replacement: replacement.clone(),
        };
        let cancel = CancelToken::new();
        let summary = replace_with(&options, &cancel, None).unwrap();
        prop_assert_eq!(summary.matches_replaced, count);

        let new_content = fs::read_to_string(dir.path().join("a.txt")).unwrap();
        for line in new_content.lines() {
            prop_assert!(
                line.contains(&replacement),
                "replaced line should contain '{replacement}', got '{line}'"
            );
        }
    }
}

proptest! {
    #[test]
    fn max_results_caps_output(
        file_count in 3usize..8,
        max in 1usize..3,
    ) {
        let dir = tempdir().unwrap();
        for i in 0..file_count {
            fs::write(dir.path().join(format!("f{i}.txt")), "TODO\n").unwrap();
        }

        let mut options = SearchOptions::new(dir.path(), "TODO");
        options.max_results = Some(max);
        let summary = search(&options).unwrap();
        prop_assert!(
            summary.results.len() <= max,
            "results {} should be <= max {}",
            summary.results.len(),
            max
        );
    }
}

proptest! {
    #[test]
    fn case_insensitive_match_works_for_ascii(
        term in "[a-z]{3,6}",
        upper_mode in 0usize..3,
    ) {
        let cased = match upper_mode {
            0 => term.to_uppercase(),
            1 => {
                let mut chars: Vec<char> = term.chars().collect();
                if !chars.is_empty() { chars[0] = chars[0].to_ascii_uppercase(); }
                chars.into_iter().collect()
            }
            _ => term.clone(),
        };

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), format!("{cased}\n")).unwrap();

        let options = SearchOptions::new(dir.path(), &term);
        let summary = search(&options).unwrap();
        prop_assert!(
            summary.matches >= 1,
            "case-insensitive: '{term}' should match '{cased}'"
        );
    }
}

proptest! {
    #[test]
    fn search_finds_multiple_occurrences_per_line(
        term in "[a-z]{2,4}",
        count in 2usize..5,
    ) {
        let dir = tempdir().unwrap();
        let line = format!("{} ", term).repeat(count);
        fs::write(dir.path().join("a.txt"), format!("{line}\n")).unwrap();

        let options = SearchOptions::new(dir.path(), &term);
        let summary = search(&options).unwrap();
        prop_assert_eq!(summary.results.len(), 1);
        prop_assert!(
            summary.results[0].match_count >= count,
            "expected >= {count} matches, got {}",
            summary.results[0].match_count
        );
    }
}

proptest! {
    #[test]
    fn unicode_multiline_file_searches_correctly(
        prefix in "[a-z]{1,3}",
        suffix in "[a-z]{1,3}",
    ) {
        let dir = tempdir().unwrap();
        let body = format!(
            "{prefix}\ncaf\u{00E9}\n{suffix}\nna\u{00EF}ve\n"
        );
        fs::write(dir.path().join("a.txt"), &body).unwrap();

        let mut options = SearchOptions::new(dir.path(), "caf");
        options.diacritic_sensitive = true;
        let summary = search(&options).unwrap();
        prop_assert!(summary.matches >= 1);

        let mut options2 = SearchOptions::new(dir.path(), "na");
        options2.diacritic_sensitive = true;
        let summary2 = search(&options2).unwrap();
        prop_assert!(summary2.matches >= 1);
    }
}
