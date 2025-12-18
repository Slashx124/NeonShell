pub mod commands;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use regex::Regex;
use once_cell::sync::Lazy;

/// Maximum lines to keep in memory ring buffer
const MAX_RING_BUFFER_LINES: usize = 10_000;

/// Maximum line length before truncation
const MAX_LINE_LENGTH: usize = 2048;

/// Maximum log file size before rotation (5 MB)
const MAX_LOG_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// Maximum total bundle size (10 MB)
const MAX_BUNDLE_SIZE: u64 = 10 * 1024 * 1024;

/// Sensitive patterns to redact
static SENSITIVE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // SSH Private key blocks
        Regex::new(r"(?s)-----BEGIN[^-]*PRIVATE KEY-----.*?-----END[^-]*PRIVATE KEY-----").unwrap(),
        Regex::new(r"(?s)-----BEGIN[^-]*KEY-----.*?-----END[^-]*KEY-----").unwrap(),
        // GitHub tokens
        Regex::new(r"ghp_[A-Za-z0-9_]{36,}").unwrap(),
        Regex::new(r"gho_[A-Za-z0-9_]{36,}").unwrap(),
        Regex::new(r"ghs_[A-Za-z0-9_]{36,}").unwrap(),
        Regex::new(r"GHSAT[A-Za-z0-9_]{40,}").unwrap(),
        Regex::new(r"github_pat_[A-Za-z0-9_]{22,}").unwrap(),
        // AWS keys
        Regex::new(r"AKIA[A-Z0-9]{16}").unwrap(),
        Regex::new(r#"(?i)aws[_-]?secret[_-]?access[_-]?key['"]?\s*[:=]\s*['"]?[A-Za-z0-9/+=]{40}"#).unwrap(),
        // JWTs (base64.base64.base64)
        Regex::new(r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").unwrap(),
        // Authorization headers
        Regex::new(r"(?i)authorization\s*:\s*bearer\s+[^\s]+").unwrap(),
        Regex::new(r"(?i)authorization\s*:\s*basic\s+[^\s]+").unwrap(),
        // Generic secrets by key name (key=value patterns)
        Regex::new(r#"(?i)(password|passwd|pwd|secret|token|api[_-]?key|private[_-]?key|passphrase|auth[_-]?token|access[_-]?token)\s*[:=]\s*["']?[^\s"']+["']?"#).unwrap(),
        // Base64 encoded potential secrets (long base64 strings)
        Regex::new(r"[A-Za-z0-9+/]{64,}={0,2}").unwrap(),
    ]
});

/// Sanitize a string by removing sensitive information
pub fn sanitize(input: &str) -> String {
    let mut result = input.to_string();
    
    // Apply all sensitive patterns
    for pattern in SENSITIVE_PATTERNS.iter() {
        result = pattern.replace_all(&result, "[REDACTED]").to_string();
    }
    
    // Truncate long lines
    if result.len() > MAX_LINE_LENGTH {
        result = format!("{}... [truncated]", &result[..MAX_LINE_LENGTH]);
    }
    
    result
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Log subsystem
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogSubsystem {
    Ssh,
    Config,
    Plugins,
    Python,
    Keychain,
    App,
    Unknown,
}

impl std::fmt::Display for LogSubsystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogSubsystem::Ssh => write!(f, "ssh"),
            LogSubsystem::Config => write!(f, "config"),
            LogSubsystem::Plugins => write!(f, "plugins"),
            LogSubsystem::Python => write!(f, "python"),
            LogSubsystem::Keychain => write!(f, "keychain"),
            LogSubsystem::App => write!(f, "app"),
            LogSubsystem::Unknown => write!(f, "unknown"),
        }
    }
}

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub timestamp: i64,
    pub level: LogLevel,
    pub subsystem: LogSubsystem,
    pub session_id: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl LogLine {
    pub fn new(level: LogLevel, subsystem: LogSubsystem, message: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp_millis(),
            level,
            subsystem,
            session_id: None,
            message: sanitize(&message.into()),
            details: None,
        }
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        // Sanitize details too
        self.details = Some(sanitize_json(&details));
        self
    }
}

/// Sanitize a JSON value recursively
fn sanitize_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => serde_json::Value::String(sanitize(s)),
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                let key_lower = k.to_lowercase();
                // Check if key suggests sensitive data
                if key_lower.contains("password")
                    || key_lower.contains("secret")
                    || key_lower.contains("token")
                    || key_lower.contains("key")
                    || key_lower.contains("passphrase")
                    || key_lower.contains("authorization")
                    || key_lower.contains("auth")
                    || key_lower.contains("credential")
                {
                    new_map.insert(k.clone(), serde_json::Value::String("[REDACTED]".to_string()));
                } else {
                    new_map.insert(k.clone(), sanitize_json(v));
                }
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sanitize_json).collect())
        }
        other => other.clone(),
    }
}

/// Log filter for querying logs
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LogFilter {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub level: Option<LogLevel>,
    #[serde(default)]
    pub subsystem: Option<LogSubsystem>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub since: Option<i64>,
}

/// Debug bundle options
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DebugBundleOptions {
    #[serde(default)]
    pub max_lines: Option<u32>,
    #[serde(default)]
    pub include_config: Option<bool>,
    #[serde(default)]
    pub include_sessions: Option<bool>,
    #[serde(default)]
    pub include_plugins: Option<bool>,
    #[serde(default)]
    pub redact_hostnames: Option<bool>,
}

/// App info for debug bundle
#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub build_type: String,
    pub tauri_version: String,
    pub rust_version: String,
    pub generated_at: i64,
}

impl AppInfo {
    pub fn collect() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            build_type: if cfg!(debug_assertions) { "debug" } else { "release" }.to_string(),
            tauri_version: "2.0".to_string(),
            rust_version: option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("unknown").to_string(),
            generated_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Log manager with ring buffer and file persistence
pub struct LogManager {
    ring_buffer: RwLock<VecDeque<LogLine>>,
    log_file_path: PathBuf,
    config_dir: PathBuf,
}

impl LogManager {
    pub fn new(config_dir: PathBuf) -> std::io::Result<Arc<Self>> {
        let logs_dir = config_dir.join("logs");
        fs::create_dir_all(&logs_dir)?;
        
        let log_file_path = logs_dir.join("neonshell.log");
        
        let manager = Arc::new(Self {
            ring_buffer: RwLock::new(VecDeque::with_capacity(MAX_RING_BUFFER_LINES)),
            log_file_path,
            config_dir,
        });
        
        // Load existing logs from file into ring buffer
        manager.load_existing_logs();
        
        Ok(manager)
    }

    /// Load existing log lines from file into the ring buffer
    fn load_existing_logs(&self) {
        if !self.log_file_path.exists() {
            return;
        }

        if let Ok(file) = File::open(&self.log_file_path) {
            let reader = BufReader::new(file);
            let mut buffer = self.ring_buffer.write();
            
            for line in reader.lines().filter_map(|l| l.ok()) {
                if let Ok(log_line) = serde_json::from_str::<LogLine>(&line) {
                    if buffer.len() >= MAX_RING_BUFFER_LINES {
                        buffer.pop_front();
                    }
                    buffer.push_back(log_line);
                }
            }
        }
    }

    /// Add a log entry
    pub fn log(&self, entry: LogLine) {
        // Add to ring buffer
        {
            let mut buffer = self.ring_buffer.write();
            if buffer.len() >= MAX_RING_BUFFER_LINES {
                buffer.pop_front();
            }
            buffer.push_back(entry.clone());
        }

        // Persist to file
        self.write_to_file(&entry);
    }

    /// Write a log entry to the file with rotation
    fn write_to_file(&self, entry: &LogLine) {
        // Check file size and rotate if needed
        if let Ok(metadata) = fs::metadata(&self.log_file_path) {
            if metadata.len() > MAX_LOG_FILE_SIZE {
                self.rotate_log_file();
            }
        }

        // Append to log file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
        {
            if let Ok(json) = serde_json::to_string(entry) {
                let _ = writeln!(file, "{}", json);
            }
        }
    }

    /// Rotate log file
    fn rotate_log_file(&self) {
        let rotated_path = self.log_file_path.with_extension("log.1");
        
        // Remove old rotated file if exists
        let _ = fs::remove_file(&rotated_path);
        
        // Rename current to .1
        let _ = fs::rename(&self.log_file_path, &rotated_path);
    }

    /// Get recent logs with optional filtering
    pub fn get_recent_logs(&self, max_lines: u32, filter: Option<LogFilter>) -> Vec<LogLine> {
        let buffer = self.ring_buffer.read();
        
        let mut logs: Vec<LogLine> = buffer
            .iter()
            .filter(|log| {
                if let Some(ref f) = filter {
                    // Filter by session_id
                    if let Some(ref sid) = f.session_id {
                        if log.session_id.as_ref() != Some(sid) {
                            return false;
                        }
                    }
                    // Filter by level
                    if let Some(level) = f.level {
                        if log.level != level {
                            return false;
                        }
                    }
                    // Filter by subsystem
                    if let Some(ref subsystem) = f.subsystem {
                        if &log.subsystem != subsystem {
                            return false;
                        }
                    }
                    // Filter by search term
                    if let Some(ref search) = f.search {
                        let search_lower = search.to_lowercase();
                        if !log.message.to_lowercase().contains(&search_lower) {
                            return false;
                        }
                    }
                    // Filter by timestamp
                    if let Some(since) = f.since {
                        if log.timestamp < since {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();
        
        // Take last N entries
        if logs.len() > max_lines as usize {
            logs = logs.split_off(logs.len() - max_lines as usize);
        }
        
        logs
    }

    /// Clear the ring buffer (does not delete file logs)
    pub fn clear_view(&self) {
        self.ring_buffer.write().clear();
    }

    /// Get logs directory path
    pub fn get_logs_dir(&self) -> PathBuf {
        self.config_dir.join("logs")
    }

    /// Get config directory path
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }
}

/// Global log manager instance - must be initialized in setup
static LOG_MANAGER: once_cell::sync::OnceCell<Arc<LogManager>> = once_cell::sync::OnceCell::new();

/// Initialize the global log manager
pub fn init_log_manager(config_dir: PathBuf) -> std::io::Result<()> {
    let manager = LogManager::new(config_dir)?;
    LOG_MANAGER.set(manager).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::AlreadyExists, "LogManager already initialized")
    })?;
    Ok(())
}

/// Get the global log manager
pub fn get_log_manager() -> Option<&'static Arc<LogManager>> {
    LOG_MANAGER.get()
}

/// Helper to log a message (convenience function)
pub fn log(level: LogLevel, subsystem: LogSubsystem, message: impl Into<String>) {
    if let Some(manager) = get_log_manager() {
        manager.log(LogLine::new(level, subsystem, message));
    }
}

/// Helper to log with session ID
pub fn log_session(
    level: LogLevel,
    subsystem: LogSubsystem,
    session_id: impl Into<String>,
    message: impl Into<String>,
) {
    if let Some(manager) = get_log_manager() {
        manager.log(LogLine::new(level, subsystem, message).with_session(session_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_private_key() {
        let input = "Key: -----BEGIN RSA PRIVATE KEY-----\nMIIE...secret...\n-----END RSA PRIVATE KEY-----";
        let result = sanitize(input);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("MIIE"));
    }

    #[test]
    fn test_sanitize_github_token() {
        let input = "Token: ghp_1234567890abcdefghijklmnopqrstuvwxyz12";
        let result = sanitize(input);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("ghp_"));
    }

    #[test]
    fn test_sanitize_aws_key() {
        let input = "AWS Key: AKIAIOSFODNN7EXAMPLE";
        let result = sanitize(input);
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_password_field() {
        let input = "password=mysecretpassword123";
        let result = sanitize(input);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("mysecretpassword"));
    }

    #[test]
    fn test_sanitize_jwt() {
        let input = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let result = sanitize(input);
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_json() {
        let json = serde_json::json!({
            "username": "test",
            "password": "secret123",
            "api_key": "12345"
        });
        let result = sanitize_json(&json);
        assert_eq!(result["password"], "[REDACTED]");
        assert_eq!(result["api_key"], "[REDACTED]");
        assert_eq!(result["username"], "test");
    }

    #[test]
    fn test_truncate_long_line() {
        let long_input = "a".repeat(5000);
        let result = sanitize(&long_input);
        assert!(result.len() < 3000);
        assert!(result.ends_with("[truncated]"));
    }
}

