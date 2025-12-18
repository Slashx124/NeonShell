pub mod commands;
pub mod session;

pub use session::*;

use crate::config::get_config_dir;
use crate::error::{AppError, AppResult};
use dashmap::DashMap;
use std::sync::Arc;
use tauri::AppHandle;
use uuid::Uuid;

/// Manages all SSH sessions
pub struct SessionManager {
    app_handle: AppHandle,
    sessions: DashMap<String, Arc<SessionHandle>>,
}

impl SessionManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            sessions: DashMap::new(),
        }
    }

    /// Create a new SSH session with the given config
    pub fn create_session(&self, config: SessionConfig) -> AppResult<String> {
        let id = Uuid::new_v4().to_string();
        let handle = SessionHandle::new(id.clone(), config, self.app_handle.clone());
        self.sessions.insert(id.clone(), Arc::new(handle));
        
        tracing::info!("Created SSH session: {}", id);
        Ok(id)
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &str) -> Option<Arc<SessionHandle>> {
        self.sessions.get(id).map(|s| Arc::clone(&s))
    }

    /// Remove a session
    pub fn remove_session(&self, id: &str) -> Option<Arc<SessionHandle>> {
        self.sessions.remove(id).map(|(_, s)| s)
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions
            .iter()
            .map(|entry| entry.value().info())
            .collect()
    }

    /// Start connection for a session
    pub fn connect_session(
        &self,
        session_id: &str,
        password: Option<String>,
        private_key: Option<String>,
        passphrase: Option<String>,
    ) -> AppResult<()> {
        let session = self.get_session(session_id)
            .ok_or_else(|| AppError::Ssh(format!("Session not found: {}", session_id)))?;

        let config_dir = get_config_dir()?;
        
        // Start connection in background
        session.start_connection(password, private_key, passphrase, config_dir);
        
        Ok(())
    }

    /// Set host key decision for a session
    pub fn set_hostkey_decision(&self, session_id: &str, decision: HostKeyDecision) -> AppResult<()> {
        let session = self.get_session(session_id)
            .ok_or_else(|| AppError::Ssh(format!("Session not found: {}", session_id)))?;
        
        session.set_hostkey_decision(decision);
        Ok(())
    }

    /// Send data to a session
    pub fn send_data(&self, session_id: &str, data: &[u8]) -> AppResult<()> {
        let session = self.get_session(session_id)
            .ok_or_else(|| AppError::Ssh(format!("Session not found: {}", session_id)))?;
        
        session.send_data(data)
    }

    /// Resize PTY for a session
    pub fn resize_pty(&self, session_id: &str, cols: u32, rows: u32) -> AppResult<()> {
        let session = self.get_session(session_id)
            .ok_or_else(|| AppError::Ssh(format!("Session not found: {}", session_id)))?;
        
        session.resize_pty(cols, rows)
    }

    /// Disconnect a session
    pub fn disconnect(&self, session_id: &str) -> AppResult<()> {
        let session = self.get_session(session_id)
            .ok_or_else(|| AppError::Ssh(format!("Session not found: {}", session_id)))?;
        
        session.disconnect()?;
        
        // Give it a moment to clean up, then remove
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.remove_session(session_id);
        
        tracing::info!("Disconnected SSH session: {}", session_id);
        Ok(())
    }
}
