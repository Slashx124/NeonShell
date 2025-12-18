use crate::config::{AppSettings, ProfileManager};
use crate::error::AppResult;
use crate::plugins::PluginManager;
use crate::python::ScriptManager;
use crate::ssh::SessionManager;
use parking_lot::RwLock;
use std::sync::Arc;
use tauri::AppHandle;

/// Global application state
pub struct AppState {
    pub app_handle: AppHandle,
    pub sessions: Arc<SessionManager>,
    pub profiles: Arc<RwLock<ProfileManager>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub plugins: Arc<RwLock<PluginManager>>,
    pub scripts: Arc<RwLock<ScriptManager>>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> AppResult<Self> {
        let config_dir = crate::config::get_config_dir()?;

        // Load settings
        let settings = AppSettings::load(&config_dir)?;
        
        // Load profiles
        let profiles = ProfileManager::load(&config_dir)?;

        // Initialize managers
        let sessions = SessionManager::new(app_handle.clone());
        let plugins = PluginManager::new(&config_dir)?;
        let scripts = ScriptManager::new(&config_dir)?;

        Ok(Self {
            app_handle,
            sessions: Arc::new(sessions),
            profiles: Arc::new(RwLock::new(profiles)),
            settings: Arc::new(RwLock::new(settings)),
            plugins: Arc::new(RwLock::new(plugins)),
            scripts: Arc::new(RwLock::new(scripts)),
        })
    }
}

