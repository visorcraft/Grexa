// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cmd() -> Command {
    Command::cargo_bin("grexa-cli").expect("grexa-cli binary")
}

fn write(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

#[test]
fn text_output_lists_matches() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("alpha.txt"), "hello\nTODO fix\n");

    cmd()
        .arg(dir.path())
        .arg("TODO")
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha.txt:2:1:TODO fix"));
}

#[test]
fn json_output_is_pretty_array() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "TODO\n");

    cmd()
        .args([dir.path().to_str().unwrap(), "TODO", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("[\n"))
        .stdout(predicate::str::contains("\"line_number\": 1"));
}

#[test]
fn csv_output_has_header_and_escaping() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "TODO, with comma\n");

    cmd()
        .args([dir.path().to_str().unwrap(), "TODO", "--format", "csv"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("File,Line,Column,Content"))
        .stdout(predicate::str::contains("\"TODO, with comma\""));
}

#[test]
fn csv_output_neutralizes_spreadsheet_formulas() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "=HYPERLINK(\"https://example.invalid\",\"TODO\")\n");

    cmd()
        .args([dir.path().to_str().unwrap(), "TODO", "--format", "csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"'=HYPERLINK(\"\"https://example.invalid\"\""));
}

#[test]
fn count_output_prints_total() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "TODO\nTODO\n");

    cmd()
        .args([dir.path().to_str().unwrap(), "TODO", "--count"])
        .assert()
        .success()
        .stdout(predicate::eq("2\n"));
}

#[test]
fn files_only_dedups_and_sorts() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "TODO\nTODO\n");
    write(&dir.path().join("b.txt"), "TODO\n");

    let assert = cmd()
        .args([dir.path().to_str().unwrap(), "TODO", "--files-only"])
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].ends_with("a.txt"));
    assert!(lines[1].ends_with("b.txt"));
}

#[test]
fn quiet_exit_one_when_no_matches() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "irrelevant\n");

    cmd()
        .args([dir.path().to_str().unwrap(), "TODO", "--quiet"])
        .assert()
        .code(1)
        .stdout(predicate::eq(""));
}

#[test]
fn quiet_exit_zero_when_matches() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "TODO\n");

    cmd()
        .args([dir.path().to_str().unwrap(), "TODO", "--quiet"])
        .assert()
        .code(0)
        .stdout(predicate::eq(""));
}

#[test]
fn nonexistent_path_returns_error_exit_two() {
    cmd()
        .args(["/nonexistent/path/should/fail", "TODO"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn regex_flag_finds_pattern_matches() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("data.txt"), "abc-1\nabc-22\nxyz\n");

    cmd()
        .args([dir.path().to_str().unwrap(), r"abc-\d+", "--regex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("data.txt:1:1:abc-1"))
        .stdout(predicate::str::contains("data.txt:2:1:abc-22"));
}

#[test]
fn match_files_filter_limits_extensions() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("keep.rs"), "TODO\n");
    write(&dir.path().join("skip.log"), "TODO\n");

    let assert = cmd()
        .args([
            dir.path().to_str().unwrap(),
            "TODO",
            "--match-files",
            "*.rs",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("keep.rs"));
    assert!(!stdout.contains("skip.log"));
}

#[test]
fn utf16_le_files_are_searchable() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("notes.txt");
    let mut bytes = vec![0xFF, 0xFE];
    for ch in "first\nTODO café\n".encode_utf16() {
        bytes.extend_from_slice(&ch.to_le_bytes());
    }
    fs::write(&path, bytes).unwrap();

    cmd()
        .args([dir.path().to_str().unwrap(), "TODO"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TODO café"));
}

#[test]
fn ignore_diacritics_finds_match_when_haystack_has_accent() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "café\n");

    cmd()
        .args([dir.path().to_str().unwrap(), "cafe", "--ignore-diacritics"])
        .assert()
        .success()
        .stdout(predicate::str::contains("café"));
}

#[test]
fn comparison_invariant_culture_succeeds_against_basic_input() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "hello world\n");

    cmd()
        .args([
            dir.path().to_str().unwrap(),
            "hello",
            "--comparison",
            "invariant-culture",
        ])
        .assert()
        .success();
}

#[test]
fn use_index_and_no_index_are_mutually_exclusive() {
    let dir = tempdir().unwrap();
    write(&dir.path().join("a.txt"), "TODO\n");

    cmd()
        .args([
            dir.path().to_str().unwrap(),
            "TODO",
            "--use-index",
            "--no-index",
        ])
        .assert()
        .failure();
}

#[test]
fn completions_subcommand_emits_bash_script() {
    cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_grexa-cli()"));
}

#[test]
fn manpage_subcommand_emits_roff() {
    cmd()
        .arg("manpage")
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH grexa-cli"));
}

#[test]
fn replace_subcommand_rewrites_matching_files() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, "TODO write\nTODO fix\n").unwrap();

    cmd()
        .args(["replace", dir.path().to_str().unwrap(), "TODO", "DONE"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2 replacements"))
        .stdout(predicate::str::contains("a.txt"));

    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("DONE write"));
    assert!(content.contains("DONE fix"));
    assert!(!content.contains("TODO"));
}

#[test]
fn replace_dry_run_does_not_modify_files() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, "TODO write\n").unwrap();

    cmd()
        .args([
            "replace",
            dir.path().to_str().unwrap(),
            "TODO",
            "DONE",
            "--dry-run",
        ])
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("TODO write"));
}

#[test]
fn replace_regex_mode_uses_captures() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("data.txt");
    fs::write(&file, "foo-123\nbar-456\n").unwrap();

    cmd()
        .args([
            "replace",
            dir.path().to_str().unwrap(),
            r"(\w+)-(\d+)",
            "$2-$1",
            "--regex",
        ])
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("123-foo"));
    assert!(content.contains("456-bar"));
}

#[test]
fn replace_case_insensitive_mode() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, "Hello HELLO hello\n").unwrap();

    cmd()
        .args(["replace", dir.path().to_str().unwrap(), "hello", "hey"])
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();
    assert_eq!(content, "hey hey hey\n");
}

#[test]
fn replace_reports_zero_matches_exit_code() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, "nothing relevant\n").unwrap();

    cmd()
        .args(["replace", dir.path().to_str().unwrap(), "TODO", "DONE"])
        .assert()
        .code(1);
}
