use super::{
    AuthMethod, AuthRequest, ConnectRequest, ConnectionResult, 
    HostKeyDecision, SessionConfig, SessionInfo, SessionHandle, default_keepalive,
};
use crate::config::{Profile, ProfileOptions, get_config_dir};
use crate::error::{AppError, AppResult};
use crate::keychain;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

/// Create a new SSH session (does not connect yet)
#[tauri::command]
pub async fn create_session(
    state: State<'_, Arc<AppState>>,
    config: SessionConfig,
) -> AppResult<String> {
    tracing::info!(
        "Creating session for {}@{}:{}",
        config.username,
        config.host,
        config.port
    );
    state.sessions.create_session(config)
}

/// Connect with full connection request (new API)
/// This is the main connection command that handles auth internally
#[tauri::command]
pub async fn ssh_connect(
    state: State<'_, Arc<AppState>>,
    request: ConnectRequest,
) -> AppResult<ConnectionResult> {
    tracing::info!(
        "SSH connect request for {}@{}:{}",
        request.username,
        request.host,
        request.port
    );

    // Generate profile ID if we're saving
    let profile_id = if request.save_profile {
        Some(uuid::Uuid::new_v4().to_string())
    } else {
        None
    };

    // Determine auth method and store secrets securely if saving profile
    let auth_method = match &request.auth {
        AuthRequest::Agent => AuthMethod::Agent,
        AuthRequest::Password { .. } => {
            if let Some(ref pid) = profile_id {
                AuthMethod::Password { 
                    password_key: format!("password:{}", pid) 
                }
            } else {
                AuthMethod::Password { password_key: String::new() }
            }
        }
        AuthRequest::PrivateKey { .. } => {
            if let Some(ref pid) = profile_id {
                AuthMethod::Key { 
                    key_id: format!("key:{}", pid) 
                }
            } else {
                AuthMethod::Key { key_id: String::new() }
            }
        }
    };

    // Create session config
    let config = SessionConfig {
        host: request.host.clone(),
        port: request.port,
        username: request.username.clone(),
        auth_method: auth_method.clone(),
        jump_hosts: vec![],
        keepalive_interval: super::default_keepalive(),
        agent_forwarding: false,
        known_hosts_policy: super::KnownHostsPolicy::Ask,
        profile_id: profile_id.clone(),
    };

    // Create session
    let session_id = state.sessions.create_session(config)?;

    // Extract auth credentials (SECURITY: never log these!)
    let (password, private_key, passphrase) = match &request.auth {
        AuthRequest::Agent => (None, None, None),
        AuthRequest::Password { password } => (Some(password.clone()), None, None),
        AuthRequest::PrivateKey { private_key, passphrase } => {
            (None, Some(private_key.clone()), passphrase.clone())
        }
    };

    // Start connection in background
    if let Err(e) = state.sessions.connect_session(
        &session_id,
        password.clone(),
        private_key.clone(),
        passphrase.clone(),
    ) {
        // Clean up failed session
        state.sessions.remove_session(&session_id);
        return Err(e);
    }

    // Save profile if requested (after connection initiated - profile saved regardless of outcome)
    // SECURITY: Store secrets in OS keychain, never in plaintext
    if request.save_profile {
        if let Some(ref pid) = profile_id {
            // Store secrets in keychain
            match &request.auth {
                AuthRequest::Password { password } => {
                    let key = format!("password:{}", pid);
                    if let Err(e) = keychain::store_secret(&key, password) {
                        tracing::warn!("Failed to store password in keychain: {}", e);
                    }
                }
                AuthRequest::PrivateKey { private_key, passphrase } => {
                    let key_id = format!("key:{}", pid);
                    if let Err(e) = keychain::store_secret(&key_id, private_key) {
                        tracing::warn!("Failed to store private key in keychain: {}", e);
                    }
                    // Store passphrase if provided
                    if let Some(pass) = passphrase {
                        let pass_key = format!("passphrase:{}", pid);
                        if let Err(e) = keychain::store_secret(&pass_key, pass) {
                            tracing::warn!("Failed to store passphrase in keychain: {}", e);
                        }
                    }
                }
                AuthRequest::Agent => {}
            }

            // Create and save profile (without secrets - they're in keychain)
            let profile_name = request.name.unwrap_or_else(|| {
                format!("{}@{}", request.username, request.host)
            });
            
            let now = chrono::Utc::now().timestamp();
            let profile = Profile {
                id: pid.clone(),
                name: profile_name,
                host: request.host.clone(),
                port: request.port,
                username: request.username.clone(),
                auth_method,
                jump_hosts: vec![],
                options: ProfileOptions::default(),
                theme: None,
                tags: vec![],
                notes: String::new(),
                created_at: now,
                updated_at: now,
            };

            // Save profile to disk
            if let Err(e) = state.profiles.write().add(profile) {
                tracing::warn!("Failed to save profile: {}", e);
            } else {
                tracing::info!("Saved connection profile: {}", pid);
            }
        }
    }

    Ok(ConnectionResult {
        success: true,
        session_id,
        host: request.host,
        connected_at: None, // Will be set when actually connected
        error: None,
        profile_id,
    })
}

/// Connect an existing session (legacy API for compatibility)
#[tauri::command]
pub async fn connect(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    password: Option<String>,
    private_key: Option<String>,
) -> AppResult<ConnectionResult> {
    let session = state
        .sessions
        .get_session(&session_id)
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;

    tracing::info!(
        "Connecting session {} to {}",
        session_id,
        session.config.host
    );

    // Start connection in background
    state.sessions.connect_session(&session_id, password, private_key, None)?;

    Ok(ConnectionResult {
        success: true,
        session_id,
        host: session.config.host.clone(),
        connected_at: None,
        error: None,
        profile_id: session.config.profile_id.clone(),
    })
}

/// Connect using a saved profile (credentials retrieved from keychain)
#[tauri::command]
pub async fn connect_profile(
    state: State<'_, Arc<AppState>>,
    profile_id: String,
) -> AppResult<ConnectionResult> {
    // Get the profile
    let profile = state
        .profiles
        .read()
        .get(&profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.clone()))?;

    tracing::info!(
        "Connecting with saved profile {} ({}@{}:{})",
        profile.name,
        profile.username,
        profile.host,
        profile.port
    );

    // Retrieve credentials from keychain based on auth method
    let (password, private_key, passphrase) = match &profile.auth_method {
        AuthMethod::Password { password_key } => {
            if password_key.is_empty() {
                return Err(AppError::Auth("No password stored for this profile".to_string()));
            }
            let pwd = keychain::get_secret(password_key)?
                .ok_or_else(|| AppError::Auth("Password not found in keychain".to_string()))?;
            (Some(pwd), None, None)
        }
        AuthMethod::Key { key_id } => {
            if key_id.is_empty() {
                return Err(AppError::Auth("No private key stored for this profile".to_string()));
            }
            let key = keychain::get_secret(key_id)?
                .ok_or_else(|| AppError::Auth("Private key not found in keychain".to_string()))?;
            
            // Try to get passphrase if stored
            let passphrase_key = key_id.replace("key:", "passphrase:");
            let pass = keychain::get_secret(&passphrase_key).ok().flatten();
            
            (None, Some(key), pass)
        }
        AuthMethod::Agent => (None, None, None),
        AuthMethod::Interactive => {
            return Err(AppError::Auth("Interactive auth not supported for saved profiles".to_string()));
        }
    };

    // Create session config
    let config = SessionConfig {
        host: profile.host.clone(),
        port: profile.port,
        username: profile.username.clone(),
        auth_method: profile.auth_method.clone(),
        jump_hosts: profile.jump_hosts.clone(),
        keepalive_interval: profile.options.keepalive_interval,
        agent_forwarding: profile.options.agent_forwarding,
        known_hosts_policy: profile.options.known_hosts_policy.clone(),
        profile_id: Some(profile_id.clone()),
    };

    // Create session
    let session_id = state.sessions.create_session(config)?;

    // Start connection in background
    if let Err(e) = state.sessions.connect_session(
        &session_id,
        password,
        private_key,
        passphrase,
    ) {
        state.sessions.remove_session(&session_id);
        return Err(e);
    }

    Ok(ConnectionResult {
        success: true,
        session_id,
        host: profile.host.clone(),
        connected_at: None,
        error: None,
        profile_id: Some(profile_id),
    })
}

/// Handle host key verification decision from user
#[tauri::command]
pub async fn ssh_hostkey_decision(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    decision: String,
) -> AppResult<()> {
    let decision = match decision.as_str() {
        "once" => HostKeyDecision::TrustOnce,
        "always" => HostKeyDecision::TrustAlways,
        "reject" => HostKeyDecision::Reject,
        _ => return Err(AppError::InvalidConfig("Invalid decision".to_string())),
    };

    tracing::info!("Host key decision for {}: {:?}", session_id, decision);
    state.sessions.set_hostkey_decision(&session_id, decision)
}

/// Disconnect a session
#[tauri::command]
pub async fn disconnect(
    state: State<'_, Arc<AppState>>,
    session_id: String,
) -> AppResult<()> {
    tracing::info!("Disconnecting session {}", session_id);
    state.sessions.disconnect(&session_id)
}

/// Disconnect a session (alternate name for consistency)
#[tauri::command]
pub async fn ssh_disconnect(
    state: State<'_, Arc<AppState>>,
    session_id: String,
) -> AppResult<()> {
    disconnect(state, session_id).await
}

/// Send data to a session
#[tauri::command]
pub async fn send_data(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    data: Vec<u8>,
) -> AppResult<()> {
    // SECURITY: Don't log the data!
    state.sessions.send_data(&session_id, &data)
}

/// Send data to a session (alternate API with string)
#[tauri::command]
pub async fn ssh_write(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    data: String,
) -> AppResult<()> {
    // SECURITY: Don't log the data!
    state.sessions.send_data(&session_id, data.as_bytes())
}

/// Resize PTY
#[tauri::command]
pub async fn resize_pty(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> AppResult<()> {
    tracing::debug!("Resize PTY for {} to {}x{}", session_id, cols, rows);
    state.sessions.resize_pty(&session_id, cols, rows)
}

/// Resize PTY (alternate name)
#[tauri::command]
pub async fn ssh_resize(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> AppResult<()> {
    resize_pty(state, session_id, cols as u32, rows as u32).await
}

/// List all sessions
#[tauri::command]
pub async fn list_sessions(state: State<'_, Arc<AppState>>) -> AppResult<Vec<SessionInfo>> {
    Ok(state.sessions.list_sessions())
}

/// Get session info
#[tauri::command]
pub async fn get_session(
    state: State<'_, Arc<AppState>>,
    session_id: String,
) -> AppResult<SessionInfo> {
    let session = state
        .sessions
        .get_session(&session_id)
        .ok_or_else(|| AppError::SessionNotFound(session_id.clone()))?;
    
    Ok(session.info())
}

/// Headless debug probe: attempt one connection synchronously, logging state transitions (no UI).
#[tauri::command]
pub async fn ssh_debug_probe(
    state: State<'_, Arc<AppState>>,
    request: ConnectRequest,
) -> AppResult<String> {
    let config_dir = get_config_dir()?;
    let session_id = uuid::Uuid::new_v4().to_string();

    let auth_method = match &request.auth {
        AuthRequest::Agent => AuthMethod::Agent,
        AuthRequest::Password { .. } => AuthMethod::Password { password_key: String::new() },
        AuthRequest::PrivateKey { .. } => AuthMethod::Key { key_id: String::new() },
    };

    let config = SessionConfig {
        host: request.host.clone(),
        port: request.port,
        username: request.username.clone(),
        auth_method,
        jump_hosts: vec![],
        keepalive_interval: default_keepalive(),
        agent_forwarding: false,
        known_hosts_policy: super::KnownHostsPolicy::Ask,
        profile_id: None,
    };

    let (password, private_key, passphrase) = match &request.auth {
        AuthRequest::Agent => (None, None, None),
        AuthRequest::Password { password } => (Some(password.clone()), None, None),
        AuthRequest::PrivateKey { private_key, passphrase } => {
            (None, Some(private_key.clone()), passphrase.clone())
        }
    };

    let handle = SessionHandle::new(session_id.clone(), config, state.app_handle.clone());
    handle.connect_once(password, private_key, passphrase, config_dir)?;
    Ok(session_id)
}

/// Stress action: enqueue many tiny writes quickly to test backpressure/fast typing
#[tauri::command]
pub async fn ssh_stress_write(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    count: usize,
    chunk: String,
) -> AppResult<()> {
    let data = chunk.into_bytes();
    for _ in 0..count {
        state.sessions.send_data(&session_id, &data)?;
    }
    Ok(())
}
