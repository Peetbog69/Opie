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

#[derive(Debug)]
pub struct Session {
    pub messages: Vec<Message>,
    max_context: usize,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            max_context: 6000, // Conservative limit (server has 8192)
        }
    }
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_max_context(max_context: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_context,
        }
    }
    
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message {
            role: Role::User,
            content: content.into(),
        });
        self.trim_if_needed();
    }
    
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message {
            role: Role::Assistant,
            content: content.into(),
        });
        self.trim_if_needed();
    }
    
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message {
            role: Role::System,
            content: content.into(),
        });
        self.trim_if_needed();
    }
    
    /// Estimate token count (rough: 1 token ≈ 4 chars)
    fn estimate_tokens(&self) -> usize {
        self.messages.iter()
            .map(|m| m.content.len() / 4)
            .sum()
    }
    
    /// Trim old messages if context is getting full
    /// Keeps: system prompt (first message) + recent N messages
    fn trim_if_needed(&mut self) {
        let tokens = self.estimate_tokens();
        
        if tokens > self.max_context {
            // Always keep the first system message
            if self.messages.is_empty() {
                return;
            }
            
            let system_msg = if matches!(self.messages[0].role, Role::System) {
                Some(self.messages[0].clone())
            } else {
                None
            };
            
            // Remove oldest messages until we're under the limit
            let target = self.max_context * 3 / 4; // Trim to 75% to avoid constant trimming
            let mut current_tokens = tokens;
            let mut remove_count = 0;
            
            let start_idx = if system_msg.is_some() { 1 } else { 0 };
            
            for i in start_idx..self.messages.len() {
                if current_tokens <= target {
                    break;
                }
                current_tokens -= self.messages[i].content.len() / 4;
                remove_count += 1;
            }
            
            if remove_count > 0 {
                let mut new_messages = Vec::new();
                
                // Add system message back
                if let Some(sys_msg) = system_msg {
                    new_messages.push(sys_msg);
                }
                
                // Add a summary message
                new_messages.push(Message {
                    role: Role::System,
                    content: format!("[Context trimmed: {} older messages removed to stay within token limit]", remove_count),
                });
                
                // Add remaining messages
                new_messages.extend_from_slice(&self.messages[start_idx + remove_count..]);
                
                self.messages = new_messages;
                
                eprintln!("⚠️  Context trimmed: removed {} old messages ({}→{} tokens)", 
                    remove_count, tokens, self.estimate_tokens());
            }
        }
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
        
        // Prompt for next assistant response
        prompt.push_str("Assistant: ");
        
        prompt
    }
    
    /// Get token count estimate
    pub fn token_count(&self) -> usize {
        self.estimate_tokens()
    }
    
    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}
