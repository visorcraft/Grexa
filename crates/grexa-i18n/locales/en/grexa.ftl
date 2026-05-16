# Canonical English translation catalog for Grexa.
#
# This file is the source of truth: every key here must appear in every
# other locale. The CI script `scripts/check_locale_sync.py` enforces that.
#
# Key naming:
#   <area>-<feature>-<detail>
# e.g. `search-status-running`, `replace-confirm-warning`.

## App chrome
app-name = Grexa
app-tagline = Fast Linux file content search

## Search status — wired to TabViewModel::StatusText.
##
## ICU MessageFormat plural selectors keep the right grammatical agreement
## for non-English locales. The Rust runtime reads these via
## `fluent::FluentBundle::format_pattern`.
search-status-ready = Ready
search-status-running = Searching…
search-status-cancelled = Cancelled
search-status-error = Error: {$message}

search-status-found = {$matches ->
    [one] Found 1 match
   *[other] Found {$matches} matches
} in {$files ->
    [one] 1 file
   *[other] {$files} files
} in {$elapsed}

search-status-filtered = Showing {$shown} of {$total} matches in {$files} files

## Elapsed-time formatting helpers. Used by both search and replace.
elapsed-subsecond = under a second
elapsed-seconds = {$seconds ->
    [one] 1 second
   *[other] {$seconds} seconds
}
elapsed-minutes-only = {$minutes ->
    [one] 1 minute
   *[other] {$minutes} minutes
}
elapsed-minutes-and-seconds = {$minutes ->
    [one] 1 minute
   *[other] {$minutes} minutes
} {$seconds ->
    [one] and 1 second
   *[other] and {$seconds} seconds
}

## Replace flow.
replace-status-running = Replacing…
replace-status-completed = {$matches ->
    [one] Replaced 1 match
   *[other] Replaced {$matches} matches
} in {$files ->
    [one] 1 file
   *[other] {$files} files
} in {$elapsed}
replace-confirm-title = Confirm replace
replace-confirm-message = Replace { $matches } matches in { $files } files? This cannot be undone.

## Container search.
container-target-local = Local files
container-target-docker = Docker containers
container-target-podman = Podman containers
container-mirror-fallback-badge = (mirrored)

## AI search opt-in.
ai-disabled-banner = AI search is off. Enable it in Settings → AI Search.
ai-empty-state = Click AI to start an AI-assisted search discussion.
ai-error-not-configured = AI endpoint is not configured.
ai-error-empty-response = AI endpoint returned an empty response.

## File manager / editor actions.
action-open-in-editor = Open in editor
action-reveal-in-file-manager = Show in file manager
action-copy-path = Copy path
