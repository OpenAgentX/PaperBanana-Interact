#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub history: String,
    pub current_image_b64: String,
    pub detailed_description: String,
}

impl Session {
    pub fn new() -> Self {
        Self {
            history: String::new(),
            current_image_b64: String::new(),
            detailed_description: String::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let session = serde_json::from_str(&content)?;
        Ok(session)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
