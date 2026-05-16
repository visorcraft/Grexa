use std::fs;
use std::io::{self, Read};
use std::path::Path;

use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE};
use serde::{Deserialize, Serialize};

/// Detected file encoding, paired with the human-readable label used in
/// `FileSearchResult::encoding`. Labels intentionally match Grex's display
/// names so result tables look the same across the two apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectedEncoding {
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    Utf32Le,
    Utf32Be,
}

impl DetectedEncoding {
    pub fn label(self) -> &'static str {
        match self {
            DetectedEncoding::Utf8 => "UTF-8",
            DetectedEncoding::Utf8Bom => "UTF-8 BOM",
            DetectedEncoding::Utf16Le => "UTF-16 LE",
            DetectedEncoding::Utf16Be => "UTF-16 BE",
            DetectedEncoding::Utf32Le => "UTF-32 LE",
            DetectedEncoding::Utf32Be => "UTF-32 BE",
        }
    }

    fn bom_len(self) -> usize {
        match self {
            DetectedEncoding::Utf8 => 0,
            DetectedEncoding::Utf8Bom => 3,
            DetectedEncoding::Utf16Le | DetectedEncoding::Utf16Be => 2,
            DetectedEncoding::Utf32Le | DetectedEncoding::Utf32Be => 4,
        }
    }

    fn encoding_rs(self) -> Option<&'static Encoding> {
        match self {
            DetectedEncoding::Utf8 | DetectedEncoding::Utf8Bom => Some(UTF_8),
            DetectedEncoding::Utf16Le => Some(UTF_16LE),
            DetectedEncoding::Utf16Be => Some(UTF_16BE),
            // encoding_rs does not handle UTF-32; we currently report it but
            // do not decode. A future change can add iconv or a manual UTF-32
            // decoder if real-world files demand it.
            DetectedEncoding::Utf32Le | DetectedEncoding::Utf32Be => None,
        }
    }
}

/// Detect encoding by peeking at the first bytes of `path`. Always returns
/// some encoding so the caller can proceed; bytes that don't match any BOM
/// default to `Utf8`.
pub fn detect_from_path(path: &Path) -> io::Result<DetectedEncoding> {
    let mut file = fs::File::open(path)?;
    let mut buf = [0u8; 4];
    let read = file.read(&mut buf)?;
    Ok(detect_from_bytes(&buf[..read]))
}

/// Detect encoding from a byte prefix. BOM-driven only; unrecognized prefixes
/// fall back to UTF-8.
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

/// Read a file end-to-end and return its decoded text.
///
/// - UTF-8/UTF-8 BOM/UTF-16 LE/UTF-16 BE: decoded via `encoding_rs`, with
///   invalid bytes replaced by U+FFFD.
/// - UTF-32 LE/BE: read but currently surfaced as lossy UTF-8 because
///   `encoding_rs` does not decode UTF-32. Listed in the audit as a future
///   spike point.
pub fn read_text(path: &Path) -> io::Result<(String, DetectedEncoding)> {
    let bytes = fs::read(path)?;
    let detected = detect_from_bytes(&bytes);
    let payload = &bytes[detected.bom_len().min(bytes.len())..];

    let text = match detected.encoding_rs() {
        Some(encoding) => {
            let (decoded, _, _) = encoding.decode(payload);
            decoded.into_owned()
        }
        None => String::from_utf8_lossy(payload).into_owned(),
    };
    Ok((text, detected))
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
        assert_eq!(
            detect_from_bytes(&[0xEF, 0xBB, 0xBF, b'h']),
            DetectedEncoding::Utf8Bom
        );
    }

    #[test]
    fn detect_utf16_le() {
        assert_eq!(
            detect_from_bytes(&[0xFF, 0xFE, b'h', 0]),
            DetectedEncoding::Utf16Le
        );
    }

    #[test]
    fn detect_utf16_be() {
        assert_eq!(
            detect_from_bytes(&[0xFE, 0xFF, 0, b'h']),
            DetectedEncoding::Utf16Be
        );
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
    fn read_invalid_utf8_yields_replacement_chars() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, [b'h', 0xFF, 0xFE, b'i']).unwrap();
        let (text, encoding) = read_text(&path).unwrap();
        // Two raw bytes in the middle would normally crash a strict UTF-8
        // decode. The lossy path replaces them with U+FFFD so search keeps
        // working.
        assert!(text.starts_with('h'));
        assert!(text.contains('\u{FFFD}'));
        assert_eq!(encoding, DetectedEncoding::Utf8);
    }
}
