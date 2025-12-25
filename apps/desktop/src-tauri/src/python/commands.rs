use super::ScriptInfo;
use crate::error::AppResult;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn list_scripts(state: State<'_, Arc<AppState>>) -> AppResult<Vec<ScriptInfo>> {
    Ok(state.scripts.read().list())
}

#[tauri::command]
pub async fn run_script(
    state: State<'_, Arc<AppState>>,
    id: String,
    function: String,
    args: serde_json::Value,
) -> AppResult<serde_json::Value> {
    let script = state
        .scripts
        .read()
        .get(&id)
        .ok_or_else(|| crate::error::AppError::Python(format!("Script not found: {}", id)))?;

    super::run_script(&script.path, &function, args).await
}

#[tauri::command]
pub async fn enable_script(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> AppResult<()> {
    state.scripts.write().enable(&id)
}

#[tauri::command]
pub async fn disable_script(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> AppResult<()> {
    state.scripts.write().disable(&id)
}




