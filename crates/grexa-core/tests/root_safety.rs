// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Root-search safety tests.
//!
//! PLAN.md phase 15 line 457 requires that searches initiated against the
//! Linux root never recurse into `/proc`, `/sys`, `/dev`, or `/run` — these
//! pseudo filesystems return wildly different content per process, regularly
//! generate spurious matches, and can hang the walker indefinitely on
//! special files.
//!
//! Rather than searching the real `/` (which is filesystem-dependent), we
//! construct a fake root in a tempdir that mirrors the structure
//! `system path → file with matches`, and verify the walker skips the
//! sensitive paths.

use std::fs;

use grexa_core::{SearchOptions, search};
use tempfile::tempdir;

fn write_under(root: &std::path::Path, sub: &str, body: &str) {
    let path = root.join(sub);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
}

#[test]
fn search_excludes_sys_proc_dev_by_default() {
    let dir = tempdir().unwrap();
    write_under(dir.path(), "proc/1/cmdline", "TODO process state\n");
    write_under(dir.path(), "sys/kernel/notes", "TODO kernel content\n");
    write_under(dir.path(), "dev/disk/notes", "TODO device entry\n");
    write_under(dir.path(), "home/me/notes.txt", "TODO real match\n");

    let summary = search(&SearchOptions::new(dir.path(), "TODO")).unwrap();
    let paths: Vec<_> = summary
        .results
        .iter()
        .map(|r| r.full_path.to_string_lossy().to_string())
        .collect();

    assert!(
        paths.iter().any(|p| p.ends_with("/home/me/notes.txt")),
        "real match should be found"
    );
    assert!(
        !paths.iter().any(|p| p.contains("/proc/")),
        "/proc must be skipped, got {paths:?}"
    );
    assert!(
        !paths.iter().any(|p| p.contains("/sys/")),
        "/sys must be skipped, got {paths:?}"
    );
    assert!(
        !paths.iter().any(|p| p.contains("/dev/")),
        "/dev must be skipped, got {paths:?}"
    );
}

#[test]
fn search_includes_pseudo_paths_when_user_overrides() {
    let dir = tempdir().unwrap();
    write_under(dir.path(), "proc/1/cmdline", "TODO process state\n");
    write_under(dir.path(), "home/me/notes.txt", "TODO real match\n");

    let mut options = SearchOptions::new(dir.path(), "TODO");
    options.include_system = true;
    let summary = search(&options).unwrap();
    let paths: Vec<_> = summary
        .results
        .iter()
        .map(|r| r.full_path.to_string_lossy().to_string())
        .collect();

    assert!(
        paths.iter().any(|p| p.contains("/proc/")),
        "with --include-system the user is in control; /proc should be searchable: {paths:?}"
    );
}

#[test]
fn search_tolerates_unreadable_directory() {
    // Simulate a permission-denied subdirectory by creating a directory we
    // can't enter. The walker should skip it without aborting the search.
    let dir = tempdir().unwrap();
    write_under(dir.path(), "ok/a.txt", "TODO ok\n");
    let denied = dir.path().join("denied");
    fs::create_dir_all(&denied).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(denied.join("hidden.txt"), "TODO hidden\n").unwrap();
        let mut perms = fs::metadata(&denied).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&denied, perms).unwrap();
    }

    let summary = search(&SearchOptions::new(dir.path(), "TODO"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Restore so tempdir cleanup works.
        let mut perms = fs::metadata(&denied).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&denied, perms).unwrap();
    }

    let summary = summary.unwrap();
    // The "ok" file must be found; the denied file is allowed to be either
    // visible (if running as root) or skipped (the normal case).
    assert!(summary.matches >= 1);
}

#[test]
fn searches_files_under_a_root_named_like_a_system_dir() {
    // The auto-exclusion of `bin`, `vendor`, `node_modules`, etc. must apply
    // only to directories *below* the user-chosen root. If a user points Grexa
    // at a directory that itself happens to be named `bin` (or sits under a
    // path component like `dev`/`vendor`), every file would otherwise be
    // silently skipped — a security-relevant false "no matches".
    let dir = tempdir().unwrap();
    let root = dir.path().join("bin");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("notes.txt"), "TODO real match\n").unwrap();

    let summary = search(&SearchOptions::new(&root, "TODO")).unwrap();
    assert_eq!(
        summary.results.len(),
        1,
        "a file under a root named 'bin' must still be searched"
    );
}

#[test]
fn still_excludes_system_dirs_below_a_root_named_like_one() {
    // The relative-to-root check must not regress the core invariant: a
    // `node_modules` *under* the root is still skipped even when the root
    // itself is named `bin`.
    let dir = tempdir().unwrap();
    let root = dir.path().join("bin");
    fs::create_dir(&root).unwrap();
    write_under(&root, "src/app.rs", "TODO keep me\n");
    write_under(&root, "node_modules/pkg/index.js", "TODO drop me\n");

    let summary = search(&SearchOptions::new(&root, "TODO")).unwrap();
    let paths: Vec<_> = summary
        .results
        .iter()
        .map(|r| r.full_path.to_string_lossy().to_string())
        .collect();
    assert!(
        paths.iter().any(|p| p.ends_with("/src/app.rs")),
        "real source must be found, got {paths:?}"
    );
    assert!(
        !paths.iter().any(|p| p.contains("/node_modules/")),
        "node_modules below the root must still be skipped, got {paths:?}"
    );
}
