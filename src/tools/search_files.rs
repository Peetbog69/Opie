use super::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::process::Command;

pub struct SearchFilesTool;

#[derive(Deserialize)]
struct SearchParams {
    pattern: String,
    #[serde(default = "default_path")]
    path: String,
    #[serde(default)]
    file_glob: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_path() -> String { ".".to_string() }
fn default_limit() -> usize { 20 }

#[async_trait]
impl Tool for SearchFilesTool {
    fn name(&self) -> &str {
        "search_files"
    }
    
    fn description(&self) -> &str {
        "Search for a pattern in files (like grep). Returns matching lines with file names and line numbers."
    }
    
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Text or regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (default: current directory)",
                    "default": "."
                },
                "file_glob": {
                    "type": "string",
                    "description": "File pattern to filter (e.g., '*.rs', '*.py')"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 20)",
                    "default": 20
                }
            },
            "required": ["pattern"]
        })
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult> {
        let params: SearchParams = serde_json::from_value(params)?;
        
        // Build grep command
        let mut cmd = Command::new("grep");
        cmd.arg("-rn") // recursive, line numbers
            .arg("-I") // skip binary files
            .arg("--color=never");
        
        // Add file glob if specified
        if let Some(glob) = &params.file_glob {
            cmd.arg("--include").arg(glob);
        }
        
        cmd.arg(&params.pattern)
            .arg(&params.path);
        
        let output = cmd.output()?;
        
        if !output.status.success() && output.stdout.is_empty() {
            return Ok(ToolResult {
                success: true,
                output: "No matches found.".to_string(),
                error: None,
            });
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().take(params.limit).collect();
        
        let result = if lines.is_empty() {
            "No matches found.".to_string()
        } else {
            let mut output = format!("Found {} matches:\n\n", lines.len());
            for line in lines {
                output.push_str(line);
                output.push('\n');
            }
            output
        };
        
        Ok(ToolResult {
            success: true,
            output: result,
            error: None,
        })
    }
}
