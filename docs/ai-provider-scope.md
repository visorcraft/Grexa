# AI Provider Scope

Grexa supports one provider-neutral wire contract: OpenAI-compatible model
listing and chat completions. Provider-native APIs, tool execution, and
authentication schemes beyond a bearer token are out of scope.

AI is optional at runtime and disabled by default, but the GUI always links the
`grexa-ai` crate. There is no Cargo feature that removes the AI code from the
desktop binary.

## Required HTTP contract

### Model listing

Grexa tests an endpoint and discovers a model with:

```http
GET <base>/v1/models
Authorization: Bearer <key>
```

The response must contain:

```json
{
  "data": [
    { "id": "model-id" }
  ]
}
```

Discovery selects the first non-empty `id`. If listing fails or yields no ID,
the client falls back to `gpt-4o-mini`.

### Chat completions

Grexa sends:

```http
POST <base>/v1/chat/completions
Content-Type: application/json; charset=utf-8
Authorization: Bearer <key>
```

Payload shape:

```json
{
  "model": "model-id",
  "temperature": 0.2,
  "messages": [
    { "role": "system", "content": "..." },
    { "role": "user", "content": "..." }
  ]
}
```

The preferred response shape is:

```json
{
  "choices": [
    {
      "message": {
        "content": "assistant text"
      }
    }
  ]
}
```

For compatibility, Grexa also accepts:

```text
choices[0].text
output_text
```

Error parsing checks `error.message`, then top-level `message`, then the raw
response body.

## Endpoint normalization

These settings resolve to the same base:

```text
https://example.test
https://example.test/
https://example.test/v1
https://example.test/v1/chat/completions
```

Grexa strips the known suffix and then appends `/v1/models` or
`/v1/chat/completions`.

A host without a scheme receives `https://`. For a local plaintext server,
enter the scheme explicitly:

```text
http://localhost:11434
```

## Authentication and transport

Grexa supports one optional header:

```http
Authorization: Bearer <API key>
```

It does not support custom headers, cookies, query-string API versions,
mutual TLS configuration, OAuth flows, or provider-specific signing.

Bearer keys are sent only to:

- any `https://` URL;
- `http://localhost`;
- `http://127.0.0.1`;
- `http://[::1]`.

A remote `http://` endpoint receives no bearer header. Redirects are disabled.
Requests time out after 90 seconds, and response bodies are capped at 4 MiB.

## Compatible server categories

A server is compatible when it implements the contract above. Common
categories include:

- OpenAI's chat-completions-compatible endpoints;
- local servers exposing an OpenAI compatibility layer;
- vLLM, llama.cpp, Ollama, LM Studio, or LocalAI when their OpenAI-compatible
  routes are enabled;
- gateways such as LiteLLM;
- organization proxies that preserve the documented paths, bearer header, and
  JSON shapes.

These projects evolve independently. Treat the protocol checklist, not the
brand list, as authoritative and use **Test endpoint** before chat.

## Not supported directly

- Anthropic Messages API;
- Google Gemini native API;
- Cohere native chat API;
- AWS Bedrock request signing;
- Azure OpenAI deployments requiring `api-key` and `api-version` parameters;
- streaming Server-Sent Events;
- tool/function calls;
- multimodal message blocks;
- provider-specific response content arrays;
- Responses API request format;
- embeddings or vector search;
- automatic model download or server management.

Use an OpenAI-compatible gateway when a provider has only a native format.

Grexa accepts a top-level `output_text` response for compatibility, but it
still sends a Chat Completions request. It is not a full Responses API client.

## Desktop behavior

Enable AI under **Settings → AI Search**. The same opt-in gate is checked
before endpoint tests, chat, and summaries.

- Saving a key stores it in the Secret Service.
- Testing sends only `GET /v1/models`.
- A typed chat sends that turn plus fixed system instructions and the current
  search path/query/modes/filter suggestions; visible earlier bubbles are not
  resent.
- Summarize Results sends bounded path/line/file excerpts from visible search
  rows.
- One request may run at a time.
- Responses are displayed as text and never executed or applied to files.

See [Security and privacy](SECURITY.md#outbound-traffic) for exact data
disclosure and [Using Grexa](usage.md#ai-search) for setup.

## Library behavior

`AiSearchClient::send_chat` accepts an arbitrary slice of
`AiConversationTurn`, so Rust callers may supply multi-turn history even though
the current GUI sends one turn.

`send_chat_with_evidence` adds text packed from `EvidenceMatch` records.
`pack_evidence` spreads the budget across files before adding more snippets
from the same file, favoring breadth without exceeding the character ceiling.
