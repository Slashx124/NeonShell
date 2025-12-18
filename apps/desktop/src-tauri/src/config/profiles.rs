use crate::error::{AppError, AppResult};
use crate::ssh::{AuthMethod, JumpHost, KnownHostsPolicy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Connection profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    #[serde(default)]
    pub jump_hosts: Vec<JumpHost>,
    #[serde(default)]
    pub options: ProfileOptions,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

fn default_port() -> u16 {
    22
}

/// Profile-specific options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileOptions {
    #[serde(default = "default_keepalive")]
    pub keepalive_interval: u32,
    #[serde(default)]
    pub agent_forwarding: bool,
    #[serde(default)]
    pub known_hosts_policy: KnownHostsPolicy,
    #[serde(default)]
    pub startup_commands: Vec<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

fn default_keepalive() -> u32 {
    60
}

impl Profile {
    pub fn new(name: String, host: String, username: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            host,
            port: 22,
            username,
            auth_method: AuthMethod::Agent,
            jump_hosts: vec![],
            options: ProfileOptions::default(),
            theme: None,
            tags: vec![],
            notes: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Profile file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilesFile {
    #[serde(default)]
    pub profiles: Vec<Profile>,
}

/// Profile manager
pub struct ProfileManager {
    profiles: HashMap<String, Profile>,
    config_path: PathBuf,
}

impl ProfileManager {
    pub fn load(config_dir: &Path) -> AppResult<Self> {
        let config_path = config_dir.join("profiles.toml");
        let profiles = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let file: ProfilesFile = toml::from_str(&content)?;
            file.profiles
                .into_iter()
                .map(|p| (p.id.clone(), p))
                .collect()
        } else {
            HashMap::new()
        };

        Ok(Self {
            profiles,
            config_path,
        })
    }

    pub fn save(&self) -> AppResult<()> {
        let profiles: Vec<_> = self.profiles.values().cloned().collect();
        let file = ProfilesFile { profiles };
        let content = toml::to_string_pretty(&file)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn list(&self) -> Vec<Profile> {
        self.profiles.values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<Profile> {
        self.profiles.get(id).cloned()
    }

    pub fn add(&mut self, profile: Profile) -> AppResult<()> {
        self.profiles.insert(profile.id.clone(), profile);
        self.save()
    }

    pub fn update(&mut self, profile: Profile) -> AppResult<()> {
        if !self.profiles.contains_key(&profile.id) {
            return Err(AppError::ProfileNotFound(profile.id));
        }
        let mut profile = profile;
        profile.updated_at = chrono::Utc::now().timestamp();
        self.profiles.insert(profile.id.clone(), profile);
        self.save()
    }

    pub fn delete(&mut self, id: &str) -> AppResult<()> {
        self.profiles
            .remove(id)
            .ok_or_else(|| AppError::ProfileNotFound(id.to_string()))?;
        self.save()
    }
}

/// Parse OpenSSH config file
pub fn parse_openssh_config(content: &str) -> Vec<Profile> {
    let mut profiles = Vec::new();
    let mut current_profile: Option<Profile> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].to_lowercase();
        let value = parts[1].trim();

        match key.as_str() {
            "host" => {
                // Save previous profile if exists
                if let Some(profile) = current_profile.take() {
                    profiles.push(profile);
                }
                current_profile = Some(Profile::new(
                    value.to_string(),
                    String::new(),
                    String::new(),
                ));
            }
            "hostname" => {
                if let Some(ref mut profile) = current_profile {
                    profile.host = value.to_string();
                }
            }
            "user" => {
                if let Some(ref mut profile) = current_profile {
                    profile.username = value.to_string();
                }
            }
            "port" => {
                if let Some(ref mut profile) = current_profile {
                    if let Ok(port) = value.parse() {
                        profile.port = port;
                    }
                }
            }
            "identityfile" => {
                if let Some(ref mut profile) = current_profile {
                    profile.auth_method = AuthMethod::Key {
                        key_id: format!("imported:{}", value),
                    };
                }
            }
            "proxyjump" => {
                if let Some(ref mut profile) = current_profile {
                    profile.jump_hosts = value
                        .split(',')
                        .map(|host| {
                            let parts: Vec<&str> = host.trim().split('@').collect();
                            let (user, host_port) = if parts.len() == 2 {
                                (parts[0].to_string(), parts[1])
                            } else {
                                (String::new(), parts[0])
                            };
                            let hp: Vec<&str> = host_port.split(':').collect();
                            let (host, port) = if hp.len() == 2 {
                                (hp[0].to_string(), hp[1].parse().unwrap_or(22))
                            } else {
                                (hp[0].to_string(), 22)
                            };
                            JumpHost {
                                host,
                                port,
                                username: user,
                                auth_method: AuthMethod::Agent,
                            }
                        })
                        .collect();
                }
            }
            "forwardagent" => {
                if let Some(ref mut profile) = current_profile {
                    profile.options.agent_forwarding =
                        value.to_lowercase() == "yes" || value == "true";
                }
            }
            "serveralivecountmax" | "serveraliveinterval" => {
                if let Some(ref mut profile) = current_profile {
                    if let Ok(interval) = value.parse() {
                        profile.options.keepalive_interval = interval;
                    }
                }
            }
            _ => {}
        }
    }

    // Don't forget the last profile
    if let Some(profile) = current_profile {
        profiles.push(profile);
    }

    // Filter out incomplete profiles and wildcards
    profiles
        .into_iter()
        .filter(|p| !p.host.is_empty() && !p.host.contains('*') && !p.host.contains('?'))
        .collect()
}

/// Export profiles to OpenSSH config format
pub fn export_openssh_config(profiles: &[Profile]) -> String {
    let mut output = String::new();
    output.push_str("# Generated by NeonShell\n\n");

    for profile in profiles {
        output.push_str(&format!("Host {}\n", profile.name));
        output.push_str(&format!("    HostName {}\n", profile.host));
        output.push_str(&format!("    User {}\n", profile.username));
        if profile.port != 22 {
            output.push_str(&format!("    Port {}\n", profile.port));
        }
        
        if let AuthMethod::Key { ref key_id } = profile.auth_method {
            // Only export if it looks like a file path
            if key_id.contains('/') || key_id.contains('\\') {
                output.push_str(&format!("    IdentityFile {}\n", key_id));
            }
        }
        
        if profile.options.agent_forwarding {
            output.push_str("    ForwardAgent yes\n");
        }
        
        if !profile.jump_hosts.is_empty() {
            let jumps: Vec<String> = profile
                .jump_hosts
                .iter()
                .map(|j| {
                    if j.username.is_empty() {
                        if j.port == 22 {
                            j.host.clone()
                        } else {
                            format!("{}:{}", j.host, j.port)
                        }
                    } else {
                        if j.port == 22 {
                            format!("{}@{}", j.username, j.host)
                        } else {
                            format!("{}@{}:{}", j.username, j.host, j.port)
                        }
                    }
                })
                .collect();
            output.push_str(&format!("    ProxyJump {}\n", jumps.join(",")));
        }
        
        output.push('\n');
    }

    output
}

