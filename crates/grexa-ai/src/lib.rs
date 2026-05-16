use serde::{Deserialize, Serialize};

pub const DEFAULT_MODEL: &str = "gpt-4o-mini";

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSearchResponse {
    pub success: bool,
    pub message: String,
    pub error_message: String,
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
        value = stripped.to_string();
    }

    value.trim_end_matches("/v1").to_string()
}

pub fn chat_completions_endpoint(endpoint: &str) -> String {
    format!("{}/v1/chat/completions", normalize_endpoint_base(endpoint))
}

pub fn models_endpoint(endpoint: &str) -> String {
    format!("{}/v1/models", normalize_endpoint_base(endpoint))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
