// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Budget-aware packing of search-result evidence into AI prompt text.
//!
//! Grexa's grep already produces the query-relevant lines; this packs them into
//! a fixed character budget so the model can answer ABOUT the matched files
//! instead of only suggesting filters. The budget is spread breadth-first — one
//! match per file before any file gets a second line — so many files are
//! represented under a tight budget, the regime where dumping the whole top file
//! degrades worst.

use serde::{Deserialize, Serialize};

/// One matched line within a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLine {
    pub line: u32,
    pub text: String,
}

/// A file and the search matches found in it, best lines first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceMatch {
    pub path: String,
    pub lines: Vec<EvidenceLine>,
}

/// The packed prompt block plus what fit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedEvidence {
    pub text: String,
    pub truncated: bool,
    pub files_included: usize,
    pub lines_included: usize,
}

/// Rough chars-per-token for English/code; a caller budgeting in model tokens
/// multiplies by this to get a character budget for [`pack_evidence`].
pub const CHARS_PER_TOKEN_ESTIMATE: usize = 4;

/// Pack `matches` (already ranked best-first by the search) into at most
/// `budget_chars` characters of prompt text, spreading the budget breadth-first
/// across files. The returned `text` never exceeds `budget_chars`.
pub fn pack_evidence(matches: &[EvidenceMatch], budget_chars: usize) -> PackedEvidence {
    let mut out = String::new();
    let mut files_included = 0usize;
    let mut lines_included = 0usize;

    let total_lines: usize = matches.iter().map(|m| m.lines.len()).sum();
    let mut cursor = vec![0usize; matches.len()];
    let mut header_written = vec![false; matches.len()];

    // Each pass adds at most one more line per file; stop once a full pass fits
    // nothing more (budget exhausted) or every line is in.
    loop {
        let mut added_any = false;
        for (i, m) in matches.iter().enumerate() {
            let li = cursor[i];
            let Some(line) = m.lines.get(li) else {
                continue;
            };
            let mut chunk = String::new();
            if !header_written[i] {
                chunk.push_str(&m.path);
                chunk.push_str(":\n");
            }
            chunk.push_str("  ");
            chunk.push_str(&line.line.to_string());
            chunk.push_str(": ");
            chunk.push_str(line.text.trim_end());
            chunk.push('\n');

            if out.len() + chunk.len() > budget_chars {
                continue; // this line doesn't fit; a smaller one elsewhere might
            }
            if !header_written[i] {
                header_written[i] = true;
                files_included += 1;
            }
            out.push_str(&chunk);
            cursor[i] = li + 1;
            lines_included += 1;
            added_any = true;
        }
        if !added_any {
            break;
        }
    }

    PackedEvidence {
        text: out,
        truncated: lines_included < total_lines,
        files_included,
        lines_included,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(path: &str, lines: &[(u32, &str)]) -> EvidenceMatch {
        EvidenceMatch {
            path: path.to_string(),
            lines: lines
                .iter()
                .map(|(n, t)| EvidenceLine {
                    line: *n,
                    text: t.to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn never_exceeds_budget() {
        let matches = vec![
            m("a.rs", &[(1, "alpha"), (2, "beta")]),
            m("b.rs", &[(9, "gamma")]),
        ];
        for budget in [0, 5, 12, 30, 1000] {
            let packed = pack_evidence(&matches, budget);
            assert!(packed.text.len() <= budget, "budget {budget}: {:?}", packed.text);
        }
    }

    #[test]
    fn spreads_breadth_first_across_files() {
        // Budget fits all three files' first lines (3 x 14 = 42 chars) plus room
        // for one deeper line, so breadth lands before any second line is added.
        let matches = vec![
            m("a.rs", &[(1, "a1"), (2, "a2"), (3, "a3")]),
            m("b.rs", &[(1, "b1"), (2, "b2")]),
            m("c.rs", &[(1, "c1")]),
        ];
        let packed = pack_evidence(&matches, 52);
        assert_eq!(packed.files_included, 3, "all three files should appear: {:?}", packed.text);
        assert!(packed.truncated, "deeper lines were dropped");
        // The first line of every file precedes any file's second line.
        let a1 = packed.text.find("a1").unwrap();
        let b1 = packed.text.find("b1").unwrap();
        let c1 = packed.text.find("c1").unwrap();
        if let Some(a2) = packed.text.find("a2") {
            assert!(a1 < a2 && b1 < a2 && c1 < a2, "breadth before depth: {:?}", packed.text);
        }
    }

    #[test]
    fn includes_everything_when_budget_is_ample() {
        let matches = vec![m("a.rs", &[(1, "only")])];
        let packed = pack_evidence(&matches, 10_000);
        assert!(!packed.truncated);
        assert_eq!(packed.files_included, 1);
        assert_eq!(packed.lines_included, 1);
        assert!(packed.text.contains("a.rs:"));
        assert!(packed.text.contains("1: only"));
    }

    #[test]
    fn empty_input_is_empty_not_truncated() {
        let packed = pack_evidence(&[], 100);
        assert!(packed.text.is_empty());
        assert!(!packed.truncated);
        assert_eq!(packed.files_included, 0);
    }

    #[test]
    fn tiny_budget_drops_oversized_line_but_keeps_smaller_one() {
        let matches = vec![
            m("verylongfilename.rs", &[(1, "this line is quite long indeed")]),
            m("s", &[(1, "x")]),
        ];
        // Enough for "s:\n  1: x\n" but not the long file's header+line.
        let packed = pack_evidence(&matches, 10);
        assert!(packed.text.contains("s:"), "small file should fit: {:?}", packed.text);
        assert!(!packed.text.contains("verylongfilename"), "oversized entry dropped");
        assert!(packed.truncated);
    }
}
