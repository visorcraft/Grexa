# Grex Models Map

This document records every model class in Grex `Models/` and its mapping to
the Grexa Rust equivalent. It complements:

- `docs/grex-storage-services-audit.md` for `RecentSearch`, `SearchProfile`,
  and `DefaultSettings`.
- `docs/grex-context-preview-audit.md` for the `ContextPreviewResult` /
  `ContextLine` runtime semantics.
- `docs/grex-docker-search-service-audit.md` for `DockerContainerInfo`,
  `DockerContainerOption`, `DockerMirrorInfo`.
- `docs/grex-ai-search-service-audit.md` for `AiChatMessage`.
- `docs/grex-search-service-audit.md` for `SearchResult`, `FileSearchResult`,
  `SizeLimitType`, `SizeUnit`, `StringComparisonMode`,
  `UnicodeNormalizationMode`.
- `docs/linux-decisions.md` for the Windows-vs-Linux scope decisions.

Source evidence:

- `Models/AiChatMessage.cs`
- `Models/ContextPreviewResult.cs`
- `Models/DockerContainerInfo.cs`
- `Models/DockerContainerOption.cs`
- `Models/DockerMirrorInfo.cs`
- `Models/FileSearchResult.cs`
- `Models/PathSuggestion.cs`
- `Models/RecentSearch.cs`
- `Models/SearchProfile.cs`
- `Models/SearchResult.cs`
- `Models/SearchResultSortField.cs`
- `Models/SizeLimitType.cs`
- `Models/SizeUnit.cs`
- `Models/StringComparisonMode.cs`
- `Models/UnicodeNormalizationMode.cs`

Grexa targets:

- `crates/grexa-core/src/models.rs`
- `crates/grexa-core/src/storage.rs`
- `crates/grexa-core/src/preview.rs`
- `crates/grexa-containers/src/lib.rs`
- `crates/grexa-ai/src/lib.rs`

## Mapping Status Legend

| Status | Meaning |
| ------ | ------- |
| Ported | A Grexa struct/enum already provides the same semantics. The row links to the Rust definition. |
| Renamed | An equivalent exists with a different name. The row notes both names. |
| Linux-specific replacement | Semantics changed because the Grex behavior is Windows-only. Grexa carries a Linux-native replacement (e.g., `UseWindowsSearchIndex` → `use_file_index`). |
| Non-applicable | Windows-only model; intentionally not ported. The row justifies the decision and cross-references `docs/linux-decisions.md`. |
| Pending | Needs a Grexa equivalent. The row proposes the Rust struct shape and points at the Phase that creates it. |

Phases referenced below come from the Grex-to-Grexa PLAN (see
`linux-decisions.md` for cross-references):

- Phase 2 — Settings, history, profiles, ICU/locale.
- Phase 3 — Search engine core (results, summary, sort).
- Phase 4 — GUI scaffolding (Qt/QML/Kirigami).
- Phase 5 — Preview, encoding detection.
- Phase 7 — Container runtimes (Docker/Podman).
- Phase 8 — AI search client, KWallet/Secret Service.
- Phase 10 — Importer for Grex backups.
- Phase 12 — CLI.
- Phase 13 — Baloo seeding spike.

## Summary Table

| # | Grex model | Kind | Grexa equivalent | Status | Notes |
| - | ---------- | ---- | ----------------- | ------ | ----- |
| 1 | `AiChatMessage` | class | `grexa_ai::AiConversationTurn` (+ `AiRole`) | Renamed | Linux drops the `Speaker` and `TimestampText` UI helpers; timestamps move to the GUI layer. |
| 2 | `ContextPreviewResult` | class | `grexa_core::preview::ContextPreviewResult` | Ported | Field renames documented below. |
| 3 | `ContextLine` (nested in `ContextPreviewResult.cs`) | class | `grexa_core::preview::ContextLine` | Ported | `IsMatchLine` → `is_match`. |
| 4 | `DockerContainerInfo` | class | `grexa_containers::ContainerInfo` | Renamed | Generalized to cover Docker + Podman. Computed `ShortId` / `DisplayName` move to the GUI. |
| 5 | `DockerContainerOption` | class | _(GUI-only adapter)_ | Non-applicable | Pure XAML/ComboBox adapter; replaced by the QML model in Phase 4. |
| 6 | `DockerMirrorInfo` | class | `grexa_containers::ContainerMirrorInfo` | Renamed | Adds `runtime: ContainerRuntimeKind`; serialization stable. |
| 7 | `FileSearchResult` | class | `grexa_core::models::FileSearchResult` | Ported | `DateModified` becomes `date_modified_unix: Option<u64>`; formatters move to the GUI. |
| 8 | `PathSuggestion` | class | _(GUI-only adapter)_ | Pending | Lightweight UI tuple. Proposed crate-local Rust type in Phase 4. |
| 9 | `RecentSearch` | class | `grexa_core::storage::RecentSearch` | Ported | Mapped in `docs/grex-storage-services-audit.md` §RecentSearch Schema. |
| 10 | `SearchProfile` | class | `grexa_core::storage::SearchProfile` (+ inlined `SearchOptions`) | Ported | Mapped in `docs/grex-storage-services-audit.md` §SearchProfile. |
| 11 | `SearchResult` | class | `grexa_core::models::SearchResult` | Ported | Computed UI helpers (`DirectoryPath`, `TrimmedLineContent`) move to GUI. |
| 12 | `SearchResultSortField` | enum | `grexa_core::models::SearchResultSortField` | Renamed | Variant names shortened: `FileName`→`Name`, `LineNumber`→`Line`, etc. |
| 13 | `SizeLimitType` | enum | `grexa_core::models::SizeLimitType` | Ported | Variant set identical. |
| 14 | `SizeUnit` | enum | `grexa_core::models::SizeUnit` | Ported | Variant set identical. |
| 15 | `StringComparisonMode` | enum | `grexa_core::models::StringComparisonMode` | Ported | Variant set identical. |
| 16 | `UnicodeNormalizationMode` | enum | `grexa_core::models::UnicodeNormalizationMode` | Ported | Variant set identical. The `ToNormalizationForm` extension is replaced by ICU in Phase 2. |

Additional Grex-derived types that live outside `Models/` but are reachable
from the same audits — `DefaultSettings`, `ThemePreference`, `SearchSummary`,
`SearchOptions`, `OutputFormat` — are tracked in their own audits and are not
duplicated in this map.

### Counts

- Total Grex model files: **15** (13 classes + 2 enums-only files, including
  the nested `ContextLine` class).
- Ported: **10** (`ContextPreviewResult`, `ContextLine`, `FileSearchResult`,
  `RecentSearch`, `SearchProfile`, `SearchResult`, `SizeLimitType`,
  `SizeUnit`, `StringComparisonMode`, `UnicodeNormalizationMode`).
- Renamed: **4** (`AiChatMessage`, `DockerContainerInfo`, `DockerMirrorInfo`,
  `SearchResultSortField`).
- Non-applicable: **1** (`DockerContainerOption`; see row 5).
- Pending: **1** (`PathSuggestion`; Phase 4 GUI scaffolding).
- Linux-specific replacement: 0 of the model classes themselves — the only
  Linux-specific rewrite (`UseWindowsSearchIndex` → `use_file_index`) lives on
  `SearchProfile`/`DefaultSettings` and is recorded in the storage audit.

## 1. `AiChatMessage`

Source: `Models/AiChatMessage.cs:1-14`. Used by `Services/AiSearchService.cs`
to model one turn of conversation between the user and the AI assistant.

| Field | C# type | Default | Grexa equivalent | Notes |
| ----- | ------- | ------- | ----------------- | ----- |
| `Role` | `string` | `"assistant"` | `AiConversationTurn.role: AiRole` | Parsed via `AiRole::parse` (case-insensitive, falls back to `User`). |
| `Speaker` | `string` | `string.Empty` | _(none)_ | Displayed in chat bubbles. Belongs to the GUI view-model, not the protocol struct. |
| `Content` | `string` | `string.Empty` | `AiConversationTurn.content: String` | Trimmed at send time by `grexa_ai::build_messages`. |
| `Timestamp` | `DateTime` | `DateTime.Now` | _(none)_ | Local time, only used for `TimestampText`; not transmitted. |
| `TimestampText` | computed `string` | n/a | _(none)_ | Pure formatter; GUI concern. |

**Status**: Renamed.

**Linked Rust types** (in `crates/grexa-ai/src/lib.rs`):

```rust
pub enum AiRole { User, Assistant, System }
pub struct AiConversationTurn { pub role: AiRole, pub content: String }
```

**GUI plumbing (Phase 4)**: a Qt/QML `AiChatMessage` view-model will carry the
extra `Speaker` and `Timestamp` fields, plus a localized `timestampText()`
helper. The wire-level type stays the lean `AiConversationTurn`.

## 2. `ContextPreviewResult` / `ContextLine`

Source: `Models/ContextPreviewResult.cs:1-56`.

### `ContextPreviewResult` fields

| Field | C# type | Default | Grexa equivalent | Notes |
| ----- | ------- | ------- | ----------------- | ----- |
| `Lines` | `List<ContextLine>` | `new()` | `lines: Vec<ContextLine>` | Same semantics. |
| `MatchLineIndex` | `int` | `0` | `match_line_index: Option<usize>` | `None` when the file is shorter than `match_line_number`; Grex used `0` as a sentinel. |
| `FileName` | `string` | `string.Empty` | `file_name: String` | Same. |
| `FullPath` | `string` | `string.Empty` | `full_path: PathBuf` | Typed path on Linux. |
| `MatchLineNumber` | `int` | `0` | `match_line_number: usize` | Same. |
| _(none)_ | — | — | `encoding: DetectedEncoding` | New: Grexa surfaces the detected encoding so the GUI can show it without re-reading the file. |

### `ContextLine` fields

| Field | C# type | Default | Grexa equivalent | Notes |
| ----- | ------- | ------- | ----------------- | ----- |
| `LineNumber` | `int` | `0` | `line_number: usize` | 1-based. |
| `Content` | `string` | `string.Empty` | `content: String` | Same. |
| `IsMatchLine` | `bool` | `false` | `is_match: bool` | Renamed for brevity; serialization key kept stable enough that the GUI binds without translation. |

**Status**: Ported. See `crates/grexa-core/src/preview.rs:14-34`.

## 3. `DockerContainerInfo`

Source: `Models/DockerContainerInfo.cs:1-37`.

| Field | C# type | Default | Grexa equivalent | Notes |
| ----- | ------- | ------- | ----------------- | ----- |
| `Id` | `string` | `string.Empty` | `id: String` | Same. |
| `Name` | `string` | `string.Empty` | `name: String` | Same. |
| `Image` | `string` | `string.Empty` | `image: String` | Same. |
| `Status` | `string` | `string.Empty` | `status: String` | Same. |
| `State` | `string` | `string.Empty` | `state: String` | Same. |
| _(none)_ | — | — | `runtime: ContainerRuntimeKind` | New: distinguishes Docker from Podman (rootful or rootless). |
| `ShortId` | computed `string` | n/a | _(none)_ | GUI helper; reproduce in Qt view-model. |
| `DisplayName` | computed `string` | n/a | _(none)_ | GUI helper. |

**Status**: Renamed (`ContainerInfo` in `grexa-containers`).

See `crates/grexa-containers/src/lib.rs:26-34`. The `runtime` field is the
Linux-specific replacement for Docker-Desktop-on-Windows assumptions.

## 4. `DockerContainerOption`

Source: `Models/DockerContainerOption.cs:1-11`.

| Field | C# type | Default | Notes |
| ----- | ------- | ------- | ----- |
| `Label` | `string` | `string.Empty` | UI display string. |
| `Container` | `DockerContainerInfo?` | `null` | Optional container reference. |
| `IsLocal` | computed `bool` | `Container == null` | Marks the "Local filesystem" entry in the ComboBox. |

**Status**: Non-applicable.

`DockerContainerOption` is a pure WinUI ComboBox adapter (it overrides
`ToString()` to feed the XAML template). The Qt/QML port creates an
equivalent QAbstractListModel during Phase 4, but no Rust struct is required
because Grexa exposes containers via `Vec<ContainerInfo>` and the GUI layer
prepends a local-filesystem sentinel row.

Cross-reference: `docs/grex-docker-search-service-audit.md` and
`docs/linux-decisions.md` §Containers.

## 5. `DockerMirrorInfo`

Source: `Models/DockerMirrorInfo.cs:1-13`.

| Field | C# type | Default | Grexa equivalent | Notes |
| ----- | ------- | ------- | ----------------- | ----- |
| `ContainerId` | `string` | `string.Empty` | `container_id: String` | Same. |
| `ContainerName` | `string` | `string.Empty` | `container_name: String` | Same. |
| `ContainerPath` | `string` | `string.Empty` | `container_path: String` | Same. |
| `LocalMirrorPath` | `string` | `string.Empty` | `local_mirror_path: PathBuf` | Typed path. |
| `LocalSearchPath` | `string` | `string.Empty` | `local_search_path: PathBuf` | Typed path. |
| `CreatedUtc` | `DateTime` | `DateTime.UtcNow` | `created_unix: u64` | Same encoding rule as `RecentSearch::timestamp_unix`. |
| _(none)_ | — | — | `runtime: ContainerRuntimeKind` | New: required for Podman parity. |

**Status**: Renamed (`ContainerMirrorInfo` in `grexa-containers`).

See `crates/grexa-containers/src/lib.rs:43-52`.

## 6. `FileSearchResult`

Source: `Models/FileSearchResult.cs:1-81`.

| Field | C# type | Default | Grexa equivalent | Notes |
| ----- | ------- | ------- | ----------------- | ----- |
| `FileName` | `string` | `string.Empty` | `file_name: String` | Same. |
| `Size` | `long` | `0` | `size: u64` | Unsigned in Rust. |
| `MatchCount` | `int` | `0` | `match_count: usize` | Same. |
| `FirstMatchLineNumber` | `int` | `0` | `first_match_line_number: usize` | Same; `0` still means "not available". |
| `MatchPreviewBefore` | `string` | `string.Empty` | `match_preview_before: String` | Same. |
| `MatchPreviewMatch` | `string` | `string.Empty` | `match_preview_match: String` | Same. |
| `MatchPreviewAfter` | `string` | `string.Empty` | `match_preview_after: String` | Same. |
| `PreviewMatches` | `List<SearchResult>` | `new()` | `preview_matches: Vec<SearchResult>` | Same. |
| `FullPath` | `string` | `string.Empty` | `full_path: PathBuf` | Typed path. |
| `RelativePath` | `string` | `string.Empty` | `relative_path: PathBuf` | Typed path. |
| `Extension` | `string` | `string.Empty` | `extension: String` | Lowercased at producer in Grex; Grexa keeps the same convention. |
| `Encoding` | `string` | `"Unknown"` | `encoding: String` | Default still `"Unknown"`. |
| `DateModified` | `DateTime` | `default(DateTime)` | `date_modified_unix: Option<u64>` | `None` represents the unset case Grex used `default(DateTime)` for. |
| `DirectoryPath` | computed `string` | n/a | _(none)_ | GUI helper. |
| `FormattedSize` | computed `string` | n/a | _(none)_ | GUI helper; QML uses Kirigami formatters. |
| `FormattedDateModified` | computed `string` | n/a | _(none)_ | GUI helper. |

**Status**: Ported. See `crates/grexa-core/src/models.rs:121-136`.

## 7. `PathSuggestion`

Source: `Models/PathSuggestion.cs:1-16`.

| Field | C# type | Default | Notes |
| ----- | ------- | ------- | ----- |
| `FullPath` | `string` | `string.Empty` | The path. |
| `DisplayText` | `string` | `string.Empty` | Truncated/formatted display variant. |

The constructor requires both fields. `ToString()` returns `DisplayText`.

**Status**: Pending (Phase 4).

**Proposed Rust shape** (will land in a Qt-facing model, not a wire type):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSuggestion {
    pub full_path: PathBuf,
    pub display_text: String,
}
```

`PathSuggestion` exists only to feed the path autocomplete dropdown. Phase 4
will own it inside the GUI controller (the recent-paths store already provides
`Vec<PathBuf>`; `PathSuggestion` is the projected presentation row).

## 8. `RecentSearch`

Source: `Models/RecentSearch.cs:1-58`.

**Status**: Ported. See `crates/grexa-core/src/storage.rs:226-278` and the
existing `docs/grex-storage-services-audit.md` §RecentSearch Schema for the
full field-by-field table, including the byte-exact `GetKey()` /
`RecentSearch::key()` compatibility test.

Highlights captured already in the storage audit:

- `Timestamp` (`DateTime.Now`) → `timestamp_unix: u64`.
- `ResultCount` → `result_count: usize`.
- `IncludeSubfolders` defaults to `true` on both sides.
- The dedupe key uses `True`/`False` casing from `Boolean.ToString()` in C#.

UI helpers `DisplayText` and `SecondaryText` are not ported; the Qt view-model
re-derives them.

## 9. `SearchProfile`

Source: `Models/SearchProfile.cs:1-48`.

**Status**: Ported. See `crates/grexa-core/src/storage.rs:284-304` and
`docs/grex-storage-services-audit.md` §SearchProfile for the table. Notable
shape differences:

- Grexa pulls all the per-search options into an embedded
  `SearchOptions` value rather than flattening them onto `SearchProfile`.
- `CreatedAt` / `UpdatedAt` (`DateTime.Now`) → `created_unix` / `updated_unix`.
- `UseWindowsSearchIndex` is mapped to `SearchOptions.use_file_index`
  (Linux-specific replacement; see `linux-decisions.md` §Windows Search
  Index).
- `SecondaryText` is a UI helper that does not exist on the Rust side.

## 10. `SearchResult`

Source: `Models/SearchResult.cs:1-63`.

| Field | C# type | Default | Grexa equivalent | Notes |
| ----- | ------- | ------- | ----------------- | ----- |
| `FileName` | `string` | `string.Empty` | `file_name: String` | Same. |
| `LineNumber` | `int` | `0` | `line_number: usize` | Same. |
| `ColumnNumber` | `int` | `0` | `column_number: usize` | Same. |
| `LineContent` | `string` | `string.Empty` | `line_content: String` | Same. |
| `MatchPreviewBefore` | `string` | `string.Empty` | `match_preview_before: String` | Same. |
| `MatchPreviewMatch` | `string` | `string.Empty` | `match_preview_match: String` | Same. |
| `MatchPreviewAfter` | `string` | `string.Empty` | `match_preview_after: String` | Same. |
| `FullPath` | `string` | `string.Empty` | `full_path: PathBuf` | Typed path. |
| `RelativePath` | `string` | `string.Empty` | `relative_path: PathBuf` | Typed path. |
| `MatchCount` | `int` | `1` | `match_count: usize` | Default `1` preserved. |
| `DirectoryPath` | computed `string` | n/a | _(none)_ | GUI helper. |
| `TrimmedLineContent` | computed `string` | n/a | _(none)_ | GUI helper. |

**Status**: Ported. See `crates/grexa-core/src/models.rs:107-119`.

## 11. `SearchResultSortField`

Source: `Models/SearchResultSortField.cs:1-14`.

| Grex variant | Numeric | Grexa variant | Notes |
| ------------ | ------- | ------------- | ----- |
| `None` | `0` | `None` | Same. |
| `FileName` | `1` | `Name` | Renamed for brevity. |
| `LineNumber` | `2` | `Line` | Renamed. |
| `ColumnNumber` | `3` | `Column` | Renamed. |
| `RelativePath` | `4` | `Path` | Renamed. |
| `Extension` | `5` | `Extension` | Same. |
| `Encoding` | `6` | `Encoding` | Same. |
| `MatchCount` | `7` | `Matches` | Renamed. |

**Status**: Renamed. See `crates/grexa-core/src/models.rs:37-47`.

**Import note (Phase 10)**: settings backups serialize `SearchResultSortField`
only inside column-visibility state, not as a top-level field — so the
variant rename does not require an importer translation table.

## 12. `SizeLimitType`

Source: `Models/SizeLimitType.cs:1-10`.

| Grex variant | Numeric | Grexa variant | Notes |
| ------------ | ------- | ------------- | ----- |
| `NoLimit` | `0` | `NoLimit` | Same. |
| `LessThan` | `1` | `LessThan` | Same. |
| `EqualTo` | `2` | `EqualTo` | Same. |
| `GreaterThan` | `3` | `GreaterThan` | Same. |

**Status**: Ported. See `crates/grexa-core/src/models.rs:6-12`.

## 13. `SizeUnit`

Source: `Models/SizeUnit.cs:1-9`.

| Grex variant | Numeric | Grexa variant | Notes |
| ------------ | ------- | ------------- | ----- |
| `KB` | `0` | `KB` | Same. |
| `MB` | `1` | `MB` | Same. |
| `GB` | `2` | `GB` | Same. |

**Status**: Ported. See `crates/grexa-core/src/models.rs:14-19`.

## 14. `StringComparisonMode`

Source: `Models/StringComparisonMode.cs:1-9`.

| Grex variant | Numeric | Grexa variant | Notes |
| ------------ | ------- | ------------- | ----- |
| `Ordinal` | `0` | `Ordinal` | Same. |
| `CurrentCulture` | `1` | `CurrentCulture` | Same (semantics resolved via ICU in Phase 2). |
| `InvariantCulture` | `2` | `InvariantCulture` | Same. |

**Status**: Ported. See `crates/grexa-core/src/models.rs:21-26`.

Cross-reference: `docs/grex-culture-comparison-audit.md` and Phase 2
linux-decisions row covering ICU semantics.

## 15. `UnicodeNormalizationMode`

Source: `Models/UnicodeNormalizationMode.cs:1-28`.

| Grex variant | Numeric | Grexa variant | Notes |
| ------------ | ------- | ------------- | ----- |
| `None` | `0` | `None` | Same. |
| `FormC` | `1` | `FormC` | Same. |
| `FormD` | `2` | `FormD` | Same. |
| `FormKC` | `3` | `FormKC` | Same. |
| `FormKD` | `4` | `FormKD` | Same. |

**Status**: Ported. See `crates/grexa-core/src/models.rs:28-35`.

The companion `UnicodeNormalizationExtensions.ToNormalizationForm` extension
is .NET-specific and replaced by ICU calls inside the Rust comparator
(Phase 2).

## Cross-Cutting: SearchSummary

`SearchSummary` is not a class under `Models/`; it is materialized by the
search engine but it bundles the model types above and merits inclusion here
because the audit prompt called it out explicitly.

Grex evidence:

- `Services/SearchService.cs` returns `List<SearchResult>` and emits ad-hoc
  `(int filesScanned, int filesMatched, int matches, int skipped,
  TimeSpan elapsed, bool cancelled)` tuples through its
  `ProgressEvent` callbacks (see `docs/grex-search-service-audit.md`
  §Progress Reporting).

Grexa equivalent: `grexa_core::models::SearchSummary` (lines 138-149 of
`crates/grexa-core/src/models.rs`):

| Grexa field | Type | Notes |
| ----------- | ---- | ----- |
| `results` | `Vec<SearchResult>` | Content-mode results. |
| `file_results` | `Vec<FileSearchResult>` | Files-mode results. |
| `files_scanned` | `usize` | Same accounting as Grex. |
| `files_matched` | `usize` | Same. |
| `matches` | `usize` | Same. |
| `skipped_files` | `usize` | Same. |
| `elapsed_ms` | `u128` | Replaces `TimeSpan`. Accessor `elapsed()` returns `Duration`. |
| `cancelled` | `bool` | Defaults to `false` via `#[serde(default)]`. |

**Status**: Ported (struct exists, populated by the Phase 3 search engine).

## Cross-Cutting: SearchOptions

`SearchOptions` is referenced by `SearchProfile`. It is not a separate Grex
model file — Grex flattens it onto `SearchProfile` — but Grexa folds the
options into a reusable struct.

Grexa shape (`crates/grexa-core/src/models.rs:56-78`):

| Field | Type | Default (`SearchOptions::new`) |
| ----- | ---- | -------------------------------- |
| `path` | `PathBuf` | argument |
| `search_term` | `String` | argument |
| `regex` | `bool` | `false` |
| `case_sensitive` | `bool` | `false` |
| `respect_gitignore` | `bool` | `false` |
| `include_hidden` | `bool` | `false` |
| `include_binary` | `bool` | `false` |
| `include_system` | `bool` | `false` |
| `include_subfolders` | `bool` | `true` |
| `include_symlinks` | `bool` | `false` |
| `match_file_names` | `String` | `""` |
| `exclude_dirs` | `String` | `""` |
| `size_limit_type` | `SizeLimitType` | `NoLimit` |
| `size_limit_kb` | `Option<u64>` | `None` |
| `size_unit` | `SizeUnit` | `KB` |
| `string_comparison_mode` | `StringComparisonMode` | `Ordinal` |
| `unicode_normalization_mode` | `UnicodeNormalizationMode` | `None` |
| `diacritic_sensitive` | `bool` | `true` |
| `culture` | `Option<String>` | `None` |
| `use_file_index` | `bool` | `false` (Linux-specific replacement for Grex `UseWindowsSearchIndex`) |

**Status**: Ported. The Phase 3 search engine consumes `SearchOptions`
end-to-end; the importer in Phase 10 builds them from imported
`SearchProfile` JSON.

## Linux-Specific Replacements

The following renamings carry semantic changes rather than mechanical port:

| Grex field | Grexa field | Where | Justification |
| ---------- | ----------- | ----- | ------------- |
| `SearchProfile.UseWindowsSearchIndex` | `SearchOptions.use_file_index` | `SearchOptions` | `linux-decisions.md` §Windows Search Index. Backed by optional Baloo candidate seeding (Phase 13). |
| `DockerContainerInfo` (Docker-only) | `ContainerInfo` w/ `runtime: ContainerRuntimeKind` | `grexa-containers` | Phase 7 ships Docker + rootless Podman + rootful Podman. |
| `DockerMirrorInfo` (Docker-only) | `ContainerMirrorInfo` w/ `runtime` | `grexa-containers` | Same as above. |
| `RecentSearch.Timestamp` / `SearchProfile.CreatedAt`/`UpdatedAt` (`DateTime`) | `*_unix: u64` | `grexa-core::storage` | Localized `DateTime.Now` cannot round-trip across JSON without a TZ stamp; Unix seconds are explicit. |
| `FileSearchResult.DateModified` (`DateTime`) | `date_modified_unix: Option<u64>` | `grexa-core::models` | Same reasoning; `None` replaces `default(DateTime)`. |

## Non-Applicable Models

Compared with `docs/grex-audit-inventory.md`, this map adds one
non-applicable model that the inventory does not call out:

- `DockerContainerOption` — Windows ComboBox adapter; no Rust analog. The
  audit inventory lists the file but does not classify it as
  non-applicable. Phase 4 supplies a QML list model that prepends the
  "Local filesystem" sentinel; no wire type is needed.

`AiChatMessage`'s `Speaker` / `Timestamp` / `TimestampText` are GUI-only
helpers but the class as a whole is renamed, not dropped, so it is not listed
here.

## Pending Models

| Grex model | Reason | Phase |
| ---------- | ------ | ----- |
| `PathSuggestion` | GUI autocomplete view-model. Recent paths already live in `grexa_core::storage::RecentPathStore`; the suggestion projection belongs to the Qt controller. | Phase 4 |

No model classes from `Models/` are pending in `grexa-core` or
`grexa-containers` after Phase 2 / Phase 3 / Phase 5 / Phase 7 work that has
already landed.

## Verification Checklist

- [x] Every file in `Grex/Models/` (15 files) appears in the summary table.
- [x] Every Ported entry links to a Rust definition under
      `crates/grexa-core/`, `crates/grexa-containers/`, or `crates/grexa-ai/`.
- [x] Every Renamed entry names both sides of the rename.
- [x] Every Pending entry proposes a Rust shape and a Phase.
- [x] `SearchResultSortField`, `ResultsListItem`, `SearchSummary`,
      `ContextPreviewResult`, and the container models each have their own
      dedicated section.
- [x] Storage-audited models (`RecentSearch`, `SearchProfile`,
      `DefaultSettings`) are referenced rather than re-tabulated.

**Note on `ResultsListItem`**: the prompt named `ResultsListItem` as a
focus model. No `Models/ResultsListItem.cs` exists in Grex —
`Controls/ResultsTemplateSelector.cs` selects between `SearchResult` and
`FileSearchResult` as the displayed row type. Both row types are covered
above (Sections 6 and 10). If a future Grex revision introduces a unified
`ResultsListItem`, it would map to a Grexa-side `enum ResultsRow { Content(SearchResult), Files(FileSearchResult) }`,
which Phase 4 would own.
