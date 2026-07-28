<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Result and Resource Budgets

Grexa is synchronous but bounded. This document explains the limits that keep
large searches, tabs, documents, containers, regexes, replace operations, and
AI summaries from growing without control.

Constants in the named implementation files are authoritative.

## Search-engine limits

| Resource | Limit | Source |
| -------- | ----- | ------ |
| Search term | 4,096 Unicode scalar values | CLI validation |
| File size | 512 MiB maximum accepted setting | `grexa-core` search options |
| Result rows without a smaller `max_results` | 1,000,000 | `DEFAULT_MAX_RESULTS` |
| Matches retained from one line | 10,000 | pattern matcher |
| Line bytes used for culture/normalization comparison | 2 MiB | `MAX_COMPARE_LINE_BYTES` |
| Recursion depth | 64 | filesystem walker |
| Extended-regex time for one line | 100 ms | pattern engine |
| Extended-regex backtracks | 100,000 | pattern engine |

When the row limit is reached, `SearchSummary.capped` is true. The GUI and CLI
report a partial/capped result rather than silently claiming a complete scan.
`max_results` can lower the ceiling but cannot raise it above the hard cap.

Cancellation is checked during traversal and per-line matching. A cancelled
summary keeps rows already found and sets `SearchSummary.cancelled`.

## What a result row owns

A content result owns:

- file name;
- full and relative paths;
- line and column numbers;
- line content;
- preview text before, inside, and after the match;
- match count and encoding label.

Text fields and paths are heap allocations, so there is no honest constant
per-row byte estimate. Long UTF-8 paths and lines cost more than short ASCII
ones. The engine bounds preview material, but a large result set can still
consume substantial memory.

Files mode aggregates content rows and retains preview matches for each file.
Peak engine memory can therefore include both the content results and their
file aggregation. Choose a lower `--max-results` when automation does not need
every row.

## GUI delivery and tab budgets

The worker thread never mutates QML objects directly. It sends progress back
through cxx-qt's queued GUI-thread path.

| Resource | Limit or policy |
| -------- | --------------- |
| Pending row batch | At most 4,096 rows |
| Row flush cadence | Coalesced to roughly one update per 16 ms |
| Counter update cadence | At most every 250 ms |
| Active tab rows | Same 1,000,000 engine ceiling |
| Open tabs | 8 |
| Inactive tab snapshot rows | 2,000,000 total |

The active tab owns the live Qt model. Inactive tabs store Rust snapshots.
Snapshot eviction keeps the aggregate inactive-row budget bounded. Restoring a
tab moves rows back into the active model instead of keeping a second active
copy.

The within-filter rebuilds the visible projection and, in files mode, one
dedupe set. An invalid within regex matches nothing.

## CLI memory behavior

The CLI receives a complete `SearchSummary` before rendering text, JSON, or
CSV. It is not an unbounded streaming printer. `--count`, `--files-only`, and
`--quiet` change output, not the engine's collection model.

For broad roots, use:

```bash
grexa-cli /path term --max-results 100000
```

Use narrower roots, file globs, excluded directories, or a lower size limit
when a full million-row summary is unnecessary.

## Document extraction

| Resource | Limit |
| -------- | ----- |
| ZIP/container entries inspected | 1,024 |
| Uncompressed bytes per entry | 4 MiB |
| Extracted text per document | 16 MiB |
| PDF extraction process | 30 seconds |

These limits apply to searchable documents such as Office, OpenDocument, ZIP,
and PDF files. Truncation or extraction failure is reported without treating
container bytes as normal source text.

## Replace

| Resource | Limit |
| -------- | ----- |
| Files collected for one replace | 100,000 |
| Matches in one file | 1,000,000 |
| Extended-regex work in one file | 5 seconds |

Replace uses the options captured by the preceding search. It skips searchable
binary container formats and writes ordinary files atomically. The limits
prevent a broad or pathological replacement from collecting unlimited work.

## Containers

| Resource | Limit |
| -------- | ----- |
| Runtime command | 30 seconds |
| Captured stdout | 10 MiB per command |
| Captured stderr | 10 MiB per command |

Direct grep stops at the requested result limit. Mirror fallback copies only
the selected container path into a temporary cache directory and then applies
the normal engine bounds. Mirror cleanup is best-effort and constrained to the
Grexa cache subtree.

## AI evidence

AI chat sends one conversation turn plus the current search context. Search
summaries inspect at most 400 visible rows and fit evidence into the configured
`ai_summary_max_chars` range of 2,000 to 40,000 characters. The prompt says
when rows or evidence were omitted.

Provider responses are capped at 4 MiB and requests time out after 90 seconds.
Only one GUI AI request may be in flight.

## Operational guidance

- Start with the default cap for interactive work.
- Lower `max_results` for scripts and intentionally broad roots.
- Use files mode only when aggregation is useful.
- Close inactive tabs that no longer matter.
- Cancel a runaway search. Partial rows remain available.
- Treat `capped` and `cancelled` as incomplete-result states in automation.
- Measure representative paths before changing a constant. Do not infer memory
  use from row count alone.
