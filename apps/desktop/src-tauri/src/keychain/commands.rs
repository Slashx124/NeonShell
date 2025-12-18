use crate::error::{AppError, AppResult};

// =============================================================================
// SECURITY: Keychain key validation
// =============================================================================

/// Allowed key prefixes for keychain access
/// This prevents arbitrary key enumeration and limits what frontend can access
const ALLOWED_KEY_PREFIXES: &[&str] = &[
    "password:",   // SSH passwords by profile ID
    "key:",        // SSH private keys by key ID
    "passphrase:", // Key passphrases
];

/// Validate that a keychain key follows allowed patterns
/// 
/// SECURITY: This prevents the frontend from accessing arbitrary keychain entries.
/// Only keys with specific prefixes are allowed.
fn validate_keychain_key(key: &str) -> AppResult<()> {
    // SECURITY: Check key is not empty and not too long
    if key.is_empty() || key.len() > 256 {
        return Err(AppError::Keychain("Invalid key length".to_string()));
    }
    
    // SECURITY: Key must start with an allowed prefix
    let has_valid_prefix = ALLOWED_KEY_PREFIXES.iter().any(|prefix| key.starts_with(prefix));
    
    if !has_valid_prefix {
        return Err(AppError::PermissionDenied(format!(
            "Keychain key must start with one of: {}",
            ALLOWED_KEY_PREFIXES.join(", ")
        )));
    }
    
    // SECURITY: Validate the ID portion (after the prefix)
    for prefix in ALLOWED_KEY_PREFIXES {
        if key.starts_with(prefix) {
            let id = &key[prefix.len()..];
            if id.is_empty() {
                return Err(AppError::Keychain("Key ID cannot be empty".to_string()));
            }
            // SECURITY: ID must be alphanumeric with limited special chars
            if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                return Err(AppError::Keychain("Key ID contains invalid characters".to_string()));
            }
            break;
        }
    }
    
    Ok(())
}

/// Store a secret in the OS keychain
/// 
/// SECURITY: 
/// - Key must follow allowed patterns (password:*, key:*, passphrase:*)
/// - The actual secret value is never logged
#[tauri::command]
pub async fn store_secret(key: String, secret: String) -> AppResult<()> {
    // SECURITY: Validate key before storing
    validate_keychain_key(&key)?;
    
    // SECURITY: Log only the key pattern, not the full key (could contain identifiers)
    let key_type = key.split(':').next().unwrap_or("unknown");
    tracing::info!("Storing secret of type: {}", key_type);
    
    super::store_secret(&key, &secret)
}

/// Retrieve a secret from the OS keychain
/// 
/// SECURITY: Only allows retrieval of keys with valid prefixes
#[tauri::command]
pub async fn get_secret(key: String) -> AppResult<Option<String>> {
    // SECURITY: Validate key before retrieval
    validate_keychain_key(&key)?;
    
    tracing::debug!("Retrieving secret (validated key)");
    super::get_secret(&key)
}

/// Delete a secret from the OS keychain
/// 
/// SECURITY: Only allows deletion of keys with valid prefixes
#[tauri::command]
pub async fn delete_secret(key: String) -> AppResult<()> {
    // SECURITY: Validate key before deletion
    validate_keychain_key(&key)?;
    
    let key_type = key.split(':').next().unwrap_or("unknown");
    tracing::info!("Deleting secret of type: {}", key_type);
    
    super::delete_secret(&key)
}

/// Check if a secret exists in the OS keychain
/// 
/// SECURITY: Only allows checking keys with valid prefixes
#[tauri::command]
pub async fn has_secret(key: String) -> AppResult<bool> {
    // SECURITY: Validate key before checking
    validate_keychain_key(&key)?;
    
    super::has_secret(&key)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_keychain_keys() {
        assert!(validate_keychain_key("password:profile-123").is_ok());
        assert!(validate_keychain_key("key:my_ssh_key").is_ok());
        assert!(validate_keychain_key("passphrase:key-456").is_ok());
    }
    
    #[test]
    fn test_invalid_keychain_keys() {
        // Missing prefix
        assert!(validate_keychain_key("profile-123").is_err());
        assert!(validate_keychain_key("arbitrary_key").is_err());
        
        // Empty
        assert!(validate_keychain_key("").is_err());
        
        // Invalid prefix
        assert!(validate_keychain_key("other:something").is_err());
        assert!(validate_keychain_key("admin:secret").is_err());
        
        // Empty ID after prefix
        assert!(validate_keychain_key("password:").is_err());
        
        // Invalid characters in ID
        assert!(validate_keychain_key("password:../../../etc").is_err());
        assert!(validate_keychain_key("password:test;rm -rf /").is_err());
    }
}

