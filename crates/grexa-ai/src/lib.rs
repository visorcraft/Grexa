use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use grexa_core::SearchOptions;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_TIMEOUT_SECS: u64 = 90;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSearchConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

impl Default for AiSearchConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: None,
            model: Some(DEFAULT_MODEL.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSearchContext {
    pub search_path: String,
    pub search_query: String,
    pub filter_suggestions: Vec<String>,
    pub regex_search: bool,
    pub files_search: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiConversationTurn {
    pub role: AiRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiRole {
    User,
    Assistant,
    System,
}

impl AiRole {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "assistant" => Self::Assistant,
            "system" => Self::System,
            _ => Self::User,
        }
    }

    pub fn wire(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSearchResponse {
    pub success: bool,
    pub message: String,
    pub error_message: String,
}

impl AiSearchResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            error_message: String::new(),
        }
    }

    pub fn fail(error: impl Into<String>) -> Self {
        Self {
            success: false,
            message: String::new(),
            error_message: error.into(),
        }
    }
}

/// Wire-level HTTP transport. Production code uses [`UreqTransport`]; tests
/// inject a fake to avoid network access.
pub trait HttpTransport: Send + Sync {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError>;
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("transport error: {0}")]
    Transport(String),
}

/// Synchronous ureq-backed transport.
pub struct UreqTransport;

impl HttpTransport for UreqTransport {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        let agent = ureq::AgentBuilder::new().timeout(request.timeout).build();
        let mut builder = match request.method {
            HttpMethod::Get => agent.get(&request.url),
            HttpMethod::Post => agent.post(&request.url),
        };
        for (name, value) in &request.headers {
            builder = builder.set(name, value);
        }

        let result = match request.body {
            Some(body) => builder.send_bytes(&body),
            None => builder.call(),
        };

        match result {
            Ok(response) => {
                let status = response.status();
                let mut buf = Vec::new();
                response
                    .into_reader()
                    .read_to_end(&mut buf)
                    .map_err(|err| HttpError::Transport(err.to_string()))?;
                Ok(HttpResponse { status, body: buf })
            }
            Err(ureq::Error::Status(status, response)) => {
                let mut buf = Vec::new();
                let _ = response.into_reader().read_to_end(&mut buf);
                Ok(HttpResponse { status, body: buf })
            }
            Err(err) => Err(HttpError::Transport(err.to_string())),
        }
    }
}

/// High-level AI search client. Tests construct with `with_transport`; the
/// runtime defaults to [`UreqTransport`].
pub struct AiSearchClient<T: HttpTransport = UreqTransport> {
    transport: T,
    timeout: Duration,
}

impl AiSearchClient<UreqTransport> {
    pub fn new() -> Self {
        Self::with_transport(UreqTransport)
    }
}

impl Default for AiSearchClient<UreqTransport> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: HttpTransport> AiSearchClient<T> {
    pub fn with_transport(transport: T) -> Self {
        Self {
            transport,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a one-shot test to `/v1/models`. Returns `(success, message)`.
    pub fn test_endpoint(&self, config: &AiSearchConfig) -> AiSearchResponse {
        if config.endpoint.trim().is_empty() {
            return AiSearchResponse::fail("AI endpoint is not configured.");
        }
        let url = models_endpoint(&config.endpoint);
        let request = HttpRequest {
            method: HttpMethod::Get,
            url,
            headers: auth_headers(config.api_key.as_deref()),
            body: None,
            timeout: self.timeout,
        };
        match self.transport.send(request) {
            Ok(resp) if resp.is_success() => {
                AiSearchResponse::ok("Endpoint reachable.")
            }
            Ok(resp) => AiSearchResponse::fail(format!(
                "AI endpoint returned HTTP {}: {}",
                resp.status,
                extract_error_message(&resp.body)
            )),
            Err(err) => AiSearchResponse::fail(err.to_string()),
        }
    }

    /// Discover the first available model id at `/v1/models`, falling back to
    /// [`DEFAULT_MODEL`] on any failure.
    pub fn discover_model(&self, config: &AiSearchConfig) -> String {
        if config.endpoint.trim().is_empty() {
            return DEFAULT_MODEL.to_string();
        }
        let request = HttpRequest {
            method: HttpMethod::Get,
            url: models_endpoint(&config.endpoint),
            headers: auth_headers(config.api_key.as_deref()),
            body: None,
            timeout: self.timeout,
        };
        match self.transport.send(request) {
            Ok(resp) if resp.is_success() => parse_first_model_id(&resp.body)
                .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            _ => DEFAULT_MODEL.to_string(),
        }
    }

    /// Send a chat completion turn. Mirrors Grex `SendDiscussionTurnAsync`
    /// behavior: blank endpoint → typed error, blank assistant text → error,
    /// otherwise trimmed assistant text in `message`.
    pub fn send_chat(
        &self,
        config: &AiSearchConfig,
        context: &AiSearchContext,
        conversation: &[AiConversationTurn],
    ) -> AiSearchResponse {
        if config.endpoint.trim().is_empty() {
            return AiSearchResponse::fail("AI endpoint is not configured.");
        }

        let model = config
            .model
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| self.discover_model(config));

        let messages = build_messages(context, conversation);
        let payload = serde_json::json!({
            "model": model,
            "temperature": 0.2,
            "messages": messages,
        });

        let mut headers = auth_headers(config.api_key.as_deref());
        headers.insert(
            "Content-Type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );

        let request = HttpRequest {
            method: HttpMethod::Post,
            url: chat_completions_endpoint(&config.endpoint),
            headers,
            body: Some(
                serde_json::to_vec(&payload)
                    .unwrap_or_else(|_| b"{}".to_vec()),
            ),
            timeout: self.timeout,
        };

        match self.transport.send(request) {
            Ok(resp) if resp.is_success() => match extract_assistant_content(&resp.body) {
                Some(message) => AiSearchResponse::ok(message),
                None => AiSearchResponse::fail("AI endpoint returned an empty response."),
            },
            Ok(resp) => AiSearchResponse::fail(format!(
                "AI endpoint returned HTTP {}: {}",
                resp.status,
                extract_error_message(&resp.body)
            )),
            Err(err) => AiSearchResponse::fail(err.to_string()),
        }
    }
}

fn auth_headers(api_key: Option<&str>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    if let Some(key) = api_key.map(str::trim).filter(|key| !key.is_empty()) {
        headers.insert("Authorization".to_string(), format!("Bearer {key}"));
    }
    headers
}

fn build_messages(
    context: &AiSearchContext,
    conversation: &[AiConversationTurn],
) -> Vec<serde_json::Value> {
    let mut out = Vec::with_capacity(conversation.len() + 2);
    out.push(serde_json::json!({
        "role": "system",
        "content": "You are Grexa AI Search. Help the user locate relevant files and code using the provided path, query, and filter suggestions. Ask concise follow-up questions when needed.",
    }));
    out.push(serde_json::json!({
        "role": "system",
        "content": build_context_prompt(context),
    }));
    for turn in conversation {
        let trimmed = turn.content.trim();
        if trimmed.is_empty() {
            continue;
        }
        out.push(serde_json::json!({
            "role": turn.role.wire(),
            "content": trimmed,
        }));
    }
    out
}

fn build_context_prompt(context: &AiSearchContext) -> String {
    let mut prompt = String::from("AI search context:\n");
    prompt.push_str(&format!("- path: {}\n", context.search_path));
    prompt.push_str(&format!("- query: {}\n", context.search_query));
    prompt.push_str(&format!(
        "- search type: {}\n",
        if context.regex_search { "Regex" } else { "Text" }
    ));
    prompt.push_str(&format!(
        "- result mode: {}\n",
        if context.files_search {
            "Files"
        } else {
            "Content lines"
        }
    ));
    prompt.push_str("Filter suggestions:\n");
    let active: Vec<_> = context
        .filter_suggestions
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if active.is_empty() {
        prompt.push_str("- No additional filters\n");
    } else {
        for suggestion in active {
            prompt.push_str(&format!("- {suggestion}\n"));
        }
    }
    prompt
        .push_str("Treat filters as suggestions and explain reasoning with concrete next steps.");
    prompt
}

fn parse_first_model_id(body: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let arr = value.get("data")?.as_array()?;
    for item in arr {
        if let Some(id) = item.get("id").and_then(|value| value.as_str()) {
            let trimmed = id.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn extract_assistant_content(body: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;

    // OpenAI-style: choices[0].message.content
    if let Some(content) = value
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
    {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // Legacy completions: choices[0].text
    if let Some(text) = value
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("text"))
        .and_then(|text| text.as_str())
    {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // Responses-API style: output_text
    if let Some(text) = value.get("output_text").and_then(|value| value.as_str()) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

fn extract_error_message(body: &[u8]) -> String {
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) {
        if let Some(message) = value
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(|message| message.as_str())
        {
            return message.to_string();
        }
        if let Some(message) = value.get("message").and_then(|message| message.as_str()) {
            return message.to_string();
        }
    }
    String::from_utf8_lossy(body).to_string()
}

/// Build Linux-aware filter suggestions to seed an [`AiSearchContext`]. The
/// strings are user-facing and rendered verbatim in the AI prompt.
///
/// The audit (`grex-ai-search-service-audit.md`) calls out that Linux phrasing
/// replaces "Windows Search index" with "Linux file index (Baloo)" and adds
/// hints for hidden files, symlinks, mounted shares, pseudo filesystems, and
/// container targets when relevant.
pub fn linux_suggestions_for(options: &SearchOptions) -> Vec<String> {
    let mut hints = Vec::new();

    if options.respect_gitignore {
        hints.push("respect .gitignore".to_string());
    }
    if options.case_sensitive {
        hints.push("case-sensitive search".to_string());
    }
    if !options.include_subfolders {
        hints.push("limit to the chosen directory (no subfolders)".to_string());
    }
    if options.include_hidden {
        hints.push("include hidden dotfiles and dotdirs".to_string());
    }
    if options.include_binary {
        hints.push("include searchable binary/document files".to_string());
    }
    if options.include_symlinks {
        hints.push("follow symbolic links (watch for mount loops)".to_string());
    }
    if options.include_system {
        hints.push(
            "include system/dependency directories (.git, vendor, node_modules, /proc, /sys)"
                .to_string(),
        );
    }
    if options.use_file_index {
        hints.push("seed candidates from the Linux file index (Baloo, KDE)".to_string());
    }
    if !options.match_file_names.trim().is_empty() {
        hints.push(format!(
            "match file names: {}",
            options.match_file_names.trim()
        ));
    }
    if !options.exclude_dirs.trim().is_empty() {
        hints.push(format!("exclude dirs: {}", options.exclude_dirs.trim()));
    }

    if let Some(label) = mount_kind_hint(&options.path) {
        hints.push(label);
    }

    hints
}

fn mount_kind_hint(path: &Path) -> Option<String> {
    let s = path.to_string_lossy();
    let s_lower = s.to_ascii_lowercase();
    if s == "/" {
        return Some(
            "root filesystem search; Grexa already guards /proc, /sys, /dev, /run pseudo filesystems"
                .to_string(),
        );
    }
    if s_lower.starts_with("/proc")
        || s_lower.starts_with("/sys")
        || s_lower.starts_with("/dev")
        || s_lower.starts_with("/run")
    {
        return Some(format!(
            "pseudo filesystem path {} — many files are virtual and may be empty or unstable",
            s
        ));
    }
    if s_lower.starts_with("/mnt/")
        || s_lower.starts_with("/media/")
        || s_lower.contains("/gvfs/")
        || s_lower.contains("/kio-fuse/")
    {
        return Some(format!("mounted share or removable media path: {}", s));
    }
    None
}

pub fn normalize_endpoint_base(endpoint: &str) -> String {
    let mut value = endpoint.trim().trim_end_matches('/').to_string();
    if value.is_empty() {
        return value;
    }

    if !value.starts_with("http://") && !value.starts_with("https://") {
        value = format!("https://{value}");
    }

    if let Some(stripped) = value.strip_suffix("/chat/completions") {
        value = stripped.trim_end_matches('/').to_string();
    }

    value.trim_end_matches("/v1").trim_end_matches('/').to_string()
}

pub fn chat_completions_endpoint(endpoint: &str) -> String {
    format!("{}/v1/chat/completions", normalize_endpoint_base(endpoint))
}

pub fn models_endpoint(endpoint: &str) -> String {
    format!("{}/v1/models", normalize_endpoint_base(endpoint))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[derive(Debug, Default, Clone)]
    struct MockTransport {
        responses: Arc<Mutex<Vec<HttpResponse>>>,
        captured: Arc<Mutex<Vec<HttpRequest>>>,
    }

    impl MockTransport {
        fn with_responses(values: Vec<HttpResponse>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(values)),
                captured: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn captured(&self) -> Vec<HttpRequest> {
            self.captured.lock().unwrap().clone()
        }
    }

    impl HttpTransport for MockTransport {
        fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
            self.captured.lock().unwrap().push(request);
            let mut queue = self.responses.lock().unwrap();
            if queue.is_empty() {
                return Err(HttpError::Transport("no canned response".into()));
            }
            Ok(queue.remove(0))
        }
    }

    fn response(status: u16, body: &str) -> HttpResponse {
        HttpResponse {
            status,
            body: body.as_bytes().to_vec(),
        }
    }

    #[test]
    fn normalizes_common_endpoint_shapes() {
        assert_eq!(
            chat_completions_endpoint("api.example.com/v1/"),
            "https://api.example.com/v1/chat/completions"
        );
        assert_eq!(
            models_endpoint("https://api.example.com/v1/chat/completions"),
            "https://api.example.com/v1/models"
        );
        assert_eq!(
            chat_completions_endpoint("https://api.example.com/"),
            "https://api.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_endpoint_succeeds_on_200() {
        let transport = MockTransport::with_responses(vec![response(200, "{\"data\":[]}")]);
        let client = AiSearchClient::with_transport(transport.clone());
        let config = AiSearchConfig {
            endpoint: "https://api.example.com/v1".into(),
            api_key: Some("sk-test".into()),
            model: None,
        };
        let result = client.test_endpoint(&config);
        assert!(result.success);

        let captured = transport.captured();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].url, "https://api.example.com/v1/models");
        assert_eq!(
            captured[0].headers.get("Authorization"),
            Some(&"Bearer sk-test".to_string())
        );
    }

    #[test]
    fn test_endpoint_fails_on_non_200() {
        let transport = MockTransport::with_responses(vec![response(
            401,
            "{\"error\":{\"message\":\"bad key\"}}",
        )]);
        let client = AiSearchClient::with_transport(transport);
        let config = AiSearchConfig {
            endpoint: "https://api.example.com/v1".into(),
            api_key: Some("sk-test".into()),
            model: None,
        };
        let result = client.test_endpoint(&config);
        assert!(!result.success);
        assert!(result.error_message.contains("HTTP 401"));
        assert!(result.error_message.contains("bad key"));
    }

    #[test]
    fn discover_model_returns_first_data_id() {
        let transport = MockTransport::with_responses(vec![response(
            200,
            "{\"data\":[{\"id\":\"gpt-4.1-mini\"},{\"id\":\"gpt-3.5-turbo\"}]}",
        )]);
        let client = AiSearchClient::with_transport(transport);
        let config = AiSearchConfig {
            endpoint: "https://api.example.com/v1".into(),
            api_key: None,
            model: None,
        };
        assert_eq!(client.discover_model(&config), "gpt-4.1-mini");
    }

    #[test]
    fn discover_model_falls_back_on_error() {
        let transport = MockTransport::with_responses(vec![response(500, "{}")]);
        let client = AiSearchClient::with_transport(transport);
        let config = AiSearchConfig {
            endpoint: "https://api.example.com/v1".into(),
            api_key: None,
            model: None,
        };
        assert_eq!(client.discover_model(&config), DEFAULT_MODEL);
    }

    #[test]
    fn send_chat_parses_openai_choices_message_content() {
        let transport = MockTransport::with_responses(vec![response(
            200,
            "{\"choices\":[{\"message\":{\"content\":\"  hello user  \"}}]}",
        )]);
        let client = AiSearchClient::with_transport(transport.clone());
        let config = AiSearchConfig {
            endpoint: "https://api.example.com/v1".into(),
            api_key: Some("sk".into()),
            model: Some("gpt-4o-mini".into()),
        };
        let context = AiSearchContext {
            search_path: "/home/me/code".into(),
            search_query: "TODO".into(),
            filter_suggestions: vec!["respect gitignore".into(), "  ".into()],
            regex_search: false,
            files_search: false,
        };
        let conversation = vec![AiConversationTurn {
            role: AiRole::User,
            content: "Help me find todos".to_string(),
        }];
        let result = client.send_chat(&config, &context, &conversation);
        assert!(result.success, "{result:?}");
        assert_eq!(result.message, "hello user");

        let request = transport.captured().pop().unwrap();
        assert_eq!(request.url, "https://api.example.com/v1/chat/completions");
        assert_eq!(
            request.headers.get("Content-Type").map(String::as_str),
            Some("application/json; charset=utf-8")
        );
        let body: serde_json::Value =
            serde_json::from_slice(request.body.as_ref().unwrap()).unwrap();
        assert_eq!(body["model"], "gpt-4o-mini");
        let messages = body["messages"].as_array().unwrap();
        assert!(messages.len() >= 3);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["role"], "system");
        assert!(messages[1]["content"]
            .as_str()
            .unwrap()
            .contains("respect gitignore"));
    }

    #[test]
    fn send_chat_returns_empty_response_when_choice_blank() {
        let transport = MockTransport::with_responses(vec![response(
            200,
            "{\"choices\":[{\"message\":{\"content\":\"\"}}]}",
        )]);
        let client = AiSearchClient::with_transport(transport);
        let result = client.send_chat(
            &AiSearchConfig {
                endpoint: "https://api.example.com/v1".into(),
                api_key: None,
                model: Some("gpt-4o-mini".into()),
            },
            &AiSearchContext {
                search_path: "/".into(),
                search_query: "x".into(),
                filter_suggestions: vec![],
                regex_search: false,
                files_search: false,
            },
            &[],
        );
        assert!(!result.success);
        assert!(result.error_message.contains("empty response"));
    }

    #[test]
    fn send_chat_parses_legacy_choices_text() {
        let transport = MockTransport::with_responses(vec![response(
            200,
            "{\"choices\":[{\"text\":\"legacy reply\"}]}",
        )]);
        let client = AiSearchClient::with_transport(transport);
        let result = client.send_chat(
            &AiSearchConfig {
                endpoint: "https://api.example.com/v1".into(),
                api_key: None,
                model: Some("local".into()),
            },
            &AiSearchContext {
                search_path: "/".into(),
                search_query: "x".into(),
                filter_suggestions: vec![],
                regex_search: false,
                files_search: false,
            },
            &[],
        );
        assert!(result.success);
        assert_eq!(result.message, "legacy reply");
    }

    #[test]
    fn send_chat_parses_output_text() {
        let transport =
            MockTransport::with_responses(vec![response(200, "{\"output_text\":\"resp\"}")]);
        let client = AiSearchClient::with_transport(transport);
        let result = client.send_chat(
            &AiSearchConfig {
                endpoint: "https://api.example.com/v1".into(),
                api_key: None,
                model: Some("local".into()),
            },
            &AiSearchContext {
                search_path: "/".into(),
                search_query: "x".into(),
                filter_suggestions: vec![],
                regex_search: false,
                files_search: false,
            },
            &[],
        );
        assert!(result.success);
        assert_eq!(result.message, "resp");
    }

    #[test]
    fn send_chat_blank_endpoint_short_circuits() {
        let client = AiSearchClient::with_transport(MockTransport::default());
        let result = client.send_chat(
            &AiSearchConfig {
                endpoint: " ".into(),
                api_key: None,
                model: None,
            },
            &AiSearchContext {
                search_path: "/".into(),
                search_query: "".into(),
                filter_suggestions: vec![],
                regex_search: false,
                files_search: false,
            },
            &[],
        );
        assert!(!result.success);
        assert!(result.error_message.contains("not configured"));
    }

    #[test]
    fn send_chat_falls_back_to_model_discovery_when_model_blank() {
        let transport = MockTransport::with_responses(vec![
            // Discovery
            response(200, "{\"data\":[{\"id\":\"discovered-model\"}]}"),
            // Chat
            response(
                200,
                "{\"choices\":[{\"message\":{\"content\":\"hi\"}}]}",
            ),
        ]);
        let client = AiSearchClient::with_transport(transport.clone());
        let result = client.send_chat(
            &AiSearchConfig {
                endpoint: "https://api.example.com/v1".into(),
                api_key: None,
                model: None,
            },
            &AiSearchContext {
                search_path: "/".into(),
                search_query: "".into(),
                filter_suggestions: vec![],
                regex_search: false,
                files_search: false,
            },
            &[],
        );
        assert!(result.success);
        let chat_request = transport.captured().pop().unwrap();
        let body: serde_json::Value =
            serde_json::from_slice(chat_request.body.as_ref().unwrap()).unwrap();
        assert_eq!(body["model"], "discovered-model");
    }

    #[test]
    fn role_parser_is_tolerant() {
        assert_eq!(AiRole::parse("assistant"), AiRole::Assistant);
        assert_eq!(AiRole::parse("  System "), AiRole::System);
        assert_eq!(AiRole::parse("anything"), AiRole::User);
        assert_eq!(AiRole::parse(""), AiRole::User);
    }

    #[test]
    fn linux_suggestions_cover_active_flags() {
        let mut options = SearchOptions::new("/home/me/code", "TODO");
        options.respect_gitignore = true;
        options.include_hidden = true;
        options.include_symlinks = true;
        options.match_file_names = "*.rs".to_string();
        options.exclude_dirs = "target".to_string();
        options.use_file_index = true;

        let hints = linux_suggestions_for(&options);
        assert!(hints.iter().any(|h| h.contains("respect .gitignore")));
        assert!(hints.iter().any(|h| h.contains("hidden dotfiles")));
        assert!(hints.iter().any(|h| h.contains("symbolic links")));
        assert!(hints.iter().any(|h| h.contains("match file names: *.rs")));
        assert!(hints.iter().any(|h| h.contains("exclude dirs: target")));
        assert!(hints.iter().any(|h| h.contains("Baloo")));
    }

    #[test]
    fn linux_suggestions_flags_pseudo_filesystems() {
        let options = SearchOptions::new("/proc/1", "x");
        let hints = linux_suggestions_for(&options);
        assert!(hints.iter().any(|h| h.contains("pseudo filesystem")));
    }

    #[test]
    fn linux_suggestions_flags_mounts() {
        let options = SearchOptions::new("/mnt/data", "x");
        let hints = linux_suggestions_for(&options);
        assert!(hints.iter().any(|h| h.contains("mounted share")));
    }

    #[test]
    fn context_prompt_renders_with_no_filters() {
        let prompt = build_context_prompt(&AiSearchContext {
            search_path: "/p".into(),
            search_query: "q".into(),
            filter_suggestions: vec![],
            regex_search: true,
            files_search: false,
        });
        assert!(prompt.contains("No additional filters"));
        assert!(prompt.contains("search type: Regex"));
        assert!(prompt.contains("result mode: Content lines"));
    }
}
