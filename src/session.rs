use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Default)]
pub struct Session {
    pub messages: Vec<Message>,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message {
            role: Role::User,
            content: content.into(),
        });
    }
    
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message {
            role: Role::Assistant,
            content: content.into(),
        });
    }
    
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message {
            role: Role::System,
            content: content.into(),
        });
    }
    
    /// Format messages for llama.cpp prompt
    pub fn to_prompt(&self) -> String {
        let mut prompt = String::new();
        
        for msg in &self.messages {
            match msg.role {
                Role::System => {
                    prompt.push_str(&format!("System: {}\n\n", msg.content));
                }
                Role::User => {
                    prompt.push_str(&format!("User: {}\n\n", msg.content));
                }
                Role::Assistant => {
                    prompt.push_str(&format!("Assistant: {}\n\n", msg.content));
                }
            }
        }
        
        prompt.push_str("Assistant: ");
        prompt
    }
    
    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.messages)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let messages: Vec<Message> = serde_json::from_str(&json)?;
        Ok(Self { messages })
    }
}
