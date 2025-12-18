use serde::{Deserialize, Serialize};

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

/// Port forwarding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub id: String,
    pub forward_type: PortForwardType,
    pub local_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortForwardType {
    Local,
    Remote,
    Dynamic,
}

/// SFTP file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<i64>,
    pub permissions: String,
    pub owner: Option<String>,
    pub group: Option<String>,
}

