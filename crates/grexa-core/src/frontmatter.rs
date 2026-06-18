// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Frontmatter utilities for grexa-core's search engine.
//!
//! Uses [`grexa_db::frontmatter`] to extract YAML frontmatter from file
//! content. This lets the search engine understand structured fields in
//! markdown files — a bridge between grexa-db's typed world and grexa-core's
//! text-search world.

pub fn extract_fields(content: &str) -> Option<grexa_db::Value> {
    grexa_db::frontmatter::split(content).ok()?.frontmatter
}

pub fn extract_string_field(content: &str, field: &str) -> Option<String> {
    extract_fields(content)?
        .get(field)?
        .as_str()
        .map(String::from)
}

pub fn extract_int_field(content: &str, field: &str) -> Option<i64> {
    extract_fields(content)?.get(field)?.as_i64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_from_file_with_frontmatter() {
        let content = "---\ntitle: Hello\nrating: 4\n---\nBody text.\n";
        let fields = extract_fields(content).unwrap();
        assert_eq!(fields["title"].as_str(), Some("Hello"));
        assert_eq!(fields["rating"].as_i64(), Some(4));
    }

    #[test]
    fn extract_returns_none_without_frontmatter() {
        assert!(extract_fields("just body\n").is_none());
    }

    #[test]
    fn extract_string_field_works() {
        let content = "---\ntitle: Test\n---\nbody\n";
        assert_eq!(extract_string_field(content, "title"), Some("Test".to_string()));
        assert_eq!(extract_string_field(content, "missing"), None);
    }

    #[test]
    fn extract_int_field_works() {
        let content = "---\ncount: 42\n---\nbody\n";
        assert_eq!(extract_int_field(content, "count"), Some(42));
    }
}
