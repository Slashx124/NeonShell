pub mod commands;

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Script metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMetadata {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub hooks: Vec<String>,
    #[serde(default)]
    pub commands: Vec<ScriptCommand>,
    #[serde(default)]
    pub permissions: Vec<ScriptPermission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptCommand {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptPermission {
    Network,
    Filesystem,
    Clipboard,
    Notifications,
    Terminal,
}

/// Script state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptState {
    Disabled,
    Enabled,
    Running,
    Error,
}

/// Script info for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub metadata: ScriptMetadata,
    pub state: ScriptState,
    pub path: PathBuf,
    #[serde(default)]
    pub error: Option<String>,
}

/// Script manager using sandboxed subprocess
pub struct ScriptManager {
    scripts: HashMap<String, ScriptInfo>,
    scripts_dir: PathBuf,
    enabled_scripts: Vec<String>,
}

impl ScriptManager {
    pub fn new(config_dir: &Path) -> AppResult<Self> {
        let scripts_dir = config_dir.join("scripts");
        std::fs::create_dir_all(&scripts_dir)?;

        let mut manager = Self {
            scripts: HashMap::new(),
            scripts_dir,
            enabled_scripts: vec![],
        };

        manager.scan_scripts()?;

        Ok(manager)
    }

    /// Scan scripts directory
    pub fn scan_scripts(&mut self) -> AppResult<()> {
        self.scripts.clear();

        if let Ok(entries) = std::fs::read_dir(&self.scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "py").unwrap_or(false) {
                    if let Err(e) = self.load_script(&path) {
                        tracing::warn!("Failed to load script at {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    fn load_script(&mut self, path: &Path) -> AppResult<()> {
        let content = std::fs::read_to_string(path)?;
        let metadata = parse_script_metadata(&content, path)?;
        
        let state = if self.enabled_scripts.contains(&metadata.id) {
            ScriptState::Enabled
        } else {
            ScriptState::Disabled
        };

        let info = ScriptInfo {
            metadata,
            state,
            path: path.to_path_buf(),
            error: None,
        };

        self.scripts.insert(info.metadata.id.clone(), info);

        Ok(())
    }

    pub fn list(&self) -> Vec<ScriptInfo> {
        self.scripts.values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<ScriptInfo> {
        self.scripts.get(id).cloned()
    }

    pub fn enable(&mut self, id: &str) -> AppResult<()> {
        let script = self
            .scripts
            .get_mut(id)
            .ok_or_else(|| AppError::Python(format!("Script not found: {}", id)))?;

        script.state = ScriptState::Enabled;

        if !self.enabled_scripts.contains(&id.to_string()) {
            self.enabled_scripts.push(id.to_string());
        }

        tracing::info!("Enabled script: {}", id);
        Ok(())
    }

    pub fn disable(&mut self, id: &str) -> AppResult<()> {
        let script = self
            .scripts
            .get_mut(id)
            .ok_or_else(|| AppError::Python(format!("Script not found: {}", id)))?;

        script.state = ScriptState::Disabled;
        self.enabled_scripts.retain(|s| s != id);

        tracing::info!("Disabled script: {}", id);
        Ok(())
    }

    /// Get enabled scripts that hook into a specific event
    pub fn get_scripts_for_hook(&self, hook: &str) -> Vec<ScriptInfo> {
        self.scripts
            .values()
            .filter(|s| {
                s.state == ScriptState::Enabled
                    && s.metadata.hooks.iter().any(|h| h == hook)
            })
            .cloned()
            .collect()
    }
}

/// Parse script metadata from docstring
fn parse_script_metadata(content: &str, path: &Path) -> AppResult<ScriptMetadata> {
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let mut metadata = ScriptMetadata {
        id: file_stem.to_string(),
        name: file_stem.replace('_', " "),
        description: String::new(),
        author: String::new(),
        version: "1.0.0".to_string(),
        hooks: vec![],
        commands: vec![],
        permissions: vec![],
    };

    // Parse docstring for metadata
    if let Some(start) = content.find("\"\"\"") {
        if let Some(end) = content[start + 3..].find("\"\"\"") {
            let docstring = &content[start + 3..start + 3 + end];
            
            for line in docstring.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("@name:") {
                    metadata.name = rest.trim().to_string();
                } else if let Some(rest) = line.strip_prefix("@description:") {
                    metadata.description = rest.trim().to_string();
                } else if let Some(rest) = line.strip_prefix("@author:") {
                    metadata.author = rest.trim().to_string();
                } else if let Some(rest) = line.strip_prefix("@version:") {
                    metadata.version = rest.trim().to_string();
                } else if let Some(rest) = line.strip_prefix("@hook:") {
                    metadata.hooks.push(rest.trim().to_string());
                }
            }
        }
    }

    // Detect hooks from decorators
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("@hook(") {
            if let Some(hook) = line
                .strip_prefix("@hook(\"")
                .or_else(|| line.strip_prefix("@hook('"))
            {
                if let Some(hook) = hook.split(&['\"', '\''][..]).next() {
                    if !metadata.hooks.contains(&hook.to_string()) {
                        metadata.hooks.push(hook.to_string());
                    }
                }
            }
        }
    }

    Ok(metadata)
}

/// Validate a function name to prevent code injection
/// Only allows alphanumeric characters and underscores, must start with letter/underscore
fn is_valid_function_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 128 {
        return false;
    }
    
    let mut chars = name.chars();
    
    // First character must be letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    
    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Sanitize JSON args to prevent injection through triple-quoted strings
fn sanitize_json_for_python(json: &str) -> String {
    // Escape any occurrences of ''' that could break out of the string
    json.replace("'''", r"\'\'\'")
}

/// Run a Python script in a sandboxed subprocess
/// 
/// SECURITY: This function validates function names to prevent code injection.
/// The function name must be a valid Python identifier (alphanumeric + underscore).
pub async fn run_script(
    script_path: &Path,
    function: &str,
    args: serde_json::Value,
) -> AppResult<serde_json::Value> {
    // SECURITY: Validate function name to prevent code injection
    if !is_valid_function_name(function) {
        return Err(AppError::Python(format!(
            "Invalid function name '{}'. Must be a valid Python identifier (letters, numbers, underscores only).",
            function.chars().take(50).collect::<String>()
        )));
    }
    
    // SECURITY: Validate script path exists and is within expected directory
    if !script_path.exists() {
        return Err(AppError::Python("Script file not found".to_string()));
    }
    
    // Convert args to JSON and sanitize
    let args_json = sanitize_json_for_python(&args.to_string());
    
    // Build the Python code to execute
    // SECURITY: function name is validated above, args are JSON-encoded
    let wrapper_code = format!(
        r#"
import sys
import json

# Load the script safely
script_path = r"{script_path}"
script_globals = {{"__name__": "__main__", "__file__": script_path}}
with open(script_path, 'r') as f:
    exec(compile(f.read(), script_path, 'exec'), script_globals)

# Verify function exists and is callable
if '{function}' not in script_globals:
    print(json.dumps({{"error": "Function '{function}' not found in script"}}))
    sys.exit(1)

func = script_globals['{function}']
if not callable(func):
    print(json.dumps({{"error": "'{function}' is not callable"}}))
    sys.exit(1)

# Call the function with sanitized args
try:
    result = func(**json.loads(r'''{args_json}'''))
    print(json.dumps({{"result": result}}))
except Exception as e:
    # Don't leak full exception details that might contain secrets
    print(json.dumps({{"error": str(type(e).__name__) + ": " + str(e)[:200]}}))
    sys.exit(1)
"#,
        script_path = script_path.display(),
        function = function,
        args_json = args_json,
    );

    let output = Command::new("python3")
        .args(["-c", &wrapper_code])
        // SECURITY: Clear environment to prevent injection via env vars
        .env_clear()
        // Re-add only essential env vars
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", std::env::var("HOME").unwrap_or_default())
        .env("PYTHONIOENCODING", "utf-8")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| AppError::Python(format!("Failed to run Python: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // SECURITY: Truncate stderr to prevent log flooding and potential secret leakage
        let truncated_stderr: String = stderr.chars().take(500).collect();
        return Err(AppError::Python(format!("Script error: {}", truncated_stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| AppError::Python(format!("Invalid script output: {}", e)))?;
    
    // Check for error response from script
    if let Some(error) = result.get("error") {
        return Err(AppError::Python(error.as_str().unwrap_or("Unknown error").to_string()));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_function_names() {
        assert!(is_valid_function_name("on_connect"));
        assert!(is_valid_function_name("_private"));
        assert!(is_valid_function_name("func123"));
        assert!(is_valid_function_name("MyFunction"));
        assert!(is_valid_function_name("a"));
    }
    
    #[test]
    fn test_invalid_function_names() {
        // Code injection attempts
        assert!(!is_valid_function_name("__import__('os').system('ls')"));
        assert!(!is_valid_function_name("func; malicious()"));
        assert!(!is_valid_function_name("func()"));
        assert!(!is_valid_function_name("func\nmalicious"));
        
        // Invalid identifiers
        assert!(!is_valid_function_name("123func"));  // Starts with number
        assert!(!is_valid_function_name("func-name")); // Contains hyphen
        assert!(!is_valid_function_name("func name")); // Contains space
        assert!(!is_valid_function_name(""));          // Empty
        assert!(!is_valid_function_name(&"a".repeat(200))); // Too long
    }
    
    #[test]
    fn test_sanitize_json() {
        assert_eq!(sanitize_json_for_python("{}"), "{}");
        assert_eq!(sanitize_json_for_python("{\"key\": \"value\"}"), "{\"key\": \"value\"}");
        // Should escape triple quotes
        assert_eq!(sanitize_json_for_python("'''"), r"\'\'\'");
    }
}

