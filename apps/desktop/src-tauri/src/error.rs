use serde::Serialize;
use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("SSH error: {0}")]
    Ssh(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Python error: {0}")]
    Python(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Serializable error for frontend
#[derive(Serialize)]
pub struct SerializableError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl From<&AppError> for SerializableError {
    fn from(err: &AppError) -> Self {
        let (code, message, details) = match err {
            AppError::Ssh(msg) => ("SSH_ERROR", msg.clone(), None),
            AppError::Connection(msg) => ("CONNECTION_ERROR", msg.clone(), None),
            AppError::Auth(msg) => ("AUTH_ERROR", "Authentication failed".to_string(), Some(msg.clone())),
            AppError::Keychain(msg) => ("KEYCHAIN_ERROR", msg.clone(), None),
            AppError::Config(msg) => ("CONFIG_ERROR", msg.clone(), None),
            AppError::Plugin(msg) => ("PLUGIN_ERROR", msg.clone(), None),
            AppError::Python(msg) => ("PYTHON_ERROR", msg.clone(), None),
            AppError::Io(e) => ("IO_ERROR", e.to_string(), None),
            AppError::Serialization(msg) => ("SERIALIZATION_ERROR", msg.clone(), None),
            AppError::SessionNotFound(id) => ("SESSION_NOT_FOUND", format!("Session {} not found", id), None),
            AppError::ProfileNotFound(id) => ("PROFILE_NOT_FOUND", format!("Profile {} not found", id), None),
            AppError::InvalidConfig(msg) => ("INVALID_CONFIG", msg.clone(), None),
            AppError::PermissionDenied(msg) => ("PERMISSION_DENIED", msg.clone(), None),
            AppError::Unknown(msg) => ("UNKNOWN_ERROR", msg.clone(), None),
        };

        SerializableError {
            code: code.to_string(),
            message,
            details,
        }
    }
}

// Implement Serialize for AppError so Tauri can send it to frontend
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableError::from(self).serialize(serializer)
    }
}

impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        AppError::Config(err.to_string())
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(err: toml::ser::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl From<keyring::Error> for AppError {
    fn from(err: keyring::Error) -> Self {
        AppError::Keychain(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Unknown(err.to_string())
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(err: zip::result::ZipError) -> Self {
        AppError::Config(format!("ZIP error: {}", err))
    }
}

pub type AppResult<T> = Result<T, AppError>;

