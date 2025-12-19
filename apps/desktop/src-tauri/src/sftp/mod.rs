//! SFTP file transfer support
//! 
//! This module provides SFTP commands for file browsing and transfer.
//! SFTP operations run on separate connections to avoid blocking terminal I/O.

pub mod commands;

use crate::error::{AppError, AppResult};
use crate::keychain;
use crate::config::Profile;
use serde::{Deserialize, Serialize};
use ssh2::{Session as Ssh2Session, Sftp, FileStat};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

/// SFTP file/directory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: Option<i64>,
    pub permissions: String,
}

/// SFTP session manager
/// Creates a separate SFTP connection per operation (stateless for simplicity)
pub struct SftpManager;

impl SftpManager {
    /// Create an SFTP session from a profile
    pub fn connect_from_profile(profile: &Profile) -> AppResult<SftpConnection> {
        // Retrieve credentials from keychain
        let (password, private_key, passphrase) = match &profile.auth_method {
            crate::ssh::AuthMethod::Password { password_key } => {
                if password_key.is_empty() {
                    return Err(AppError::Auth("No password stored for this profile".to_string()));
                }
                let pwd = keychain::get_secret(password_key)?
                    .ok_or_else(|| AppError::Auth("Password not found in keychain".to_string()))?;
                (Some(pwd), None, None)
            }
            crate::ssh::AuthMethod::Key { key_id } => {
                if key_id.is_empty() {
                    return Err(AppError::Auth("No private key stored for this profile".to_string()));
                }
                let key = keychain::get_secret(key_id)?
                    .ok_or_else(|| AppError::Auth("Private key not found in keychain".to_string()))?;
                
                let passphrase_key = key_id.replace("key:", "passphrase:");
                let pass = keychain::get_secret(&passphrase_key).ok().flatten();
                
                (None, Some(key), pass)
            }
            crate::ssh::AuthMethod::Agent => (None, None, None),
            crate::ssh::AuthMethod::Interactive => {
                return Err(AppError::Auth("Interactive auth not supported for SFTP".to_string()));
            }
        };

        SftpConnection::connect(
            &profile.host,
            profile.port,
            &profile.username,
            password.as_deref(),
            private_key.as_deref(),
            passphrase.as_deref(),
        )
    }
}

/// An active SFTP connection
pub struct SftpConnection {
    pub sftp: Sftp,
    #[allow(dead_code)]
    session: Ssh2Session,
    #[allow(dead_code)]
    tcp: TcpStream,
}

impl SftpConnection {
    /// Connect and establish SFTP session
    pub fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
        private_key: Option<&str>,
        passphrase: Option<&str>,
    ) -> AppResult<Self> {
        // Connect TCP
        let addr = format!("{}:{}", host, port);
        let tcp = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| AppError::Ssh(format!("Invalid address: {}", e)))?,
            Duration::from_secs(30),
        )?;
        tcp.set_read_timeout(Some(Duration::from_secs(60)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(60)))?;

        // Create SSH session
        let mut session = Ssh2Session::new()
            .map_err(|e| AppError::Ssh(format!("Failed to create session: {}", e)))?;
        session.set_tcp_stream(tcp.try_clone()?);
        session.handshake()
            .map_err(|e| AppError::Ssh(format!("SSH handshake failed: {}", e)))?;

        // Authenticate
        if let Some(password) = password {
            session.userauth_password(username, password)
                .map_err(|e| AppError::Auth(format!("Password auth failed: {}", e)))?;
        } else if let Some(private_key) = private_key {
            // Write key to a secure temp file (will be deleted after auth)
            let temp_dir = std::env::temp_dir();
            let key_file_path = temp_dir.join(format!("neonshell_sftp_key_{}", uuid::Uuid::new_v4()));
            
            std::fs::write(&key_file_path, private_key)
                .map_err(|e| AppError::Auth(format!("Failed to write temp key file: {}", e)))?;
            
            // Set restrictive permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o600);
                let _ = std::fs::set_permissions(&key_file_path, perms);
            }
            
            // Authenticate with the key file
            let auth_result = session.userauth_pubkey_file(
                username,
                None,
                &key_file_path,
                passphrase,
            );
            
            // Always delete the temp key file
            let _ = std::fs::remove_file(&key_file_path);
            
            auth_result.map_err(|e| AppError::Auth(format!("Key auth failed: {}", e)))?;
        } else {
            // Try agent
            let mut agent = session.agent()
                .map_err(|e| AppError::Auth(format!("Agent not available: {}", e)))?;
            agent.connect()
                .map_err(|e| AppError::Auth(format!("Agent connect failed: {}", e)))?;
            agent.list_identities()
                .map_err(|e| AppError::Auth(format!("Agent list failed: {}", e)))?;
            
            let identities: Vec<_> = agent.identities()
                .map_err(|e| AppError::Auth(format!("No agent identities: {}", e)))?;
            
            let mut authenticated = false;
            for identity in identities {
                if agent.userauth(username, &identity).is_ok() {
                    authenticated = true;
                    break;
                }
            }
            
            if !authenticated {
                return Err(AppError::Auth("Agent authentication failed".to_string()));
            }
        }

        if !session.authenticated() {
            return Err(AppError::Auth("Authentication failed".to_string()));
        }

        // Open SFTP subsystem
        let sftp = session.sftp()
            .map_err(|e| AppError::Ssh(format!("Failed to open SFTP: {}", e)))?;

        Ok(Self { sftp, session, tcp })
    }

    /// List directory contents
    pub fn list_dir(&self, path: &str) -> AppResult<Vec<SftpEntry>> {
        let path = if path.is_empty() { "." } else { path };
        let dir_path = Path::new(path);
        
        let entries = self.sftp.readdir(dir_path)
            .map_err(|e| AppError::Ssh(format!("Failed to list directory: {}", e)))?;

        let mut result = Vec::new();
        for (file_path, stat) in entries {
            let name = file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            
            // Skip . and ..
            if name == "." || name == ".." {
                continue;
            }

            result.push(SftpEntry {
                name,
                path: file_path.to_string_lossy().to_string(),
                is_dir: stat.is_dir(),
                is_symlink: stat.file_type().is_symlink(),
                size: stat.size.unwrap_or(0),
                modified: stat.mtime.map(|t| t as i64),
                permissions: format_permissions(&stat),
            });
        }

        // Sort: directories first, then by name
        result.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        Ok(result)
    }

    /// Get file/directory info
    pub fn stat(&self, path: &str) -> AppResult<SftpEntry> {
        let stat = self.sftp.stat(Path::new(path))
            .map_err(|e| AppError::Ssh(format!("Failed to stat: {}", e)))?;
        
        let name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        Ok(SftpEntry {
            name,
            path: path.to_string(),
            is_dir: stat.is_dir(),
            is_symlink: stat.file_type().is_symlink(),
            size: stat.size.unwrap_or(0),
            modified: stat.mtime.map(|t| t as i64),
            permissions: format_permissions(&stat),
        })
    }

    /// Download a file and return its contents
    pub fn download(&self, path: &str) -> AppResult<Vec<u8>> {
        let mut file = self.sftp.open(Path::new(path))
            .map_err(|e| AppError::Ssh(format!("Failed to open file: {}", e)))?;
        
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|e| AppError::Ssh(format!("Failed to read file: {}", e)))?;
        
        Ok(contents)
    }

    /// Upload a file
    pub fn upload(&self, path: &str, contents: &[u8]) -> AppResult<()> {
        let mut file = self.sftp.create(Path::new(path))
            .map_err(|e| AppError::Ssh(format!("Failed to create file: {}", e)))?;
        
        file.write_all(contents)
            .map_err(|e| AppError::Ssh(format!("Failed to write file: {}", e)))?;
        
        Ok(())
    }

    /// Create a directory
    pub fn mkdir(&self, path: &str) -> AppResult<()> {
        self.sftp.mkdir(Path::new(path), 0o755)
            .map_err(|e| AppError::Ssh(format!("Failed to create directory: {}", e)))?;
        Ok(())
    }

    /// Delete a file
    pub fn delete_file(&self, path: &str) -> AppResult<()> {
        self.sftp.unlink(Path::new(path))
            .map_err(|e| AppError::Ssh(format!("Failed to delete file: {}", e)))?;
        Ok(())
    }

    /// Delete a directory
    pub fn delete_dir(&self, path: &str) -> AppResult<()> {
        self.sftp.rmdir(Path::new(path))
            .map_err(|e| AppError::Ssh(format!("Failed to delete directory: {}", e)))?;
        Ok(())
    }

    /// Rename/move a file or directory
    pub fn rename(&self, from: &str, to: &str) -> AppResult<()> {
        self.sftp.rename(Path::new(from), Path::new(to), None)
            .map_err(|e| AppError::Ssh(format!("Failed to rename: {}", e)))?;
        Ok(())
    }

    /// Get home directory
    pub fn home_dir(&self) -> AppResult<String> {
        // Try to get realpath of ~
        match self.sftp.realpath(Path::new(".")) {
            Ok(path) => Ok(path.to_string_lossy().to_string()),
            Err(_) => Ok("/".to_string()),
        }
    }
}

/// Format file permissions as a string like "rwxr-xr-x"
fn format_permissions(stat: &FileStat) -> String {
    let perms = stat.perm.unwrap_or(0);
    
    let mut s = String::with_capacity(10);
    
    // File type
    if stat.is_dir() {
        s.push('d');
    } else if stat.file_type().is_symlink() {
        s.push('l');
    } else {
        s.push('-');
    }
    
    // Owner permissions
    s.push(if perms & 0o400 != 0 { 'r' } else { '-' });
    s.push(if perms & 0o200 != 0 { 'w' } else { '-' });
    s.push(if perms & 0o100 != 0 { 'x' } else { '-' });
    
    // Group permissions
    s.push(if perms & 0o040 != 0 { 'r' } else { '-' });
    s.push(if perms & 0o020 != 0 { 'w' } else { '-' });
    s.push(if perms & 0o010 != 0 { 'x' } else { '-' });
    
    // Other permissions
    s.push(if perms & 0o004 != 0 { 'r' } else { '-' });
    s.push(if perms & 0o002 != 0 { 'w' } else { '-' });
    s.push(if perms & 0o001 != 0 { 'x' } else { '-' });
    
    s
}

