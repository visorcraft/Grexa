// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! YAML frontmatter extraction and parsing.
//!
//! Splits a file's content into an optional YAML frontmatter block and the
//! remaining body. The frontmatter format follows the Jekyll / Hugo /
//! Obsidian convention: an opening `---` on the first line, YAML content,
//! and a closing `---` on its own line.
//!
//! ```text
//! ---
//! title: Hello
//! tags: [a, b]
//! ---
//! Body content here.
//! ```
//!
//! The opening `---` must be the very first line (an optional UTF-8 BOM is
//! tolerated). A line that is exactly `---` (after trimming trailing
//! whitespace) closes the block. Files without frontmatter pass through
//! unchanged.

use serde_yaml::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrontmatterError {
    #[error("frontmatter opened with `---` but never closed")]
    Unclosed,
    #[error("invalid YAML in frontmatter: {0}")]
    Yaml(String),
}

/// The result of splitting a file into frontmatter and body.
#[derive(Debug)]
pub struct Split<'a> {
    /// Parsed YAML frontmatter, or `None` if the file had no frontmatter
    /// block (or the block was empty).
    pub frontmatter: Option<Value>,
    /// The body content — everything after the frontmatter block, verbatim.
    pub body: &'a str,
}

/// Split a file's content into optional YAML frontmatter and body.
///
/// See the [module docs](self) for the format. Returns
/// `Split { frontmatter: None, body: <input> }` for files without an
/// opening `---` delimiter.
pub fn split(content: &str) -> Result<Split<'_>, FrontmatterError> {
    let stripped = content.strip_prefix('\u{feff}').unwrap_or(content);

    let rest = stripped
        .strip_prefix("---\r\n")
        .or_else(|| stripped.strip_prefix("---\n"));

    let Some(rest) = rest else {
        return Ok(Split {
            frontmatter: None,
            body: stripped,
        });
    };

    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if trimmed == "---" {
            let yaml_str = &rest[..offset];
            let body = &rest[offset + line.len()..];
            let frontmatter = if yaml_str.trim().is_empty() {
                None
            } else {
                Some(
                    serde_yaml::from_str(yaml_str)
                        .map_err(|e| FrontmatterError::Yaml(e.to_string()))?,
                )
            };
            return Ok(Split { frontmatter, body });
        }
        offset += line.len();
    }

    Err(FrontmatterError::Unclosed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_returns_body_unchanged() {
        let result = split("just body\n").unwrap();
        assert!(result.frontmatter.is_none());
        assert_eq!(result.body, "just body\n");
    }

    #[test]
    fn empty_file() {
        let result = split("").unwrap();
        assert!(result.frontmatter.is_none());
        assert_eq!(result.body, "");
    }

    #[test]
    fn dashes_not_at_start_are_body() {
        let result = split("--- not frontmatter\n").unwrap();
        assert!(result.frontmatter.is_none());
        assert_eq!(result.body, "--- not frontmatter\n");
    }

    #[test]
    fn simple_frontmatter_and_body() {
        let content = "---\ntitle: Hello\ntags: [a, b]\n---\nBody text.\n";
        let result = split(content).unwrap();
        let fm = result.frontmatter.expect("frontmatter should parse");
        assert_eq!(fm["title"].as_str(), Some("Hello"));
        assert_eq!(fm["tags"][0].as_str(), Some("a"));
        assert_eq!(fm["tags"][1].as_str(), Some("b"));
        assert_eq!(result.body, "Body text.\n");
    }

    #[test]
    fn frontmatter_preserves_body_verbatim() {
        let content = "---\nk: v\n---\n# Heading\n\nParagraph with --- in it.\n";
        let result = split(content).unwrap();
        assert_eq!(result.body, "# Heading\n\nParagraph with --- in it.\n");
    }

    #[test]
    fn empty_frontmatter_block() {
        let result = split("---\n---\nbody\n").unwrap();
        assert!(result.frontmatter.is_none());
        assert_eq!(result.body, "body\n");
    }

    #[test]
    fn whitespace_only_frontmatter_is_none() {
        let result = split("---\n   \n  \n---\nbody\n").unwrap();
        assert!(result.frontmatter.is_none());
        assert_eq!(result.body, "body\n");
    }

    #[test]
    fn unclosed_frontmatter_errors() {
        let result = split("---\ntitle: Hello\nbody without closer\n");
        assert!(matches!(result, Err(FrontmatterError::Unclosed)));
    }

    #[test]
    fn closing_delimiter_at_eof_without_newline() {
        let content = "---\ntitle: Hello\n---";
        let result = split(content).unwrap();
        let fm = result.frontmatter.expect("frontmatter");
        assert_eq!(fm["title"].as_str(), Some("Hello"));
        assert_eq!(result.body, "");
    }

    #[test]
    fn body_containing_dashes_is_preserved() {
        let content = "---\nk: v\n---\n---\n---\n";
        let result = split(content).unwrap();
        assert_eq!(result.body, "---\n---\n");
    }

    #[test]
    fn crlf_line_endings() {
        let content = "---\r\ntitle: Hello\r\ntags: [a, b]\r\n---\r\nBody.\r\n";
        let result = split(content).unwrap();
        let fm = result.frontmatter.expect("frontmatter");
        assert_eq!(fm["title"].as_str(), Some("Hello"));
        assert_eq!(fm["tags"][0].as_str(), Some("a"));
        assert_eq!(result.body, "Body.\r\n");
    }

    #[test]
    fn trailing_whitespace_after_closing_delimiter() {
        let result = split("---\nk: v\n---   \nbody\n").unwrap();
        assert_eq!(result.body, "body\n");
    }

    #[test]
    fn bom_at_start_is_stripped() {
        let content = "\u{feff}---\nk: v\n---\nbody\n";
        let result = split(content).unwrap();
        let fm = result.frontmatter.expect("frontmatter");
        assert_eq!(fm["k"].as_str(), Some("v"));
        assert_eq!(result.body, "body\n");
    }

    #[test]
    fn invalid_yaml_errors() {
        let result = split("---\n: invalid\n---\nbody\n");
        assert!(matches!(result, Err(FrontmatterError::Yaml(_))));
    }

    #[test]
    fn nested_and_typed_values() {
        let content = "---\nmetadata:\n  author: Jane\n  year: 2024\nrating: 4.5\npublished: true\n---\nbody\n";
        let result = split(content).unwrap();
        let fm = result.frontmatter.expect("frontmatter");
        assert_eq!(fm["metadata"]["author"].as_str(), Some("Jane"));
        assert_eq!(fm["metadata"]["year"].as_i64(), Some(2024));
        assert_eq!(fm["rating"].as_f64(), Some(4.5));
        assert_eq!(fm["published"].as_bool(), Some(true));
    }

    #[test]
    fn empty_array_value() {
        let content = "---\ntags: []\n---\nbody\n";
        let result = split(content).unwrap();
        let fm = result.frontmatter.expect("frontmatter");
        let tags = fm["tags"].as_sequence().expect("should be a sequence");
        assert!(tags.is_empty());
    }
}
