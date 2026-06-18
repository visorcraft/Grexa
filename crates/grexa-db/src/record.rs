// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! A single record — one file's parsed frontmatter and body.

use crate::frontmatter;
use serde_yaml::Value;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecordError {
    #[error("frontmatter error: {0}")]
    Frontmatter(String),
    #[error("failed to read record file `{path}`: {reason}")]
    ReadFile { path: String, reason: String },
    #[error("invalid record path `{0}`: absolute paths, `..`, and backslashes are not allowed")]
    InvalidPath(String),
}

/// A single record within a collection.
///
/// A record has:
/// - a **path** relative to its collection root (forward-slash normalized),
/// - optional **frontmatter** parsed from YAML,
/// - a **body** (everything after the frontmatter block).
pub struct Record {
    path: String,
    frontmatter: Value,
    body: String,
}

impl Record {
    /// Parse a record from its collection-relative path and raw file content.
    pub fn from_content(path: impl Into<String>, content: &str) -> Result<Self, RecordError> {
        let split =
            frontmatter::split(content).map_err(|e| RecordError::Frontmatter(e.to_string()))?;
        Ok(Self {
            path: path.into(),
            frontmatter: split.frontmatter.unwrap_or(Value::Null),
            body: split.body.to_string(),
        })
    }

    /// The record's path relative to its collection root.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The body content — everything after the frontmatter block.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Whether this record has a (non-null) frontmatter block.
    pub fn has_frontmatter(&self) -> bool {
        !self.frontmatter.is_null()
    }

    /// Raw YAML value of a frontmatter field, or `None` if absent.
    pub fn field(&self, name: &str) -> Option<&Value> {
        self.frontmatter.get(name)
    }
}

impl fmt::Debug for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Record")
            .field("path", &self.path)
            .field("frontmatter", &self.frontmatter)
            .field("body_len", &self.body.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_with_frontmatter_and_body() {
        let content = "---\ntitle: Hello\nrating: 4\n---\nBody text.\n";
        let r = Record::from_content("note.md", content).unwrap();
        assert_eq!(r.path(), "note.md");
        assert!(r.has_frontmatter());
        assert_eq!(r.field("title").unwrap().as_str(), Some("Hello"));
        assert_eq!(r.field("rating").unwrap().as_i64(), Some(4));
        assert_eq!(r.body(), "Body text.\n");
    }

    #[test]
    fn record_without_frontmatter() {
        let content = "Just markdown, no frontmatter.\n";
        let r = Record::from_content("note.md", content).unwrap();
        assert!(!r.has_frontmatter());
        assert!(r.field("title").is_none());
        assert_eq!(r.body(), "Just markdown, no frontmatter.\n");
    }

    #[test]
    fn missing_field_returns_none() {
        let content = "---\ntitle: Hello\n---\nbody\n";
        let r = Record::from_content("note.md", content).unwrap();
        assert!(r.field("nonexistent").is_none());
    }

    #[test]
    fn array_field_access() {
        let content = "---\ntags: [rust, db, flat-file]\n---\nbody\n";
        let r = Record::from_content("note.md", content).unwrap();
        let tags = r.field("tags").unwrap();
        let seq = tags.as_sequence().unwrap();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq[0].as_str(), Some("rust"));
        assert_eq!(seq[2].as_str(), Some("flat-file"));
    }

    #[test]
    fn debug_does_not_dump_full_body() {
        let content = "---\nk: v\n---\nvery long body ".repeat(100);
        let r = Record::from_content("note.md", &content).unwrap();
        let debug = format!("{:?}", r);
        assert!(debug.contains("body_len"));
        assert!(!debug.contains("very long body very long body"));
    }
}
