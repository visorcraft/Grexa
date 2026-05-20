# Result-Set Memory Budgets

PLAN.md phase 15 line 449 requires a memory budget for million-result
scans. This doc records the budgets the core types are designed for, and
the back-pressure mechanisms the GUI uses to keep the workspace usable
when a search produces more results than the budget allows.

## Per-row cost (current `SearchResult`)

The struct is in `crates/grexa-core/src/models.rs`. A typical row holds:

| Field                       | Type      | Typical bytes |
| --------------------------- | --------- | ------------- |
| `file_name`                 | `String`  | 20            |
| `line_number`               | `usize`   | 8             |
| `column_number`             | `usize`   | 8             |
| `line_content`              | `String`  | 80 (truncated to 400 chars) |
| `match_preview_before`      | `String`  | 30 (truncated to 120 chars) |
| `match_preview_match`       | `String`  | 12            |
| `match_preview_after`       | `String`  | 30 (truncated to 120 chars) |
| `full_path`                 | `PathBuf` | 64            |
| `relative_path`             | `PathBuf` | 24            |
| `match_count`               | `usize`   | 8             |
| **Total per row (~average)**| —         | **≈ 280 bytes** |

`SearchResult` heap allocations are dominated by the four `String` /
`PathBuf` fields; the `line_content` cap at 400 chars means worst case
is ~1.6 kB (UTF-8 bytes) per row.

## Budgets

| Scenario           | Row count | Memory estimate | Notes |
| ------------------ | --------- | --------------- | ----- |
| Average dev tree   | 1k        | ~300 kB         | comfortably under any sensible cap |
| Large monorepo     | 100k      | ~30 MB          | acceptable; matches Grex's WPF heap behavior |
| Pathological scan  | 1M        | ~300 MB         | hard ceiling for v1.0 |
| Above 1M           | —         | —               | the GUI surfaces a "truncated, refine your search" status; CLI keeps streaming |

The numbers are deliberate over-estimates. Real-world dev trees scanned
during development land at 50–80 bytes per `SearchResult` once short
file names + short snippets are factored in.

## Back-pressure

The Rust core never imposes a row cap — `search_with` keeps emitting
into `SearchSummary.results` regardless of size. The constraints live
in two layers above:

1. **`ProgressEvent::Match`** lets the GUI batch row inserts. The
   recommended pattern (used by the Phase 4 spike) is to coalesce
   matches into 256-row batches before flushing to the QML table model.
2. **Bounded mpsc channel** between the worker thread and the GUI
   controller. The default channel depth is 1024 batches; when the
   channel fills, the worker `send_blocking` stalls naturally. The
   walker continues; only the GUI rendering blocks.

## File aggregation (`FileSearchResult`)

Files-mode rows aggregate every `SearchResult` for one file. A file
with N matches keeps every preview row plus a head/best summary; budget
roughly `N × 280 bytes + 200 bytes (header)`. The aggregator runs in
`search.rs::aggregate_file_results` after the main loop completes, so
the peak memory cost is the union of `results` + `file_results` —
double the per-row estimate above when the user has selected Files mode.

## Result-cap policy

The Rust core never silently truncates. The GUI is responsible for:

- showing a "showing N of M" banner when filtering kicks in,
- offering a "Cancel search" affordance when memory pressure crosses
  a configurable threshold (default 256 MB),
- writing structured `tracing::warn!` events when `results.len()`
  exceeds 500k so an operator can correlate slowness with the search
  scope after the fact.

## Open follow-ups

- Memory-bounded streaming for the CLI: today the CLI buffers the full
  `SearchSummary` before printing. For `--quiet` and `--files-only`
  this is fine; for large `--format text` runs we should switch to
  emit-on-match via the `ProgressSink` and free `results` immediately
  after rendering each row. Tracked in Phase 15 line 446.
- Profile-guided shrinking: when 95% of `SearchResult` rows in a real
  workload show `line_content.len() < 80`, swap `String` for
  `SmolStr` / `CompactString` to reclaim the per-row pointer overhead.
  Defer until benchmarks justify the dep.
