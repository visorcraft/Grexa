// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Shared constants used across the Grexa search/replace engine.

/// Hard upper bound on the size of a single file the search/replace engine
/// will read into memory. The user-facing size filter defaults to "no limit",
/// so without this safety net a single multi-gigabyte file could exhaust
/// memory (decoding/normalization roughly doubles the footprint). Files above
/// the cap are rejected rather than scanned or rewritten.
pub const MAX_SEARCH_FILE_BYTES: u64 = 512 * 1024 * 1024;

/// Internal ceiling on the number of individual match rows returned by a
/// single search when the caller does not supply a `max_results` value.
/// Keeps memory usage and downstream GUI rendering bounded on broad queries.
pub const ENGINE_RESULT_CAP: usize = 1_000_000;

/// `true` when a file of `len` bytes exceeds the hard in-memory read cap. The
/// cap itself is allowed; only strictly larger files are rejected.
pub fn file_exceeds_hard_cap(len: u64) -> bool {
    len > MAX_SEARCH_FILE_BYTES
}
