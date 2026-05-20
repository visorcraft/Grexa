# Grex AI Search Service Audit

This document records Grex `Services/AiSearchService.cs` behavior for endpoint
normalization, model discovery, request construction, response parsing, error
extraction, and Search tab integration.

Source evidence:

- `Services/AiSearchService.cs`
- `Controls/SearchTabContent.xaml.cs`
- `Controls/SettingsView.xaml.cs`
- `Services/SettingsService.cs`
- `Tests/Services/AiSearchServiceTests.cs`
- `Tests/Controls/SettingsViewAiEndpointHelpersTests.cs`
- `IntegrationTests/AiSearchSettingsIntegrationTests.cs`
- `IntegrationTests/AiSearchLocalizationIntegrationTests.cs`
- `docs/architecture.md`
- `docs/features.md`
- `docs/reference.md`
- `docs/usage.md`
- `crates/grexa-ai/src/lib.rs`

## Public Types

`AiSearchContext` fields:

- `SearchPath`
- `SearchQuery`
- `FilterSuggestions`
- `IsRegexSearch`
- `IsFilesSearch`

`AiConversationTurn` fields:

- `Role`, default `user`
- `Content`

`AiSearchResponse` fields:

- `Success`
- `Message`
- `ErrorMessage`

`AiSearchService`:

- default model: `gpt-4o-mini`
- default `HttpClient.Timeout`: 90 seconds
- optional `HttpClient` injection for tests
- JSON serializer uses camelCase property naming
- caches one resolved endpoint base and one resolved model per service instance

Grexa replacement:

- Keep equivalent DTOs in `grexa-ai`.
- Prefer a typed role enum at public boundaries, but preserve import/runtime
  tolerance for string roles from Grex semantics.
- Keep the AI service testable with injectable HTTP transport.

## SendDiscussionTurnAsync

Signature:

```csharp
Task<AiSearchResponse> SendDiscussionTurnAsync(
    string endpoint,
    string? apiKey,
    string? preferredModel,
    AiSearchContext context,
    IReadOnlyList<AiConversationTurn> conversation,
    CancellationToken cancellationToken = default)
```

Validation:

- blank endpoint returns `Success=false` and
  `AI endpoint is not configured.`
- null context throws `ArgumentNullException`
- null conversation throws `ArgumentNullException`

Model selection:

- nonblank `preferredModel` is trimmed and used directly
- blank preferred model triggers `/v1/models` discovery
- discovery fallback is `gpt-4o-mini`

Request:

- method: `POST`
- URL: chat completions endpoint derived from endpoint
- content type: `application/json; charset=utf-8`
- optional authorization: `Bearer <trimmed apiKey>`
- payload:
  - `model`
  - `temperature = 0.2`
  - `messages`

Success behavior:

- parses assistant message
- blank/empty assistant message returns `Success=false` and
  `AI endpoint returned an empty response.`
- nonblank assistant message is trimmed and returned as `Message`

Error behavior:

- non-success HTTP returns `Success=false` with extracted error message
- `OperationCanceledException` is rethrown
- all other exceptions are logged to `%Temp%\Grex.log` and returned as
  `ErrorMessage = ex.Message`

Grexa replacement:

- Preserve cancellation as a real cancellation, not a synthetic AI error.
- Preserve trimmed API key in Authorization only; do not log it.
- Add request timeout/cancellation tests.
- Route logs to `$XDG_STATE_HOME/grexa` if logging is needed.

## Endpoint Normalization

`NormalizeEndpointBase(endpoint)`:

- trims whitespace
- if the value does not start with `http://` or `https://`, prefixes `https://`
- removes trailing slashes
- does not remove `/v1`
- does not remove `/chat/completions`
- does not remove `/models`

`BuildChatCompletionsEndpoint(endpoint)`:

- normalizes the endpoint first
- if normalized endpoint ends with `/chat/completions`, returns it unchanged
- if normalized endpoint ends with `/v1`, appends `/chat/completions`
- otherwise appends `/v1/chat/completions`

`BuildModelsEndpoint(endpoint)`:

- normalizes the endpoint first
- if normalized endpoint ends with `/models`, returns it unchanged
- if normalized endpoint ends with `/v1`, appends `/models`
- otherwise appends `/v1/models`

Examples:

- `api.example.test/v1/` -> `https://api.example.test/v1/chat/completions`
- `api.example.test` -> `https://api.example.test/v1/chat/completions`
- `https://api.example.test/v1` -> `https://api.example.test/v1/models`
- `https://api.example.test/models` -> `https://api.example.test/models`

Important edge:

- an endpoint saved as `/v1/chat/completions` is accepted for chat calls, but
  model discovery would build `/v1/chat/completions/v1/models`.
- this matters only when model is blank, because nonblank preferred model skips
  discovery.

Current Grexa state:

- `crates/grexa-ai/src/lib.rs` has endpoint helpers that strip
  `/chat/completions` and trim `/v1` before rebuilding URLs.
- That behavior is cleaner for shared helpers but differs from Grex's exact
  `BuildModelsEndpoint` edge cases.

Grexa replacement:

- Consolidate endpoint normalization in one helper used by AI chat and Settings
  endpoint test.
- Prefer the Grexa helper shape that canonicalizes to a base URL, then builds
  `/v1/chat/completions` and `/v1/models`.
- Add compatibility tests for Grex endpoint input shapes.

## Model Discovery

`ResolveModelAsync`:

- normalizes endpoint base
- if normalized base differs from `_resolvedEndpointBase`
  case-insensitively:
  - sets `_resolvedEndpointBase`
  - clears `_resolvedModel`
- returns cached `_resolvedModel` when nonblank
- sends `GET` to models endpoint
- includes Authorization when API key is nonblank
- on non-success HTTP, caches and returns `gpt-4o-mini`
- on success, parses JSON `data` array
- returns the first nonblank string `id`
- trims the selected id
- on parse/network error, logs and caches `gpt-4o-mini`

Cache characteristics:

- cache is per `AiSearchService` instance
- cache key is endpoint base only
- API key changes do not invalidate the cached model
- a discovery failure caches the default model and prevents retry for the same
  endpoint until the service instance or endpoint changes
- providing a preferred model skips discovery and does not update the cache

Grexa replacement:

- Decide whether model discovery failures should be cached.
- If caching, key by endpoint plus authentication identity or expose a retry path
  after Settings changes.
- Add tests for blank model, endpoint change, API key change, empty data array,
  invalid JSON, and non-success discovery responses.

## Message Construction

`BuildMessages` emits two system messages before conversation turns.

First system message:

```text
You are Grex AI Search. Help the user locate relevant files and code using the provided path, query, and filter suggestions. Ask concise follow-up questions when needed.
```

Second system message is the context prompt.

Conversation handling:

- roles are normalized:
  - `assistant` -> `assistant`
  - `system` -> `system`
  - everything else -> `user`
- content is trimmed
- blank content turns are skipped
- all retained turns are appended after the two system messages

`BuildContextPrompt` includes:

- `AI search context:`
- search path
- search query
- search type: `Regex` or `Text`
- result mode: `Files` or `Content lines`
- `Filter suggestions:`
- one `- ...` line per nonblank suggestion, or `- No additional filters`
- final instruction:
  `Treat filters as suggestions and explain reasoning with concrete next steps.`

Search tab context suggestions currently include:

- respect `.gitignore`
- case-sensitive
- include subfolders
- include hidden items
- include binary files
- include symbolic links
- use Windows Search index
- match files, when present
- exclude dirs, when present
- size limit, when enabled

Grexa replacement:

- Replace Windows Search wording with Linux file-index wording.
- Include container target/runtime/path, hidden/symlink/mount context, and Linux
  pseudo-filesystem hints where relevant.
- Add explicit privacy/opt-in UI before sending path, query, filters, or result
  context to a remote endpoint.
- Consider showing the exact context prompt before the first request.

## Response Parsing

`ExtractAssistantMessage` supports:

- `choices[0].message.content`
- `choices[0].text`
- root `output_text`

`message.content` parsing:

- string content returns the string
- array content concatenates:
  - string items
  - object `text` string values
  - object `content` string values
- unsupported shapes return empty

Limitations:

- only the first choice is used
- no streaming response support
- no tool call/function call parsing
- no root `output` array parsing from newer Responses-style payloads
- parse errors are logged and treated as empty response

Grexa replacement:

- Preserve current OpenAI-compatible chat-completions parsing.
- Add streaming support only if the UI implements backpressure and cancellation
  cleanly.
- Add tests for string content, array content, legacy `text`, `output_text`,
  empty choices, malformed JSON, and modern Responses payloads if supported.

## Error Extraction

`ExtractErrorMessage(responseJson, fallbackReason)`:

- tries to parse JSON when body is nonblank
- supports OpenAI-style `{ "error": { "message": "..." } }`
- supports `{ "error": "..." }`
- trims nonblank extracted messages
- ignores parse failures
- falls back to nonblank HTTP reason phrase
- final fallback is `AI request failed.`

Settings endpoint test has a separate helper with the same payload parsing but
uses final fallback `Request failed.`

Grexa replacement:

- Use one shared error extraction helper for chat and endpoint test paths.
- Include HTTP status code in user-facing errors when helpful.
- Avoid logging request bodies or API keys.

## Search Tab Integration

AI chat flow:

1. AI button starts a new conversation when no request is in flight.
2. AI button cancels the active request when a request is in flight.
3. endpoint must be configured.
4. search path and search query must be nonblank.
5. filter pane is collapsed.
6. AI mode hides normal results and shows the chat panel.
7. new conversations clear existing chat messages/history.
8. the initial user message is the current search query.
9. follow-up messages are taken from the chat input box.
10. user turns are appended to UI and `_aiConversationHistory`.
11. service receives endpoint, API key, model, context, full conversation, and
    cancellation token.
12. successful assistant messages are appended to UI and history.
13. failed requests append localized `AI request failed: {0}` as assistant text.
14. cancellation appends localized `AI request cancelled.`

Current privacy behavior:

- Grex sends local path, query, mode, result mode, and active filter
  suggestions once the user clicks AI.
- There is no separate context preview or consent confirmation.

Grexa replacement:

- Keep follow-up conversation behavior.
- Add explicit first-use opt-in and context disclosure.
- Keep cancellation visible and responsive.

## Settings Integration

Defaults:

- endpoint: `https://api.openai.com/v1`
- API key: empty
- model: `gpt-4o-mini`

Persistence:

- endpoint is trimmed
- API key preserves exact user input
- model is trimmed
- settings export/import currently includes API key

Behavioral implication:

- because the default model is nonblank, default Grex AI chat does not run model
  discovery. Users must blank the model setting to auto-detect from `/v1/models`.

Grexa replacement:

- Store endpoint and optional model in normal settings.
- Store API key in a secret store or mark export-with-secrets as explicit.
- Make auto-detect behavior clear in Settings.

## Test Coverage

Existing tests cover:

- preferred model trims and skips model discovery
- endpoint without scheme and with `/v1/` normalizes for chat
- Authorization header trims API key
- request payload model and messages
- context prompt includes path, query, and filter suggestions
- blank model triggers discovery
- discovered model is cached for repeated calls to the same endpoint
- discovery failure falls back to `gpt-4o-mini`
- blank endpoint returns validation error and skips HTTP
- API error extracts nested `error.message`
- AI settings defaults, persistence, and export/import
- AI localization keys for Settings endpoint test
- Settings endpoint helper URL normalization and error extraction

Current test gaps:

- no direct tests for `choices[0].text`, `output_text`, or array content
- no tests for malformed success JSON or empty choices
- no tests for root string `error`
- no tests for model cache invalidation on API key change
- no tests for discovery invalid JSON or empty `data`
- no cancellation test
- no null context/conversation tests
- no privacy/consent test

## Current Grexa Status

`crates/grexa-ai` now implements OpenAI-compatible chat requests, model
discovery, endpoint normalization, response/error parsing, Linux-aware context
prompting, and Secret-Service-backed API key helpers. The GUI gates AI calls on
the explicit `ai_search_enabled` setting and stores keys per endpoint.

Remaining gaps:

- decide whether model discovery needs caching and test invalidation behavior
- add cancellation coverage for in-flight AI requests
- add malformed-response and null-context tests beyond the current parser cases
- add first-use privacy/context disclosure UI if required by policy
