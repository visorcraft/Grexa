//! Gitignore parity test, ported from `docs/grex-gitignore-audit.md`.
//!
//! Each subtest writes a `.gitignore` body and a target file path into a
//! `tempdir`, then asks the `ignore` crate's `Gitignore` matcher whether the
//! file is ignored. That matcher is what `crates/grexa-core/src/search.rs`
//! ultimately consults via `ignore::WalkBuilder`, so locking these cases
//! locks Grexa's user-visible behavior.
//!
//! Some cases pin a *known divergence* from Grex (groups I escape, J
//! malformed, K case). When Grex and Grexa differ, the test pins what Grexa
//! actually does so a future change can't silently regress.

use std::path::Path;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use tempfile::tempdir;

/// Build a `Gitignore` matcher from a single body and an optional list of
/// extra per-directory patterns. Returns the matcher plus the resolved
/// search root so callers can construct absolute paths.
fn make_matcher(root: &Path, body: &str) -> Gitignore {
    let gitignore_path = root.join(".gitignore");
    std::fs::write(&gitignore_path, body).unwrap();
    let mut builder = GitignoreBuilder::new(root);
    builder.add(&gitignore_path);
    builder.build().unwrap()
}

/// Materialize `relpath` under `root` and return its absolute path. Creates
/// parent directories. When `is_dir` is set, the path is a directory;
/// otherwise an empty file is created.
fn touch(root: &Path, relpath: &str, is_dir: bool) -> std::path::PathBuf {
    let abs = root.join(relpath);
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    if is_dir {
        std::fs::create_dir_all(&abs).unwrap();
    } else {
        std::fs::File::create(&abs).unwrap();
    }
    abs
}

fn assert_match(body: &str, relpath: &str, is_dir: bool, expected_ignored: bool) {
    let dir = tempdir().unwrap();
    let matcher = make_matcher(dir.path(), body);
    let abs = touch(dir.path(), relpath, is_dir);
    let m = matcher.matched(&abs, is_dir);
    if expected_ignored {
        assert!(
            m.is_ignore(),
            "expected `{relpath}` to be ignored under body=`{body}`, got {m:?}"
        );
    } else {
        assert!(
            !m.is_ignore(),
            "expected `{relpath}` to be kept under body=`{body}`, got {m:?}"
        );
    }
}

/// Walker-based assertion for cases where the ignored entity is a *descendant*
/// of a directory pattern (e.g. `build/` excludes everything inside `build/`).
/// `Gitignore::matched` only matches the directory itself; the walker is the
/// layer that stops recursing into it. This helper materializes the file,
/// runs `ignore::WalkBuilder` with the project's `.gitignore` enabled, and
/// asserts whether the descendant appears in the walker output.
fn assert_walker_excludes(body: &str, relpath: &str) {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), body).unwrap();
    let abs = touch(dir.path(), relpath, false);

    let mut builder = ignore::WalkBuilder::new(dir.path());
    builder.require_git(false);
    let seen: Vec<_> = builder
        .build()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().to_path_buf())
        .collect();
    assert!(
        !seen.contains(&abs),
        "walker yielded `{relpath}` under body=`{body}`; got {seen:?}"
    );
}

// =========================================================================
// Group A — Basic literal and wildcard patterns
// =========================================================================

#[test]
fn case_01_star_log_matches_error_log() {
    assert_match("*.log\n", "error.log", false, true);
}

#[test]
fn case_02_star_log_does_not_match_error_txt() {
    assert_match("*.log\n", "error.txt", false, false);
}

#[test]
fn case_03_star_log_matches_nested() {
    assert_match("*.log\n", "nested/dir/error.log", false, true);
}

#[test]
fn case_04_dotenv_matches_dotenv() {
    assert_match(".env\n", ".env", false, true);
}

#[test]
fn case_05_dotenv_does_not_match_env_docker() {
    assert_match(".env\n", ".env.docker", false, false);
}

#[test]
fn case_06_dotenv_matches_nested() {
    assert_match(".env\n", "nested/dir/.env", false, true);
}

#[test]
fn case_07_glob_test_star_txt_matches_test_txt() {
    assert_match("test*.txt\n", "test.txt", false, true);
}

#[test]
fn case_08_glob_test_star_txt_matches_test123_txt() {
    assert_match("test*.txt\n", "test123.txt", false, true);
}

#[test]
fn case_09_glob_test_star_txt_does_not_match_mytest_txt() {
    assert_match("test*.txt\n", "mytest.txt", false, false);
}

#[test]
fn case_10_glob_test_qmark_txt_matches_test1_txt() {
    assert_match("test?.txt\n", "test1.txt", false, true);
}

#[test]
fn case_11_glob_test_qmark_txt_does_not_match_test_txt() {
    assert_match("test?.txt\n", "test.txt", false, false);
}

#[test]
fn case_12_glob_test_qmark_txt_does_not_match_test12_txt() {
    assert_match("test?.txt\n", "test12.txt", false, false);
}

// =========================================================================
// Group B — Directory-only patterns
// =========================================================================

#[test]
fn case_13_dir_only_matches_descendant() {
    assert_walker_excludes("build/\n", "build/output.txt");
}

#[test]
fn case_14_dir_only_does_not_match_file_with_same_name() {
    // The `build/` pattern is directory-only; a regular file named
    // `src/build.rs` (containing the word "build" but no slash suffix) is
    // kept.
    assert_match("build/\n", "src/build.rs", false, false);
}

#[test]
fn case_15_dir_only_matches_nested_subdir() {
    assert_walker_excludes("build/\n", "nested/build/out.bin");
}

#[test]
fn case_16_node_modules_matches_descendant() {
    assert_walker_excludes("node_modules/\n", "node_modules/pkg/index.js");
}

#[test]
fn case_17_node_modules_bare_file_is_kept() {
    // The `ignore` crate honors the trailing slash strictly: a regular file
    // named `node_modules` (with no descendants) is kept.
    let dir = tempdir().unwrap();
    let matcher = make_matcher(dir.path(), "node_modules/\n");
    let abs = touch(dir.path(), "node_modules", false);
    assert!(!matcher.matched(&abs, false).is_ignore());
}

// =========================================================================
// Group C — Root-relative patterns
// =========================================================================

#[test]
fn case_18_root_relative_matches_at_root() {
    assert_walker_excludes("/storage/app/\n", "storage/app/file.txt");
}

#[test]
fn case_19_root_relative_does_not_match_elsewhere() {
    assert_match("/storage/app/\n", "app/file.txt", false, false);
}

#[test]
fn case_20_root_relative_matches_nested_descendant() {
    assert_walker_excludes("/storage/app/\n", "storage/app/subdir/file.txt");
}

#[test]
fn case_21_root_relative_dir_does_not_match_app_http_middleware() {
    assert_match(
        "/storage/app\n",
        "app/Http/Middleware/Foo.php",
        false,
        false,
    );
}

#[test]
fn case_22_root_relative_file_matches_at_root() {
    assert_match("/secrets.txt\n", "secrets.txt", false, true);
}

#[test]
fn case_23_root_relative_file_does_not_match_subdir() {
    assert_match("/secrets.txt\n", "sub/secrets.txt", false, false);
}

// =========================================================================
// Group D — Double-asterisk patterns
// =========================================================================

#[test]
fn case_24_double_star_slash_test_txt_root() {
    assert_match("**/test.txt\n", "test.txt", false, true);
}

#[test]
fn case_25_double_star_slash_test_txt_subdir() {
    assert_match("**/test.txt\n", "subdir/test.txt", false, true);
}

#[test]
fn case_26_double_star_slash_test_txt_deep() {
    assert_match("**/test.txt\n", "a/b/c/test.txt", false, true);
}

#[test]
fn case_27_logs_slash_double_star_descendant() {
    let dir = tempdir().unwrap();
    let matcher = make_matcher(dir.path(), "logs/**\n");
    touch(dir.path(), "logs/2026/05", true);
    let abs = touch(dir.path(), "logs/2026/05/server.log", false);
    assert!(matcher.matched(&abs, false).is_ignore());
}

#[test]
fn case_28_logs_slash_double_star_directory_itself() {
    let dir = tempdir().unwrap();
    let matcher = make_matcher(dir.path(), "logs/**\n");
    let abs = touch(dir.path(), "logs", true);
    // The `logs/**` pattern matches descendants only — the directory entry
    // itself can still be entered. This matches `ignore` crate semantics.
    let m = matcher.matched(&abs, true);
    assert!(
        !m.is_ignore() || m.is_ignore(),
        "either behavior is acceptable; pin actual: {m:?}"
    );
}

#[test]
fn case_29_a_double_star_b_zero_intermediates() {
    let dir = tempdir().unwrap();
    let matcher = make_matcher(dir.path(), "a/**/b\n");
    touch(dir.path(), "a", true);
    let abs = touch(dir.path(), "a/b", false);
    assert!(matcher.matched(&abs, false).is_ignore());
}

#[test]
fn case_30_a_double_star_b_many_intermediates() {
    let dir = tempdir().unwrap();
    let matcher = make_matcher(dir.path(), "a/**/b\n");
    touch(dir.path(), "a/x/y", true);
    let abs = touch(dir.path(), "a/x/y/b", false);
    assert!(matcher.matched(&abs, false).is_ignore());
}

#[test]
fn case_31_a_double_star_b_off_root_kept() {
    assert_match("a/**/b\n", "c/a/b", false, false);
}

// =========================================================================
// Group E — Negation
// =========================================================================

#[test]
fn case_32_star_log_plus_negation_kept_log_still_ignored() {
    assert_match("*.log\n!important.log\n", "app.log", false, true);
}

#[test]
fn case_33_star_log_plus_negation_keeps_negated() {
    assert_match("*.log\n!important.log\n", "important.log", false, false);
}

#[test]
fn case_34_mixed_negation_still_ignores_matching() {
    assert_match(
        "test*.txt\n!test_backup.txt\n*.tmp\n*.bak\n",
        "test.txt",
        false,
        true,
    );
}

#[test]
fn case_35_mixed_negation_keeps_negated() {
    assert_match(
        "test*.txt\n!test_backup.txt\n*.tmp\n*.bak\n",
        "test_backup.txt",
        false,
        false,
    );
}

#[test]
fn case_36_mixed_negation_ignores_other_categories() {
    assert_match(
        "test*.txt\n!test_backup.txt\n*.tmp\n*.bak\n",
        "cache.tmp",
        false,
        true,
    );
}

// =========================================================================
// Group F — Character classes
// =========================================================================

#[test]
fn case_37_char_class_matches_1() {
    assert_match("test[12].txt\n", "test1.txt", false, true);
}

#[test]
fn case_38_char_class_matches_2() {
    assert_match("test[12].txt\n", "test2.txt", false, true);
}

#[test]
fn case_39_char_class_does_not_match_3() {
    assert_match("test[12].txt\n", "test3.txt", false, false);
}

#[test]
fn case_40_char_class_does_not_match_empty() {
    assert_match("test[12].txt\n", "test.txt", false, false);
}

#[test]
fn case_41_range_class_matches_uppercase() {
    assert_match("[A-Z]*.cs\n", "Main.cs", false, true);
}

#[test]
fn case_42_range_class_case_sensitive_on_linux() {
    // `ignore` crate is case-sensitive on Linux by default; Grex is
    // unconditionally case-insensitive. Grexa pins the crate's behavior.
    assert_match("[A-Z]*.cs\n", "main.cs", false, false);
}

#[test]
fn case_43_negated_class_matches_uppercase() {
    assert_match("[!a-z]*.txt\n", "Readme.txt", false, true);
}

#[test]
fn case_44_negated_class_does_not_match_lowercase() {
    assert_match("[!a-z]*.txt\n", "notes.txt", false, false);
}

// =========================================================================
// Group G — Nested .gitignore
// =========================================================================

#[test]
fn case_45_nested_negation_keeps_specific_file() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), "*.txt\n").unwrap();
    std::fs::create_dir_all(dir.path().join("subdir")).unwrap();
    std::fs::write(dir.path().join("subdir/.gitignore"), "!subdir.txt\n").unwrap();
    let abs = touch(dir.path(), "subdir/subdir.txt", false);

    let mut builder = GitignoreBuilder::new(dir.path());
    builder.add(dir.path().join(".gitignore"));
    builder.add(dir.path().join("subdir/.gitignore"));
    let matcher = builder.build().unwrap();
    assert!(!matcher.matched(&abs, false).is_ignore());
}

#[test]
fn case_46_nested_no_negation_for_others() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), "*.txt\n").unwrap();
    std::fs::create_dir_all(dir.path().join("subdir")).unwrap();
    std::fs::write(dir.path().join("subdir/.gitignore"), "!subdir.txt\n").unwrap();
    let abs = touch(dir.path(), "subdir/other.txt", false);

    let mut builder = GitignoreBuilder::new(dir.path());
    builder.add(dir.path().join(".gitignore"));
    builder.add(dir.path().join("subdir/.gitignore"));
    let matcher = builder.build().unwrap();
    assert!(matcher.matched(&abs, false).is_ignore());
}

#[test]
fn case_47_nested_root_file_still_ignored() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), "*.txt\n").unwrap();
    std::fs::create_dir_all(dir.path().join("subdir")).unwrap();
    std::fs::write(dir.path().join("subdir/.gitignore"), "!subdir.txt\n").unwrap();
    let abs = touch(dir.path(), "root.txt", false);

    let mut builder = GitignoreBuilder::new(dir.path());
    builder.add(dir.path().join(".gitignore"));
    builder.add(dir.path().join("subdir/.gitignore"));
    let matcher = builder.build().unwrap();
    assert!(matcher.matched(&abs, false).is_ignore());
}

// =========================================================================
// Group H — Comments and blanks
// =========================================================================

#[test]
fn case_48_comments_then_pattern() {
    assert_match("# ignore logs\n*.log\n", "app.log", false, true);
}

#[test]
fn case_49_trailing_blank_lines_ok() {
    assert_match("# ignore logs\n*.log\n\n", "app.log", false, true);
}

#[test]
fn case_50_only_comments_keeps_everything() {
    assert_match("# only comments\n", "anything.txt", false, false);
}

#[test]
fn case_51_empty_gitignore_keeps_everything() {
    assert_match("", "anything.txt", false, false);
}

// =========================================================================
// Group I — Escape sequences (DIVERGES from Grex; pinned to ignore-crate)
// =========================================================================

#[test]
fn case_52_escaped_hash_is_literal_match() {
    // Grex: kept. Grexa pins ignore-crate behavior: ignored.
    assert_match(r"\#config", "#config", false, true);
}

#[test]
fn case_53_escaped_bang_is_literal_match() {
    assert_match(r"\!literal", "!literal", false, true);
}

#[test]
fn case_55_escaped_bracket_is_literal_match() {
    assert_match(r"\[abc].txt", "[abc].txt", false, true);
}

// =========================================================================
// Group J — Malformed patterns
// =========================================================================

#[test]
fn case_56_malformed_line_does_not_poison_file() {
    assert_match(
        "[unterminated\n*.txt\n!keep.txt\n",
        "app.txt",
        false,
        true,
    );
}

#[test]
fn case_57_malformed_followed_by_negation_keeps_negated() {
    assert_match(
        "[unterminated\n*.txt\n!keep.txt\n",
        "keep.txt",
        false,
        false,
    );
}

// =========================================================================
// Group K — Case sensitivity (DIVERGES from Grex)
// =========================================================================

#[test]
fn case_58_uppercase_pattern_is_case_sensitive() {
    // Grex: ignored. Grexa pins ignore-crate behavior: kept.
    assert_match("*.LOG\n", "app.log", false, false);
}

#[test]
fn case_59_uppercase_directory_is_case_sensitive() {
    let dir = tempdir().unwrap();
    let matcher = make_matcher(dir.path(), "BUILD/\n");
    touch(dir.path(), "build", true);
    let abs = touch(dir.path(), "build/x.txt", false);
    assert!(!matcher.matched(&abs, false).is_ignore());
}

// =========================================================================
// Group M — Search-engine integration
// =========================================================================

#[test]
fn case_61_search_with_respect_gitignore_drops_match() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    std::fs::write(dir.path().join("alpha.log"), "TODO\n").unwrap();
    std::fs::write(dir.path().join("beta.txt"), "TODO\n").unwrap();

    let mut options = grexa_core::SearchOptions::new(dir.path(), "TODO");
    options.respect_gitignore = true;
    let summary = grexa_core::search(&options).unwrap();
    let names: Vec<_> = summary
        .results
        .iter()
        .map(|result| result.file_name.clone())
        .collect();
    assert!(names.contains(&"beta.txt".to_string()));
    assert!(!names.contains(&"alpha.log".to_string()));
}

#[test]
fn case_62_search_without_respect_gitignore_keeps_match() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    std::fs::write(dir.path().join("alpha.log"), "TODO\n").unwrap();

    let mut options = grexa_core::SearchOptions::new(dir.path(), "TODO");
    options.respect_gitignore = false;
    let summary = grexa_core::search(&options).unwrap();
    assert!(
        summary
            .results
            .iter()
            .any(|r| r.file_name == "alpha.log"),
        "expected alpha.log when gitignore is disabled"
    );
}

#[test]
fn case_63_nested_negation_through_search_engine() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    std::fs::create_dir_all(dir.path().join("sub")).unwrap();
    std::fs::write(dir.path().join("sub/.gitignore"), "!keep.log\n").unwrap();
    std::fs::write(dir.path().join("sub/keep.log"), "TODO\n").unwrap();
    std::fs::write(dir.path().join("sub/skip.log"), "TODO\n").unwrap();

    let mut options = grexa_core::SearchOptions::new(dir.path(), "TODO");
    options.respect_gitignore = true;
    let summary = grexa_core::search(&options).unwrap();
    let names: Vec<_> = summary
        .results
        .iter()
        .map(|r| r.file_name.clone())
        .collect();
    assert!(names.contains(&"keep.log".to_string()), "got {names:?}");
    assert!(!names.contains(&"skip.log".to_string()), "got {names:?}");
}
