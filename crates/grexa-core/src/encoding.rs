// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use std::fs;
use std::io::{self, Read};
use std::path::Path;

use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE};
use serde::{Deserialize, Serialize};

/// How many bytes to peek for BOM detection. Four is enough for every BOM
/// Grexa knows about (UTF-32 LE/BE need the full four; everything else fits
/// in two or three bytes).
const BOM_PEEK: usize = 4;

/// How many bytes to feed `chardetng` when BOM detection returns plain UTF-8
/// but the file isn't actually valid UTF-8. 64 KiB is the upper bound the
/// chardetng documentation recommends — beyond that the additional confidence
/// gain is marginal.
const CHARDET_PEEK: usize = 64 * 1024;

/// Detected file encoding, paired with the human-readable label used in
/// `FileSearchResult::encoding`. The first six variants match the BOM table
/// exactly. `Heuristic` carries the canonical [`encoding_rs::Encoding::name`]
/// returned by `chardetng` when the file lacks a BOM and isn't valid UTF-8.
///
/// Labels intentionally mirror Grex's display names so result tables look the
/// same across the two apps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectedEncoding {
    /// Plain UTF-8 without a BOM. The default for almost everything Grexa
    /// reads.
    Utf8,
    /// UTF-8 with byte-order mark `EF BB BF`.
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    /// UTF-32 LE — detected only. Grexa does not decode UTF-32 yet; reading a
    /// UTF-32 file falls back to a lossy UTF-8 decode.
    Utf32Le,
    Utf32Be,
    /// Encoding picked by `chardetng`. The string is the canonical
    /// `encoding_rs::Encoding::name()` (e.g. `"windows-1252"`, `"Shift_JIS"`).
    /// Round-trips through serde.
    Heuristic(String),
}

impl DetectedEncoding {
    pub fn label(&self) -> String {
        match self {
            DetectedEncoding::Utf8 => "UTF-8".to_string(),
            DetectedEncoding::Utf8Bom => "UTF-8 BOM".to_string(),
            DetectedEncoding::Utf16Le => "UTF-16 LE".to_string(),
            DetectedEncoding::Utf16Be => "UTF-16 BE".to_string(),
            DetectedEncoding::Utf32Le => "UTF-32 LE".to_string(),
            DetectedEncoding::Utf32Be => "UTF-32 BE".to_string(),
            DetectedEncoding::Heuristic(name) => name.clone(),
        }
    }

    fn bom_len(&self) -> usize {
        match self {
            DetectedEncoding::Utf8 | DetectedEncoding::Heuristic(_) => 0,
            DetectedEncoding::Utf8Bom => 3,
            DetectedEncoding::Utf16Le | DetectedEncoding::Utf16Be => 2,
            DetectedEncoding::Utf32Le | DetectedEncoding::Utf32Be => 4,
        }
    }

    fn encoding_rs(&self) -> Option<&'static Encoding> {
        match self {
            DetectedEncoding::Utf8 | DetectedEncoding::Utf8Bom => Some(UTF_8),
            DetectedEncoding::Utf16Le => Some(UTF_16LE),
            DetectedEncoding::Utf16Be => Some(UTF_16BE),
            DetectedEncoding::Utf32Le | DetectedEncoding::Utf32Be => None,
            DetectedEncoding::Heuristic(name) => Encoding::for_label(name.as_bytes()),
        }
    }
}

/// Detect encoding by peeking at the first bytes of `path`. Always returns
/// some encoding so the caller can proceed; bytes that don't match any BOM
/// default to [`DetectedEncoding::Utf8`].
pub fn detect_from_path(path: &Path) -> io::Result<DetectedEncoding> {
    let mut file = fs::File::open(path)?;
    let mut buf = [0u8; BOM_PEEK];
    let read = file.read(&mut buf)?;
    Ok(detect_from_bytes(&buf[..read]))
}

/// Detect encoding from a byte prefix. BOM-driven only; unrecognized prefixes
/// fall back to UTF-8. Use [`read_text`] for the full BOM → strict-UTF-8 →
/// chardetng cascade.
pub fn detect_from_bytes(bytes: &[u8]) -> DetectedEncoding {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return DetectedEncoding::Utf8Bom;
    }
    if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
        return DetectedEncoding::Utf32Le;
    }
    if bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
        return DetectedEncoding::Utf32Be;
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return DetectedEncoding::Utf16Le;
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return DetectedEncoding::Utf16Be;
    }
    DetectedEncoding::Utf8
}

/// Run `chardetng` against `bytes` and return the canonical
/// `encoding_rs::Encoding::name()` of whatever it guessed. This is only used
/// as a third-tier fallback: BOM detection picks the encoding when one exists,
/// the strict-UTF-8 fast path takes over when the file is clean UTF-8, and
/// this function fires only when both of those fail.
pub fn heuristic_label(bytes: &[u8]) -> &'static str {
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Allow);
    let sample = &bytes[..bytes.len().min(CHARDET_PEEK)];
    detector.feed(sample, true);
    detector.guess(None, Utf8Detection::Allow).name()
}

/// Read a file end-to-end and return its decoded text.
///
/// Cascade:
/// 1. **BOM**: if the file starts with a recognized byte-order mark, decode
///    via the matching `encoding_rs` decoder.
/// 2. **Strict UTF-8 fast path**: BOM-less files that validate as UTF-8 get
///    a zero-copy conversion (no chardetng feed, no allocation beyond the
///    final `String`).
/// 3. **Heuristic**: BOM-less files that are *not* valid UTF-8 are handed to
///    `chardetng`. The result is decoded via `encoding_rs` (which always
///    succeeds because chardetng only ever picks encodings `encoding_rs` can
///    decode).
/// 4. **UTF-32**: detected but not decoded; falls back to lossy UTF-8 to keep
///    search runnable. A future change can layer an `iconv` helper.
///
/// Invalid bytes are replaced with U+FFFD. The function never returns an
/// error from decoding — only filesystem errors propagate.
pub fn read_text(path: &Path) -> io::Result<(String, DetectedEncoding)> {
    let bytes = fs::read(path)?;
    Ok(decode_text(&bytes))
}

/// Decode a byte buffer using the same cascade as [`read_text`].
pub fn decode_text(bytes: &[u8]) -> (String, DetectedEncoding) {
    let detected = detect_from_bytes(bytes);
    let payload = &bytes[detected.bom_len().min(bytes.len())..];

    // BOM-less UTF-8: short-circuit when the bytes are already valid UTF-8.
    if matches!(detected, DetectedEncoding::Utf8) {
        if let Ok(text) = std::str::from_utf8(payload) {
            return (text.to_string(), DetectedEncoding::Utf8);
        }

        // Strict UTF-8 failed → ask chardetng.
        let label = heuristic_label(payload);
        if let Some(encoding) = Encoding::for_label(label.as_bytes()) {
            let (decoded, _, _) = encoding.decode(payload);
            return (decoded.into_owned(), DetectedEncoding::Heuristic(label.to_string()));
        }
        // chardetng returned a label encoding_rs doesn't recognize — fall
        // back to lossy UTF-8 rather than failing the search.
        return (String::from_utf8_lossy(payload).into_owned(), DetectedEncoding::Utf8);
    }

    let text = match detected.encoding_rs() {
        Some(encoding) => {
            let (decoded, _, _) = encoding.decode(payload);
            decoded.into_owned()
        }
        None => String::from_utf8_lossy(payload).into_owned(),
    };
    (text, detected)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn detect_plain_utf8() {
        assert_eq!(detect_from_bytes(b"hello"), DetectedEncoding::Utf8);
    }

    #[test]
    fn detect_utf8_bom() {
        assert_eq!(detect_from_bytes(&[0xEF, 0xBB, 0xBF, b'h']), DetectedEncoding::Utf8Bom);
    }

    #[test]
    fn detect_utf16_le() {
        assert_eq!(detect_from_bytes(&[0xFF, 0xFE, b'h', 0]), DetectedEncoding::Utf16Le);
    }

    #[test]
    fn detect_utf16_be() {
        assert_eq!(detect_from_bytes(&[0xFE, 0xFF, 0, b'h']), DetectedEncoding::Utf16Be);
    }

    #[test]
    fn detect_utf32_le_takes_precedence_over_utf16() {
        // UTF-32 LE BOM begins with the UTF-16 LE BOM bytes; make sure the
        // 4-byte check wins.
        assert_eq!(
            detect_from_bytes(&[0xFF, 0xFE, 0x00, 0x00, b'h', 0, 0, 0]),
            DetectedEncoding::Utf32Le
        );
    }

    #[test]
    fn read_utf8_with_bom_strips_marker() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
        f.write_all("héllo".as_bytes()).unwrap();
        drop(f);
        let (text, encoding) = read_text(&path).unwrap();
        assert_eq!(text, "héllo");
        assert_eq!(encoding, DetectedEncoding::Utf8Bom);
    }

    #[test]
    fn read_utf16_le_decodes_correctly() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        let mut bytes = vec![0xFF, 0xFE];
        for ch in "héllo".encode_utf16() {
            bytes.extend_from_slice(&ch.to_le_bytes());
        }
        std::fs::write(&path, &bytes).unwrap();
        let (text, encoding) = read_text(&path).unwrap();
        assert_eq!(text, "héllo");
        assert_eq!(encoding, DetectedEncoding::Utf16Le);
    }

    #[test]
    fn read_clean_utf8_returns_utf8_without_invoking_chardet() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, "hello — clean text").unwrap();
        let (text, encoding) = read_text(&path).unwrap();
        assert_eq!(text, "hello — clean text");
        assert_eq!(encoding, DetectedEncoding::Utf8);
    }

    #[test]
    fn read_invalid_utf8_triggers_heuristic_detection() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        // Build a realistic Windows-1252 payload: "naïve résumé"
        // 0xEF (ï) and 0xE9 (é) are valid Win-1252 bytes that are *not* valid
        // standalone UTF-8 leading bytes when followed by ASCII, so the
        // strict-UTF-8 fast path will fail and chardetng will get its turn.
        let bytes = b"na\xEFve r\xE9sum\xE9 \xA9 2026";
        std::fs::write(&path, bytes).unwrap();

        let (text, encoding) = read_text(&path).unwrap();
        assert!(text.contains("naïve"), "got {text:?}");
        assert!(text.contains("résumé"), "got {text:?}");
        assert!(text.contains('©'), "got {text:?}");
        match encoding {
            DetectedEncoding::Heuristic(label) => {
                // chardetng picks one of the cp125x family; assert it's not UTF-8
                // and that encoding_rs would round-trip back to a real codec.
                assert!(
                    label.eq_ignore_ascii_case("windows-1252")
                        || label.eq_ignore_ascii_case("ISO-8859-1")
                        || label.eq_ignore_ascii_case("windows-1250")
                        || label.eq_ignore_ascii_case("windows-1254"),
                    "unexpected heuristic label: {label}"
                );
                assert!(Encoding::for_label(label.as_bytes()).is_some());
            }
            other => panic!("expected Heuristic, got {other:?}"),
        }
    }

    #[test]
    fn read_garbage_utf8_falls_back_safely() {
        // Bytes that no encoding can produce meaningfully. The decoder still
        // succeeds via replacement chars; encoding may stay UTF-8 or be a
        // heuristic guess — either is acceptable as long as we don't crash.
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, [0xFF, 0xFE, 0xFD, 0xFC, 0xFB]).unwrap();
        let (_text, _encoding) = read_text(&path).unwrap();
    }

    #[test]
    fn heuristic_label_returns_static_str() {
        // Pure smoke test: chardetng + encoding_rs labels are static.
        let label = heuristic_label(b"plain ASCII");
        assert!(!label.is_empty());
    }
}
