pub mod commands;

use crate::error::{AppError, AppResult};
use keyring::Entry;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;

const SERVICE_NAME: &str = "neonshell";

// Track if we've already warned about fallback mode
static FALLBACK_WARNING_SHOWN: AtomicBool = AtomicBool::new(false);

/// Check if we're using the insecure fallback
pub fn is_using_fallback() -> bool {
    // Try a test operation to see if keyring works
    let test_entry = Entry::new(SERVICE_NAME, "__neonshell_keyring_test__");
    match test_entry {
        Ok(entry) => {
            // Try to set and delete a test value
            if entry.set_password("test").is_ok() {
                let _ = entry.delete_password();
                false // Keyring works
            } else {
                true // Keyring doesn't work
            }
        }
        Err(_) => true, // Keyring doesn't work
    }
}

/// Get the fallback secrets file path
fn get_fallback_path() -> AppResult<PathBuf> {
    let config_dir = crate::config::get_config_dir()?;
    Ok(config_dir.join(".secrets.enc"))
}

/// Get the encryption key path (derived from machine-specific data)
fn get_key_path() -> AppResult<PathBuf> {
    let config_dir = crate::config::get_config_dir()?;
    Ok(config_dir.join(".keyfile"))
}

/// Get or create the encryption key for fallback storage
/// WARNING: This is NOT as secure as OS keychain - the key is stored on disk
fn get_or_create_fallback_key() -> AppResult<[u8; 32]> {
    use rand::RngCore;
    
    let key_path = get_key_path()?;
    
    if key_path.exists() {
        let key_data = fs::read(&key_path)
            .map_err(|e| AppError::Keychain(format!("Failed to read key file: {}", e)))?;
        if key_data.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&key_data);
            return Ok(key);
        }
    }
    
    // Generate new key
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    
    // Save key with restrictive permissions
    fs::write(&key_path, &key)
        .map_err(|e| AppError::Keychain(format!("Failed to write key file: {}", e)))?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        let _ = fs::set_permissions(&key_path, perms);
    }
    
    Ok(key)
}

/// Load the fallback secrets store
fn load_fallback_store() -> AppResult<HashMap<String, String>> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    
    let path = get_fallback_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    
    let encrypted_data = fs::read(&path)
        .map_err(|e| AppError::Keychain(format!("Failed to read secrets file: {}", e)))?;
    
    if encrypted_data.len() < 12 {
        return Ok(HashMap::new());
    }
    
    let key = get_or_create_fallback_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Keychain(format!("Failed to create cipher: {}", e)))?;
    
    let nonce = Nonce::from_slice(&encrypted_data[..12]);
    let ciphertext = &encrypted_data[12..];
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Keychain(format!("Failed to decrypt secrets: {}", e)))?;
    
    let json_str = String::from_utf8(plaintext)
        .map_err(|e| AppError::Keychain(format!("Invalid UTF-8 in secrets: {}", e)))?;
    
    serde_json::from_str(&json_str)
        .map_err(|e| AppError::Keychain(format!("Failed to parse secrets: {}", e)))
}

/// Save the fallback secrets store
fn save_fallback_store(store: &HashMap<String, String>) -> AppResult<()> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use rand::RngCore;
    
    let path = get_fallback_path()?;
    let key = get_or_create_fallback_key()?;
    
    let json_str = serde_json::to_string(store)
        .map_err(|e| AppError::Keychain(format!("Failed to serialize secrets: {}", e)))?;
    
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Keychain(format!("Failed to create cipher: {}", e)))?;
    
    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, json_str.as_bytes())
        .map_err(|e| AppError::Keychain(format!("Failed to encrypt secrets: {}", e)))?;
    
    // Prepend nonce to ciphertext
    let mut output = nonce_bytes.to_vec();
    output.extend(ciphertext);
    
    fs::write(&path, &output)
        .map_err(|e| AppError::Keychain(format!("Failed to write secrets file: {}", e)))?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        let _ = fs::set_permissions(&path, perms);
    }
    
    Ok(())
}

/// Store secret using fallback (encrypted local file)
fn store_secret_fallback(key: &str, secret: &str) -> AppResult<()> {
    let mut store = load_fallback_store()?;
    store.insert(key.to_string(), secret.to_string());
    save_fallback_store(&store)?;
    tracing::warn!("Stored secret in INSECURE fallback storage: {}", key);
    Ok(())
}

/// Get secret from fallback
fn get_secret_fallback(key: &str) -> AppResult<Option<String>> {
    let store = load_fallback_store()?;
    Ok(store.get(key).cloned())
}

/// Delete secret from fallback
#[allow(dead_code)]
fn delete_secret_fallback(key: &str) -> AppResult<()> {
    let mut store = load_fallback_store()?;
    store.remove(key);
    save_fallback_store(&store)?;
    Ok(())
}

/// Emit a warning to the frontend about insecure storage
pub fn emit_fallback_warning(app_handle: &tauri::AppHandle) {
    if FALLBACK_WARNING_SHOWN.swap(true, Ordering::SeqCst) {
        return; // Already shown
    }
    
    let _ = app_handle.emit("keychain:fallback_warning", serde_json::json!({
        "title": "⚠️ Insecure Secret Storage",
        "message": "Your system does not have a secure keyring available. Secrets are being stored in an encrypted local file, which is LESS SECURE than the OS keychain.\n\nFor better security, please install a keyring:\n• GNOME: gnome-keyring (usually pre-installed)\n• KDE: KWallet with ksecretservice\n• Other: keepassxc with Secret Service enabled\n\nAfter installing, restart NeonShell.",
        "severity": "warning"
    }));
    
    tracing::warn!(
        "⚠️ SECURITY WARNING: No OS keyring available! Using insecure fallback storage. \
        Install gnome-keyring, KWallet, or keepassxc for secure secret storage."
    );
}

/// Store a secret in the OS keychain (with fallback)
pub fn store_secret(key: &str, secret: &str) -> AppResult<()> {
    // Try OS keychain first
    match Entry::new(SERVICE_NAME, key) {
        Ok(entry) => {
            match entry.set_password(secret) {
                Ok(()) => {
                    tracing::debug!("Stored secret in OS keychain: {}", key);
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("OS keychain failed, using fallback: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("OS keychain unavailable, using fallback: {}", e);
        }
    }
    
    // Fallback to encrypted local storage
    store_secret_fallback(key, secret)
}

/// Store a secret with app handle for warning emission
pub fn store_secret_with_warning(key: &str, secret: &str, app_handle: &tauri::AppHandle) -> AppResult<()> {
    // Try OS keychain first
    match Entry::new(SERVICE_NAME, key) {
        Ok(entry) => {
            match entry.set_password(secret) {
                Ok(()) => {
                    tracing::debug!("Stored secret in OS keychain: {}", key);
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("OS keychain failed, using fallback: {}", e);
                    emit_fallback_warning(app_handle);
                }
            }
        }
        Err(e) => {
            tracing::warn!("OS keychain unavailable, using fallback: {}", e);
            emit_fallback_warning(app_handle);
        }
    }
    
    // Fallback to encrypted local storage
    store_secret_fallback(key, secret)
}

/// Retrieve a secret from the OS keychain (with fallback)
pub fn get_secret(key: &str) -> AppResult<Option<String>> {
    // Try OS keychain first
    match Entry::new(SERVICE_NAME, key) {
        Ok(entry) => {
            match entry.get_password() {
                Ok(password) => return Ok(Some(password)),
                Err(keyring::Error::NoEntry) => {
                    // Not in keychain, try fallback
                }
                Err(e) => {
                    tracing::debug!("OS keychain get failed, trying fallback: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::debug!("OS keychain unavailable for get, trying fallback: {}", e);
        }
    }
    
    // Try fallback
    get_secret_fallback(key)
}

/// Delete a secret from the OS keychain (and fallback)
pub fn delete_secret(key: &str) -> AppResult<()> {
    // Try to delete from both locations
    let mut deleted = false;
    
    // Try OS keychain
    if let Ok(entry) = Entry::new(SERVICE_NAME, key) {
        match entry.delete_password() {
            Ok(()) => {
                tracing::debug!("Deleted secret from OS keychain: {}", key);
                deleted = true;
            }
            Err(keyring::Error::NoEntry) => {}
            Err(e) => {
                tracing::debug!("OS keychain delete failed: {}", e);
            }
        }
    }
    
    // Also try fallback
    if let Ok(mut store) = load_fallback_store() {
        if store.remove(key).is_some() {
            let _ = save_fallback_store(&store);
            deleted = true;
        }
    }
    
    if deleted {
        tracing::debug!("Deleted secret: {}", key);
    }
    Ok(())
}

/// Check if a secret exists in the OS keychain (or fallback)
pub fn has_secret(key: &str) -> AppResult<bool> {
    // Check OS keychain
    if let Ok(entry) = Entry::new(SERVICE_NAME, key) {
        if entry.get_password().is_ok() {
            return Ok(true);
        }
    }
    
    // Check fallback
    if let Ok(store) = load_fallback_store() {
        if store.contains_key(key) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Store a private key in the keychain
pub fn store_private_key(key_id: &str, key_content: &str) -> AppResult<()> {
    let key = format!("key:{}", key_id);
    store_secret(&key, key_content)
}

/// Retrieve a private key from the keychain
pub fn get_private_key(key_id: &str) -> AppResult<Option<String>> {
    let key = format!("key:{}", key_id);
    get_secret(&key)
}

/// Store a password in the keychain
pub fn store_password(profile_id: &str, password: &str) -> AppResult<()> {
    let key = format!("password:{}", profile_id);
    store_secret(&key, password)
}

/// Retrieve a password from the keychain
pub fn get_password(profile_id: &str) -> AppResult<Option<String>> {
    let key = format!("password:{}", profile_id);
    get_secret(&key)
}

/// Check keyring availability and return status info
pub fn get_keyring_status() -> KeyringStatus {
    let test_key = "__neonshell_keyring_test__";
    
    match Entry::new(SERVICE_NAME, test_key) {
        Ok(entry) => {
            match entry.set_password("test") {
                Ok(()) => {
                    let _ = entry.delete_password();
                    KeyringStatus {
                        available: true,
                        backend: detect_backend(),
                        using_fallback: false,
                        warning: None,
                    }
                }
                Err(e) => KeyringStatus {
                    available: false,
                    backend: "none".to_string(),
                    using_fallback: true,
                    warning: Some(format!(
                        "OS keyring unavailable ({}). Using encrypted local storage. \
                        For better security, install gnome-keyring, KWallet, or keepassxc.",
                        e
                    )),
                },
            }
        }
        Err(e) => KeyringStatus {
            available: false,
            backend: "none".to_string(),
            using_fallback: true,
            warning: Some(format!(
                "OS keyring unavailable ({}). Using encrypted local storage. \
                For better security, install gnome-keyring, KWallet, or keepassxc.",
                e
            )),
        },
    }
}

fn detect_backend() -> String {
    #[cfg(target_os = "macos")]
    return "macOS Keychain".to_string();
    
    #[cfg(target_os = "windows")]
    return "Windows Credential Manager".to_string();
    
    #[cfg(target_os = "linux")]
    return "Secret Service (D-Bus)".to_string();
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return "Unknown".to_string();
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct KeyringStatus {
    pub available: bool,
    pub backend: String,
    pub using_fallback: bool,
    pub warning: Option<String>,
}
