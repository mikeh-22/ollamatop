#[cfg(test)]
mod tests {
    use ollamatop::ollama::client::OllamaClient;
    use ollamatop::model::stats::OllamaModel;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_list_models() {
        let client = OllamaClient::new().unwrap();
        let models = client.list_models().await;

        // If Ollama is running we should get models; if not, an error is expected
        if let Ok(models) = models {
            assert!(!models.is_empty());
        }
    }

    #[tokio::test]
    async fn test_ping() {
        let client = OllamaClient::new().unwrap();
        // ping returns Ok(bool); it only errors on a transport failure
        assert!(client.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_get_model_stats() {
        let client = OllamaClient::new().unwrap();
        if let Ok(models) = client.list_models().await {
            if !models.is_empty() {
                let first_model = models[0].name.clone();
                if let Ok(stats) = client.get_model_stats(&first_model).await {
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

    #[tokio::test]
    async fn test_multiple_models() {
        let client = OllamaClient::new().unwrap();
        if let Ok(models) = client.list_models().await {
            if models.len() > 1 {
                let first = models[0].name.clone();
                let second = models[1].name.clone();

                let first_stats = client.get_model_stats(&first).await.unwrap();
                let second_stats = client.get_model_stats(&second).await.unwrap();

                assert_ne!(first_stats.name, second_stats.name);
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let client = OllamaClient::new().unwrap();
        let stats = client.get_model_stats("non-existent-model-12345").await;
        assert!(stats.is_err());
    }
}
