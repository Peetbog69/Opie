pub mod local;

use anyhow::Result;
use async_trait::async_trait;

/// Core trait for inference providers
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Generate a response to the given prompt
    async fn generate(&self, prompt: &str) -> Result<String>;
    
    /// Get provider name for logging
    fn name(&self) -> &str;
    
    /// Estimated cost per message (tokens * cost, or 0 for local)
    fn cost_estimate(&self, _prompt: &str, _response: &str) -> f64 {
        0.0
    }
}
