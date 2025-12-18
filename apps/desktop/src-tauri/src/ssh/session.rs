use crate::error::{AppError, AppResult};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ssh2::{Session as Ssh2Session, Channel, HostKeyType, KnownHostFileKind, CheckResult};
use std::io::{Read, Write};
use std::env;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use serde_json::json;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{error::TrySendError, error::TryRecvError};

/// SSH session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    #[serde(default)]
    pub jump_hosts: Vec<JumpHost>,
    #[serde(default = "default_keepalive")]
    pub keepalive_interval: u32,
    #[serde(default)]
    pub agent_forwarding: bool,
    #[serde(default)]
    pub known_hosts_policy: KnownHostsPolicy,
    #[serde(default)]
    pub profile_id: Option<String>,
}

pub fn default_keepalive() -> u32 {
    20 // send keepalives every 20s by default (within requested 15-30s)
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthMethod {
    #[serde(rename = "password")]
    Password {
        /// Keychain key reference (for saved profiles)
        #[serde(default)]
        password_key: String,
    },
    #[serde(rename = "key")]
    Key {
        /// Keychain key reference (for saved profiles) 
        #[serde(default)]
        key_id: String,
    },
    #[serde(rename = "agent")]
    Agent,
    #[serde(rename = "interactive")]
    Interactive,
}

impl Default for AuthMethod {
    fn default() -> Self {
        AuthMethod::Agent
    }
}

/// Jump host configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpHost {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
}

/// Known hosts policy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum KnownHostsPolicy {
    #[default]
    Strict,
    Ask,
    Accept,
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Created,
    Connecting,
    WaitingForHostKey,
    Connected,
    Disconnected,
    Error,
}

/// Session info for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub state: SessionState,
    pub profile_id: Option<String>,
    pub connected_at: Option<i64>,
}

/// Host key information for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostKeyInfo {
    pub session_id: String,
    pub host: String,
    pub port: u16,
    pub key_type: String,
    pub fingerprint_sha256: String,
}

/// Host key decision from user
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HostKeyDecision {
    #[serde(rename = "once")]
    TrustOnce,
    #[serde(rename = "always")]
    TrustAlways,
    #[serde(rename = "reject")]
    Reject,
}

/// Connect request from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthRequest,
    pub name: Option<String>,
    pub save_profile: bool,
}

/// Auth details for connect request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthRequest {
    #[serde(rename = "agent")]
    Agent,
    #[serde(rename = "password")]
    Password { password: String },
    #[serde(rename = "private_key")]
    PrivateKey { 
        private_key: String,
        #[serde(default)]
        passphrase: Option<String>,
    },
}

/// Connection result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub session_id: String,
    pub host: String,
    pub connected_at: Option<i64>,
    pub error: Option<String>,
    /// Profile ID if the connection was saved as a profile
    pub profile_id: Option<String>,
}

/// Internal session handle for managing SSH connection
pub struct SessionHandle {
    pub id: String,
    pub config: SessionConfig,
    state: RwLock<SessionState>,
    app_handle: AppHandle,
    // Bounded channel to send data to SSH writer loop
    write_tx: RwLock<Option<mpsc::Sender<SessionCommand>>>,
    connected_at: RwLock<Option<i64>>,
    // For host key verification
    hostkey_decision: RwLock<Option<HostKeyDecision>>,
}

enum SessionCommand {
    Write(Vec<u8>),
    Resize(u32, u32),
    Close,
}

const MAX_PENDING_BYTES: usize = 256 * 1024; // 256KB write buffer
const WRITE_CHUNK_BYTES: usize = 8 * 1024; // limit each write call

impl SessionHandle {
    pub fn new(id: String, config: SessionConfig, app_handle: AppHandle) -> Self {
        Self {
            id,
            config,
            state: RwLock::new(SessionState::Created),
            app_handle,
            write_tx: RwLock::new(None),
            connected_at: RwLock::new(None),
            hostkey_decision: RwLock::new(None),
        }
    }

    pub fn info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id.clone(),
            host: self.config.host.clone(),
            port: self.config.port,
            username: self.config.username.clone(),
            state: *self.state.read(),
            profile_id: self.config.profile_id.clone(),
            connected_at: *self.connected_at.read(),
        }
    }

    pub fn state(&self) -> SessionState {
        *self.state.read()
    }

    pub fn set_state(&self, state: SessionState) {
        *self.state.write() = state;
        self.emit_state_change();
    }

    fn emit_state_change(&self) {
        let info = self.info();
        let _ = self.app_handle.emit(&format!("ssh:session:{}", self.id), &info);
        let _ = self.app_handle.emit("ssh:sessions", &info);
    }

    /// Emit a concise debug event (throttled by caller)
    fn emit_debug(&self, stage: &str, details: serde_json::Value) {
        let payload = json!({
            "session_id": self.id,
            "stage": stage,
            "details": details
        });
        let _ = self.app_handle.emit("ssh:debug", payload);
    }

    pub fn set_hostkey_decision(&self, decision: HostKeyDecision) {
        *self.hostkey_decision.write() = Some(decision);
    }

    pub fn get_hostkey_decision(&self) -> Option<HostKeyDecision> {
        *self.hostkey_decision.read()
    }

    pub fn clear_hostkey_decision(&self) {
        *self.hostkey_decision.write() = None;
    }

    /// Start the SSH connection in a background thread
    pub fn start_connection(
        self: Arc<Self>,
        password: Option<String>,
        private_key: Option<String>,
        passphrase: Option<String>,
        config_dir: PathBuf,
    ) {
        let session = self.clone();
        
        thread::spawn(move || {
            let result = session.connect_blocking(password, private_key, passphrase, config_dir);
            
            if let Err(e) = result {
                // SECURITY: Don't log the actual error which might contain sensitive info
                tracing::error!("SSH connection failed for session {}", session.id);
                session.set_state(SessionState::Error);
                
                // Sanitize error message before sending to frontend
                let sanitized_error = sanitize_error_message(&e.to_string());
                let _ = session.app_handle.emit("ssh:error", serde_json::json!({
                    "session_id": session.id,
                    "message": sanitized_error
                }));
            }
        });
    }

    /// Blocking SSH connection (runs in background thread)
    pub fn connect_blocking(
        &self,
        password: Option<String>,
        private_key: Option<String>,
        passphrase: Option<String>,
        config_dir: PathBuf,
    ) -> AppResult<()> {
        self.set_state(SessionState::Connecting);

        // Log connection attempt (no secrets!)
        tracing::info!(
            "Connecting to {}@{}:{} (session {})",
            self.config.username,
            self.config.host,
            self.config.port,
            self.id
        );

        // Connect TCP
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let tcp = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| AppError::Connection(format!("Invalid address: {}", e)))?,
            Duration::from_secs(30),
        ).map_err(|e| AppError::Connection(format!("TCP connect failed: {}", e)))?;
        
        // Don't set read timeout - we'll handle blocking in the I/O loop
        tcp.set_nodelay(true)?; // Disable Nagle's algorithm for better latency
        tcp.set_write_timeout(Some(Duration::from_secs(30)))?;

        // Create SSH session
        let mut ssh_session = Ssh2Session::new()
            .map_err(|e| AppError::Ssh(format!("Failed to create SSH session: {}", e)))?;
        
        ssh_session.set_tcp_stream(tcp);
        ssh_session.set_timeout(30_000); // 30 seconds for operations
        
        // Enable SSH keepalive to prevent timeout
        ssh_session.set_keepalive(true, self.config.keepalive_interval);
        
        // SSH handshake
        ssh_session.handshake()
            .map_err(|e| AppError::Ssh(format!("SSH handshake failed: {}", e)))?;

        // Verify host key
        self.verify_host_key(&ssh_session, &config_dir)?;

        // Authenticate
        self.authenticate(&mut ssh_session, password, private_key, passphrase)?;

        // Open interactive shell with fallbacks
        let mut channel = self.open_interactive_channel(&mut ssh_session)?;

        // Set non-blocking mode for reads
        ssh_session.set_blocking(false);

        // Setup bounded command channel
        let (write_tx, mut write_rx) = mpsc::channel::<SessionCommand>(1024);
        *self.write_tx.write() = Some(write_tx);

        // Update state
        *self.connected_at.write() = Some(chrono::Utc::now().timestamp());
        self.set_state(SessionState::Connected);

        // Emit connected event
        let _ = self.app_handle.emit("ssh:connected", self.info());

        tracing::info!("SSH connected successfully (session {})", self.id);

        // Main I/O loop
        tracing::debug!("Entering I/O loop (session {})", self.id);
        self.run_io_loop(&mut channel, &mut write_rx, ssh_session)?;

        Ok(())
    }

    /// Direct connect for debug probes (no background thread)
    pub fn connect_once(
        &self,
        password: Option<String>,
        private_key: Option<String>,
        passphrase: Option<String>,
        config_dir: PathBuf,
    ) -> AppResult<()> {
        self.connect_blocking(password, private_key, passphrase, config_dir)
    }

    /// Open an interactive shell with fallbacks
    fn open_interactive_channel(&self, ssh_session: &mut Ssh2Session) -> AppResult<Channel> {
        // Helper to request PTY + merge stderr
        let open_channel = |label: &str| -> AppResult<Channel> {
            tracing::debug!("Opening channel [{}] (session {})", label, self.id);
            self.emit_debug("channel_open", json!({"label": label}));
            let mut ch = ssh_session
                .channel_session()
                .map_err(|e| AppError::Ssh(format!("Failed to open channel [{}]: {}", label, e)))?;
            ch.handle_extended_data(ssh2::ExtendedData::Merge)
                .map_err(|e| AppError::Ssh(format!("Failed to merge stderr [{}]: {}", label, e)))?;
            ch.request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
                .map_err(|e| AppError::Ssh(format!("Failed to request PTY [{}]: {}", label, e)))?;
            self.emit_debug("pty_ok", json!({"label": label}));
            Ok(ch)
        };

        // Primary: shell()
        if let Ok(mut ch) = open_channel("primary") {
            match ch.shell() {
                Ok(_) => {
                    tracing::debug!("Shell started (primary) session {}", self.id);
                    self.emit_debug("shell_ok", json!({"label": "primary"}));
                    return Ok(ch);
                }
                Err(e) => {
                    self.emit_debug("shell_fail", json!({"label": "primary", "error": e.to_string()}));
                    let _ = ch.close();
                }
            }
        }

        // Fallback A: exec $SHELL -l or /bin/sh -l
        let candidate_shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let fallback_cmds = vec![
            format!("{} -l", candidate_shell),
            "/bin/sh -l".to_string(),
        ];

        for cmd in fallback_cmds {
            if let Ok(mut ch) = open_channel("fallback_exec_shell") {
                self.emit_debug("exec_try", json!({"cmd": cmd}));
                match ch.exec(&cmd) {
                    Ok(_) => {
                        self.emit_debug("exec_ok", json!({"cmd": cmd}));
                        tracing::debug!("Exec shell started with cmd '{}' (session {})", cmd, self.id);
                        return Ok(ch);
                    }
                    Err(e) => {
                        self.emit_debug("exec_fail", json!({"cmd": cmd, "error": e.to_string()}));
                        let _ = ch.close();
                    }
                }
            }
        }

        // Fallback B: explicit bash -l then sh -l
        for cmd in ["bash -l", "sh -l"] {
            if let Ok(mut ch) = open_channel("fallback_exec_generic") {
                self.emit_debug("exec_try", json!({"cmd": cmd}));
                match ch.exec(cmd) {
                    Ok(_) => {
                        self.emit_debug("exec_ok", json!({"cmd": cmd}));
                        tracing::debug!("Exec shell started with cmd '{}' (session {})", cmd, self.id);
                        return Ok(ch);
                    }
                    Err(e) => {
                        self.emit_debug("exec_fail", json!({"cmd": cmd, "error": e.to_string()}));
                        let _ = ch.close();
                    }
                }
            }
        }

        Err(AppError::Ssh("Failed to start interactive shell".to_string()))
    }

    /// Verify the host key against known_hosts
    fn verify_host_key(&self, ssh_session: &Ssh2Session, config_dir: &PathBuf) -> AppResult<()> {
        let known_hosts_path = config_dir.join("known_hosts");
        
        // Get host key from server
        let (key, key_type) = ssh_session.host_key()
            .ok_or_else(|| AppError::Ssh("No host key received".to_string()))?;

        // Compute SHA256 fingerprint
        let fingerprint = compute_sha256_fingerprint(key);
        let key_type_str = match key_type {
            HostKeyType::Rsa => "ssh-rsa",
            HostKeyType::Dss => "ssh-dss",
            HostKeyType::Ecdsa256 => "ecdsa-sha2-nistp256",
            HostKeyType::Ecdsa384 => "ecdsa-sha2-nistp384",
            HostKeyType::Ecdsa521 => "ecdsa-sha2-nistp521",
            HostKeyType::Ed25519 => "ssh-ed25519",
            _ => "unknown",
        };

        // Try to load existing known_hosts
        let mut known_hosts = ssh_session.known_hosts()
            .map_err(|e| AppError::Ssh(format!("Failed to create known_hosts: {}", e)))?;

        if known_hosts_path.exists() {
            let _ = known_hosts.read_file(&known_hosts_path, KnownHostFileKind::OpenSSH);
        }

        // Check if host is known
        let check_result = known_hosts.check_port(&self.config.host, self.config.port, key);

        match check_result {
            CheckResult::Match => {
                tracing::debug!("Host key matched for {}:{}", self.config.host, self.config.port);
                Ok(())
            }
            CheckResult::NotFound => {
                // Unknown host - ask user
                tracing::info!("Unknown host key for {}:{}", self.config.host, self.config.port);
                
                self.set_state(SessionState::WaitingForHostKey);
                
                // Emit host key request to frontend
                let hostkey_info = HostKeyInfo {
                    session_id: self.id.clone(),
                    host: self.config.host.clone(),
                    port: self.config.port,
                    key_type: key_type_str.to_string(),
                    fingerprint_sha256: fingerprint.clone(),
                };
                
                let _ = self.app_handle.emit("ssh:hostkey_request", &hostkey_info);

                // Wait for user decision (poll with timeout)
                let decision = self.wait_for_hostkey_decision()?;

                match decision {
                    HostKeyDecision::TrustOnce => {
                        tracing::info!("User accepted host key once");
                        Ok(())
                    }
                    HostKeyDecision::TrustAlways => {
                        tracing::info!("User accepted host key permanently");
                        
                        // Determine the key format based on key type
                        let key_format = match key_type {
                            HostKeyType::Rsa => ssh2::KnownHostKeyFormat::SshRsa,
                            HostKeyType::Dss => ssh2::KnownHostKeyFormat::SshDss,
                            _ => ssh2::KnownHostKeyFormat::Unknown,
                        };
                        
                        // Add to known_hosts
                        known_hosts.add(
                            &self.config.host,
                            key,
                            &format!("Added by NeonShell on {}", chrono::Utc::now()),
                            key_format,
                        ).map_err(|e| AppError::Ssh(format!("Failed to add known host: {}", e)))?;
                        
                        // Ensure parent directory exists
                        if let Some(parent) = known_hosts_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        
                        // Write to file
                        known_hosts.write_file(&known_hosts_path, KnownHostFileKind::OpenSSH)
                            .map_err(|e| AppError::Ssh(format!("Failed to write known_hosts: {}", e)))?;
                        
                        Ok(())
                    }
                    HostKeyDecision::Reject => {
                        Err(AppError::Ssh("Host key rejected by user".to_string()))
                    }
                }
            }
            CheckResult::Mismatch => {
                // HOST KEY CHANGED - SECURITY RISK!
                tracing::error!(
                    "HOST KEY MISMATCH for {}:{}! Possible MITM attack!",
                    self.config.host,
                    self.config.port
                );
                
                let _ = self.app_handle.emit("ssh:error", serde_json::json!({
                    "session_id": self.id,
                    "message": format!(
                        "SECURITY WARNING: Host key has changed for {}:{}! \
                        This could indicate a man-in-the-middle attack. \
                        Connection rejected. \
                        If you trust this change, remove the old key from known_hosts.",
                        self.config.host,
                        self.config.port
                    )
                }));
                
                Err(AppError::Ssh("Host key mismatch - possible security risk".to_string()))
            }
            CheckResult::Failure => {
                Err(AppError::Ssh("Failed to check known hosts".to_string()))
            }
        }
    }

    /// Wait for host key decision from user with timeout
    fn wait_for_hostkey_decision(&self) -> AppResult<HostKeyDecision> {
        let timeout = Duration::from_secs(60); // 60 second timeout
        let start = std::time::Instant::now();
        
        loop {
            if let Some(decision) = self.get_hostkey_decision() {
                self.clear_hostkey_decision();
                return Ok(decision);
            }
            
            if start.elapsed() > timeout {
                return Err(AppError::Ssh("Host key verification timed out".to_string()));
            }
            
            thread::sleep(Duration::from_millis(100));
        }
    }

    /// Authenticate with the SSH server
    fn authenticate(
        &self,
        ssh_session: &mut Ssh2Session,
        password: Option<String>,
        private_key: Option<String>,
        passphrase: Option<String>,
    ) -> AppResult<()> {
        match &self.config.auth_method {
            AuthMethod::Password { .. } => {
                let password = password
                    .ok_or_else(|| AppError::Auth("Password required".to_string()))?;
                
                ssh_session.userauth_password(&self.config.username, &password)
                    .map_err(|_| AppError::Auth("Password authentication failed".to_string()))?;
            }
            AuthMethod::Key { .. } => {
                let key_data = private_key
                    .ok_or_else(|| AppError::Auth("Private key required".to_string()))?;
                
                // Write key to a secure temp file (will be deleted when dropped)
                let temp_dir = std::env::temp_dir();
                let key_file_path = temp_dir.join(format!("neonshell_key_{}", uuid::Uuid::new_v4()));
                
                // Write key data to temp file
                std::fs::write(&key_file_path, &key_data)
                    .map_err(|e| AppError::Auth(format!("Failed to write temp key file: {}", e)))?;
                
                // Set restrictive permissions on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = std::fs::Permissions::from_mode(0o600);
                    let _ = std::fs::set_permissions(&key_file_path, perms);
                }
                
                // Authenticate with the key file
                let auth_result = ssh_session.userauth_pubkey_file(
                    &self.config.username,
                    None, // No separate public key
                    &key_file_path,
                    passphrase.as_deref(),
                );
                
                // Always delete the temp key file
                let _ = std::fs::remove_file(&key_file_path);
                
                auth_result.map_err(|e| {
                    let msg = e.to_string();
                    if msg.to_lowercase().contains("passphrase") || msg.to_lowercase().contains("decrypt") || msg.to_lowercase().contains("parse") {
                        AppError::Auth("Invalid passphrase or key format. Ensure the key is in PEM or OpenSSH format.".to_string())
                    } else if msg.to_lowercase().contains("denied") || msg.to_lowercase().contains("auth") {
                        AppError::Auth("Private key not accepted by server".to_string())
                    } else {
                        AppError::Auth("Private key authentication failed".to_string())
                    }
                })?;
            }
            AuthMethod::Agent => {
                // Try SSH agent authentication
                let mut agent = ssh_session.agent()
                    .map_err(|_| AppError::Auth("SSH agent not available. Make sure ssh-agent is running.".to_string()))?;
                
                agent.connect()
                    .map_err(|_| AppError::Auth("Failed to connect to SSH agent. Is it running?".to_string()))?;
                
                agent.list_identities()
                    .map_err(|_| AppError::Auth("Failed to list SSH agent identities".to_string()))?;
                
                let identities: Vec<_> = agent.identities().unwrap_or_default();
                
                if identities.is_empty() {
                    return Err(AppError::Auth("No identities found in SSH agent. Add keys with ssh-add.".to_string()));
                }
                
                let mut auth_success = false;
                for identity in identities {
                    if agent.userauth(&self.config.username, &identity).is_ok() {
                        auth_success = true;
                        break;
                    }
                }
                
                if !auth_success {
                    return Err(AppError::Auth("SSH agent authentication failed. No matching key accepted.".to_string()));
                }
            }
            AuthMethod::Interactive => {
                // Keyboard-interactive auth not fully supported yet
                return Err(AppError::Auth("Keyboard-interactive auth not yet supported".to_string()));
            }
        }

        if !ssh_session.authenticated() {
            return Err(AppError::Auth("Authentication failed".to_string()));
        }

        tracing::info!("SSH authentication successful (session {})", self.id);
        Ok(())
    }

    /// Main I/O loop - reads from SSH channel and writes to frontend
    #[allow(unused_mut)] // set_blocking takes &self but may need mut in some versions
    fn run_io_loop(
        &self,
        channel: &mut Channel,
        write_rx: &mut mpsc::Receiver<SessionCommand>,
        mut ssh_session: Ssh2Session,
    ) -> AppResult<()> {
        let mut read_buf = [0u8; 32768]; // 32KB read buffer
        let mut last_keepalive = std::time::Instant::now();
        let keepalive_interval = Duration::from_secs(self.config.keepalive_interval as u64);
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 5;
        let mut pending: Vec<u8> = Vec::new();
        let mut read_error_count: u32 = 0;
        
        loop {
            // Send SSH keepalive periodically
            if last_keepalive.elapsed() >= keepalive_interval {
                ssh_session.set_blocking(true);
                match ssh_session.keepalive_send() {
                    Ok(_) => tracing::debug!("Keepalive sent (session {})", self.id),
                    Err(e) => tracing::warn!("Keepalive send failed (session {}): {}", self.id, e),
                }
                ssh_session.set_blocking(false);
                last_keepalive = std::time::Instant::now();
            }

            // Drain commands quickly
            for _ in 0..32 {
                match write_rx.try_recv() {
                    Ok(SessionCommand::Write(data)) => {
                        let enqueue_len = data.len();
                        if pending.len() + enqueue_len > MAX_PENDING_BYTES {
                            self.emit_debug("enqueue_dropped", json!({"len": enqueue_len, "pending": pending.len()}));
                            continue;
                        }
                        pending.extend_from_slice(&data);
                        self.emit_debug("enqueue", json!({"len": enqueue_len, "pending": pending.len()}));
                    }
                    Ok(SessionCommand::Resize(cols, rows)) => {
                        ssh_session.set_blocking(true);
                        if let Err(e) = channel.request_pty_size(cols, rows, None, None) {
                            tracing::warn!("Failed to resize PTY: {}", e);
                        }
                        ssh_session.set_blocking(false);
                    }
                    Ok(SessionCommand::Close) => {
                        tracing::info!("Close command received (session {})", self.id);
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        tracing::info!("Command channel disconnected (session {})", self.id);
                        break;
                    }
                }
            }

            // Check for too many consecutive errors
            if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                tracing::error!("Too many consecutive errors, closing session {}", self.id);
                let _ = self.app_handle.emit("ssh:error", serde_json::json!({
                    "session_id": self.id,
                    "message": "Connection lost - too many errors"
                }));
                break;
            }

            // Reads first: drain until EAGAIN
            loop {
                match channel.stream(0).read(&mut read_buf) {
                    Ok(0) => {
                        // No data; don't treat as EOF
                        break;
                    }
                    Ok(n) => {
                        read_error_count = 0;
                        consecutive_errors = 0;
                        let data = read_buf[..n].to_vec();
                        let _ = self.app_handle.emit("ssh:data", serde_json::json!({
                            "session_id": self.id,
                            "data": data
                        }));
                        continue;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        break;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {
                        continue;
                    }
                    Err(e) => {
                        let err_str = e.to_string().to_lowercase();
                        if Self::is_recoverable_error(&err_str) {
                            break;
                        }
                        read_error_count += 1;
                        tracing::warn!(
                            "Read error (session {}): {}; count={}",
                            self.id,
                            e,
                            read_error_count
                        );
                        if channel.eof() {
                            tracing::info!("EOF after read error (session {}): {}", self.id, e);
                            break;
                        }
                        // brief backoff
                        thread::sleep(Duration::from_millis(5));
                        break;
                    }
                }
                // If we just delivered data, loop to read again
            }

            // Writes with backoff and partial handling
            if !pending.is_empty() {
                ssh_session.set_blocking(true);
                loop {
                    if pending.is_empty() {
                        break;
                    }
                    let write_len = pending.len().min(WRITE_CHUNK_BYTES);
                    match channel.write(&pending[..write_len]) {
                        Ok(0) => {
                            // treat as would-block
                            thread::sleep(Duration::from_millis(4));
                        }
                        Ok(n) => {
                            let _ = pending.drain(..n);
                            consecutive_errors = 0;
                            self.emit_debug("write_progress", json!({"written": n, "pending": pending.len()}));
                        }
                        Err(e) => {
                            let err_str = e.to_string().to_lowercase();
                            if Self::is_recoverable_error(&err_str) {
                                thread::sleep(Duration::from_millis(4));
                                continue;
                            }
                            tracing::error!("Write error (session {}): {}", self.id, e);
                            let _ = self.app_handle.emit("ssh:error", serde_json::json!({
                                "session_id": self.id,
                                "message": "Write failed"
                            }));
                            pending.clear();
                            break;
                        }
                    }
                    // allow interleave
                    if pending.is_empty() {
                        break;
                    }
                }
                let _ = channel.flush();
                ssh_session.set_blocking(false);
            }

            // Check if channel is closed before reading next loop
            if channel.eof() {
                tracing::info!("SSH channel closed (session {})", self.id);
                break;
            }

            // small sleep to avoid busy spin
            thread::sleep(Duration::from_millis(2));
        }

        // Cleanup
        // Try to wait for channel close to gather exit info
        let _ = channel.wait_close();

        self.set_state(SessionState::Disconnected);
        Self::log_channel_state(channel, &ssh_session, self.id.clone(), "loop_exit");
        let _ = self.app_handle.emit("ssh:closed", serde_json::json!({
            "session_id": self.id,
            "reason": "Connection closed"
        }));

        Ok(())
    }

    /// Log channel/session state for diagnostics (no payloads/secrets)
    fn log_channel_state(channel: &Channel, _ssh_session: &Ssh2Session, session_id: String, ctx: &str) {
        let eof = channel.eof();
        let exit_status = channel.exit_status().unwrap_or_default();
        let exit_signal = match channel.exit_signal() {
            Ok(sig) => {
                let name = sig.exit_signal.unwrap_or_else(|| "unknown".to_string());
                let msg = sig.error_message.unwrap_or_else(|| "".to_string());
                let lang = sig.lang_tag.unwrap_or_else(|| "".to_string());
                format!("{} msg={} lang={}", name, msg, lang)
            }
            Err(_) => "none".to_string(),
        };

        tracing::info!(
            "Channel state [{ctx}] (session {session_id}): eof={eof} exit_status={exit_status} exit_signal={exit_signal}",
            ctx = ctx,
            session_id = session_id,
            eof = eof,
            exit_status = exit_status,
            exit_signal = exit_signal
        );
    }

    pub fn send_data(&self, data: &[u8]) -> AppResult<()> {
        if let Some(tx) = self.write_tx.read().as_ref() {
            match tx.try_send(SessionCommand::Write(data.to_vec())) {
                Ok(_) => Ok(()),
                Err(TrySendError::Full(_)) => Err(AppError::Ssh("Output queue full".to_string())),
                Err(_) => Err(AppError::Ssh("Session closed".to_string())),
            }
        } else {
            Err(AppError::Ssh("Session closed".to_string()))
        }
    }

    pub fn resize_pty(&self, cols: u32, rows: u32) -> AppResult<()> {
        if let Some(tx) = self.write_tx.read().as_ref() {
            tx.try_send(SessionCommand::Resize(cols, rows))
                .map_err(|_| AppError::Ssh("Session closed".to_string()))?;
        }
        Ok(())
    }

    pub fn disconnect(&self) -> AppResult<()> {
        if let Some(tx) = self.write_tx.write().take() {
            let _ = tx.try_send(SessionCommand::Close);
        }
        self.set_state(SessionState::Disconnected);
        let _ = self.app_handle.emit("ssh:disconnected", self.info());
        Ok(())
    }
    
    /// Check if an error message indicates a recoverable (transient) error
    fn is_recoverable_error(err_str: &str) -> bool {
        // These are errors that indicate "try again later" rather than a fatal error
        err_str.contains("would block") ||
        err_str.contains("wouldblock") ||
        err_str.contains("eagain") ||
        err_str.contains("try again") ||
        err_str.contains("temporarily") ||
        err_str.contains("resource temporarily unavailable") ||
        err_str.contains("timeout") ||
        err_str.contains("timed out") ||
        err_str.contains("-37") // libssh2 EAGAIN code
    }
}

/// Compute SHA256 fingerprint of a key
fn compute_sha256_fingerprint(key: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key);
    let result = hasher.finalize();
    
    // Format as SHA256:base64
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &result);
    format!("SHA256:{}", b64.trim_end_matches('='))
}

/// Sanitize error messages to remove potential secrets
fn sanitize_error_message(msg: &str) -> String {
    // Remove anything that looks like it might contain sensitive data
    let msg = msg.to_string();
    
    // Truncate long messages
    if msg.len() > 200 {
        format!("{}...", &msg[..200])
    } else {
        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_format() {
        let test_key = b"test key data";
        let fp = compute_sha256_fingerprint(test_key);
        assert!(fp.starts_with("SHA256:"));
    }

    #[test]
    fn test_sanitize_error() {
        let short = "Short error";
        assert_eq!(sanitize_error_message(short), short);

        let long = "a".repeat(300);
        let sanitized = sanitize_error_message(&long);
        assert!(sanitized.len() < 210);
        assert!(sanitized.ends_with("..."));
    }
}
