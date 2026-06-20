// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Budget-aware packing of search-result evidence into AI prompt text.
//!
//! Grexa's grep already produces the query-relevant lines; this packs them —
//! each match together with a few lines of surrounding context — into a fixed
//! character budget so the model can answer ABOUT the matched code, not just the
//! bare hit line. The budget is spread breadth-first — one whole snippet per
//! file before any file gets a second — so many files are represented under a
//! tight budget, the regime where dumping the whole top file degrades worst.
//! Every snippet is self-describing (carries its own `path:` header) so the
//! breadth-first interleaving can never mis-attribute a line to the wrong file.

use serde::{Deserialize, Serialize};

/// One source line within a snippet. `is_match` flags the line the search
/// actually hit; the rest are surrounding context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLine {
    pub line: u32,
    pub text: String,
    #[serde(default)]
    pub is_match: bool,
}

/// A contiguous snippet around one match: the matched line plus a few context
/// lines before/after, packed as one atomic unit so the budget never splits it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSnippet {
    pub lines: Vec<EvidenceLine>,
}

/// A file and the match snippets found in it, best-first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceMatch {
    pub path: String,
    pub snippets: Vec<EvidenceSnippet>,
}

/// The packed prompt block plus what fit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedEvidence {
    pub text: String,
    pub truncated: bool,
    pub files_included: usize,
    pub snippets_included: usize,
}

/// Rough chars-per-token for English/code; a caller budgeting in model tokens
/// multiplies by this to get a character budget for [`pack_evidence`].
pub const CHARS_PER_TOKEN_ESTIMATE: usize = 4;

/// Pack `matches` (best-first from the search) into at most `budget_chars`
/// characters, spreading the budget breadth-first across files — one whole
/// snippet per file before any file gets a second. The returned `text` never
/// exceeds `budget_chars`, and a snippet is always emitted whole or not at all.
pub fn pack_evidence(matches: &[EvidenceMatch], budget_chars: usize) -> PackedEvidence {
    let mut out = String::new();
    let mut files_included = 0usize;
    let mut snippets_included = 0usize;

    let total_snippets: usize = matches.iter().map(|m| m.snippets.len()).sum();
    let mut cursor = vec![0usize; matches.len()];
    let mut file_seen = vec![false; matches.len()];

    // Each pass adds at most one more snippet per file; stop once a full pass
    // fits nothing more (budget exhausted) or every snippet is in.
    loop {
        let mut added_any = false;
        for (i, m) in matches.iter().enumerate() {
            let si = cursor[i];
            let Some(snippet) = m.snippets.get(si) else {
                continue;
            };
            let chunk = render_snippet(&m.path, snippet);
            if out.len() + chunk.len() > budget_chars {
                continue; // this snippet doesn't fit; a smaller one elsewhere might
            }
            out.push_str(&chunk);
            cursor[i] = si + 1;
            snippets_included += 1;
            if !file_seen[i] {
                file_seen[i] = true;
                files_included += 1;
            }
            added_any = true;
        }
        if !added_any {
            break;
        }
    }

    PackedEvidence {
        text: out,
        truncated: snippets_included < total_snippets,
        files_included,
        snippets_included,
    }
}

/// Render one self-describing snippet: a `path:` header then its lines, the
/// matched line flagged with `>` so the model can tell the hit from its context.
fn render_snippet(path: &str, snippet: &EvidenceSnippet) -> String {
    let mut s = String::new();
    s.push_str(path);
    s.push_str(":\n");
    for ln in &snippet.lines {
        s.push_str(if ln.is_match { "> " } else { "  " });
        s.push_str(&ln.line.to_string());
        s.push_str(": ");
        s.push_str(ln.text.trim_end());
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(n: u32, text: &str, is_match: bool) -> EvidenceLine {
        EvidenceLine {
            line: n,
            text: text.to_string(),
            is_match,
        }
    }

    fn snippet(lines: &[(u32, &str, bool)]) -> EvidenceSnippet {
        EvidenceSnippet {
            lines: lines.iter().map(|(n, t, m)| line(*n, t, *m)).collect(),
        }
    }

    fn file(path: &str, snippets: Vec<EvidenceSnippet>) -> EvidenceMatch {
        EvidenceMatch {
            path: path.to_string(),
            snippets,
        }
    }

    #[test]
    fn never_exceeds_budget() {
        let matches = vec![
            file(
                "a.rs",
                vec![
                    snippet(&[(1, "alpha", true)]),
                    snippet(&[(9, "beta", true)]),
                ],
            ),
            file("b.rs", vec![snippet(&[(2, "gamma", true)])]),
        ];
        for budget in [0, 5, 14, 40, 1000] {
            let packed = pack_evidence(&matches, budget);
            assert!(packed.text.len() <= budget, "budget {budget}: {:?}", packed.text);
        }
    }

    #[test]
    fn spreads_breadth_first_across_files() {
        // Each first snippet renders to 14 chars ("X.rs:\n> 1: X1\n"); budget 56
        // fits all three firsts (42) plus exactly one second, so breadth lands
        // before any file's second snippet.
        let matches = vec![
            file("a.rs", vec![snippet(&[(1, "a1", true)]), snippet(&[(2, "a2", true)])]),
            file("b.rs", vec![snippet(&[(1, "b1", true)]), snippet(&[(2, "b2", true)])]),
            file("c.rs", vec![snippet(&[(1, "c1", true)])]),
        ];
        let packed = pack_evidence(&matches, 56);
        assert_eq!(packed.files_included, 3, "all three files: {:?}", packed.text);
        assert!(packed.truncated, "a second snippet was dropped");
        let a1 = packed.text.find("a1").unwrap();
        let b1 = packed.text.find("b1").unwrap();
        let c1 = packed.text.find("c1").unwrap();
        if let Some(a2) = packed.text.find("a2") {
            assert!(a1 < a2 && b1 < a2 && c1 < a2, "breadth before depth: {:?}", packed.text);
        }
    }

    #[test]
    fn includes_context_lines_and_flags_the_match() {
        let matches = vec![file(
            "src/x.rs",
            vec![snippet(&[
                (40, "ctx", false),
                (41, "hit", true),
                (42, "ctx", false),
            ])],
        )];
        let packed = pack_evidence(&matches, 10_000);
        assert!(!packed.truncated);
        assert_eq!(packed.snippets_included, 1);
        assert!(packed.text.contains("> 41: hit"), "match flagged: {:?}", packed.text);
        assert!(packed.text.contains("  40: ctx"), "context present: {:?}", packed.text);
    }

    #[test]
    fn empty_input_is_empty_not_truncated() {
        let packed = pack_evidence(&[], 100);
        assert!(packed.text.is_empty());
        assert!(!packed.truncated);
        assert_eq!(packed.files_included, 0);
        assert_eq!(packed.snippets_included, 0);
    }

    #[test]
    fn tiny_budget_drops_oversized_snippet_but_keeps_smaller() {
        let matches = vec![
            file(
                "verylongfilename.rs",
                vec![snippet(&[(1, "this line is quite long indeed", true)])],
            ),
            file("s", vec![snippet(&[(1, "x", true)])]),
        ];
        // Enough for "s:\n> 1: x\n" (10 chars) but not the long file's header.
        let packed = pack_evidence(&matches, 11);
        assert!(packed.text.contains("s:"), "small file should fit: {:?}", packed.text);
        assert!(!packed.text.contains("verylongfilename"), "oversized entry dropped");
        assert!(packed.truncated);
    }
}
