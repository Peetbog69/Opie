use super::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub struct PatchTool;

#[derive(Deserialize)]
struct PatchParams {
    path: String,
    old_string: String,
    new_string: String,
    #[serde(default)]
    replace_all: bool,
}

#[async_trait]
impl Tool for PatchTool {
    fn name(&self) -> &str {
        "patch"
    }
    
    fn description(&self) -> &str {
        "Make targeted edits to a file by replacing old_string with new_string. More precise than rewriting the whole file."
    }
    
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "Text to find and replace (must be unique unless replace_all=true)"
                },
                "new_string": {
                    "type": "string",
                    "description": "Replacement text (can be empty to delete)"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences instead of requiring unique match (default: false)",
                    "default": false
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult> {
        let params: PatchParams = serde_json::from_value(params)?;
        
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
        
        // Count occurrences
        let count = content.matches(&params.old_string).count();
        
        if count == 0 {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("String not found in file".to_string()),
            });
        }
        
        if count > 1 && !params.replace_all {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "String appears {} times. Use replace_all=true or make the string more specific.",
                    count
                )),
            });
        }
        
        // Perform replacement
        let new_content = if params.replace_all {
            content.replace(&params.old_string, &params.new_string)
        } else {
            content.replacen(&params.old_string, &params.new_string, 1)
        };
        
        fs::write(path, new_content)?;
        
        let action = if params.replace_all {
            format!("Replaced {} occurrence(s)", count)
        } else {
            "Replaced 1 occurrence".to_string()
        };
        
        Ok(ToolResult {
            success: true,
            output: format!("{} in {}", action, params.path),
            error: None,
        })
    }
}
