//! SFTP Tauri commands

use super::{SftpEntry, SftpManager};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// SFTP list directory request
#[derive(Debug, Deserialize)]
pub struct SftpListRequest {
    pub profile_id: String,
    pub path: String,
}

/// SFTP list directory response
#[derive(Debug, Serialize)]
pub struct SftpListResponse {
    pub entries: Vec<SftpEntry>,
    pub current_path: String,
}

/// SFTP file operation request
#[derive(Debug, Deserialize)]
pub struct SftpFileRequest {
    pub profile_id: String,
    pub path: String,
}

/// SFTP rename request
#[derive(Debug, Deserialize)]
pub struct SftpRenameRequest {
    pub profile_id: String,
    pub from_path: String,
    pub to_path: String,
}

/// SFTP upload request
#[derive(Debug, Deserialize)]
pub struct SftpUploadRequest {
    pub profile_id: String,
    pub remote_path: String,
    pub contents: Vec<u8>,
}

/// List directory contents via SFTP
#[tauri::command]
pub async fn sftp_list(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
    path: String,
) -> AppResult<SftpListResponse> {
    tracing::info!("SFTP list: profile={}, path={}", profile_id, path);
    
    // Get the profile
    let profile = state
        .profiles
        .read()
        .get(&profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.clone()))?;

    // Connect SFTP
    let conn = SftpManager::connect_from_profile(&profile)?;
    
    // Determine the path to list
    let list_path = if path.is_empty() {
        conn.home_dir()?
    } else {
        path.clone()
    };

    // List directory
    let entries = conn.list_dir(&list_path)?;
    
    // Get the actual resolved path
    let current_path = conn.sftp.realpath(std::path::Path::new(&list_path))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(list_path);

    Ok(SftpListResponse {
        entries,
        current_path,
    })
}

/// Get file/directory info via SFTP
#[tauri::command]
pub async fn sftp_stat(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
    path: String,
) -> AppResult<SftpEntry> {
    tracing::info!("SFTP stat: profile={}, path={}", profile_id, path);
    
    let profile = state
        .profiles
        .read()
        .get(&profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.clone()))?;

    let conn = SftpManager::connect_from_profile(&profile)?;
    conn.stat(&path)
}

/// Download a file via SFTP
#[tauri::command]
pub async fn sftp_download(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
    path: String,
) -> AppResult<Vec<u8>> {
    tracing::info!("SFTP download: profile={}, path={}", profile_id, path);
    
    let profile = state
        .profiles
        .read()
        .get(&profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.clone()))?;

    let conn = SftpManager::connect_from_profile(&profile)?;
    conn.download(&path)
}

/// Upload a file via SFTP
#[tauri::command]
pub async fn sftp_upload(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
    remote_path: String,
    contents: Vec<u8>,
) -> AppResult<()> {
    tracing::info!("SFTP upload: profile={}, path={}, size={}", profile_id, remote_path, contents.len());
    
    let profile = state
        .profiles
        .read()
        .get(&profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.clone()))?;

    let conn = SftpManager::connect_from_profile(&profile)?;
    conn.upload(&remote_path, &contents)
}

/// Create a directory via SFTP
#[tauri::command]
pub async fn sftp_mkdir(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
    path: String,
) -> AppResult<()> {
    tracing::info!("SFTP mkdir: profile={}, path={}", profile_id, path);
    
    let profile = state
        .profiles
        .read()
        .get(&profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.clone()))?;

    let conn = SftpManager::connect_from_profile(&profile)?;
    conn.mkdir(&path)
}

/// Delete a file via SFTP
#[tauri::command]
pub async fn sftp_delete(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
    path: String,
    is_dir: bool,
) -> AppResult<()> {
    tracing::info!("SFTP delete: profile={}, path={}, is_dir={}", profile_id, path, is_dir);
    
    let profile = state
        .profiles
        .read()
        .get(&profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.clone()))?;

    let conn = SftpManager::connect_from_profile(&profile)?;
    
    if is_dir {
        conn.delete_dir(&path)
    } else {
        conn.delete_file(&path)
    }
}

/// Rename/move a file or directory via SFTP
#[tauri::command]
pub async fn sftp_rename(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
    from_path: String,
    to_path: String,
) -> AppResult<()> {
    tracing::info!("SFTP rename: profile={}, from={}, to={}", profile_id, from_path, to_path);
    
    let profile = state
        .profiles
        .read()
        .get(&profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.clone()))?;

    let conn = SftpManager::connect_from_profile(&profile)?;
    conn.rename(&from_path, &to_path)
}

/// Get home directory path via SFTP  
#[tauri::command]
pub async fn sftp_home(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
) -> AppResult<String> {
    tracing::info!("SFTP home: profile={}", profile_id);
    
    let profile = state
        .profiles
        .read()
        .get(&profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.clone()))?;

    let conn = SftpManager::connect_from_profile(&profile)?;
    conn.home_dir()
}

