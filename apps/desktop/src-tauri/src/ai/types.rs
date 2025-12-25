//! AI Types for NeonShell
//! 
//! These types mirror the @neonshell/shared types for the desktop app.

use serde::{Deserialize, Serialize};

/// Model source - where the model comes from
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelSource {
    /// Hosted by NeonShell (billed through gateway)
    Hosted,
    /// Organization's BYOK model
    Org,
    /// Local model (Ollama, etc.)
    Local,
    /// Personal BYOK (stored in user's keychain)
    Personal,
}

/// Model provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    OpenAI,
    Anthropic,
    Google,
    Azure,
    Ollama,
    OpenAICompatible,
    Custom,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    pub chat: bool,
    pub completion: bool,
    pub embeddings: bool,
    pub vision: bool,
    pub function_calling: bool,
    pub streaming: bool,
}

/// Model pricing (per 1M tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_per_1m_tokens: f64,
    pub output_per_1m_tokens: f64,
    pub currency: String,
}

/// AI Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: ModelProvider,
    pub source: ModelSource,
    pub model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub context_window: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    pub capabilities: ModelCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<ModelPricing>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    /// For local models - the endpoint URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

/// Chat message role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

/// Tool call function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunctionDefinition,
}

/// Tool function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Chat request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model_id: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stream: bool,
}

/// Chat response usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Chat response choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

/// Chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub created: u64,
    pub choices: Vec<ChatChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ChatUsage>,
}

/// Model catalog response from gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalog {
    pub models: Vec<Model>,
    pub last_updated: String,
}

/// Local model configuration (stored in settings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelConfig {
    pub id: String,
    pub name: String,
    pub provider: ModelProvider,
    pub model_id: String,
    pub endpoint: String,
    pub enabled: bool,
}

/// Personal BYOK configuration (key stored in keychain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalKeyConfig {
    pub id: String,
    pub provider: ModelProvider,
    pub name: String,
    /// Keychain key for the API key (never stores the actual key)
    pub key_id: String,
    pub enabled: bool,
}

/// AI Settings stored in config
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AISettings {
    /// Enable hosted/org models from gateway
    pub enable_gateway: bool,
    /// Gateway API URL
    pub gateway_url: String,
    /// Local models (Ollama, etc.)
    pub local_models: Vec<LocalModelConfig>,
    /// Personal BYOK keys
    pub personal_keys: Vec<PersonalKeyConfig>,
    /// Default model ID
    pub default_model: Option<String>,
    /// Tool execution requires approval
    pub require_tool_approval: bool,
    /// Auto-mode enabled
    pub auto_mode: bool,
}

impl AISettings {
    pub fn default_settings() -> Self {
        Self {
            enable_gateway: true,
            gateway_url: "https://api.neonshell.dev".to_string(),
            local_models: vec![
                LocalModelConfig {
                    id: "ollama-llama3".to_string(),
                    name: "Llama 3 (Ollama)".to_string(),
                    provider: ModelProvider::Ollama,
                    model_id: "llama3".to_string(),
                    endpoint: "http://localhost:11434".to_string(),
                    enabled: false,
                },
            ],
            personal_keys: vec![],
            default_model: None,
            require_tool_approval: true,
            auto_mode: false,
        }
    }
}




