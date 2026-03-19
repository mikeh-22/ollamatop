use serde::{Deserialize, Serialize};

/// Represents a single Ollama model
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaModel {
    /// Model name
    pub name: String,
    /// Total parameters in billions
    pub parameters: u64,
    /// Quantization type (e.g., "q4_0", "q5_0", "q8_0", etc.)
    pub quantization: Option<String>,
    /// Model size in GB
    pub size: f64,
    /// Modified date
    pub modified_at: String,
}

impl std::fmt::Display for OllamaModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Represents usage statistics for a model
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    /// Total tokens used
    pub total_tokens: u64,
    #[serde(default)]
    pub prompt_tokens: Option<u64>,
    #[serde(default)]
    pub completion_tokens: Option<u64>,
}

impl Default for Usage {
    fn default() -> Self {
        Self {
            total_tokens: 0,
            prompt_tokens: None,
            completion_tokens: None,
        }
    }
}

/// Represents response from Ollama /api/generate
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaResponse {
    pub model: String,
    pub created_at: String,
    /// Text response (from /api/generate)
    #[serde(default)]
    pub response: Option<String>,
    /// Chat message (from /api/chat)
    #[serde(default)]
    pub message: Option<Message>,
    pub eval_count: u64,
    #[serde(default)]
    pub eval_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u64>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
}

/// Represents a chat message in Ollama response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Context window size assumed when the model doesn't report one
pub const DEFAULT_CONTEXT_WINDOW: u64 = 4096;

/// Represents the full stats for a model
#[derive(Debug, Clone)]
pub struct ModelStats {
    pub name: String,
    pub usage: Usage,
    pub response_time_ms: Option<f64>,
    /// Cumulative number of stat refreshes for this model
    pub completion_count: u64,
    /// Token count from the most recent response
    pub current_token_count: u64,
    /// Rolling history of token counts (newest last, capped at 20)
    pub token_history: Vec<u64>,
}

impl ModelStats {
    /// Calculate percentage of context window used
    pub fn context_usage_percent(&self) -> f64 {
        if self.current_token_count == 0 {
            return 0.0;
        }
        (self.current_token_count as f64 / DEFAULT_CONTEXT_WINDOW as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_usage_calculation() {
        let stats = ModelStats {
            name: "test-model".to_string(),
            usage: Usage::default(),
            response_time_ms: None,
            completion_count: 0,
            current_token_count: 1024,
            token_history: Vec::new(),
        };

        assert_eq!(stats.context_usage_percent(), 25.0);
    }

    #[test]
    fn test_zero_context_usage() {
        let stats = ModelStats {
            name: "test-model".to_string(),
            usage: Usage::default(),
            response_time_ms: None,
            completion_count: 0,
            current_token_count: 0,
            token_history: Vec::new(),
        };

        assert_eq!(stats.context_usage_percent(), 0.0);
    }
}
