use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub terminal: TerminalSettings,
    #[serde(default)]
    pub ssh: SshSettings,
    #[serde(default)]
    pub security: SecuritySettings,
    #[serde(default)]
    pub plugins: PluginSettings,
    #[serde(default)]
    pub ui: UiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_true")]
    pub check_updates: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default = "default_true")]
    pub restore_sessions: bool,
}

fn default_theme() -> String {
    "neon-default".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            language: default_language(),
            check_updates: true,
            start_minimized: false,
            restore_sessions: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,
    #[serde(default = "default_true")]
    pub cursor_blink: bool,
    #[serde(default = "default_scrollback")]
    pub scrollback: u32,
    #[serde(default = "default_true")]
    pub copy_on_select: bool,
    #[serde(default)]
    pub bell_sound: bool,
    #[serde(default)]
    pub bell_visual: bool,
}

fn default_font_family() -> String {
    "JetBrains Mono".to_string()
}

fn default_font_size() -> u32 {
    14
}

fn default_cursor_style() -> String {
    "block".to_string()
}

fn default_scrollback() -> u32 {
    10000
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            cursor_style: default_cursor_style(),
            cursor_blink: true,
            scrollback: default_scrollback(),
            copy_on_select: true,
            bell_sound: false,
            bell_visual: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSettings {
    #[serde(default = "default_port")]
    pub default_port: u16,
    #[serde(default = "default_keepalive")]
    pub keepalive_interval: u32,
    #[serde(default = "default_true")]
    pub strict_host_checking: bool,
    #[serde(default)]
    pub agent_forwarding: bool,
    #[serde(default)]
    pub compression: bool,
    #[serde(default = "default_ciphers")]
    pub preferred_ciphers: Vec<String>,
}

fn default_port() -> u16 {
    22
}

fn default_keepalive() -> u32 {
    60
}

fn default_ciphers() -> Vec<String> {
    vec![
        "chacha20-poly1305@openssh.com".to_string(),
        "aes256-gcm@openssh.com".to_string(),
        "aes128-gcm@openssh.com".to_string(),
    ]
}

impl Default for SshSettings {
    fn default() -> Self {
        Self {
            default_port: default_port(),
            keepalive_interval: default_keepalive(),
            strict_host_checking: true,
            agent_forwarding: false,
            compression: false,
            preferred_ciphers: default_ciphers(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    #[serde(default = "default_store")]
    pub store_passwords: String,
    #[serde(default = "default_auto_lock")]
    pub auto_lock_timeout: u32,
    #[serde(default = "default_true")]
    pub clear_clipboard: bool,
    #[serde(default = "default_clipboard_timeout")]
    pub clipboard_timeout: u32,
}

fn default_store() -> String {
    "keychain".to_string()
}

fn default_auto_lock() -> u32 {
    300
}

fn default_clipboard_timeout() -> u32 {
    30
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            store_passwords: default_store(),
            auto_lock_timeout: default_auto_lock(),
            clear_clipboard: true,
            clipboard_timeout: default_clipboard_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(default)]
    pub allow_unsigned: bool,
    #[serde(default)]
    pub auto_update: bool,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            enabled: vec![],
            allow_unsigned: false,
            auto_update: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    #[serde(default = "default_true")]
    pub show_sidebar: bool,
    #[serde(default)]
    pub sidebar_position: String,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
    #[serde(default = "default_true")]
    pub show_statusbar: bool,
    #[serde(default)]
    pub tab_position: String,
    #[serde(default)]
    pub confirm_close: bool,
}

fn default_sidebar_width() -> u32 {
    250
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            sidebar_position: "left".to_string(),
            sidebar_width: default_sidebar_width(),
            show_statusbar: true,
            tab_position: "top".to_string(),
            confirm_close: false,
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            terminal: TerminalSettings::default(),
            ssh: SshSettings::default(),
            security: SecuritySettings::default(),
            plugins: PluginSettings::default(),
            ui: UiSettings::default(),
        }
    }
}

impl AppSettings {
    pub fn load(config_dir: &Path) -> AppResult<Self> {
        let config_path = config_dir.join("config.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let settings: AppSettings = toml::from_str(&content)?;
            Ok(settings)
        } else {
            let settings = AppSettings::default();
            settings.save(config_dir)?;
            Ok(settings)
        }
    }

    pub fn save(&self, config_dir: &Path) -> AppResult<()> {
        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}




