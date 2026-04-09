use super::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::process::Command;

pub struct TerminalTool;

#[derive(Deserialize)]
struct TerminalParams {
    command: String,
    #[serde(default = "default_timeout")]
    timeout: u64,
}

fn default_timeout() -> u64 { 30 }

#[async_trait]
impl Tool for TerminalTool {
    fn name(&self) -> &str {
        "terminal"
    }
    
    fn description(&self) -> &str {
        "Execute a shell command and return the output"
    }
    
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds (default: 30)",
                    "default": 30
                }
            },
            "required": ["command"]
        })
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult> {
        let params: TerminalParams = serde_json::from_value(params)?;
        
        // Execute command with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(params.timeout),
            tokio::task::spawn_blocking(move || {
                Command::new("sh")
                    .arg("-c")
                    .arg(&params.command)
                    .output()
            })
        ).await;
        
        match output {
            Ok(Ok(Ok(result))) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                let mut output = String::new();
                if !stdout.is_empty() {
                    output.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !output.is_empty() {
                        output.push_str("\nSTDERR:\n");
                    }
                    output.push_str(&stderr);
                }
                
                Ok(ToolResult {
                    success: result.status.success(),
                    output: output.trim().to_string(),
                    error: if result.status.success() {
                        None
                    } else {
                        Some(format!("Command exited with code: {}", 
                            result.status.code().unwrap_or(-1)))
                    },
                })
            }
            Ok(Ok(Err(e))) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to execute command: {}", e)),
            }),
            Ok(Err(e)) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Task panicked: {}", e)),
            }),
            Err(_) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Command timed out".to_string()),
            }),
        }
    }
}
