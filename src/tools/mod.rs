use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod read_file;
pub mod write_file;
pub mod terminal;
pub mod search_files;
pub mod patch;

/// Tool call request from the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub parameters: serde_json::Value,
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Tool trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name
    fn name(&self) -> &str;
    
    /// Tool description for the LLM
    fn description(&self) -> &str;
    
    /// Parameter schema (JSON Schema format)
    fn parameters(&self) -> serde_json::Value;
    
    /// Execute the tool
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult>;
}

/// Tool registry that manages all available tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self { tools: Vec::new() };
        
        // Register built-in tools
        registry.register(Box::new(read_file::ReadFileTool));
        registry.register(Box::new(write_file::WriteFileTool));
        registry.register(Box::new(terminal::TerminalTool));
        registry.register(Box::new(search_files::SearchFilesTool));
        registry.register(Box::new(patch::PatchTool));
        
        registry
    }
    
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == name)
    }
    
    pub fn all(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }
    
    /// Get tool descriptions for the LLM system prompt
    pub fn system_prompt(&self) -> String {
        let mut prompt = String::from("You have access to the following tools:\n\n");
        
        for tool in &self.tools {
            prompt.push_str(&format!("- {}: {}\n", tool.name(), tool.description()));
        }
        
        prompt.push_str("\nTo use a tool, respond with:\n");
        prompt.push_str("TOOL_CALL: {\"name\": \"tool_name\", \"parameters\": {...}}\n");
        prompt.push_str("\nOnly use tools when necessary. Respond normally otherwise.");
        
        prompt
    }
}
