use super::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub struct ReadFileTool;

#[derive(Deserialize)]
struct ReadFileParams {
    path: String,
    #[serde(default = "default_offset")]
    offset: usize,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_offset() -> usize { 1 }
fn default_limit() -> usize { 500 }

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    
    fn description(&self) -> &str {
        "Read a text file with line numbers and pagination"
    }
    
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start from (1-indexed, default: 1)",
                    "default": 1
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read (default: 500)",
                    "default": 500
                }
            },
            "required": ["path"]
        })
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult> {
        let params: ReadFileParams = serde_json::from_value(params)?;
        
        let path = shellexpand::tilde(&params.path);
        let path = Path::new(path.as_ref());
        
        if !path.exists() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("File not found: {}", params.path)),
            });
        }
        
        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        let start = params.offset.saturating_sub(1);
        let end = (start + params.limit).min(total_lines);
        
        let mut output = String::new();
        for (idx, line) in lines[start..end].iter().enumerate() {
            output.push_str(&format!("{:6}|{}\n", start + idx + 1, line));
        }
        
        output.push_str(&format!("\nTotal lines: {}", total_lines));
        
        Ok(ToolResult {
            success: true,
            output,
            error: None,
        })
    }
}
