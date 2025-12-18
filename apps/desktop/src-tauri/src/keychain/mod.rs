pub mod commands;

use crate::error::{AppError, AppResult};
use keyring::Entry;

const SERVICE_NAME: &str = "neonshell";

/// Store a secret in the OS keychain
pub fn store_secret(key: &str, secret: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    entry.set_password(secret)?;
    tracing::debug!("Stored secret: {}", key);
    Ok(())
}

/// Retrieve a secret from the OS keychain
pub fn get_secret(key: &str) -> AppResult<Option<String>> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keychain(e.to_string())),
    }
}

/// Delete a secret from the OS keychain
pub fn delete_secret(key: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    match entry.delete_password() {
        Ok(()) => {
            tracing::debug!("Deleted secret: {}", key);
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(AppError::Keychain(e.to_string())),
    }
}

/// Check if a secret exists in the OS keychain
pub fn has_secret(key: &str) -> AppResult<bool> {
    let entry = Entry::new(SERVICE_NAME, key)?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(AppError::Keychain(e.to_string())),
    }
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
