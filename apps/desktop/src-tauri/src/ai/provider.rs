//! AI Provider Abstraction
//! 
//! Supports multiple AI provider types:
//! - GatewayProvider: Routes through neonshell.dev
//! - LocalOllamaProvider: Local Ollama instance
//! - LocalOpenAICompatProvider: Any OpenAI-compatible endpoint
//! - PersonalCloudProvider: Direct to cloud with user's BYOK keys

use crate::error::{AppError, AppResult};
use crate::keychain;
use super::types::*;
use std::time::Duration;

/// Base trait for AI providers
pub trait AIProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;
    
    /// Check if provider is available/configured
    fn is_available(&self) -> bool;
    
    /// Get available models from this provider
    fn get_models(&self) -> AppResult<Vec<Model>>;
}

/// Gateway provider - routes through neonshell.dev API
pub struct GatewayProvider {
    base_url: String,
    access_token: Option<String>,
    client: reqwest::blocking::Client,
}

impl GatewayProvider {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            base_url: base_url.to_string(),
            access_token: None,
            client,
        }
    }
    
    pub fn with_token(mut self, token: String) -> Self {
        self.access_token = Some(token);
        self
    }
    
    pub fn set_token(&mut self, token: Option<String>) {
        self.access_token = token;
    }
    
    /// Fetch models from gateway
    pub fn fetch_models(&self) -> AppResult<ModelCatalog> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| AppError::Auth("Not authenticated with gateway".to_string()))?;
        
        let response = self.client
            .get(format!("{}/v1/models", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| AppError::Network(format!("Gateway request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Gateway returned {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            )));
        }
        
        response.json::<ModelCatalog>()
            .map_err(|e| AppError::Network(format!("Failed to parse models: {}", e)))
    }
    
    /// Send chat request through gateway
    pub fn chat(&self, request: &ChatRequest) -> AppResult<ChatResponse> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| AppError::Auth("Not authenticated with gateway".to_string()))?;
        
        let response = self.client
            .post(format!("{}/v1/ai/chat", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .map_err(|e| AppError::Network(format!("Chat request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Chat failed {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            )));
        }
        
        response.json::<ChatResponse>()
            .map_err(|e| AppError::Network(format!("Failed to parse response: {}", e)))
    }
}

impl AIProvider for GatewayProvider {
    fn name(&self) -> &str {
        "NeonShell Gateway"
    }
    
    fn is_available(&self) -> bool {
        self.access_token.is_some()
    }
    
    fn get_models(&self) -> AppResult<Vec<Model>> {
        self.fetch_models().map(|c| c.models)
    }
}

/// Local Ollama provider
pub struct LocalOllamaProvider {
    endpoint: String,
    client: reqwest::blocking::Client,
}

impl LocalOllamaProvider {
    pub fn new(endpoint: &str) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            endpoint: endpoint.to_string(),
            client,
        }
    }
    
    /// Check if Ollama is running
    pub fn is_running(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
    
    /// Get available Ollama models
    pub fn list_models(&self) -> AppResult<Vec<Model>> {
        let response = self.client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .map_err(|e| AppError::Network(format!("Ollama request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AppError::Network("Ollama not available".to_string()));
        }
        
        #[derive(serde::Deserialize)]
        struct OllamaModels {
            models: Vec<OllamaModel>,
        }
        
        #[derive(serde::Deserialize)]
        struct OllamaModel {
            name: String,
            size: Option<u64>,
        }
        
        let ollama_models: OllamaModels = response.json()
            .map_err(|e| AppError::Network(format!("Failed to parse Ollama response: {}", e)))?;
        
        Ok(ollama_models.models.into_iter().map(|m| {
            let name = m.name.clone();
            Model {
                id: format!("ollama:{}", name),
                name: name.clone(),
                provider: ModelProvider::Ollama,
                source: ModelSource::Local,
                model_id: name,
                description: Some("Local Ollama model".to_string()),
                context_window: 4096, // Default, varies by model
                max_output_tokens: None,
                capabilities: ModelCapabilities {
                    chat: true,
                    completion: true,
                    embeddings: false,
                    vision: false,
                    function_calling: false,
                    streaming: true,
                },
                pricing: None,
                enabled: true,
                badge: None,
                endpoint: Some(self.endpoint.clone()),
            }
        }).collect())
    }
    
    /// Send chat to Ollama
    pub fn chat(&self, model: &str, messages: &[ChatMessage]) -> AppResult<ChatResponse> {
        #[derive(serde::Serialize)]
        struct OllamaRequest {
            model: String,
            messages: Vec<OllamaMessage>,
            stream: bool,
        }
        
        #[derive(serde::Serialize)]
        struct OllamaMessage {
            role: String,
            content: String,
        }
        
        let request = OllamaRequest {
            model: model.to_string(),
            messages: messages.iter().map(|m| OllamaMessage {
                role: match m.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Tool => "tool".to_string(),
                },
                content: m.content.clone(),
            }).collect(),
            stream: false,
        };
        
        let response = self.client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&request)
            .send()
            .map_err(|e| AppError::Network(format!("Ollama chat failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Ollama chat failed: {}",
                response.text().unwrap_or_default()
            )));
        }
        
        #[derive(serde::Deserialize)]
        struct OllamaResponse {
            message: OllamaResponseMessage,
            eval_count: Option<u32>,
            prompt_eval_count: Option<u32>,
        }
        
        #[derive(serde::Deserialize)]
        struct OllamaResponseMessage {
            role: String,
            content: String,
        }
        
        let ollama_resp: OllamaResponse = response.json()
            .map_err(|e| AppError::Network(format!("Failed to parse Ollama response: {}", e)))?;
        
        Ok(ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            model: model.to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: ollama_resp.message.content,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Some(ChatUsage {
                prompt_tokens: ollama_resp.prompt_eval_count.unwrap_or(0),
                completion_tokens: ollama_resp.eval_count.unwrap_or(0),
                total_tokens: ollama_resp.prompt_eval_count.unwrap_or(0) + ollama_resp.eval_count.unwrap_or(0),
            }),
        })
    }
}

impl AIProvider for LocalOllamaProvider {
    fn name(&self) -> &str {
        "Local Ollama"
    }
    
    fn is_available(&self) -> bool {
        self.is_running()
    }
    
    fn get_models(&self) -> AppResult<Vec<Model>> {
        self.list_models()
    }
}

/// OpenAI-compatible provider (for personal BYOK or custom endpoints)
pub struct OpenAICompatProvider {
    endpoint: String,
    api_key: String,
    client: reqwest::blocking::Client,
}

impl OpenAICompatProvider {
    pub fn new(endpoint: &str, api_key: &str) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            endpoint: endpoint.to_string(),
            api_key: api_key.to_string(),
            client,
        }
    }
    
    /// Create from keychain-stored key
    pub fn from_keychain(endpoint: &str, key_id: &str) -> AppResult<Self> {
        let api_key = keychain::get_secret(key_id)?
            .ok_or_else(|| AppError::Auth("API key not found in keychain".to_string()))?;
        
        Ok(Self::new(endpoint, &api_key))
    }
    
    /// Send chat request
    pub fn chat(&self, request: &ChatRequest) -> AppResult<ChatResponse> {
        let response = self.client
            .post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": request.model_id,
                "messages": request.messages,
                "tools": request.tools,
                "temperature": request.temperature,
                "max_tokens": request.max_tokens,
                "stream": false,
            }))
            .send()
            .map_err(|e| AppError::Network(format!("Chat request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Chat failed {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            )));
        }
        
        response.json::<ChatResponse>()
            .map_err(|e| AppError::Network(format!("Failed to parse response: {}", e)))
    }
}

impl AIProvider for OpenAICompatProvider {
    fn name(&self) -> &str {
        "OpenAI Compatible"
    }
    
    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
    
    fn get_models(&self) -> AppResult<Vec<Model>> {
        // Most OpenAI-compatible endpoints don't list models well
        // Return empty - models should be configured manually
        Ok(vec![])
    }
}




