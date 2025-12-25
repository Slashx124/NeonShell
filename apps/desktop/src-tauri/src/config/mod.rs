pub mod commands;
pub mod profiles;
pub mod settings;
pub mod themes;

pub use profiles::*;
pub use settings::*;
pub use themes::*;

use crate::error::AppResult;
use std::path::PathBuf;

/// Get the NeonShell config directory
pub fn get_config_dir() -> AppResult<PathBuf> {
    let config_dir = dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .ok_or_else(|| crate::error::AppError::Config("Could not find config directory".into()))?
        .join("neonshell");

    Ok(config_dir)
}

/// Get the themes directory
pub fn get_themes_dir() -> AppResult<PathBuf> {
    Ok(get_config_dir()?.join("themes"))
}

/// Get the plugins directory
pub fn get_plugins_dir() -> AppResult<PathBuf> {
    Ok(get_config_dir()?.join("plugins"))
}

/// Get the scripts directory
pub fn get_scripts_dir() -> AppResult<PathBuf> {
    Ok(get_config_dir()?.join("scripts"))
}




