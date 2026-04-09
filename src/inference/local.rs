use super::InferenceProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Local inference via llama-server HTTP API
/// Expects llama-server running on localhost:8080
pub struct LocalInference {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Serialize)]
struct CompletionRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct CompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
}

impl LocalInference {
    pub fn new(server_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: server_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
        }
    }
    
    async fn generate_internal(&self, prompt: &str) -> Result<String> {
        let request = CompletionRequest {
            model: "qwen2.5-3b".to_string(),
            prompt: prompt.to_string(),
            max_tokens: 512,
            temperature: Some(0.7),
        };
        
        let response = self.client
            .post(format!("{}/v1/completions", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to connect to llama-server")?;
        
        if !response.status().is_success() {
            anyhow::bail!(
                "llama-server returned error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }
        
        let completion: CompletionResponse = response
            .json()
            .await
            .context("Failed to parse llama-server response")?;
        
        Ok(completion.choices[0].text.trim().to_string())
    }
    
    pub async fn health_check(&self) -> Result<()> {
        let response = self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .context("Failed to connect to llama-server")?;
        
        if !response.status().is_success() {
            anyhow::bail!("llama-server is not running");
        }
        
        Ok(())
    }
}

#[async_trait]
impl InferenceProvider for LocalInference {
    async fn generate(&self, prompt: &str) -> Result<String> {
        self.generate_internal(prompt).await
    }
    
    fn name(&self) -> &str {
        "local"
    }
    
    fn cost_estimate(&self, _prompt: &str, _response: &str) -> f64 {
        0.0 // Free!
    }
}
