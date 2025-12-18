pub mod commands;

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Plugin manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default)]
    pub main: String,
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,
    #[serde(default)]
    pub signed: bool,
}

/// Plugin permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginPermission {
    Network,
    Filesystem,
    Clipboard,
    Notifications,
    Terminal,
    Shell,
}

impl std::fmt::Display for PluginPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network => write!(f, "network"),
            Self::Filesystem => write!(f, "filesystem"),
            Self::Clipboard => write!(f, "clipboard"),
            Self::Notifications => write!(f, "notifications"),
            Self::Terminal => write!(f, "terminal"),
            Self::Shell => write!(f, "shell"),
        }
    }
}

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    Disabled,
    Enabled,
    Error,
}

/// Plugin info for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub path: PathBuf,
    #[serde(default)]
    pub granted_permissions: Vec<PluginPermission>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Plugin manager
pub struct PluginManager {
    plugins: HashMap<String, PluginInfo>,
    plugins_dir: PathBuf,
    enabled_plugins: Vec<String>,
}

impl PluginManager {
    pub fn new(config_dir: &Path) -> AppResult<Self> {
        let plugins_dir = config_dir.join("plugins");
        std::fs::create_dir_all(&plugins_dir)?;

        let mut manager = Self {
            plugins: HashMap::new(),
            plugins_dir,
            enabled_plugins: vec![],
        };

        manager.scan_plugins()?;

        Ok(manager)
    }

    /// Scan plugins directory for installed plugins
    pub fn scan_plugins(&mut self) -> AppResult<()> {
        self.plugins.clear();

        if let Ok(entries) = std::fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Err(e) = self.load_plugin(&path) {
                        tracing::warn!("Failed to load plugin at {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    fn load_plugin(&mut self, path: &Path) -> AppResult<()> {
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(AppError::Plugin("manifest.json not found".into()));
        }

        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| AppError::Plugin(format!("Invalid manifest: {}", e)))?;

        // Validate API version
        if !manifest.api_version.starts_with("1") {
            return Err(AppError::Plugin(format!(
                "Unsupported API version: {}",
                manifest.api_version
            )));
        }

        let state = if self.enabled_plugins.contains(&manifest.id) {
            PluginState::Enabled
        } else {
            PluginState::Disabled
        };

        let info = PluginInfo {
            manifest,
            state,
            path: path.to_path_buf(),
            granted_permissions: vec![],
            error: None,
        };

        self.plugins.insert(info.manifest.id.clone(), info);

        Ok(())
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<PluginInfo> {
        self.plugins.get(id).cloned()
    }

    pub fn enable(&mut self, id: &str, permissions: Vec<PluginPermission>) -> AppResult<()> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| AppError::Plugin(format!("Plugin not found: {}", id)))?;

        // Verify all requested permissions are granted
        for perm in &plugin.manifest.permissions {
            if !permissions.contains(perm) {
                return Err(AppError::PermissionDenied(format!(
                    "Permission '{}' not granted for plugin '{}'",
                    perm, id
                )));
            }
        }

        plugin.state = PluginState::Enabled;
        plugin.granted_permissions = permissions;

        if !self.enabled_plugins.contains(&id.to_string()) {
            self.enabled_plugins.push(id.to_string());
        }

        tracing::info!("Enabled plugin: {}", id);
        Ok(())
    }

    pub fn disable(&mut self, id: &str) -> AppResult<()> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| AppError::Plugin(format!("Plugin not found: {}", id)))?;

        plugin.state = PluginState::Disabled;
        plugin.granted_permissions.clear();

        self.enabled_plugins.retain(|p| p != id);

        tracing::info!("Disabled plugin: {}", id);
        Ok(())
    }

    pub fn install(&mut self, source_path: &Path) -> AppResult<String> {
        // Read manifest to get plugin ID
        let manifest_path = source_path.join("manifest.json");
        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| AppError::Plugin(format!("Invalid manifest: {}", e)))?;

        let id = manifest.id.clone();
        let dest_path = self.plugins_dir.join(&id);

        // Copy plugin files
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path)?;
        }
        copy_dir_recursive(source_path, &dest_path)?;

        // Load the plugin
        self.load_plugin(&dest_path)?;

        tracing::info!("Installed plugin: {}", id);
        Ok(id)
    }

    pub fn uninstall(&mut self, id: &str) -> AppResult<()> {
        let plugin = self
            .plugins
            .remove(id)
            .ok_or_else(|| AppError::Plugin(format!("Plugin not found: {}", id)))?;

        // Remove from enabled list
        self.enabled_plugins.retain(|p| p != id);

        // Delete plugin directory
        if plugin.path.exists() {
            std::fs::remove_dir_all(&plugin.path)?;
        }

        tracing::info!("Uninstalled plugin: {}", id);
        Ok(())
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> AppResult<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

