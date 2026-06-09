// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Searchable-document extractors.
//!
//! Grex declares the following extensions as "searchable binary" — i.e. files
//! that look binary on disk but carry searchable text inside:
//!
//! - OOXML: `.docx`, `.xlsx`, `.pptx`
//! - ODF: `.odt`, `.ods`, `.odp`
//! - Generic ZIP: `.zip` (names + embedded text/XML entries)
//! - PDF: `.pdf` (via `pdftotext` when available)
//! - RTF: `.rtf`
//!
//! Each extractor returns `Ok(Some(text))` with a UTF-8 string suitable for
//! handing to the line-by-line scanner, `Ok(None)` when the file is shaped
//! correctly but contains no searchable text, or `Err(ExtractError)` on
//! genuine I/O / parse failures. The search engine treats any `Err` as a
//! skip, mirroring Grex's behavior of silently dropping unreadable
//! attachments.

use std::ffi::OsStr;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use quick_xml::Reader;
use quick_xml::events::Event;
use thiserror::Error;
use zip::ZipArchive;

const MAX_ZIP_ENTRIES: usize = 1024;
/// Wall-clock budget for the `pdftotext` subprocess. A malicious or
/// pathological PDF can make `pdftotext` spin or dribble output indefinitely;
/// without a deadline the reading thread (and, on the single-threaded CLI, the
/// whole process) hangs forever. After this elapses the child is killed.
const PDF_EXTRACT_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_TEXTUAL_ENTRY_BYTES: usize = 4 * 1024 * 1024;
const MAX_EXTRACTED_TEXT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("zip parse error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("xml parse error: {0}")]
    Xml(String),
    #[error("pdftotext unavailable or failed: {0}")]
    Pdf(String),
    #[error("document extraction limit exceeded: {0}")]
    Limit(&'static str),
}

impl From<quick_xml::Error> for ExtractError {
    fn from(value: quick_xml::Error) -> Self {
        ExtractError::Xml(value.to_string())
    }
}

/// Best-effort extraction. Picks an extractor based on the file extension.
/// Unknown extensions return `Ok(None)`; the caller should fall back to its
/// plain-text reader.
pub fn extract_text(path: &Path) -> Result<Option<String>, ExtractError> {
    let Some(ext) = normalized_extension(path) else {
        return Ok(None);
    };
    match ext.as_str() {
        "docx" => extract_ooxml(path, &["word/document.xml"]).map(Some),
        "xlsx" => extract_ooxml(path, &["xl/sharedStrings.xml", "xl/comments1.xml"]).map(Some),
        "pptx" => extract_ooxml_glob(path, "ppt/slides/").map(Some),
        "odt" | "ods" | "odp" => extract_ooxml(path, &["content.xml"]).map(Some),
        "zip" => extract_zip(path).map(Some),
        "rtf" => extract_rtf(path).map(Some),
        "pdf" => extract_pdf(path).map(Some),
        _ => Ok(None),
    }
}

/// Extract text from one or more named XML entries inside a ZIP container.
fn extract_ooxml(path: &Path, entries: &[&str]) -> Result<String, ExtractError> {
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    check_zip_entry_count(&archive)?;
    let mut out = String::new();
    for entry in entries {
        let inner = match archive.by_name(entry) {
            Ok(inner) => inner,
            Err(zip::result::ZipError::FileNotFound) => continue,
            Err(err) => return Err(err.into()),
        };
        if inner.size() > MAX_TEXTUAL_ENTRY_BYTES as u64 {
            return Err(ExtractError::Limit("ZIP entry too large"));
        }
        push_xml_text(inner, &mut out)?;
        push_limited(&mut out, "\n")?;
    }
    Ok(out)
}

/// Extract text from every ZIP entry whose path starts with `prefix` and
/// ends with `.xml`. Used for OOXML formats that scatter content across many
/// entries (`ppt/slides/slide*.xml`).
fn extract_ooxml_glob(path: &Path, prefix: &str) -> Result<String, ExtractError> {
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    check_zip_entry_count(&archive)?;
    let mut targets: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.starts_with(prefix) && name.ends_with(".xml") {
            targets.push(name);
        }
    }
    targets.sort();

    let mut out = String::new();
    for name in targets {
        let inner = archive.by_name(&name)?;
        if inner.size() > MAX_TEXTUAL_ENTRY_BYTES as u64 {
            return Err(ExtractError::Limit("ZIP entry too large"));
        }
        push_xml_text(inner, &mut out)?;
        push_limited(&mut out, "\n")?;
    }
    Ok(out)
}

/// Generic ZIP extractor: file names on their own lines, plus the verbatim
/// content of every entry whose name extension looks textual.
fn extract_zip(path: &Path) -> Result<String, ExtractError> {
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    check_zip_entry_count(&archive)?;

    let mut out = String::new();
    let names: Vec<String> = (0..archive.len())
        .map(|i| {
            archive
                .by_index(i)
                .map(|entry| entry.name().to_string())
                .unwrap_or_default()
        })
        .filter(|name| !name.is_empty())
        .collect();

    out.push_str("# Entries\n");
    for name in &names {
        push_limited(&mut out, name)?;
        push_limited(&mut out, "\n")?;
    }
    push_limited(&mut out, "\n# Contents\n")?;

    for name in names {
        if !is_textual_name(&name) {
            continue;
        }
        let mut inner = match archive.by_name(&name) {
            Ok(inner) => inner,
            Err(_) => continue,
        };
        if inner.size() > MAX_TEXTUAL_ENTRY_BYTES as u64 {
            return Err(ExtractError::Limit("ZIP entry too large"));
        }
        let mut buf = Vec::new();
        read_limited(&mut inner, MAX_TEXTUAL_ENTRY_BYTES, &mut buf)?;
        push_limited(&mut out, &format!("\n--- {name} ---\n"))?;
        push_limited(&mut out, &String::from_utf8_lossy(&buf))?;
    }
    Ok(out)
}

fn check_zip_entry_count<R: Read + std::io::Seek>(
    archive: &ZipArchive<R>,
) -> Result<(), ExtractError> {
    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(ExtractError::Limit("too many ZIP entries"));
    }
    Ok(())
}

fn is_textual_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    matches!(
        Path::new(&lower).extension().and_then(OsStr::to_str),
        Some("txt")
            | Some("md")
            | Some("xml")
            | Some("json")
            | Some("yaml")
            | Some("yml")
            | Some("html")
            | Some("htm")
            | Some("css")
            | Some("js")
            | Some("ts")
            | Some("rs")
            | Some("py")
            | Some("go")
            | Some("java")
            | Some("c")
            | Some("h")
            | Some("cpp")
            | Some("hpp")
            | Some("toml")
            | Some("ini")
            | Some("conf")
            | Some("csv")
            | Some("tsv")
            | Some("log")
            | Some("sh")
            | Some("bash")
            | Some("zsh")
            | Some("fish")
    )
}

/// Strip XML markup and concatenate the textual content into `out`.
fn push_xml_text<R: Read>(reader: R, out: &mut String) -> Result<(), ExtractError> {
    let limited = reader.take((MAX_TEXTUAL_ENTRY_BYTES + 1) as u64);
    let mut reader = Reader::from_reader(std::io::BufReader::new(limited));
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                let bytes = e.into_inner();
                let unescaped =
                    quick_xml::escape::unescape(std::str::from_utf8(&bytes).unwrap_or(""))
                        .unwrap_or_default()
                        .into_owned();
                push_limited(out, &unescaped)?;
                push_limited(out, " ")?;
            }
            Ok(Event::CData(e)) => {
                let bytes = e.into_inner();
                push_limited(out, &String::from_utf8_lossy(&bytes))?;
                push_limited(out, " ")?;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(err) => return Err(ExtractError::Xml(err.to_string())),
        }
        buf.clear();
    }
    Ok(())
}

/// RTF extractor — strip control words and group braces. RTF is plain ASCII
/// with `\controlword` and `{...}` groups; the visible text is everything
/// else. This is the standard "RTF degradation to text" algorithm and matches
/// what Grex's WPF `RichTextBox` would show.
fn extract_rtf(path: &Path) -> Result<String, ExtractError> {
    let mut file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    read_limited(&mut file, MAX_EXTRACTED_TEXT_BYTES, &mut bytes)?;
    let text = std::str::from_utf8(&bytes)
        .map(str::to_string)
        .unwrap_or_else(|_| String::from_utf8_lossy(&bytes).into_owned());

    let mut out = String::with_capacity(text.len() / 2);
    let mut chars = text.chars().peekable();
    let mut group_depth = 0_i32;
    let mut skip_group_until: Option<i32> = None;

    while let Some(ch) = chars.next() {
        if let Some(target_depth) = skip_group_until {
            // Inside a group we want to skip entirely (\fonttbl, \stylesheet,
            // \colortbl, \info, \pict, \*). Track braces until we close the
            // group at the same depth we entered.
            match ch {
                '{' => group_depth += 1,
                '}' => {
                    group_depth -= 1;
                    if group_depth < target_depth {
                        skip_group_until = None;
                    }
                }
                _ => {}
            }
            continue;
        }
        match ch {
            '{' => group_depth += 1,
            '}' => {
                if group_depth > 0 {
                    group_depth -= 1;
                }
            }
            '\\' => {
                // Control word or symbol. Read until non-alphanumeric.
                let mut word = String::new();
                let starred = chars.peek() == Some(&'*');
                if starred {
                    chars.next();
                }
                while let Some(&peek) = chars.peek() {
                    if peek.is_ascii_alphabetic() {
                        word.push(peek);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Optional decimal parameter.
                let mut param = String::new();
                if let Some(&peek) = chars.peek()
                    && (peek == '-' || peek.is_ascii_digit())
                {
                    param.push(peek);
                    chars.next();
                    while let Some(&p2) = chars.peek() {
                        if p2.is_ascii_digit() {
                            param.push(p2);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                // A space after the control word is the delimiter; consume it.
                if chars.peek() == Some(&' ') {
                    chars.next();
                }

                match word.as_str() {
                    // Skip the whole containing group for these "destinations".
                    "fonttbl" | "stylesheet" | "colortbl" | "info" | "pict" | "header"
                    | "footer" | "object" => {
                        skip_group_until = Some(group_depth);
                    }
                    // `\*` introduces a destination control; skip its group.
                    _ if starred => {
                        skip_group_until = Some(group_depth);
                    }
                    "par" | "line" | "lbr" => push_limited(&mut out, "\n")?,
                    "tab" => push_limited(&mut out, "\t")?,
                    "'" => {
                        // Hex escape: \'XX
                        let hex: String = chars.by_ref().take(2).collect();
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            push_limited(&mut out, &(byte as char).to_string())?;
                        }
                    }
                    _ => {
                        // Unknown control word — drop the word and its param.
                    }
                }
            }
            '\r' | '\n' => {} // structural whitespace is ignored in RTF
            _ => push_limited(&mut out, &ch.to_string())?,
        }
    }
    Ok(out)
}

/// PDF extractor — shells out to `pdftotext` if available. Returns
/// `ExtractError::Pdf` when the binary isn't on `$PATH` or the file is
/// encrypted/malformed; callers treat that as a skip.
fn extract_pdf(path: &Path) -> Result<String, ExtractError> {
    let child = Command::new("pdftotext")
        .arg("-layout")
        // `--` terminates option parsing so a path beginning with `-` can never
        // be misread as a poppler flag.
        .arg("--")
        .arg(path)
        .arg("-") // write to stdout
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| ExtractError::Pdf(err.to_string()))?;

    let buf = run_with_timeout(child, PDF_EXTRACT_TIMEOUT)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn normalized_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
}

/// Drive a spawned child to completion under a wall-clock deadline, returning
/// its captured stdout (capped at [`MAX_EXTRACTED_TEXT_BYTES`]). stdout is read
/// on a worker thread so a hung child can be killed when the deadline elapses
/// instead of blocking the caller forever. Returns `Err` on timeout, read
/// error, or a non-zero exit status.
fn run_with_timeout(mut child: Child, timeout: Duration) -> Result<Vec<u8>, ExtractError> {
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| ExtractError::Pdf("child stdout unavailable".to_string()))?;

    // Read on a worker thread; the pipe read blocks until EOF, so a child that
    // never produces output (or spins) would otherwise hang us indefinitely.
    let (tx, rx) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut buf = Vec::new();
        let outcome = read_limited(&mut stdout, MAX_EXTRACTED_TEXT_BYTES, &mut buf);
        let _ = tx.send(outcome.map(|()| buf));
    });

    let kill_and_join = |child: &mut Child, reader: thread::JoinHandle<()>| {
        let _ = child.kill();
        let _ = child.wait();
        let _ = reader.join();
    };

    match rx.recv_timeout(timeout) {
        Ok(Ok(buf)) => {
            let status = child
                .wait()
                .map_err(|err| ExtractError::Pdf(err.to_string()))?;
            let _ = reader.join();
            if status.success() {
                Ok(buf)
            } else {
                Err(ExtractError::Pdf(format!("exit status {status}")))
            }
        }
        Ok(Err(read_err)) => {
            kill_and_join(&mut child, reader);
            Err(read_err)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            kill_and_join(&mut child, reader);
            Err(ExtractError::Pdf(format!("timed out after {}s", timeout.as_secs())))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            kill_and_join(&mut child, reader);
            Err(ExtractError::Pdf("reader thread disconnected".to_string()))
        }
    }
}

fn read_limited<R: Read>(
    reader: &mut R,
    max_bytes: usize,
    out: &mut Vec<u8>,
) -> Result<(), ExtractError> {
    reader
        .take((max_bytes + 1) as u64)
        .read_to_end(out)
        .map_err(ExtractError::Io)?;
    if out.len() > max_bytes {
        return Err(ExtractError::Limit("input too large"));
    }
    Ok(())
}

fn push_limited(out: &mut String, value: &str) -> Result<(), ExtractError> {
    if out.len().saturating_add(value.len()) > MAX_EXTRACTED_TEXT_BYTES {
        return Err(ExtractError::Limit("extracted text too large"));
    }
    out.push_str(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    use super::*;

    fn write_zip<F: FnOnce(&mut zip::ZipWriter<std::fs::File>)>(path: &Path, body: F) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        body(&mut zip);
        zip.finish().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn run_with_timeout_returns_output_for_a_fast_process() {
        let child = Command::new("printf")
            .arg("hello world")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let buf = run_with_timeout(child, Duration::from_secs(5)).unwrap();
        assert_eq!(buf, b"hello world");
    }

    #[cfg(unix)]
    #[test]
    fn run_with_timeout_kills_a_hanging_process() {
        use std::time::Instant;

        let child = Command::new("sleep")
            .arg("30")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let start = Instant::now();
        let result = run_with_timeout(child, Duration::from_millis(200));
        let elapsed = start.elapsed();

        assert!(result.is_err(), "a hung process must yield an error");
        assert!(
            elapsed >= Duration::from_millis(150),
            "must actually wait for the deadline, waited {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_secs(10),
            "must not block for the full sleep, waited {elapsed:?}"
        );
    }

    fn write_entry(zip: &mut zip::ZipWriter<std::fs::File>, name: &str, content: &[u8]) {
        zip.start_file(name, SimpleFileOptions::default()).unwrap();
        zip.write_all(content).unwrap();
    }

    #[test]
    fn unknown_extension_returns_none() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("note.txt");
        std::fs::write(&path, "hello").unwrap();
        assert!(extract_text(&path).unwrap().is_none());
    }

    #[test]
    fn docx_pulls_text_from_word_document_xml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("doc.docx");
        write_zip(&path, |zip| {
            let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://example">
  <w:body>
    <w:p><w:r><w:t>Hello docx</w:t></w:r></w:p>
    <w:p><w:r><w:t>TODO fix the typo</w:t></w:r></w:p>
  </w:body>
</w:document>"#;
            write_entry(zip, "word/document.xml", xml.as_bytes());
            write_entry(zip, "[Content_Types].xml", b"<types/>");
        });
        let text = extract_text(&path).unwrap().unwrap();
        assert!(text.contains("Hello docx"), "got {text:?}");
        assert!(text.contains("TODO fix the typo"));
    }

    #[test]
    fn xlsx_pulls_shared_strings() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("book.xlsx");
        write_zip(&path, |zip| {
            let xml = r#"<sst xmlns="http://example">
  <si><t>Sheet1</t></si>
  <si><t>TODO budget review</t></si>
</sst>"#;
            write_entry(zip, "xl/sharedStrings.xml", xml.as_bytes());
        });
        let text = extract_text(&path).unwrap().unwrap();
        assert!(text.contains("TODO budget review"));
    }

    #[test]
    fn pptx_globs_slide_xml_entries() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("deck.pptx");
        write_zip(&path, |zip| {
            write_entry(
                zip,
                "ppt/slides/slide1.xml",
                br#"<p:sld xmlns:p="x"><a:t>Slide one heading</a:t></p:sld>"#,
            );
            write_entry(
                zip,
                "ppt/slides/slide2.xml",
                br#"<p:sld xmlns:p="x"><a:t>Slide two TODO</a:t></p:sld>"#,
            );
        });
        let text = extract_text(&path).unwrap().unwrap();
        assert!(text.contains("Slide one heading"));
        assert!(text.contains("Slide two TODO"));
    }

    #[test]
    fn odt_pulls_content_xml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("doc.odt");
        write_zip(&path, |zip| {
            let xml = r#"<office:document-content xmlns:office="x">
  <office:body><text:p>ODF body TODO finish</text:p></office:body>
</office:document-content>"#;
            write_entry(zip, "content.xml", xml.as_bytes());
        });
        let text = extract_text(&path).unwrap().unwrap();
        assert!(text.contains("ODF body TODO finish"));
    }

    #[test]
    fn zip_includes_file_names_and_text_contents() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bundle.zip");
        write_zip(&path, |zip| {
            write_entry(zip, "notes.txt", b"Plain text TODO finish");
            write_entry(zip, "binary.bin", &[0xFF, 0xFE, 0xFD]);
            write_entry(zip, "readme.md", b"# Heading\n\nMore TODO content.");
        });
        let text = extract_text(&path).unwrap().unwrap();
        assert!(text.contains("notes.txt"), "file names section");
        assert!(text.contains("binary.bin"));
        assert!(text.contains("Plain text TODO finish"));
        assert!(text.contains("More TODO content."));
    }

    #[test]
    fn zip_text_entry_is_size_limited() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bundle.zip");
        write_zip(&path, |zip| {
            write_entry(zip, "huge.txt", &vec![b'A'; MAX_TEXTUAL_ENTRY_BYTES + 1]);
        });

        let err = extract_text(&path).unwrap_err();
        assert!(matches!(err, ExtractError::Limit(_)));
    }

    #[test]
    fn zip_entry_count_is_limited() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bundle.zip");
        write_zip(&path, |zip| {
            for i in 0..=MAX_ZIP_ENTRIES {
                write_entry(zip, &format!("f{i}.txt"), b"x");
            }
        });

        let err = extract_text(&path).unwrap_err();
        assert!(matches!(err, ExtractError::Limit(_)));
    }

    #[test]
    fn rtf_strips_control_words_and_keeps_text() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("note.rtf");
        let rtf = r"{\rtf1\ansi\ansicpg1252\deff0\nouicompat{\fonttbl{\f0\fnil\fcharset0 Calibri;}}\viewkind4\uc1\pard\sa200\sl240\slmult1\f0\fs22\lang9 Hello \b TODO\b0\par finish me\par}";
        std::fs::write(&path, rtf).unwrap();
        let text = extract_text(&path).unwrap().unwrap();
        assert!(text.contains("Hello"), "got {text:?}");
        assert!(text.contains("TODO"));
        assert!(text.contains("finish me"));
        assert!(!text.contains("\\fonttbl"));
    }

    #[test]
    fn pdf_extractor_is_callable_when_pdftotext_present() {
        // Smoke test only — we don't bundle a PDF fixture. Just confirm the
        // extractor surfaces a sensible error when handed a non-PDF, rather
        // than panicking.
        let dir = tempdir().unwrap();
        let path = dir.path().join("not-a.pdf");
        std::fs::write(&path, b"not actually pdf").unwrap();
        let result = extract_text(&path);
        // If pdftotext is installed and rejects the file we expect an Err;
        // if it isn't installed we also expect an Err. Both are fine.
        match result {
            Ok(Some(_)) => panic!("extractor unexpectedly succeeded for non-pdf"),
            Ok(None) => panic!("pdf branch should always attempt extraction"),
            Err(_) => {}
        }
    }

    #[test]
    fn extract_rtf_handles_hex_escapes() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("hex.rtf");
        let rtf = r#"{\rtf1 Hello \'AB world}"#;
        std::fs::write(&path, rtf).unwrap();
        let text = extract_rtf(&path).unwrap();
        assert!(text.contains("Hello"));
    }

    #[test]
    fn extract_rtf_handles_nested_groups() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested.rtf");
        let rtf = r#"{\rtf1 {\b bold} normal {\i {\b both}} end}"#;
        std::fs::write(&path, rtf).unwrap();
        let text = extract_rtf(&path).unwrap();
        assert!(text.contains("bold"));
        assert!(text.contains("normal"));
        assert!(text.contains("end"));
    }

    #[test]
    fn extract_rtf_handles_empty_input() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.rtf");
        std::fs::write(&path, b"").unwrap();
        let result = extract_rtf(&path);
        assert!(result.is_err() || result.as_ref().unwrap().is_empty());
    }
}
