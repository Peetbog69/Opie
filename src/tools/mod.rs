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
        let mut prompt = String::from(
"You are Opie, a local AI coding assistant. You help with programming tasks by reading files, \
searching code, making edits, and running commands.

TOOL USAGE:
When you need to use a tool, respond with EXACTLY this format:
TOOL_CALL: {\"name\": \"tool_name\", \"parameters\": {\"param\": \"value\"}}

After calling a tool, you'll receive the result. Then respond naturally to the user.

AVAILABLE TOOLS:\n\n"
        );
        
        for tool in &self.tools {
            prompt.push_str(&format!("- {}: {}\n", tool.name(), tool.description()));
        }
        
        prompt.push_str(
"\nGUIDANCE:
- Use tools proactively. If the user asks about code, read it first.
- For \"change X to Y\" requests, use search_files to find it, then patch to edit.
- For \"what does this do\" questions, use read_file then explain.
- Keep responses concise and helpful.
- Don't apologize or ask permission - just do the task.
- If a file path is mentioned, use it exactly as given.

TOOL CALL FORMAT EXAMPLES:
TOOL_CALL: {\"name\": \"read_file\", \"parameters\": {\"path\": \"src/main.rs\"}}
TOOL_CALL: {\"name\": \"search_files\", \"parameters\": {\"pattern\": \"TODO\", \"file_glob\": \"*.rs\"}}
TOOL_CALL: {\"name\": \"patch\", \"parameters\": {\"path\": \"config.yaml\", \"old_string\": \"debug: false\", \"new_string\": \"debug: true\"}}
TOOL_CALL: {\"name\": \"write_file\", \"parameters\": {\"path\": \"test.txt\", \"content\": \"Hello world\"}}
TOOL_CALL: {\"name\": \"terminal\", \"parameters\": {\"command\": \"cargo test\"}}

Respond naturally when no tool is needed. Only output TOOL_CALL when you need to perform an action."
        );
        
        prompt
    }
}
