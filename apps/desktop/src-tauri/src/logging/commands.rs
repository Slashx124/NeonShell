use super::{
    get_log_manager, sanitize, AppInfo, DebugBundleOptions, LogFilter, LogLine, LogLevel,
    LogSubsystem, MAX_BUNDLE_SIZE,
};
use crate::config::get_config_dir;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tauri::State;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

/// Maximum lines to include in debug bundle
const MAX_BUNDLE_LINES: u32 = 10_000;

/// Get recent logs from the ring buffer
#[tauri::command]
pub async fn get_recent_logs(
    max_lines: Option<u32>,
    filter: Option<LogFilter>,
) -> AppResult<Vec<LogLine>> {
    let manager = get_log_manager()
        .ok_or_else(|| AppError::Config("Log manager not initialized".to_string()))?;

    let lines = max_lines.unwrap_or(1000).min(MAX_BUNDLE_LINES);
    Ok(manager.get_recent_logs(lines, filter))
}

/// Clear the in-memory log view (does not delete file logs)
#[tauri::command]
pub async fn clear_log_view() -> AppResult<()> {
    let manager = get_log_manager()
        .ok_or_else(|| AppError::Config("Log manager not initialized".to_string()))?;

    manager.clear_view();
    Ok(())
}

/// Export a debug bundle as a zip file
#[tauri::command]
pub async fn export_debug_bundle(
    state: State<'_, Arc<AppState>>,
    path: String,
    options: Option<DebugBundleOptions>,
) -> AppResult<String> {
    let options = options.unwrap_or_default();
    let manager = get_log_manager()
        .ok_or_else(|| AppError::Config("Log manager not initialized".to_string()))?;

    // Validate path
    let export_path = validate_bundle_path(&path)?;

    // Collect bundle data
    let max_lines = options.max_lines.unwrap_or(MAX_BUNDLE_LINES).min(MAX_BUNDLE_LINES);
    let logs = manager.get_recent_logs(max_lines, None);
    let app_info = AppInfo::collect();

    // Create zip file
    let file = File::create(&export_path)
        .map_err(|e| AppError::Io(e))?;
    let mut zip = ZipWriter::new(file);

    let zip_options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // 1. Write logs.jsonl
    zip.start_file("logs.jsonl", zip_options)
        .map_err(|e| AppError::Config(format!("Failed to create logs.jsonl: {}", e)))?;
    
    let mut total_size: u64 = 0;
    for log in &logs {
        if let Ok(json) = serde_json::to_string(log) {
            let line = format!("{}\n", json);
            total_size += line.len() as u64;
            if total_size > MAX_BUNDLE_SIZE {
                tracing::warn!("Debug bundle size limit reached, truncating logs");
                break;
            }
            zip.write_all(line.as_bytes())
                .map_err(|e| AppError::Io(e))?;
        }
    }

    // 2. Write app_info.json
    zip.start_file("app_info.json", zip_options)
        .map_err(|e| AppError::Config(format!("Failed to create app_info.json: {}", e)))?;
    let app_info_json = serde_json::to_string_pretty(&app_info)
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    zip.write_all(app_info_json.as_bytes())
        .map_err(|e| AppError::Io(e))?;

    // 3. Write config_snapshot.json (sanitized)
    if options.include_config.unwrap_or(true) {
        zip.start_file("config_snapshot.json", zip_options)
            .map_err(|e| AppError::Config(format!("Failed to create config_snapshot.json: {}", e)))?;
        
        let config_snapshot = build_sanitized_config_snapshot()?;
        let config_json = serde_json::to_string_pretty(&config_snapshot)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        zip.write_all(config_json.as_bytes())
            .map_err(|e| AppError::Io(e))?;
    }

    // 4. Write ssh_sessions.json (if requested)
    if options.include_sessions.unwrap_or(true) {
        zip.start_file("ssh_sessions.json", zip_options)
            .map_err(|e| AppError::Config(format!("Failed to create ssh_sessions.json: {}", e)))?;
        
        let sessions = build_session_snapshot(&state, options.redact_hostnames.unwrap_or(false));
        let sessions_json = serde_json::to_string_pretty(&sessions)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        zip.write_all(sessions_json.as_bytes())
            .map_err(|e| AppError::Io(e))?;
    }

    // 5. Write plugins_themes.json (if requested)
    if options.include_plugins.unwrap_or(true) {
        zip.start_file("plugins_themes.json", zip_options)
            .map_err(|e| AppError::Config(format!("Failed to create plugins_themes.json: {}", e)))?;
        
        let plugins_themes = build_plugins_themes_snapshot()?;
        let pt_json = serde_json::to_string_pretty(&plugins_themes)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        zip.write_all(pt_json.as_bytes())
            .map_err(|e| AppError::Io(e))?;
    }

    // 6. Write README.txt
    zip.start_file("README.txt", zip_options)
        .map_err(|e| AppError::Config(format!("Failed to create README.txt: {}", e)))?;
    let readme = r#"NeonShell Debug Bundle
======================

This bundle contains sanitized debug information for troubleshooting.

Contents:
- logs.jsonl: Recent application logs (sanitized)
- app_info.json: Application version and system information
- config_snapshot.json: Settings snapshot (secrets redacted)
- ssh_sessions.json: Active session states (no credentials)
- plugins_themes.json: Installed plugins and themes list

PRIVACY NOTICE:
This bundle has been automatically sanitized to remove:
- Passwords and passphrases
- Private keys
- API tokens
- Authorization headers

However, please review the contents before sharing. Hostnames and
usernames may still be present unless you opted to redact them.

To submit this bundle:
1. Create an issue at: https://github.com/yourorg/neonshell/issues/new
2. Attach this zip file to the issue
3. Describe the problem you encountered

Or email to: support@neonshell.dev
"#;
    zip.write_all(readme.as_bytes())
        .map_err(|e| AppError::Io(e))?;

    // Finish zip
    zip.finish()
        .map_err(|e| AppError::Config(format!("Failed to finalize zip: {}", e)))?;

    tracing::info!("Debug bundle exported to: {}", export_path.display());
    super::log(
        LogLevel::Info,
        LogSubsystem::App,
        format!("Debug bundle exported with {} log entries", logs.len()),
    );

    Ok(export_path.to_string_lossy().to_string())
}

/// Validate the bundle export path
fn validate_bundle_path(path: &str) -> AppResult<std::path::PathBuf> {
    let path = std::path::Path::new(path);
    
    // Must end with .zip
    if path.extension().map(|e| e.to_str()) != Some(Some("zip")) {
        return Err(AppError::InvalidConfig("Bundle path must end with .zip".to_string()));
    }

    // Check for path traversal
    let canonical = if path.exists() {
        path.canonicalize()
            .map_err(|e| AppError::Io(e))?
    } else {
        // For new files, canonicalize parent
        let parent = path.parent()
            .ok_or_else(|| AppError::InvalidConfig("Invalid path".to_string()))?;
        
        if !parent.exists() {
            return Err(AppError::InvalidConfig("Parent directory does not exist".to_string()));
        }
        
        let canonical_parent = parent.canonicalize()
            .map_err(|e| AppError::Io(e))?;
        
        let file_name = path.file_name()
            .ok_or_else(|| AppError::InvalidConfig("Invalid filename".to_string()))?;
        
        canonical_parent.join(file_name)
    };

    // Don't allow writing to system directories
    let path_str = canonical.to_string_lossy().to_lowercase();
    let forbidden = ["windows", "system32", "program files", "/usr", "/bin", "/etc", "/var"];
    for fb in forbidden {
        if path_str.contains(fb) {
            return Err(AppError::PermissionDenied("Cannot write to system directory".to_string()));
        }
    }

    Ok(canonical)
}

/// Build sanitized config snapshot
fn build_sanitized_config_snapshot() -> AppResult<serde_json::Value> {
    let config_dir = get_config_dir()?;
    let settings_path = config_dir.join("settings.toml");
    
    let mut snapshot = serde_json::json!({
        "settings_exists": settings_path.exists(),
        "config_dir": sanitize(&config_dir.to_string_lossy()),
    });

    if settings_path.exists() {
        // Read and parse settings
        if let Ok(contents) = fs::read_to_string(&settings_path) {
            if let Ok(settings) = toml::from_str::<serde_json::Value>(&contents) {
                // Sanitize the settings
                snapshot["settings"] = super::sanitize_json(&settings);
            }
        }
    }

    Ok(snapshot)
}

/// Build session snapshot (no credentials)
fn build_session_snapshot(state: &State<'_, Arc<AppState>>, redact_hostnames: bool) -> serde_json::Value {
    let sessions = state.sessions.list_sessions();
    
    let sanitized_sessions: Vec<serde_json::Value> = sessions
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "host": if redact_hostnames { "[REDACTED]".to_string() } else { s.host },
                "port": s.port,
                "username": if redact_hostnames { "[REDACTED]".to_string() } else { s.username },
                "state": format!("{:?}", s.state),
                "connected_at": s.connected_at,
            })
        })
        .collect();

    serde_json::json!({
        "active_sessions": sanitized_sessions,
        "session_count": sanitized_sessions.len(),
    })
}

/// Build plugins and themes snapshot
fn build_plugins_themes_snapshot() -> AppResult<serde_json::Value> {
    let config_dir = get_config_dir()?;
    
    // List plugins
    let plugins_dir = config_dir.join("plugins");
    let plugins: Vec<String> = if plugins_dir.exists() {
        fs::read_dir(&plugins_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    // List themes
    let themes_dir = config_dir.join("themes");
    let themes: Vec<String> = if themes_dir.exists() {
        fs::read_dir(&themes_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    Ok(serde_json::json!({
        "plugins": plugins,
        "plugins_count": plugins.len(),
        "themes": themes,
        "themes_count": themes.len(),
    }))
}

/// Get the logs directory path
#[tauri::command]
pub async fn get_logs_dir() -> AppResult<String> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("logs").to_string_lossy().to_string())
}

/// Reveal a path in the system file explorer
#[tauri::command]
pub async fn reveal_in_explorer(path: String) -> AppResult<()> {
    let path = Path::new(&path);
    
    if !path.exists() {
        return Err(AppError::InvalidConfig("Path does not exist".to_string()));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::Io(e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::Io(e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try xdg-open on the parent directory
        if let Some(parent) = path.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| AppError::Io(e))?;
        }
    }

    Ok(())
}

