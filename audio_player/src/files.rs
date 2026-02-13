use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn list_audio_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("Failed to read {:?}", dir))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_audio_file(&path) {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

pub fn is_audio_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
        Some(ext) => matches!(ext.as_str(), "mp3" | "wav" | "flac" | "ogg" | "m4a"),
        None => false,
    }
}
