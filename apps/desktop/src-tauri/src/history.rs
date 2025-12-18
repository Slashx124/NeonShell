//! Terminal scrollback history persistence
//! 
//! SECURITY: This stores terminal output which could contain sensitive data.
//! - Files are stored in the user's config directory
//! - Each session has its own history file
//! - History is keyed by profile_id (not session_id) so reconnections restore history
//! - Maximum file size is capped to prevent disk abuse
//! - History can be cleared by the user

use crate::error::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;
use std::io::{Read, Write};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;

/// Maximum scrollback history file size (compressed) - 5 MB
const MAX_HISTORY_SIZE: u64 = 5 * 1024 * 1024;

/// Maximum uncompressed history size - 50 MB
const MAX_UNCOMPRESSED_SIZE: usize = 50 * 1024 * 1024;

/// Get the history directory path
fn get_history_dir() -> AppResult<PathBuf> {
    let config_dir = crate::config::get_config_dir()?;
    let history_dir = config_dir.join("history");
    fs::create_dir_all(&history_dir)?;
    Ok(history_dir)
}

/// Sanitize profile ID to be safe as filename
fn sanitize_filename(profile_id: &str) -> AppResult<String> {
    // SECURITY: Only allow alphanumeric, hyphen, underscore
    if profile_id.is_empty() || profile_id.len() > 64 {
        return Err(AppError::InvalidConfig("Invalid profile ID length".to_string()));
    }
    
    if !profile_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::InvalidConfig("Invalid profile ID characters".to_string()));
    }
    
    Ok(format!("{}.history.gz", profile_id))
}

/// Get history file path for a profile
fn get_history_path(profile_id: &str) -> AppResult<PathBuf> {
    let history_dir = get_history_dir()?;
    let filename = sanitize_filename(profile_id)?;
    Ok(history_dir.join(filename))
}

/// Save terminal scrollback history for a profile
/// 
/// SECURITY: This stores raw terminal output. Users should be aware this may contain
/// sensitive information visible in their terminal session.
pub fn save_history(profile_id: &str, data: &[u8]) -> AppResult<()> {
    // SECURITY: Cap uncompressed size
    if data.len() > MAX_UNCOMPRESSED_SIZE {
        tracing::warn!(
            "History too large for profile {} ({} bytes), truncating", 
            profile_id, 
            data.len()
        );
        // Save only the last MAX_UNCOMPRESSED_SIZE bytes
        let truncated = &data[data.len() - MAX_UNCOMPRESSED_SIZE..];
        return save_history_internal(profile_id, truncated);
    }
    
    save_history_internal(profile_id, data)
}

fn save_history_internal(profile_id: &str, data: &[u8]) -> AppResult<()> {
    let path = get_history_path(profile_id)?;
    
    // Compress the data
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)
        .map_err(|e| AppError::Io(e))?;
    let compressed = encoder.finish()
        .map_err(|e| AppError::Io(e))?;
    
    // SECURITY: Check compressed size before writing
    if compressed.len() as u64 > MAX_HISTORY_SIZE {
        return Err(AppError::Config("Compressed history too large".to_string()));
    }
    
    // Write atomically using temp file
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &compressed)?;
    fs::rename(&temp_path, &path)?;
    
    tracing::debug!("Saved {} bytes of history for profile {}", compressed.len(), profile_id);
    Ok(())
}

/// Load terminal scrollback history for a profile
pub fn load_history(profile_id: &str) -> AppResult<Option<Vec<u8>>> {
    let path = get_history_path(profile_id)?;
    
    if !path.exists() {
        return Ok(None);
    }
    
    // SECURITY: Check file size before reading
    let metadata = fs::metadata(&path)?;
    if metadata.len() > MAX_HISTORY_SIZE {
        tracing::warn!("History file too large for profile {}, skipping", profile_id);
        return Ok(None);
    }
    
    // Read and decompress
    let compressed = fs::read(&path)?;
    let decoder = GzDecoder::new(&compressed[..]);
    let mut data = Vec::new();
    
    // SECURITY: Limit decompression size
    let mut limited_decoder = decoder.take(MAX_UNCOMPRESSED_SIZE as u64);
    limited_decoder.read_to_end(&mut data)
        .map_err(|e| AppError::Config(format!("Failed to decompress history: {}", e)))?;
    
    tracing::debug!("Loaded {} bytes of history for profile {}", data.len(), profile_id);
    Ok(Some(data))
}

/// Clear history for a profile
pub fn clear_history(profile_id: &str) -> AppResult<()> {
    let path = get_history_path(profile_id)?;
    
    if path.exists() {
        fs::remove_file(&path)?;
        tracing::info!("Cleared history for profile {}", profile_id);
    }
    
    Ok(())
}

/// Clear all history
pub fn clear_all_history() -> AppResult<()> {
    let history_dir = get_history_dir()?;
    
    if history_dir.exists() {
        for entry in fs::read_dir(&history_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "gz") {
                let _ = fs::remove_file(&path);
            }
        }
        tracing::info!("Cleared all terminal history");
    }
    
    Ok(())
}

/// List all profiles with saved history
pub fn list_history_profiles() -> AppResult<Vec<String>> {
    let history_dir = get_history_dir()?;
    let mut profiles = Vec::new();
    
    if history_dir.exists() {
        for entry in fs::read_dir(&history_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                if name.ends_with(".history.gz") {
                    let profile_id = name.trim_end_matches(".history.gz");
                    profiles.push(profile_id.to_string());
                }
            }
        }
    }
    
    Ok(profiles)
}

// =============================================================================
// Tauri commands
// =============================================================================

pub mod commands {
    use super::*;
    use crate::error::AppResult;
    use tauri::State;
    use crate::state::AppState;
    use std::sync::Arc;

    /// Save terminal history for a profile
    #[tauri::command]
    pub async fn save_terminal_history(
        _state: State<'_, Arc<AppState>>,
        profile_id: String,
        data: Vec<u8>,
    ) -> AppResult<()> {
        save_history(&profile_id, &data)
    }

    /// Load terminal history for a profile
    #[tauri::command]
    pub async fn load_terminal_history(
        _state: State<'_, Arc<AppState>>,
        profile_id: String,
    ) -> AppResult<Option<Vec<u8>>> {
        load_history(&profile_id)
    }

    /// Clear terminal history for a profile
    #[tauri::command]
    pub async fn clear_terminal_history(
        _state: State<'_, Arc<AppState>>,
        profile_id: String,
    ) -> AppResult<()> {
        clear_history(&profile_id)
    }

    /// Clear all terminal history
    #[tauri::command]
    pub async fn clear_all_terminal_history(
        _state: State<'_, Arc<AppState>>,
    ) -> AppResult<()> {
        clear_all_history()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert!(sanitize_filename("valid-profile_123").is_ok());
        assert!(sanitize_filename("").is_err());
        assert!(sanitize_filename("../../../etc/passwd").is_err());
        assert!(sanitize_filename("profile/test").is_err());
    }
}

