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
    /// Token usage per key
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

/// Represents response from Ollama API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaResponse {
    /// The model that generated the response
    pub model: String,
    /// The created timestamp
    pub created_at: String,
    /// Message content (not used for stats)
    pub message: Message,
    /// Token usage statistics
    pub eval_count: u64,
    #[serde(default)]
    pub eval_duration: Option<u64>,
    /// Usage details
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u64>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub token_count: Option<u64>,
}

/// Represents a message in Ollama response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Represents the full stats for a model
#[derive(Debug, Clone)]
pub struct ModelStats {
    /// Model name
    pub name: String,
    /// Usage statistics
    pub usage: Usage,
    /// Response metrics
    pub response_time_ms: Option<f64>,
    /// Number of completions generated
    pub completion_count: u64,
    /// Current token count
    pub current_token_count: u64,
}

impl ModelStats {
    /// Calculate percentage of context window used
    pub fn context_usage_percent(&self) -> f64 {
        if self.current_token_count == 0 {
            return 0.0;
        }
        let total = 4096.0; // Default context window size
        (self.current_token_count as f64 / total) * 100.0
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
        };

        let percent = stats.context_usage_percent();
        assert_eq!(percent, 25.0);
    }

    #[test]
    fn test_zero_context_usage() {
        let stats = ModelStats {
            name: "test-model".to_string(),
            usage: Usage::default(),
            response_time_ms: None,
            completion_count: 0,
            current_token_count: 0,
        };

        let percent = stats.context_usage_percent();
        assert_eq!(percent, 0.0);
    }
}