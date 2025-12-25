use super::{PluginInfo, PluginPermission};
use crate::error::AppResult;
use crate::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn list_plugins(state: State<'_, Arc<AppState>>) -> AppResult<Vec<PluginInfo>> {
    Ok(state.plugins.read().list())
}

#[tauri::command]
pub async fn get_plugin(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> AppResult<Option<PluginInfo>> {
    Ok(state.plugins.read().get(&id))
}

#[tauri::command]
pub async fn enable_plugin(
    state: State<'_, Arc<AppState>>,
    id: String,
    permissions: Vec<PluginPermission>,
) -> AppResult<()> {
    state.plugins.write().enable(&id, permissions)
}

#[tauri::command]
pub async fn disable_plugin(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> AppResult<()> {
    state.plugins.write().disable(&id)
}

#[tauri::command]
pub async fn install_plugin(
    state: State<'_, Arc<AppState>>,
    path: String,
) -> AppResult<String> {
    let source_path = PathBuf::from(path);
    state.plugins.write().install(&source_path)
}




