use super::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub struct WriteFileTool;

#[derive(Deserialize)]
struct WriteFileParams {
    path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    
    fn description(&self) -> &str {
        "Write content to a file, creating it if it doesn't exist"
    }
    
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult> {
        let params: WriteFileParams = serde_json::from_value(params)?;
        
        let path = shellexpand::tilde(&params.path);
        let path = Path::new(path.as_ref());
        
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, params.content)?;
        
        Ok(ToolResult {
            success: true,
            output: format!("File written: {}", params.path),
            error: None,
        })
    }
}
