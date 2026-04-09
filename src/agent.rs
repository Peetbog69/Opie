use crate::inference::InferenceProvider;
use crate::session::Session;
use crate::tools::{ToolCall, ToolRegistry};
use anyhow::{Context, Result};

const MAX_ITERATIONS: usize = 10;

pub struct Agent {
    provider: Box<dyn InferenceProvider>,
    tools: ToolRegistry,
}

impl Agent {
    pub fn new(provider: Box<dyn InferenceProvider>) -> Self {
        Self {
            provider,
            tools: ToolRegistry::new(),
        }
    }
    
    /// Run the agent loop with tool support
    pub async fn run(&self, session: &mut Session, user_input: &str) -> Result<String> {
        // Add system message with tool descriptions if not present
        if session.messages.is_empty() {
            session.add_system_message(self.tools.system_prompt());
        }
        
        // Add user message
        session.add_user_message(user_input);
        
        // Agent loop - keep going until we get a non-tool response
        for _iteration in 0..MAX_ITERATIONS {
            use std::io::{self, Write};
            use std::sync::{Arc, Mutex};
            
            let buffer = Arc::new(Mutex::new(String::new()));
            let is_tool_call = Arc::new(Mutex::new(false));
            
            let buffer_clone = buffer.clone();
            let is_tool_call_clone = is_tool_call.clone();
            
            // Stream the response, but detect tool calls early
            let callback = Arc::new(Mutex::new(move |chunk: &str| {
                let mut buf = buffer_clone.lock().unwrap();
                buf.push_str(chunk);
                
                // Check if it looks like a tool call (starts with TOOL_CALL:)
                if buf.contains("TOOL_CALL:") {
                    *is_tool_call_clone.lock().unwrap() = true;
                } else if !*is_tool_call_clone.lock().unwrap() {
                    // Not a tool call yet, print it
                    print!("{}", chunk);
                    io::stdout().flush().unwrap();
                }
            }));
            
            let response = self.provider.generate_stream(&session.messages, callback).await
                .context("Failed to generate response")?;
            
            if !*is_tool_call.lock().unwrap() {
                println!(); // Newline after streaming normal response
            }
            
            // Check if response contains a tool call
            if let Some(tool_call) = self.parse_tool_call(&response) {
                println!("  [{}] Running...", tool_call.name);
                
                // Execute the tool
                let tool_result = match self.tools.get(&tool_call.name) {
                    Some(tool) => {
                        tool.execute(tool_call.parameters).await
                            .context("Tool execution failed")?
                    }
                    None => {
                        // Tool not found
                        crate::tools::ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Tool '{}' not found", tool_call.name)),
                        }
                    }
                };
                
                // Add assistant's tool call to history
                session.add_assistant_message(&response);
                
                // Add tool result as a system message
                let result_msg = if tool_result.success {
                    format!("Tool result: {}", tool_result.output)
                } else {
                    format!("Tool error: {}", tool_result.error.unwrap_or_default())
                };
                session.add_system_message(&result_msg);
                
                // Continue loop to get next response
                continue;
            }
            
            // No tool call - this is the final response
            session.add_assistant_message(&response);
            return Ok(response);
        }
        
        anyhow::bail!("Agent exceeded maximum iterations ({})", MAX_ITERATIONS);
    }
    
    /// Parse a tool call from LLM response
    /// Format: TOOL_CALL: {"name": "tool_name", "parameters": {...}}
    fn parse_tool_call(&self, response: &str) -> Option<ToolCall> {
        let response = response.trim();
        
        // Look for TOOL_CALL: prefix
        if let Some(json_start) = response.find("TOOL_CALL:") {
            let json_str = &response[json_start + 10..].trim();
            
            // Try to parse JSON
            if let Ok(tool_call) = serde_json::from_str::<ToolCall>(json_str) {
                return Some(tool_call);
            }
            
            // Try to find the JSON object boundaries
            if let Some(start) = json_str.find('{') {
                if let Some(end) = json_str.rfind('}') {
                    let json_str = &json_str[start..=end];
                    if let Ok(tool_call) = serde_json::from_str::<ToolCall>(json_str) {
                        return Some(tool_call);
                    }
                }
            }
        }
        
        None
    }
}
