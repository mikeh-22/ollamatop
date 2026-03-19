use crate::model::stats::{OllamaModel, OllamaResponse, ModelStats, Usage};
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

/// Ollama API client for fetching model statistics
pub struct OllamaClient {
    client: Client,
    base_url: String, // host only, e.g. "http://localhost:11434"
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new() -> Result<Self> {
        let mut host = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        // Strip any trailing /api/... path that might have been included
        if let Some(idx) = host.find("/api/") {
            host.truncate(idx);
        }
        let base_url = host.trim_end_matches('/').to_string();

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, base_url })
    }

    /// Get the list of available models
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch models")?;

        if !response.status().is_success() {
            anyhow::bail!("API request failed with status: {}", response.status());
        }

        let data: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse response")?;

        let models = data
            .get("models")
            .and_then(|m| m.as_array())
            .ok_or_else(|| anyhow::anyhow!("No models array in response"))?
            .iter()
            .filter_map(|m| {
                let model_data = m.as_object()?;
                Some(OllamaModel {
                    name: model_data.get("name")?.as_str()?.to_string(),
                    parameters: model_data.get("parameters")?.as_u64()?,
                    quantization: model_data
                        .get("quantization")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    size: model_data.get("size")?.as_f64()?,
                    modified_at: model_data.get("modified_at")?.as_str()?.to_string(),
                })
            })
            .collect();

        Ok(models)
    }

    /// Get stats for a specific model by running a minimal generate request
    pub async fn get_model_stats(&self, model_name: &str) -> Result<ModelStats> {
        let url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "model": model_name,
                "prompt": "hi",
                "stream": false,
            }))
            .send()
            .await
            .context("Failed to send generate request")?;

        if !response.status().is_success() {
            anyhow::bail!("API request failed with status: {}", response.status());
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse response")?;

        let stats = ModelStats {
            name: model_name.to_string(),
            usage: Usage {
                total_tokens: ollama_response.eval_count,
                prompt_tokens: ollama_response.prompt_eval_count,
                completion_tokens: Some(ollama_response.eval_count),
            },
            response_time_ms: ollama_response
                .eval_duration
                .map(|d| d as f64 / 1_000_000.0),
            completion_count: 1,
            current_token_count: ollama_response.eval_count,
            token_history: Vec::new(),
        };

        Ok(stats)
    }

    /// Ping the Ollama server to check if it's running
    #[allow(dead_code)]
    pub async fn ping(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new();
        assert!(client.is_ok());
    }
}
