pub mod local;

use anyhow::Result;
use async_trait::async_trait;
use crate::session::Message;
use std::sync::{Arc, Mutex};

/// Callback for streaming chunks
pub type StreamCallback = Arc<Mutex<dyn FnMut(&str) + Send>>;

/// Core trait for inference providers
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Generate a response to the given messages (non-streaming)
    async fn generate(&self, messages: &[Message]) -> Result<String>;
    
    /// Generate a streaming response, calling the callback for each chunk
    /// Returns the full response text at the end
    async fn generate_stream(&self, messages: &[Message], callback: StreamCallback) -> Result<String> {
        // Default implementation: just call generate and return it all at once
        let response = self.generate(messages).await?;
        callback.lock().unwrap()(&response);
        Ok(response)
    }
    
    /// Get provider name for logging
    fn name(&self) -> &str;
    
    /// Estimated cost per message (tokens * cost, or 0 for local)
    fn cost_estimate(&self, _messages: &[Message], _response: &str) -> f64 {
        0.0
    }
}
