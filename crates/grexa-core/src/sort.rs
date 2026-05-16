use std::cmp::Ordering;

use crate::models::{FileSearchResult, SearchResult, SearchResultSortField};

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn flip(self, ordering: Ordering) -> Ordering {
        match self {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        }
    }
}

/// Sort a Content-mode (`SearchResult`) list in place.
///
/// Behavior matches Grex `TabViewModel.SortResults`:
/// - `SearchResultSortField::None` is a no-op.
/// - Unsupported fields (Extension, Encoding, Matches) fall back to FileName.
/// - Ties are broken by `(file_name, line_number, column_number, full_path)`
///   so large-result order is deterministic across parallel runs.
pub fn sort_content(results: &mut [SearchResult], field: SearchResultSortField, dir: SortDirection) {
    if field == SearchResultSortField::None {
        return;
    }

    results.sort_by(|a, b| {
        let primary = match field {
            SearchResultSortField::Line => a.line_number.cmp(&b.line_number),
            SearchResultSortField::Column => a.column_number.cmp(&b.column_number),
            SearchResultSortField::Path => a.relative_path.cmp(&b.relative_path),
            // FileName-style fallback for unsupported keys.
            _ => name_cmp(&a.file_name, &b.file_name),
        };

        let primary = dir.flip(primary);
        if primary != Ordering::Equal {
            return primary;
        }

        // Stable tie-breaker is direction-independent: same query yields the
        // same order regardless of the active sort key.
        name_cmp(&a.file_name, &b.file_name)
            .then_with(|| a.line_number.cmp(&b.line_number))
            .then_with(|| a.column_number.cmp(&b.column_number))
            .then_with(|| a.full_path.cmp(&b.full_path))
    });
}

/// Sort a Files-mode (`FileSearchResult`) list in place.
pub fn sort_files(
    results: &mut [FileSearchResult],
    field: SearchResultSortField,
    dir: SortDirection,
) {
    if field == SearchResultSortField::None {
        return;
    }

    results.sort_by(|a, b| {
        let primary = match field {
            SearchResultSortField::Path => a.relative_path.cmp(&b.relative_path),
            SearchResultSortField::Extension => name_cmp(&a.extension, &b.extension),
            SearchResultSortField::Encoding => name_cmp(&a.encoding, &b.encoding),
            SearchResultSortField::Matches => a.match_count.cmp(&b.match_count),
            // FileName-style fallback for unsupported keys (None already
            // returned early, Line/Column are not meaningful here).
            _ => name_cmp(&a.file_name, &b.file_name),
        };

        let primary = dir.flip(primary);
        if primary != Ordering::Equal {
            return primary;
        }

        name_cmp(&a.file_name, &b.file_name).then_with(|| a.full_path.cmp(&b.full_path))
    });
}

/// Apply Grex's default sort to a freshly-completed search summary.
///
/// - Content mode: `FileName` ascending.
/// - Files mode: `MatchCount` descending.
pub fn apply_default_sort(
    content: &mut [SearchResult],
    files: &mut [FileSearchResult],
) {
    sort_content(content, SearchResultSortField::Name, SortDirection::Ascending);
    sort_files(
        files,
        SearchResultSortField::Matches,
        SortDirection::Descending,
    );
}

fn name_cmp(a: &str, b: &str) -> Ordering {
    a.to_lowercase().cmp(&b.to_lowercase()).then_with(|| a.cmp(b))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn result(name: &str, line: usize, column: usize, full: &str) -> SearchResult {
        SearchResult {
            file_name: name.to_string(),
            line_number: line,
            column_number: column,
            line_content: String::new(),
            match_preview_before: String::new(),
            match_preview_match: String::new(),
            match_preview_after: String::new(),
            full_path: PathBuf::from(full),
            relative_path: PathBuf::from(name),
            match_count: 1,
        }
    }

    fn file(name: &str, matches: usize, ext: &str) -> FileSearchResult {
        FileSearchResult {
            file_name: name.to_string(),
            size: 0,
            match_count: matches,
            first_match_line_number: 0,
            match_preview_before: String::new(),
            match_preview_match: String::new(),
            match_preview_after: String::new(),
            preview_matches: Vec::new(),
            full_path: PathBuf::from(name),
            relative_path: PathBuf::from(name),
            extension: ext.to_string(),
            encoding: "UTF-8".to_string(),
            date_modified_unix: None,
        }
    }

    #[test]
    fn sort_content_by_name_ascending_is_case_insensitive() {
        let mut results = vec![
            result("Zebra.rs", 1, 1, "/a/Zebra.rs"),
            result("apple.rs", 1, 1, "/a/apple.rs"),
            result("Banana.rs", 1, 1, "/a/Banana.rs"),
        ];
        sort_content(
            &mut results,
            SearchResultSortField::Name,
            SortDirection::Ascending,
        );
        let names: Vec<_> = results.iter().map(|r| r.file_name.clone()).collect();
        assert_eq!(names, vec!["apple.rs", "Banana.rs", "Zebra.rs"]);
    }

    #[test]
    fn sort_content_by_line_then_column_breaks_ties_deterministically() {
        let mut results = vec![
            result("b.rs", 5, 2, "/x/b.rs"),
            result("a.rs", 5, 1, "/x/a.rs"),
            result("a.rs", 1, 1, "/x/a.rs"),
        ];
        sort_content(
            &mut results,
            SearchResultSortField::Line,
            SortDirection::Ascending,
        );
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].file_name, "a.rs");
        assert_eq!(results[2].file_name, "b.rs");
    }

    #[test]
    fn sort_content_descending_reverses_primary_key_only() {
        let mut results = vec![
            result("a.rs", 1, 1, "/x/a.rs"),
            result("b.rs", 1, 1, "/x/b.rs"),
            result("c.rs", 1, 1, "/x/c.rs"),
        ];
        sort_content(
            &mut results,
            SearchResultSortField::Name,
            SortDirection::Descending,
        );
        let names: Vec<_> = results.iter().map(|r| r.file_name.clone()).collect();
        assert_eq!(names, vec!["c.rs", "b.rs", "a.rs"]);
    }

    #[test]
    fn sort_content_none_is_noop() {
        let mut results = vec![
            result("z.rs", 1, 1, "/x/z.rs"),
            result("a.rs", 1, 1, "/x/a.rs"),
        ];
        sort_content(
            &mut results,
            SearchResultSortField::None,
            SortDirection::Ascending,
        );
        assert_eq!(results[0].file_name, "z.rs");
    }

    #[test]
    fn sort_content_unsupported_field_falls_back_to_name() {
        let mut results = vec![
            result("b.rs", 1, 1, "/x/b.rs"),
            result("a.rs", 1, 1, "/x/a.rs"),
        ];
        sort_content(
            &mut results,
            SearchResultSortField::Encoding,
            SortDirection::Ascending,
        );
        assert_eq!(results[0].file_name, "a.rs");
    }

    #[test]
    fn sort_files_by_matches_descending_default() {
        let mut files = vec![
            file("low.rs", 1, "rs"),
            file("high.rs", 100, "rs"),
            file("mid.rs", 10, "rs"),
        ];
        sort_files(
            &mut files,
            SearchResultSortField::Matches,
            SortDirection::Descending,
        );
        let counts: Vec<_> = files.iter().map(|f| f.match_count).collect();
        assert_eq!(counts, vec![100, 10, 1]);
    }

    #[test]
    fn sort_files_by_extension() {
        let mut files = vec![
            file("a.toml", 1, "toml"),
            file("b.rs", 1, "rs"),
            file("c.md", 1, "md"),
        ];
        sort_files(
            &mut files,
            SearchResultSortField::Extension,
            SortDirection::Ascending,
        );
        let exts: Vec<_> = files.iter().map(|f| f.extension.clone()).collect();
        assert_eq!(exts, vec!["md", "rs", "toml"]);
    }

    #[test]
    fn apply_default_sort_uses_grex_defaults() {
        let mut content = vec![
            result("z.rs", 1, 1, "/x/z.rs"),
            result("a.rs", 1, 1, "/x/a.rs"),
        ];
        let mut files = vec![
            file("few.rs", 2, "rs"),
            file("many.rs", 50, "rs"),
        ];
        apply_default_sort(&mut content, &mut files);
        assert_eq!(content[0].file_name, "a.rs");
        assert_eq!(files[0].file_name, "many.rs");
    }
}
