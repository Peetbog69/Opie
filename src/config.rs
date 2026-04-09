use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// llama-server URL (if using external server)
    #[serde(default = "default_server_url")]
    pub server_url: String,
    
    /// Anthropic API key (optional - for fallback)
    pub anthropic_api_key: Option<String>,
    
    /// When to use API: "never", "fallback", "auto", "always"
    #[serde(default = "default_api_mode")]
    pub api_mode: String,
    
    /// Conversation history directory
    #[serde(default = "default_history_dir")]
    pub history_dir: PathBuf,
}

fn default_server_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_api_mode() -> String {
    "never".to_string()
}

fn default_history_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".opie")
        .join("history")
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if !config_path.exists() {
            return Ok(Self::default_config());
        }
        
        let content = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        toml::from_str(&content).context("Failed to parse config file")
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".opie")
            .join("config.toml")
    }
    
    fn default_config() -> Self {
        Self {
            server_url: default_server_url(),
            anthropic_api_key: None,
            api_mode: default_api_mode(),
            history_dir: default_history_dir(),
        }
    }
}
