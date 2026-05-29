# AI Provider Scope

This note pins what kinds of AI endpoints Grexa supports in v1.0, and what
falls outside the scope on purpose.

PLAN.md phase 8 lines 333-348 (and `docs/grex-ai-search-service-audit.md`)
reference this doc.

## In scope

Grexa speaks only the **OpenAI-compatible HTTP shape**: a `POST /v1/chat/completions`
endpoint that takes `{model, temperature, messages: [{role, content}]}` and
returns the OpenAI-style `{choices: [{message: {content}}]}` envelope. We
also call `GET /v1/models` for endpoint sanity tests and for the
"auto-discover model" path.

Servers that ship this shape, validated against
`crates/grexa-ai/src/lib.rs::AiSearchClient`:

- `api.openai.com` (the namesake)
- `api.anthropic.com` via OpenAI-shim proxies (LiteLLM, Anthropic's own
  OpenAI-compatible endpoints, when present)
- Local OpenAI-compatible servers: `ollama serve` (after enabling the
  `/v1` proxy), `vLLM` with the OpenAI server, `LM Studio`,
  `text-generation-webui` with the `--openai` flag, `LocalAI`
- Azure OpenAI when fronted by an API gateway that strips the
  `api-key:` / `api-version=` query-string requirements
- Self-hosted aggregators: LiteLLM, BerriAI, OneAPI, Helicone

## Out of scope

Grexa does **not** implement provider-native wire formats:

- **Anthropic Messages API** (`/v1/messages` with `tool_use` and
  `content: [{type: text}]` blocks) — proxy through LiteLLM if you need it.
- **Google Gemini** native JSON shape.
- **Cohere `/v1/chat`** native shape.
- **xAI Grok** native API.
- **AWS Bedrock**, **Azure OpenAI without an OpenAI-shim gateway**.

If a server speaks a different shape, run an OpenAI-compatible proxy
between it and Grexa. The Settings → AI Search panel has a "Test endpoint"
button that calls `GET /v1/models`; if that responds with an OpenAI-style
`{data: [{id, ...}]}` array, Grexa can talk to it.

## Feature flag

The full chat client lives behind the implicit `ai` Cargo path in
`grexa-ai`. Privacy-conscious distributions can omit the entire crate
from a build without losing search, replace, CLI, or container support:
nothing else depends on it. The GUI presents the AI tab only when the
crate is linked — that wiring lives in the GUI crate's Cargo features.

## Opt-in

`DefaultSettings.ai_search_enabled` defaults to `false`. The AI chat
panel is greyed out until the user toggles this on in Settings, and the
toggle is wired to a one-time consent dialog that summarizes what
context (search path, query, filter list) the first request will send.

The API key never lives in `settings.json`. `grexa-ai` stores it in the
system keyring (`org.freedesktop.secrets` on Linux) keyed by
`service = "com.visorcraft.Grexa.ai"` and `account = <endpoint-base>`,
which means one user can keep multiple distinct keys for multiple
endpoints.

If the keyring backend is unavailable (no D-Bus session, no
secret-service daemon), `grexa-ai::store_api_key` returns
`SecretError::Backend(_)`. The Settings UI surfaces this verbatim and
**refuses to fall back to plaintext** — that's the
`docs/linux-decisions.md` rule and `docs/grex-storage-services-audit.md`
import contract working together.
