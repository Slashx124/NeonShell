use super::{
    export_openssh_config, parse_openssh_config, AppSettings, NeonPack, Profile, ThemeManager, Theme,
};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use tauri::State;

// =============================================================================
// SECURITY: Path validation utilities
// =============================================================================

/// Validate that a path is safe and within allowed directories
/// Returns the canonicalized path if valid
fn validate_export_path(path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(path);
    
    // SECURITY: Reject obviously malicious patterns
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        return Err(AppError::Config("Path traversal not allowed".to_string()));
    }
    
    // SECURITY: Must have .zip extension
    match path.extension() {
        Some(ext) if ext == "zip" => {}
        _ => return Err(AppError::Config("Export file must have .zip extension".to_string())),
    }
    
    // SECURITY: Parent directory must exist (prevents creating arbitrary directories)
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(AppError::Config("Parent directory does not exist".to_string()));
        }
    }
    
    Ok(path)
}

/// Validate that an import path is safe
fn validate_import_path(path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(path);
    
    // SECURITY: File must exist
    if !path.exists() {
        return Err(AppError::Config("File not found".to_string()));
    }
    
    // SECURITY: Must be a file, not a directory or symlink to directory
    if !path.is_file() {
        return Err(AppError::Config("Path must be a file".to_string()));
    }
    
    // SECURITY: Must have .zip extension
    match path.extension() {
        Some(ext) if ext == "zip" => {}
        _ => return Err(AppError::Config("Import file must have .zip extension".to_string())),
    }
    
    Ok(path)
}

/// Sanitize a theme/plugin ID to prevent path traversal
/// Only allows alphanumeric, hyphen, and underscore
fn sanitize_id(id: &str) -> AppResult<String> {
    // SECURITY: Reject path traversal attempts
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err(AppError::Config(format!(
            "Invalid ID '{}': contains path traversal characters",
            id.chars().take(50).collect::<String>()
        )));
    }
    
    // SECURITY: Only allow safe characters
    if id.is_empty() || id.len() > 64 {
        return Err(AppError::Config("ID must be 1-64 characters".to_string()));
    }
    
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::Config(format!(
            "Invalid ID '{}': only alphanumeric, hyphen, and underscore allowed",
            id.chars().take(50).collect::<String>()
        )));
    }
    
    Ok(id.to_string())
}

/// Validate that a path stays within a base directory (prevents zip slip)
fn validate_path_within_base(base: &Path, relative: &str) -> AppResult<PathBuf> {
    // SECURITY: Reject obvious traversal
    if relative.contains("..") {
        return Err(AppError::Config("Path traversal not allowed in archive".to_string()));
    }
    
    let full_path = base.join(relative);
    
    // SECURITY: Canonicalize and verify it's still under base
    // Note: We can't canonicalize non-existent paths, so we normalize manually
    let normalized = normalize_path(&full_path);
    let base_normalized = normalize_path(base);
    
    if !normalized.starts_with(&base_normalized) {
        return Err(AppError::Config("Path escape attempt detected".to_string()));
    }
    
    Ok(full_path)
}

/// Normalize a path without requiring it to exist
fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            _ => normalized.push(component),
        }
    }
    normalized
}

// Profile commands

#[tauri::command]
pub async fn list_profiles(state: State<'_, Arc<AppState>>) -> AppResult<Vec<Profile>> {
    Ok(state.profiles.read().list())
}

#[tauri::command]
pub async fn get_profile(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> AppResult<Profile> {
    state
        .profiles
        .read()
        .get(&id)
        .ok_or_else(|| AppError::ProfileNotFound(id))
}

#[tauri::command]
pub async fn save_profile(
    state: State<'_, Arc<AppState>>,
    profile: Profile,
    is_new: bool,
) -> AppResult<()> {
    let mut profiles = state.profiles.write();
    if is_new {
        profiles.add(profile)
    } else {
        profiles.update(profile)
    }
}

#[tauri::command]
pub async fn delete_profile(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> AppResult<()> {
    state.profiles.write().delete(&id)
}

#[tauri::command]
pub async fn import_ssh_config(
    state: State<'_, Arc<AppState>>,
    content: String,
) -> AppResult<Vec<Profile>> {
    let profiles = parse_openssh_config(&content);
    let mut manager = state.profiles.write();
    for profile in &profiles {
        manager.add(profile.clone())?;
    }
    Ok(profiles)
}

#[tauri::command]
pub async fn export_ssh_config(
    state: State<'_, Arc<AppState>>,
) -> AppResult<String> {
    let profiles = state.profiles.read().list();
    Ok(export_openssh_config(&profiles))
}

// Settings commands

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> AppResult<AppSettings> {
    Ok(state.settings.read().clone())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> AppResult<()> {
    let config_dir = super::get_config_dir()?;
    settings.save(&config_dir)?;
    *state.settings.write() = settings;
    Ok(())
}

// Theme commands

#[tauri::command]
pub async fn list_themes(_state: State<'_, Arc<AppState>>) -> AppResult<Vec<Theme>> {
    let config_dir = super::get_config_dir()?;
    let manager = ThemeManager::load(&config_dir)?;
    Ok(manager.list())
}

#[tauri::command]
pub async fn get_theme(
    _state: State<'_, Arc<AppState>>,
    id: String,
) -> AppResult<Theme> {
    let config_dir = super::get_config_dir()?;
    let manager = ThemeManager::load(&config_dir)?;
    manager
        .get(&id)
        .ok_or_else(|| AppError::Config(format!("Theme not found: {}", id)))
}

#[tauri::command]
pub async fn set_theme(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> AppResult<()> {
    let config_dir = super::get_config_dir()?;
    let manager = ThemeManager::load(&config_dir)?;
    
    // Verify theme exists
    if manager.get(&id).is_none() {
        return Err(AppError::Config(format!("Theme not found: {}", id)));
    }
    
    // Update settings
    {
        let mut settings = state.settings.write();
        settings.general.theme = id;
        settings.save(&config_dir)?;
    }
    
    Ok(())
}

// =============================================================================
// Theme Import from ZIP - with comprehensive validation
// =============================================================================

/// Result of theme import validation
#[derive(serde::Serialize)]
pub struct ThemeImportResult {
    pub success: bool,
    pub theme_id: Option<String>,
    pub theme_name: Option<String>,
    pub error: Option<String>,
}

/// Validate a color string is a valid hex color
fn validate_color(color: &str, field_name: &str) -> AppResult<()> {
    // Allow empty for optional fields
    if color.is_empty() {
        return Ok(());
    }
    
    // Must start with #
    if !color.starts_with('#') {
        return Err(AppError::Config(format!(
            "Invalid color for '{}': must start with #",
            field_name
        )));
    }
    
    let hex_part = &color[1..];
    
    // Must be 3, 4, 6, or 8 hex digits (RGB, RGBA, RRGGBB, RRGGBBAA)
    let valid_len = matches!(hex_part.len(), 3 | 4 | 6 | 8);
    let all_hex = hex_part.chars().all(|c| c.is_ascii_hexdigit());
    
    if !valid_len || !all_hex {
        return Err(AppError::Config(format!(
            "Invalid color for '{}': must be a valid hex color (e.g., #ff0080)",
            field_name
        )));
    }
    
    Ok(())
}

/// Validate theme structure and colors
fn validate_theme_structure(theme: &Theme) -> AppResult<()> {
    // Validate required fields
    if theme.id.is_empty() {
        return Err(AppError::Config("Theme missing 'id' field".to_string()));
    }
    if theme.name.is_empty() {
        return Err(AppError::Config("Theme missing 'name' field".to_string()));
    }
    
    // Validate ID format
    sanitize_id(&theme.id)?;
    
    // Validate required colors
    validate_color(&theme.colors.background, "colors.background")?;
    validate_color(&theme.colors.foreground, "colors.foreground")?;
    validate_color(&theme.colors.accent, "colors.accent")?;
    
    // Validate optional colors
    validate_color(&theme.colors.accent_muted, "colors.accent_muted")?;
    validate_color(&theme.colors.surface_0, "colors.surface_0")?;
    validate_color(&theme.colors.surface_1, "colors.surface_1")?;
    validate_color(&theme.colors.surface_2, "colors.surface_2")?;
    validate_color(&theme.colors.surface_3, "colors.surface_3")?;
    validate_color(&theme.colors.border, "colors.border")?;
    validate_color(&theme.colors.cursor, "colors.cursor")?;
    validate_color(&theme.colors.selection, "colors.selection")?;
    validate_color(&theme.colors.error, "colors.error")?;
    validate_color(&theme.colors.warning, "colors.warning")?;
    validate_color(&theme.colors.success, "colors.success")?;
    
    // Validate ANSI colors
    let ansi = &theme.terminal.ansi_colors;
    validate_color(&ansi.black, "terminal.ansi_colors.black")?;
    validate_color(&ansi.red, "terminal.ansi_colors.red")?;
    validate_color(&ansi.green, "terminal.ansi_colors.green")?;
    validate_color(&ansi.yellow, "terminal.ansi_colors.yellow")?;
    validate_color(&ansi.blue, "terminal.ansi_colors.blue")?;
    validate_color(&ansi.magenta, "terminal.ansi_colors.magenta")?;
    validate_color(&ansi.cyan, "terminal.ansi_colors.cyan")?;
    validate_color(&ansi.white, "terminal.ansi_colors.white")?;
    validate_color(&ansi.bright_black, "terminal.ansi_colors.bright_black")?;
    validate_color(&ansi.bright_red, "terminal.ansi_colors.bright_red")?;
    validate_color(&ansi.bright_green, "terminal.ansi_colors.bright_green")?;
    validate_color(&ansi.bright_yellow, "terminal.ansi_colors.bright_yellow")?;
    validate_color(&ansi.bright_blue, "terminal.ansi_colors.bright_blue")?;
    validate_color(&ansi.bright_magenta, "terminal.ansi_colors.bright_magenta")?;
    validate_color(&ansi.bright_cyan, "terminal.ansi_colors.bright_cyan")?;
    validate_color(&ansi.bright_white, "terminal.ansi_colors.bright_white")?;
    
    // Validate font size is reasonable
    if theme.terminal.font_size < 6 || theme.terminal.font_size > 72 {
        return Err(AppError::Config(
            "Font size must be between 6 and 72".to_string()
        ));
    }
    
    // Validate CSS file name if present (no path traversal)
    if let Some(css_file) = &theme.css_file {
        if css_file.contains("..") || css_file.contains('/') || css_file.contains('\\') {
            return Err(AppError::Config(
                "CSS filename cannot contain path separators".to_string()
            ));
        }
        if !css_file.ends_with(".css") {
            return Err(AppError::Config(
                "CSS file must have .css extension".to_string()
            ));
        }
    }
    
    Ok(())
}

/// Import a theme from a ZIP file
#[tauri::command]
pub async fn import_theme_zip(
    state: State<'_, Arc<AppState>>,
    path: String,
) -> AppResult<ThemeImportResult> {
    // SECURITY: Validate the import path
    let validated_path = validate_import_path(&path)?;
    
    let config_dir = super::get_config_dir()?;
    let themes_dir = config_dir.join("themes");
    std::fs::create_dir_all(&themes_dir)?;
    
    // Open and read the zip file
    let file = std::fs::File::open(&validated_path)
        .map_err(|e| AppError::Config(format!("Failed to open file: {}", e)))?;
    
    // SECURITY: Check file size before processing
    let file_size = file.metadata()
        .map_err(|e| AppError::Config(format!("Failed to read file metadata: {}", e)))?
        .len();
    
    const MAX_THEME_ZIP_SIZE: u64 = 10 * 1024 * 1024; // 10 MB max
    if file_size > MAX_THEME_ZIP_SIZE {
        return Err(AppError::Config(format!(
            "Theme file too large: {} bytes (max {} bytes)",
            file_size, MAX_THEME_ZIP_SIZE
        )));
    }
    
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Config(format!("Invalid ZIP file: {}", e)))?;
    
    // SECURITY: Limit number of files in archive
    const MAX_THEME_FILES: usize = 20;
    if archive.len() > MAX_THEME_FILES {
        return Err(AppError::Config(format!(
            "Theme ZIP contains too many files ({} > {})",
            archive.len(), MAX_THEME_FILES
        )));
    }
    
    // First pass: find the index of theme.json
    let mut theme_json_index: Option<usize> = None;
    
    for i in 0..archive.len() {
        let file = archive.by_index(i)
            .map_err(|e| AppError::Config(format!("Failed to read archive: {}", e)))?;
        
        let name = file.name().to_string();
        
        // Look for theme.json at root or in a single subdirectory
        if name == "theme.json" || name.ends_with("/theme.json") {
            let depth = name.matches('/').count();
            if depth <= 1 {
                // SECURITY: Limit theme.json size
                if file.size() > 100 * 1024 { // 100 KB max
                    return Err(AppError::Config("theme.json too large".to_string()));
                }
                theme_json_index = Some(i);
                break;
            }
        }
    }
    
    let theme_json_idx = theme_json_index.ok_or_else(|| {
        AppError::Config("ZIP file must contain a theme.json file".to_string())
    })?;
    
    // Second pass: read theme.json content
    let mut theme_content = String::new();
    {
        let mut file = archive.by_index(theme_json_idx)
            .map_err(|e| AppError::Config(format!("Failed to read archive: {}", e)))?;
        
        file.read_to_string(&mut theme_content)
            .map_err(|e| AppError::Config(format!("Failed to read theme.json: {}", e)))?;
    }
    
    // Parse and validate theme
    let theme: Theme = serde_json::from_str(&theme_content)
        .map_err(|e| AppError::Config(format!("Invalid theme.json: {}", e)))?;
    
    // SECURITY: Validate theme structure and values
    validate_theme_structure(&theme)?;
    
    // Create sanitized theme directory
    let safe_theme_id = sanitize_id(&theme.id)?;
    let theme_dest_dir = validate_path_within_base(&themes_dir, &safe_theme_id)?;
    
    // Check if theme already exists
    if theme_dest_dir.exists() {
        return Err(AppError::Config(format!(
            "Theme '{}' already exists. Delete it first to reimport.",
            theme.name
        )));
    }
    
    std::fs::create_dir_all(&theme_dest_dir)?;
    
    // Write validated theme.json
    let validated_theme_json = serde_json::to_string_pretty(&theme)?;
    std::fs::write(theme_dest_dir.join("theme.json"), validated_theme_json)?;
    
    // Extract CSS file if referenced
    if let Some(css_filename) = &theme.css_file {
        // Reopen archive to extract CSS
        let file = std::fs::File::open(&validated_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        // First find the CSS file index
        let mut css_file_index: Option<usize> = None;
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            // Match the CSS file (at root or one level deep)
            if name == css_filename.as_str() || name.ends_with(&format!("/{}", css_filename)) {
                // SECURITY: Limit CSS file size
                if file.size() > 500 * 1024 { // 500 KB max
                    return Err(AppError::Config("CSS file too large".to_string()));
                }
                css_file_index = Some(i);
                break;
            }
        }
        
        // Then read the content if found
        if let Some(idx) = css_file_index {
            let mut file = archive.by_index(idx)?;
            let mut css_content = String::new();
            file.read_to_string(&mut css_content)
                .map_err(|e| AppError::Config(format!("Failed to read CSS file: {}", e)))?;
            
            // SECURITY: Basic CSS validation - check for dangerous patterns
            let css_lower = css_content.to_lowercase();
            if css_lower.contains("javascript:") || 
               css_lower.contains("expression(") ||
               css_lower.contains("behavior:") ||
               css_lower.contains("-moz-binding") {
                // Clean up and fail
                let _ = std::fs::remove_dir_all(&theme_dest_dir);
                return Err(AppError::Config(
                    "CSS contains potentially dangerous content".to_string()
                ));
            }
            
            // Write CSS file
            let css_dest = theme_dest_dir.join(css_filename);
            std::fs::write(css_dest, css_content)?;
        }
    }
    
    tracing::info!("Imported theme: {}", safe_theme_id);
    
    // Optionally set as active theme
    {
        let mut settings = state.settings.write();
        settings.general.theme = safe_theme_id.clone();
        settings.save(&config_dir)?;
    }
    
    Ok(ThemeImportResult {
        success: true,
        theme_id: Some(safe_theme_id),
        theme_name: Some(theme.name),
        error: None,
    })
}

// =============================================================================
// Pack export/import - with security validation
// =============================================================================

#[tauri::command]
pub async fn export_pack(
    state: State<'_, Arc<AppState>>,
    path: String,
) -> AppResult<()> {
    // SECURITY: Validate the export path
    let validated_path = validate_export_path(&path)?;
    
    let config_dir = super::get_config_dir()?;
    let settings = state.settings.read().clone();
    
    // Build the pack
    let mut pack = NeonPack {
        version: "1.0".to_string(),
        name: "NeonShell Pack".to_string(),
        description: "Exported NeonShell settings and theme".to_string(),
        theme: None,
        layout: None,
        hotkeys: None,
        snippets: None,
    };
    
    // Include current theme
    let manager = ThemeManager::load(&config_dir)?;
    pack.theme = manager.get(&settings.general.theme);
    
    let manifest_json = serde_json::to_string_pretty(&pack)?;
    
    // Write to the validated zip file path
    let file = std::fs::File::create(&validated_path)
        .map_err(|e| AppError::Config(format!("Failed to create file: {}", e)))?;
    
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    // Add manifest
    zip.start_file("manifest.json", options)
        .map_err(|e| AppError::Config(format!("Failed to write zip: {}", e)))?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| AppError::Config(format!("Failed to write manifest: {}", e)))?;
    
    // SECURITY: Add settings excluding ALL security-sensitive fields
    // Never export: ssh settings, security settings, plugin settings
    let safe_settings = serde_json::json!({
        "general": {
            "theme": settings.general.theme,
            "language": settings.general.language,
            // Explicitly exclude: check_updates, start_minimized, restore_sessions
        },
        "terminal": settings.terminal,
        "ui": settings.ui,
        // SECURITY: Explicitly NOT including:
        // - ssh (contains default_port, agent_forwarding, etc.)
        // - security (contains password storage settings)
        // - plugins (contains enabled plugins list)
    });
    let settings_json = serde_json::to_string_pretty(&safe_settings)?;
    
    zip.start_file("settings.json", options)
        .map_err(|e| AppError::Config(format!("Failed to write zip: {}", e)))?;
    zip.write_all(settings_json.as_bytes())
        .map_err(|e| AppError::Config(format!("Failed to write settings: {}", e)))?;
    
    zip.finish()
        .map_err(|e| AppError::Config(format!("Failed to finish zip: {}", e)))?;
    
    // SECURITY: Log export without full path to avoid log injection
    tracing::info!("Exported pack successfully");
    Ok(())
}

#[tauri::command]
pub async fn import_pack(
    state: State<'_, Arc<AppState>>,
    path: String,
) -> AppResult<()> {
    // SECURITY: Validate the import path
    let validated_path = validate_import_path(&path)?;
    
    let config_dir = super::get_config_dir()?;
    
    // Open and read the zip file
    let file = std::fs::File::open(&validated_path)
        .map_err(|e| AppError::Config(format!("Failed to open file: {}", e)))?;
    
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Config(format!("Invalid pack file: {}", e)))?;
    
    // SECURITY: Limit archive to prevent zip bombs
    const MAX_FILES: usize = 100;
    
    if archive.len() > MAX_FILES {
        return Err(AppError::Config(format!(
            "Pack contains too many files ({} > {})",
            archive.len(),
            MAX_FILES
        )));
    }
    
    // Read manifest first
    let mut manifest_content = String::new();
    {
        let mut manifest_file = archive.by_name("manifest.json")
            .map_err(|_| AppError::Config("Pack missing manifest.json".to_string()))?;
        
        // SECURITY: Limit manifest size
        if manifest_file.size() > 1024 * 1024 {
            return Err(AppError::Config("Manifest too large".to_string()));
        }
        
        manifest_file.read_to_string(&mut manifest_content)
            .map_err(|e| AppError::Config(format!("Failed to read manifest: {}", e)))?;
    }
    
    // SECURITY: Parse with size limits
    let pack: NeonPack = serde_json::from_str(&manifest_content)
        .map_err(|e| AppError::Config(format!("Invalid manifest: {}", e)))?;
    
    // Validate version
    if !pack.version.starts_with("1.") {
        return Err(AppError::Config(format!(
            "Unsupported pack version: {}. Expected 1.x",
            pack.version
        )));
    }
    
    // Import theme if present
    if let Some(theme) = &pack.theme {
        // SECURITY: Sanitize theme ID to prevent path traversal
        let safe_theme_id = sanitize_id(&theme.id)?;
        
        // SECURITY: Validate the destination path is within themes directory
        let themes_base = config_dir.join("themes");
        let themes_dir = validate_path_within_base(&themes_base, &safe_theme_id)?;
        
        std::fs::create_dir_all(&themes_dir)?;
        let theme_file = themes_dir.join("theme.json");
        
        // SECURITY: Create a sanitized copy of the theme with validated ID
        let mut safe_theme = theme.clone();
        safe_theme.id = safe_theme_id.clone();
        
        let theme_json = serde_json::to_string_pretty(&safe_theme)?;
        std::fs::write(theme_file, theme_json)?;
        
        // Set as active theme
        {
            let mut settings = state.settings.write();
            settings.general.theme = safe_theme_id;
            settings.save(&config_dir)?;
        }
        
        // SECURITY: Don't log untrusted theme name directly
        tracing::info!("Imported theme successfully");
    }
    
    // Import settings if present
    if let Ok(mut settings_file) = archive.by_name("settings.json") {
        // SECURITY: Limit settings file size
        if settings_file.size() > 1024 * 1024 {
            return Err(AppError::Config("Settings file too large".to_string()));
        }
        
        let mut settings_content = String::new();
        settings_file.read_to_string(&mut settings_content)
            .map_err(|e| AppError::Config(format!("Failed to read settings: {}", e)))?;
        
        // SECURITY: Parse and merge only safe fields
        // Never import: ssh, security, plugins sections
        if let Ok(imported) = serde_json::from_str::<serde_json::Value>(&settings_content) {
            let mut settings = state.settings.write();
            
            // Only import terminal and UI settings - these are safe
            if let Some(terminal) = imported.get("terminal") {
                if let Ok(term) = serde_json::from_value(terminal.clone()) {
                    settings.terminal = term;
                }
            }
            
            if let Some(ui) = imported.get("ui") {
                if let Ok(ui_settings) = serde_json::from_value(ui.clone()) {
                    settings.ui = ui_settings;
                }
            }
            
            // SECURITY: Explicitly NOT importing:
            // - general (could change update check settings)
            // - ssh (could weaken security settings)
            // - security (could change password storage)
            // - plugins (could enable malicious plugins)
            
            settings.save(&config_dir)?;
        }
    }
    
    tracing::info!("Imported pack successfully");
    Ok(())
}

// =============================================================================
// Security tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_id_valid() {
        assert!(sanitize_id("my-theme").is_ok());
        assert!(sanitize_id("my_theme").is_ok());
        assert!(sanitize_id("theme123").is_ok());
        assert!(sanitize_id("a").is_ok());
    }
    
    #[test]
    fn test_sanitize_id_path_traversal() {
        // Path traversal attempts
        assert!(sanitize_id("../etc/passwd").is_err());
        assert!(sanitize_id("..\\windows\\system32").is_err());
        assert!(sanitize_id("theme/../../../etc").is_err());
        assert!(sanitize_id("/etc/passwd").is_err());
        assert!(sanitize_id("C:\\Windows").is_err());
    }
    
    #[test]
    fn test_sanitize_id_invalid_chars() {
        assert!(sanitize_id("theme with space").is_err());
        assert!(sanitize_id("theme;rm -rf /").is_err());
        assert!(sanitize_id("").is_err());
        assert!(sanitize_id(&"a".repeat(100)).is_err()); // Too long
    }
    
    #[test]
    fn test_validate_export_path() {
        // Valid paths
        assert!(validate_export_path("/tmp/export.zip").is_ok() || cfg!(windows));
        
        // Path traversal
        assert!(validate_export_path("../../../etc/passwd.zip").is_err());
        assert!(validate_export_path("/tmp/../etc/passwd.zip").is_err());
        
        // Wrong extension
        assert!(validate_export_path("/tmp/export.exe").is_err());
        assert!(validate_export_path("/tmp/export").is_err());
    }
    
    #[test]
    fn test_normalize_path() {
        // Test that normalize_path correctly resolves .. components
        let path1 = PathBuf::from("/home/user/config/subdir/..");
        let result1 = normalize_path(&path1);
        assert_eq!(result1, PathBuf::from("/home/user/config"));
        
        // Test that . components are removed
        let path2 = PathBuf::from("/home/./user/./config");
        let result2 = normalize_path(&path2);
        assert_eq!(result2, PathBuf::from("/home/user/config"));
        
        // Test relative paths stay relative
        let path3 = PathBuf::from("foo/bar/../baz");
        let result3 = normalize_path(&path3);
        assert_eq!(result3, PathBuf::from("foo/baz"));
        
        // Note: normalize_path does NOT enforce security boundaries.
        // Callers must validate the result is within allowed directories.
    }
}
