use super::InferenceProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

/// Local inference via llama-server HTTP API
/// Expects llama-server running on localhost:8080
pub struct LocalInference {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatCompletionChunk {
    choices: Vec<DeltaChoice>,
}

#[derive(Deserialize)]
struct DeltaChoice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

impl LocalInference {
    pub fn new(server_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: server_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
        }
    }
    
    async fn generate_internal(&self, messages: &[crate::session::Message]) -> Result<String> {
        // Convert our messages to the API format
        let api_messages: Vec<ChatMessage> = messages.iter().map(|m| {
            let role = match m.role {
                crate::session::Role::User => "user",
                crate::session::Role::Assistant => "assistant",
                crate::session::Role::System => "system",
            };
            ChatMessage {
                role: role.to_string(),
                content: m.content.clone(),
            }
        }).collect();
        
        let request = ChatCompletionRequest {
            model: "qwen2.5-3b".to_string(),
            messages: api_messages,
            max_tokens: 512,
            temperature: Some(0.7),
            stream: None,
        };
        
        let response = self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
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
        
        let completion: ChatCompletionResponse = response
            .json()
            .await
            .context("Failed to parse llama-server response")?;
        
        Ok(completion.choices[0].message.content.trim().to_string())
    }
    
    async fn generate_stream_internal(
        &self,
        messages: &[crate::session::Message],
        callback: super::StreamCallback,
    ) -> Result<String> {
        // Convert our messages to the API format
        let api_messages: Vec<ChatMessage> = messages.iter().map(|m| {
            let role = match m.role {
                crate::session::Role::User => "user",
                crate::session::Role::Assistant => "assistant",
                crate::session::Role::System => "system",
            };
            ChatMessage {
                role: role.to_string(),
                content: m.content.clone(),
            }
        }).collect();
        
        let request = ChatCompletionRequest {
            model: "qwen2.5-3b".to_string(),
            messages: api_messages,
            max_tokens: 512,
            temperature: Some(0.7),
            stream: Some(true),
        };
        
        let response = self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
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
        
        let mut full_response = String::new();
        let mut stream = response.bytes_stream().eventsource();
        
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if event.data == "[DONE]" {
                        break;
                    }
                    
                    if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(&event.data) {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                callback.lock().unwrap()(content);
                                full_response.push_str(content);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    break;
                }
            }
        }
        
        Ok(full_response.trim().to_string())
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
    async fn generate(&self, messages: &[crate::session::Message]) -> Result<String> {
        self.generate_internal(messages).await
    }
    
    async fn generate_stream(&self, messages: &[crate::session::Message], callback: super::StreamCallback) -> Result<String> {
        self.generate_stream_internal(messages, callback).await
    }
    
    fn name(&self) -> &str {
        "local"
    }
    
    fn cost_estimate(&self, _messages: &[crate::session::Message], _response: &str) -> f64 {
        0.0 // Free!
    }
}
