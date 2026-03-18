#[cfg(test)]
mod tests {
    use ollamatop::ollama::client::OllamaClient;
    use ollamatop::model::stats::OllamaModel;
    use std::time::Duration;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_list_models() {
        let client = OllamaClient::new().unwrap();
        let models = client.list_models().await;

        // If Ollama is running, we should get models
        // If not, we expect an error
        if models.is_ok() {
            let models = models.unwrap();
            assert!(!models.is_empty());
        }
    }

    #[tokio::test]
    async fn test_ping() {
        let client = OllamaClient::new().unwrap();
        let is_running = client.ping().await;
        assert!(is_running.is_ok());
    }

    #[tokio::test]
    async fn test_get_model_stats() {
        let client = OllamaClient::new().unwrap();
        let models = client.list_models().await;

        if let Ok(models) = models {
            if !models.is_empty() {
                let first_model = models[0].name.clone();
                let stats = client.get_model_stats(&first_model).await;

                if stats.is_ok() {
                    let stats = stats.unwrap();
                    assert_eq!(stats.name, first_model);
                }
            }
        }
    }

    #[test]
    fn test_model_creation() {
        let model = OllamaModel {
            name: "test-model".to_string(),
            parameters: 7,
            quantization: Some("q4_0".to_string()),
            size: 4.0,
            modified_at: "2024-01-01".to_string(),
        };

        assert_eq!(model.name, "test-model");
        assert_eq!(model.parameters, 7);
    }

    #[test]
    fn test_context_usage_calculation() {
        use ollamatop::model::stats::ModelStats;

        let stats = ModelStats {
            name: "test-model".to_string(),
            usage: ollamatop::model::stats::Usage::default(),
            response_time_ms: None,
            completion_count: 0,
            current_token_count: 1024,
        };

        let percent = stats.context_usage_percent();
        assert_eq!(percent, 25.0);
    }

    #[tokio::test]
    async fn test_multiple_models() {
        let client = OllamaClient::new().unwrap();
        let models = client.list_models().await;

        if let Ok(models) = models {
            if !models.is_empty() && models.len() > 1 {
                let first_model = models[0].name.clone();
                let second_model = models[1].name.clone();

                let first_stats = client.get_model_stats(&first_model).await.unwrap();
                let second_stats = client.get_model_stats(&second_model).await.unwrap();

                assert_ne!(first_stats.name, second_stats.name);
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let client = OllamaClient::new().unwrap();

        // Try to get stats for a non-existent model
        let stats = client.get_model_stats("non-existent-model-12345").await;

        // Should fail gracefully
        assert!(stats.is_err());
    }
}