use crate::session::{Message, Session};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct SavedSession {
    version: u8,
    messages: Vec<Message>,
    created_at: String,
    updated_at: String,
}

pub struct SessionStorage {
    sessions_dir: PathBuf,
}

impl SessionStorage {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        
        let sessions_dir = PathBuf::from(home).join(".opie").join("sessions");
        fs::create_dir_all(&sessions_dir)
            .context("Failed to create sessions directory")?;
        
        Ok(Self { sessions_dir })
    }
    
    /// Save a session to disk
    pub fn save(&self, session: &Session, name: &str) -> Result<PathBuf> {
        let now = chrono::Local::now().to_rfc3339();
        
        let saved = SavedSession {
            version: 1,
            messages: session.messages.clone(),
            created_at: now.clone(),
            updated_at: now,
        };
        
        let filename = Self::sanitize_filename(name);
        let path = self.sessions_dir.join(format!("{}.json", filename));
        
        let json = serde_json::to_string_pretty(&saved)
            .context("Failed to serialize session")?;
        
        fs::write(&path, json)
            .context("Failed to write session file")?;
        
        Ok(path)
    }
    
    /// Load a session from disk
    pub fn load(&self, name: &str) -> Result<Session> {
        let filename = Self::sanitize_filename(name);
        let path = self.sessions_dir.join(format!("{}.json", filename));
        
        let json = fs::read_to_string(&path)
            .context("Failed to read session file")?;
        
        let saved: SavedSession = serde_json::from_str(&json)
            .context("Failed to parse session file")?;
        
        let mut session = Session::new();
        session.messages = saved.messages;
        
        Ok(session)
    }
    
    /// List all saved sessions
    pub fn list(&self) -> Result<Vec<String>> {
        let mut sessions = Vec::new();
        
        if !self.sessions_dir.exists() {
            return Ok(sessions);
        }
        
        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    sessions.push(name.to_string());
                }
            }
        }
        
        sessions.sort();
        Ok(sessions)
    }
    
    /// Delete a saved session
    pub fn delete(&self, name: &str) -> Result<()> {
        let filename = Self::sanitize_filename(name);
        let path = self.sessions_dir.join(format!("{}.json", filename));
        
        fs::remove_file(&path)
            .context("Failed to delete session file")?;
        
        Ok(())
    }
    
    /// Check if a session exists
    pub fn exists(&self, name: &str) -> bool {
        let filename = Self::sanitize_filename(name);
        let path = self.sessions_dir.join(format!("{}.json", filename));
        path.exists()
    }
    
    /// Sanitize a filename (remove special characters)
    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }
}
