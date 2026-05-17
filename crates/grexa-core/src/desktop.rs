use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Preset editor identifier. Used by Settings to remember the user's choice
/// and by [`open_in_editor_command`] to materialize an argv vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorPreset {
    Kate,
    KWrite,
    VsCode,
    VsCodium,
    SublimeText,
    JetBrains,
    GnomeTextEditor,
    Neovim,
    XdgOpen,
}

impl EditorPreset {
    pub fn binary_name(self) -> &'static str {
        match self {
            EditorPreset::Kate => "kate",
            EditorPreset::KWrite => "kwrite",
            EditorPreset::VsCode => "code",
            EditorPreset::VsCodium => "codium",
            EditorPreset::SublimeText => "subl",
            EditorPreset::JetBrains => "idea",
            EditorPreset::GnomeTextEditor => "gnome-text-editor",
            EditorPreset::Neovim => "nvim",
            EditorPreset::XdgOpen => "xdg-open",
        }
    }
}

/// Build the argv vector that launches `path` at `line` in the chosen editor.
/// `line` is 1-based; pass `None` to just open the file without a line jump.
pub fn open_in_editor_command(
    preset: EditorPreset,
    path: &Path,
    line: Option<usize>,
) -> Vec<OsString> {
    let path_os: OsString = path.as_os_str().to_owned();
    let binary: OsString = preset.binary_name().into();

    match preset {
        // Native KDE: `kate file --line N`. KWrite uses the same flag.
        EditorPreset::Kate | EditorPreset::KWrite => {
            let mut argv = vec![binary, path_os];
            if let Some(line) = line {
                argv.push("--line".into());
                argv.push(line.to_string().into());
            }
            argv
        }
        // `code --goto path:line` selects a line and column. We don't carry a
        // column here so just append :line.
        EditorPreset::VsCode | EditorPreset::VsCodium => {
            let target = match line {
                Some(line) => {
                    let mut combined: OsString = path_os;
                    combined.push(format!(":{line}"));
                    combined
                }
                None => path_os,
            };
            vec![binary, "--goto".into(), target]
        }
        // `subl path:line`
        EditorPreset::SublimeText => {
            let target = match line {
                Some(line) => {
                    let mut combined: OsString = path_os;
                    combined.push(format!(":{line}"));
                    combined
                }
                None => path_os,
            };
            vec![binary, target]
        }
        // `idea --line N path` — JetBrains tools share this convention.
        EditorPreset::JetBrains => {
            let mut argv = vec![binary];
            if let Some(line) = line {
                argv.push("--line".into());
                argv.push(line.to_string().into());
            }
            argv.push(path_os);
            argv
        }
        // `gnome-text-editor +line path`
        EditorPreset::GnomeTextEditor => {
            let mut argv = vec![binary];
            if let Some(line) = line {
                argv.push(format!("+{line}").into());
            }
            argv.push(path_os);
            argv
        }
        // `nvim +line path` — same convention as vi.
        EditorPreset::Neovim => {
            let mut argv = vec![binary];
            if let Some(line) = line {
                argv.push(format!("+{line}").into());
            }
            argv.push(path_os);
            argv
        }
        // `xdg-open` cannot target a line; ignore the parameter.
        EditorPreset::XdgOpen => vec![binary, path_os],
    }
}

/// Classify a path string the user pasted into the search bar so the GUI
/// can surface a useful message before launching a search. Unsupported
/// abstract URLs are flagged with the scheme that would need mounting.
pub fn classify_user_path(input: &str) -> UserPathKind {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return UserPathKind::Empty;
    }
    for scheme in &[
        "smb://", "fish://", "mtp://", "ftp://", "sftp://", "obex://",
    ] {
        if let Some(stripped) = trimmed.strip_prefix(scheme) {
            return UserPathKind::AbstractUrl {
                scheme: scheme.trim_end_matches("://").to_string(),
                rest: stripped.to_string(),
            };
        }
    }
    if trimmed.starts_with("file://") {
        return UserPathKind::FileUri(trimmed.trim_start_matches("file://").to_string());
    }
    if trimmed.starts_with('/') {
        return UserPathKind::Absolute(trimmed.to_string());
    }
    UserPathKind::Relative(trimmed.to_string())
}

/// Result of [`classify_user_path`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserPathKind {
    Empty,
    Absolute(String),
    Relative(String),
    FileUri(String),
    /// An abstract scheme that doesn't map directly to a Linux filesystem
    /// path — the user must mount it (KIO FUSE / gvfs / cifs / sshfs) and
    /// then browse the mounted path. The GUI shows
    /// `t("error-abstract-url-needs-mount", {scheme})`.
    AbstractUrl {
        scheme: String,
        rest: String,
    },
}

/// `xdg-open` fallback for "reveal in file manager" — opens the parent
/// directory. Callers should prefer the
/// `org.freedesktop.FileManager1.ShowItems` D-Bus call when available; this
/// builder is for the dispatch shim.
pub fn reveal_with_xdg_open(path: &Path) -> Vec<OsString> {
    let parent = path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    vec!["xdg-open".into(), parent.into_os_string()]
}

/// Build the D-Bus argument for FileManager1.ShowItems. Returns the `file://`
/// URI list that the dbus call would pass.
pub fn file_manager_show_items_uris(paths: &[&Path]) -> Vec<String> {
    paths.iter().map(|path| path_to_file_uri(path)).collect()
}

fn path_to_file_uri(path: &Path) -> String {
    // file:// URI: percent-encode non-ASCII and reserved chars. We use a
    // minimal encoder that covers spaces and characters the FileManager1 spec
    // explicitly lists as reserved.
    let s = path.to_string_lossy();
    let mut out = String::from("file://");
    for ch in s.chars() {
        match ch {
            '/' | 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(ch),
            _ => {
                for byte in ch.to_string().as_bytes() {
                    out.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn into_strings(argv: Vec<OsString>) -> Vec<String> {
        argv.into_iter().map(|v| v.into_string().unwrap()).collect()
    }

    #[test]
    fn kate_open_at_line() {
        let argv =
            open_in_editor_command(EditorPreset::Kate, &PathBuf::from("/tmp/a.rs"), Some(42));
        assert_eq!(into_strings(argv), vec!["kate", "/tmp/a.rs", "--line", "42"]);
    }

    #[test]
    fn vscode_open_at_line() {
        let argv =
            open_in_editor_command(EditorPreset::VsCode, &PathBuf::from("/tmp/a.rs"), Some(7));
        assert_eq!(into_strings(argv), vec!["code", "--goto", "/tmp/a.rs:7"]);
    }

    #[test]
    fn neovim_open_at_line() {
        let argv =
            open_in_editor_command(EditorPreset::Neovim, &PathBuf::from("/tmp/a.rs"), Some(3));
        assert_eq!(into_strings(argv), vec!["nvim", "+3", "/tmp/a.rs"]);
    }

    #[test]
    fn jetbrains_open_at_line_prefixes_line_flag() {
        let argv =
            open_in_editor_command(EditorPreset::JetBrains, &PathBuf::from("/tmp/a.rs"), Some(15));
        assert_eq!(into_strings(argv), vec!["idea", "--line", "15", "/tmp/a.rs"]);
    }

    #[test]
    fn no_line_omits_line_argument() {
        let argv = open_in_editor_command(EditorPreset::Kate, &PathBuf::from("/tmp/a"), None);
        assert_eq!(into_strings(argv), vec!["kate", "/tmp/a"]);

        let argv = open_in_editor_command(EditorPreset::Neovim, &PathBuf::from("/tmp/a"), None);
        assert_eq!(into_strings(argv), vec!["nvim", "/tmp/a"]);
    }

    #[test]
    fn xdg_open_ignores_line_argument() {
        let argv = open_in_editor_command(EditorPreset::XdgOpen, &PathBuf::from("/tmp/a"), Some(1));
        assert_eq!(into_strings(argv), vec!["xdg-open", "/tmp/a"]);
    }

    #[test]
    fn reveal_with_xdg_open_targets_parent_dir() {
        let argv = reveal_with_xdg_open(&PathBuf::from("/tmp/sub/file.rs"));
        assert_eq!(into_strings(argv), vec!["xdg-open", "/tmp/sub"]);
    }

    #[test]
    fn reveal_with_xdg_open_defaults_to_cwd_when_no_parent() {
        let argv = reveal_with_xdg_open(&PathBuf::from("loose-file"));
        assert_eq!(into_strings(argv), vec!["xdg-open", ""]);
    }

    #[test]
    fn classify_path_detects_abstract_schemes() {
        assert_eq!(classify_user_path(""), UserPathKind::Empty);
        match classify_user_path("smb://server/share/dir") {
            UserPathKind::AbstractUrl { scheme, rest } => {
                assert_eq!(scheme, "smb");
                assert_eq!(rest, "server/share/dir");
            }
            other => panic!("expected AbstractUrl, got {other:?}"),
        }
        match classify_user_path("/home/me/code") {
            UserPathKind::Absolute(p) => assert_eq!(p, "/home/me/code"),
            other => panic!("expected Absolute, got {other:?}"),
        }
        match classify_user_path("relative/path") {
            UserPathKind::Relative(p) => assert_eq!(p, "relative/path"),
            other => panic!("expected Relative, got {other:?}"),
        }
        match classify_user_path("file:///home/me/file.txt") {
            UserPathKind::FileUri(p) => assert_eq!(p, "/home/me/file.txt"),
            other => panic!("expected FileUri, got {other:?}"),
        }
    }

    #[test]
    fn file_manager_uri_encodes_spaces_and_unicode() {
        let uris = file_manager_show_items_uris(&[Path::new("/tmp/space file.rs")]);
        assert_eq!(uris, vec!["file:///tmp/space%20file.rs".to_string()]);

        let uris = file_manager_show_items_uris(&[Path::new("/tmp/测试.txt")]);
        assert!(uris[0].starts_with("file:///tmp/%E6"));
    }
}
