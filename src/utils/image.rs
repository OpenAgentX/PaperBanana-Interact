#![allow(dead_code)]
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn read_image_to_base64<P: AsRef<Path>>(path: P) -> Result<String> {
    let bytes = fs::read(path)?;
    use base64::{engine::general_purpose, Engine as _};
    Ok(general_purpose::STANDARD.encode(&bytes))
}

pub fn save_base64_to_image<P: AsRef<Path>>(b64: &str, path: P) -> Result<()> {
    use base64::{engine::general_purpose, Engine as _};
    let clean_b64 = b64.split(',').next_back().unwrap_or(b64);
    let bytes = general_purpose::STANDARD.decode(clean_b64)?;
    fs::write(path, bytes)?;
    Ok(())
}
