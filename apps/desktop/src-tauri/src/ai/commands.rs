//! AI Tauri Commands
//!
//! Commands for AI model management and chat.

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use super::provider::{GatewayProvider, LocalOllamaProvider, OpenAICompatProvider};
use super::types::*;
use std::sync::Arc;
use tauri::State;
use parking_lot::RwLock;

/// Cached model catalog
static MODEL_CACHE: RwLock<Option<Vec<Model>>> = RwLock::new(None);

/// Get AI settings
#[tauri::command]
pub async fn get_ai_settings(
    state: State<'_, Arc<AppState>>,
) -> AppResult<AISettings> {
    // TODO: Load from config file
    Ok(AISettings::default_settings())
}

/// Save AI settings
#[tauri::command]
pub async fn save_ai_settings(
    state: State<'_, Arc<AppState>>,
    settings: AISettings,
) -> AppResult<()> {
    // TODO: Save to config file
    tracing::info!("Saving AI settings: {:?}", settings);
    Ok(())
}

/// Get available models from all sources
#[tauri::command]
pub async fn get_models(
    state: State<'_, Arc<AppState>>,
    force_refresh: bool,
) -> AppResult<Vec<Model>> {
    // Check cache first
    if !force_refresh {
        let cache = MODEL_CACHE.read();
        if let Some(models) = cache.as_ref() {
            return Ok(models.clone());
        }
    }
    
    let mut all_models: Vec<Model> = Vec::new();
    let settings = AISettings::default_settings();
    
    // 1. Fetch from gateway (hosted + org models)
    if settings.enable_gateway {
        // Get token from keychain
        if let Ok(Some(token)) = crate::keychain::get_secret("gateway:access_token") {
            let gateway = GatewayProvider::new(&settings.gateway_url)
                .with_token(token);
            
            match gateway.fetch_models() {
                Ok(catalog) => {
                    all_models.extend(catalog.models);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch gateway models: {}", e);
                }
            }
        }
    }
    
    // 2. Get local Ollama models
    let ollama = LocalOllamaProvider::new("http://localhost:11434");
    if ollama.is_running() {
        match ollama.list_models() {
            Ok(models) => {
                all_models.extend(models);
            }
            Err(e) => {
                tracing::debug!("Ollama not available: {}", e);
            }
        }
    }
    
    // 3. Add configured local models from settings
    for local_config in &settings.local_models {
        if local_config.enabled {
            // Check if model already exists from live query
            let exists = all_models.iter().any(|m| m.id == local_config.id);
            if !exists {
                all_models.push(Model {
                    id: local_config.id.clone(),
                    name: local_config.name.clone(),
                    provider: local_config.provider.clone(),
                    source: ModelSource::Local,
                    model_id: local_config.model_id.clone(),
                    description: None,
                    context_window: 4096,
                    max_output_tokens: None,
                    capabilities: ModelCapabilities::default(),
                    pricing: None,
                    enabled: local_config.enabled,
                    badge: None,
                    endpoint: Some(local_config.endpoint.clone()),
                });
            }
        }
    }
    
    // 4. Add personal BYOK models
    for key_config in &settings.personal_keys {
        if key_config.enabled {
            // These are just indicators that the user has configured a key
            // The actual models depend on the provider
            let provider_name = match key_config.provider {
                ModelProvider::OpenAI => "OpenAI (Personal)",
                ModelProvider::Anthropic => "Anthropic (Personal)",
                _ => "Custom (Personal)",
            };
            
            all_models.push(Model {
                id: format!("personal:{}", key_config.id),
                name: key_config.name.clone(),
                provider: key_config.provider.clone(),
                source: ModelSource::Personal,
                model_id: key_config.id.clone(),
                description: Some(format!("{} - Personal API Key", provider_name)),
                context_window: 128000,
                max_output_tokens: None,
                capabilities: ModelCapabilities {
                    chat: true,
                    completion: true,
                    embeddings: false,
                    vision: true,
                    function_calling: true,
                    streaming: true,
                },
                pricing: None, // User pays directly
                enabled: key_config.enabled,
                badge: None,
                endpoint: None,
            });
        }
    }
    
    // Update cache
    *MODEL_CACHE.write() = Some(all_models.clone());
    
    Ok(all_models)
}

/// Send a chat message
#[tauri::command]
pub async fn ai_chat(
    state: State<'_, Arc<AppState>>,
    request: ChatRequest,
) -> AppResult<ChatResponse> {
    let settings = AISettings::default_settings();
    
    // Find the model
    let models = get_models(state.clone(), false).await?;
    let model = models.iter()
        .find(|m| m.id == request.model_id || m.model_id == request.model_id)
        .ok_or_else(|| AppError::NotFound(format!("Model not found: {}", request.model_id)))?;
    
    // Route to appropriate provider based on source
    match model.source {
        ModelSource::Hosted | ModelSource::Org => {
            // Use gateway
            let token = crate::keychain::get_secret("gateway:access_token")?
                .ok_or_else(|| AppError::Auth("Not authenticated with gateway".to_string()))?;
            
            let gateway = GatewayProvider::new(&settings.gateway_url)
                .with_token(token);
            
            gateway.chat(&request)
        }
        ModelSource::Local => {
            // Check provider type
            match model.provider {
                ModelProvider::Ollama => {
                    let endpoint = model.endpoint.as_deref()
                        .unwrap_or("http://localhost:11434");
                    let ollama = LocalOllamaProvider::new(endpoint);
                    ollama.chat(&model.model_id, &request.messages)
                }
                ModelProvider::OpenAICompatible | ModelProvider::Custom => {
                    let endpoint = model.endpoint.as_deref()
                        .ok_or_else(|| AppError::Config("Local model missing endpoint".to_string()))?;
                    
                    // Try to get API key from keychain
                    let key_id = format!("local:{}", model.id);
                    let api_key = crate::keychain::get_secret(&key_id)?
                        .unwrap_or_default();
                    
                    let provider = OpenAICompatProvider::new(endpoint, &api_key);
                    provider.chat(&request)
                }
                _ => {
                    Err(AppError::Config(format!("Unsupported local provider: {:?}", model.provider)))
                }
            }
        }
        ModelSource::Personal => {
            // Use personal BYOK key
            let key_id = format!("personal:key:{}", model.id.replace("personal:", ""));
            let api_key = crate::keychain::get_secret(&key_id)?
                .ok_or_else(|| AppError::Auth("Personal API key not found".to_string()))?;
            
            // Determine endpoint based on provider
            let endpoint = match model.provider {
                ModelProvider::OpenAI => "https://api.openai.com/v1",
                ModelProvider::Anthropic => "https://api.anthropic.com/v1",
                _ => {
                    return Err(AppError::Config("Personal BYOK requires OpenAI or Anthropic".to_string()));
                }
            };
            
            let provider = OpenAICompatProvider::new(endpoint, &api_key);
            provider.chat(&request)
        }
    }
}

/// Check Ollama availability
#[tauri::command]
pub async fn check_ollama() -> AppResult<bool> {
    let ollama = LocalOllamaProvider::new("http://localhost:11434");
    Ok(ollama.is_running())
}

/// Store personal API key in keychain
#[tauri::command]
pub async fn store_personal_key(
    provider: String,
    name: String,
    api_key: String,
) -> AppResult<String> {
    let key_id = uuid::Uuid::new_v4().to_string();
    let keychain_key = format!("personal:key:{}", key_id);
    
    crate::keychain::store_secret(&keychain_key, &api_key)?;
    
    tracing::info!("Stored personal API key for provider: {}", provider);
    
    Ok(key_id)
}

/// Delete personal API key from keychain
#[tauri::command]
pub async fn delete_personal_key(key_id: String) -> AppResult<()> {
    let keychain_key = format!("personal:key:{}", key_id);
    crate::keychain::delete_secret(&keychain_key)?;
    
    tracing::info!("Deleted personal API key: {}", key_id);
    
    Ok(())
}

/// Gateway authentication - start device flow
#[tauri::command]
pub async fn gateway_auth_start(
    device_name: String,
    platform: String,
) -> AppResult<serde_json::Value> {
    let settings = AISettings::default_settings();
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/auth/device/start", settings.gateway_url))
        .json(&serde_json::json!({
            "device_name": device_name,
            "platform": platform,
        }))
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Gateway auth failed: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(AppError::Network(format!(
            "Gateway auth failed: {}",
            response.text().await.unwrap_or_default()
        )));
    }
    
    response.json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Network(format!("Failed to parse response: {}", e)))
}

/// Gateway authentication - poll for token
#[tauri::command]
pub async fn gateway_auth_poll(device_code: String) -> AppResult<serde_json::Value> {
    let settings = AISettings::default_settings();
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/auth/token", settings.gateway_url))
        .json(&serde_json::json!({
            "grant_type": "device_code",
            "device_code": device_code,
        }))
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Gateway poll failed: {}", e)))?;
    
    let json: serde_json::Value = response.json()
        .await
        .map_err(|e| AppError::Network(format!("Failed to parse response: {}", e)))?;
    
    // If we got tokens, store them
    if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
        crate::keychain::store_secret("gateway:access_token", access_token)?;
    }
    if let Some(refresh_token) = json.get("refresh_token").and_then(|v| v.as_str()) {
        crate::keychain::store_secret("gateway:refresh_token", refresh_token)?;
    }
    
    Ok(json)
}

/// Gateway logout
#[tauri::command]
pub async fn gateway_logout() -> AppResult<()> {
    crate::keychain::delete_secret("gateway:access_token")?;
    crate::keychain::delete_secret("gateway:refresh_token")?;
    *MODEL_CACHE.write() = None;
    
    tracing::info!("Logged out from gateway");
    Ok(())
}

/// Check if authenticated with gateway
#[tauri::command]
pub async fn is_gateway_authenticated() -> AppResult<bool> {
    Ok(crate::keychain::has_secret("gateway:access_token")?)
}




