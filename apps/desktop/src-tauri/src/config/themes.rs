use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    pub colors: ThemeColors,
    #[serde(default)]
    pub terminal: TerminalTheme,
    #[serde(default)]
    pub ui: UiTheme,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_file: Option<String>,
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    #[serde(default)]
    pub accent_muted: String,
    #[serde(default)]
    pub surface_0: String,
    #[serde(default)]
    pub surface_1: String,
    #[serde(default)]
    pub surface_2: String,
    #[serde(default)]
    pub surface_3: String,
    #[serde(default)]
    pub border: String,
    #[serde(default)]
    pub cursor: String,
    #[serde(default)]
    pub selection: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub warning: String,
    #[serde(default)]
    pub success: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTheme {
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default)]
    pub ansi_colors: AnsiColors,
}

fn default_font_family() -> String {
    "JetBrains Mono".to_string()
}

fn default_font_size() -> u32 {
    14
}

impl Default for TerminalTheme {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            ansi_colors: AnsiColors::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsiColors {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

impl Default for AnsiColors {
    fn default() -> Self {
        // Neon-inspired default colors
        Self {
            black: "#0a0a0f".to_string(),
            red: "#ff0055".to_string(),
            green: "#00ff9f".to_string(),
            yellow: "#ffff00".to_string(),
            blue: "#00aaff".to_string(),
            magenta: "#ff00ff".to_string(),
            cyan: "#00ffff".to_string(),
            white: "#ffffff".to_string(),
            bright_black: "#333344".to_string(),
            bright_red: "#ff5588".to_string(),
            bright_green: "#55ffbb".to_string(),
            bright_yellow: "#ffff55".to_string(),
            bright_blue: "#55bbff".to_string(),
            bright_magenta: "#ff55ff".to_string(),
            bright_cyan: "#55ffff".to_string(),
            bright_white: "#ffffff".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiTheme {
    #[serde(default)]
    pub border_radius: String,
    #[serde(default)]
    pub shadows: bool,
    #[serde(default)]
    pub animations: bool,
    #[serde(default)]
    pub blur: bool,
}

/// Theme manager
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    #[allow(dead_code)]
    themes_dir: PathBuf,
}

impl ThemeManager {
    pub fn load(config_dir: &Path) -> AppResult<Self> {
        let themes_dir = config_dir.join("themes");
        std::fs::create_dir_all(&themes_dir)?;

        let mut themes = HashMap::new();

        // Add built-in default theme
        let default_theme = create_default_theme();
        themes.insert(default_theme.id.clone(), default_theme);

        // Add bundled themes (Dracula, Monokai, Nord)
        for bundled_theme in create_bundled_themes() {
            // Install bundled theme to user's themes dir if not present
            let theme_dir = themes_dir.join(&bundled_theme.id);
            if !theme_dir.exists() {
                if let Err(e) = install_bundled_theme(&bundled_theme, &theme_dir) {
                    tracing::warn!("Failed to install bundled theme {}: {}", bundled_theme.id, e);
                }
            }
            themes.insert(bundled_theme.id.clone(), bundled_theme);
        }

        // Load user themes (and overwrite bundled if user has modified them)
        if let Ok(entries) = std::fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let theme_file = path.join("theme.json");
                    if theme_file.exists() {
                        if let Ok(content) = std::fs::read_to_string(&theme_file) {
                            if let Ok(mut theme) = serde_json::from_str::<Theme>(&content) {
                                theme.path = Some(path.clone());
                                themes.insert(theme.id.clone(), theme);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self { themes, themes_dir })
    }

    pub fn list(&self) -> Vec<Theme> {
        self.themes.values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<Theme> {
        self.themes.get(id).cloned()
    }

    pub fn get_css(&self, id: &str) -> AppResult<Option<String>> {
        if let Some(theme) = self.themes.get(id) {
            if let (Some(css_file), Some(path)) = (&theme.css_file, &theme.path) {
                let css_path = path.join(css_file);
                if css_path.exists() {
                    return Ok(Some(std::fs::read_to_string(css_path)?));
                }
            }
        }
        Ok(None)
    }
}

fn create_default_theme() -> Theme {
    Theme {
        id: "neon-default".to_string(),
        name: "Neon Default".to_string(),
        version: "1.0.0".to_string(),
        author: "NeonShell".to_string(),
        description: "The default NeonShell theme with vibrant neon colors".to_string(),
        colors: ThemeColors {
            background: "#0a0a0f".to_string(),
            foreground: "#e0e0e0".to_string(),
            accent: "#ff0080".to_string(),
            accent_muted: "#aa0055".to_string(),
            surface_0: "#0a0a0f".to_string(),
            surface_1: "#12121a".to_string(),
            surface_2: "#1a1a24".to_string(),
            surface_3: "#22222e".to_string(),
            border: "#333344".to_string(),
            cursor: "#ff0080".to_string(),
            selection: "#ff008044".to_string(),
            error: "#ff0055".to_string(),
            warning: "#ffaa00".to_string(),
            success: "#00ff9f".to_string(),
        },
        terminal: TerminalTheme::default(),
        ui: UiTheme {
            border_radius: "8px".to_string(),
            shadows: true,
            animations: true,
            blur: true,
        },
        css_file: None,
        path: None,
    }
}

/// Create all bundled themes
fn create_bundled_themes() -> Vec<Theme> {
    vec![
        create_dracula_theme(),
        create_monokai_theme(),
        create_nord_theme(),
    ]
}

/// Install a bundled theme to the user's themes directory
fn install_bundled_theme(theme: &Theme, theme_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(theme_dir)?;
    
    // Write theme.json
    let theme_json = serde_json::to_string_pretty(theme)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(theme_dir.join("theme.json"), theme_json)?;
    
    // Write CSS file if present
    if let Some(css_file) = &theme.css_file {
        let css_content = get_bundled_css(&theme.id);
        if let Some(css) = css_content {
            std::fs::write(theme_dir.join(css_file), css)?;
        }
    }
    
    tracing::info!("Installed bundled theme: {}", theme.id);
    Ok(())
}

/// Get bundled CSS for a theme
fn get_bundled_css(theme_id: &str) -> Option<&'static str> {
    match theme_id {
        "dracula" => Some(DRACULA_CSS),
        "monokai" => Some(MONOKAI_CSS),
        "nord" => Some(NORD_CSS),
        _ => None,
    }
}

// =============================================================================
// Dracula Theme
// =============================================================================

fn create_dracula_theme() -> Theme {
    Theme {
        id: "dracula".to_string(),
        name: "Dracula".to_string(),
        version: "1.0.0".to_string(),
        author: "NeonShell Team".to_string(),
        description: "A dark theme with vibrant purple accents".to_string(),
        colors: ThemeColors {
            background: "#282a36".to_string(),
            foreground: "#f8f8f2".to_string(),
            accent: "#bd93f9".to_string(),
            accent_muted: "#6272a4".to_string(),
            surface_0: "#282a36".to_string(),
            surface_1: "#2d303d".to_string(),
            surface_2: "#343746".to_string(),
            surface_3: "#3b3f51".to_string(),
            border: "#44475a".to_string(),
            cursor: "#f8f8f2".to_string(),
            selection: "#44475a".to_string(),
            error: "#ff5555".to_string(),
            warning: "#ffb86c".to_string(),
            success: "#50fa7b".to_string(),
        },
        terminal: TerminalTheme {
            font_family: "JetBrains Mono".to_string(),
            font_size: 14,
            ansi_colors: AnsiColors {
                black: "#21222c".to_string(),
                red: "#ff5555".to_string(),
                green: "#50fa7b".to_string(),
                yellow: "#f1fa8c".to_string(),
                blue: "#bd93f9".to_string(),
                magenta: "#ff79c6".to_string(),
                cyan: "#8be9fd".to_string(),
                white: "#f8f8f2".to_string(),
                bright_black: "#6272a4".to_string(),
                bright_red: "#ff6e6e".to_string(),
                bright_green: "#69ff94".to_string(),
                bright_yellow: "#ffffa5".to_string(),
                bright_blue: "#d6acff".to_string(),
                bright_magenta: "#ff92df".to_string(),
                bright_cyan: "#a4ffff".to_string(),
                bright_white: "#ffffff".to_string(),
            },
        },
        ui: UiTheme {
            border_radius: "8px".to_string(),
            shadows: true,
            animations: true,
            blur: true,
        },
        css_file: Some("styles.css".to_string()),
        path: None,
    }
}

const DRACULA_CSS: &str = r#"/* Dracula Theme for NeonShell */
:root {
  --surface-0: #282a36;
  --surface-1: #2d303d;
  --surface-2: #343746;
  --surface-3: #3b3f51;
  --accent: #bd93f9;
  --accent-muted: #6272a4;
  --foreground: #f8f8f2;
  --foreground-muted: #6272a4;
  --border: #44475a;
  --border-focus: #bd93f9;
  --error: #ff5555;
  --warning: #ffb86c;
  --success: #50fa7b;
}

.neon-glow, .btn-primary {
  box-shadow: 0 0 5px rgba(189, 147, 249, 0.3), 0 0 10px rgba(189, 147, 249, 0.2);
}

.btn-primary:hover {
  box-shadow: 0 0 10px rgba(189, 147, 249, 0.4), 0 0 20px rgba(189, 147, 249, 0.3);
}

::selection {
  background: var(--accent);
  color: var(--surface-0);
}

.sidebar-active {
  border-left: 3px solid var(--accent);
  background: linear-gradient(90deg, rgba(189, 147, 249, 0.1) 0%, transparent 100%);
}

.btn-primary {
  background: linear-gradient(135deg, #bd93f9 0%, #6272a4 100%);
  color: var(--surface-0);
}

.btn-primary:hover {
  background: linear-gradient(135deg, #caa6ff 0%, #bd93f9 100%);
}

.input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 5px rgba(189, 147, 249, 0.3);
}
"#;

// =============================================================================
// Monokai Theme
// =============================================================================

fn create_monokai_theme() -> Theme {
    Theme {
        id: "monokai".to_string(),
        name: "Monokai Pro".to_string(),
        version: "1.0.0".to_string(),
        author: "NeonShell Team".to_string(),
        description: "A warm, elegant theme inspired by Monokai".to_string(),
        colors: ThemeColors {
            background: "#2d2a2e".to_string(),
            foreground: "#fcfcfa".to_string(),
            accent: "#ffd866".to_string(),
            accent_muted: "#c9a93e".to_string(),
            surface_0: "#2d2a2e".to_string(),
            surface_1: "#353236".to_string(),
            surface_2: "#403e41".to_string(),
            surface_3: "#4a474c".to_string(),
            border: "#5b595c".to_string(),
            cursor: "#fcfcfa".to_string(),
            selection: "#5b595c".to_string(),
            error: "#ff6188".to_string(),
            warning: "#fc9867".to_string(),
            success: "#a9dc76".to_string(),
        },
        terminal: TerminalTheme {
            font_family: "JetBrains Mono".to_string(),
            font_size: 14,
            ansi_colors: AnsiColors {
                black: "#2d2a2e".to_string(),
                red: "#ff6188".to_string(),
                green: "#a9dc76".to_string(),
                yellow: "#ffd866".to_string(),
                blue: "#78dce8".to_string(),
                magenta: "#ab9df2".to_string(),
                cyan: "#78dce8".to_string(),
                white: "#fcfcfa".to_string(),
                bright_black: "#727072".to_string(),
                bright_red: "#ff6188".to_string(),
                bright_green: "#a9dc76".to_string(),
                bright_yellow: "#ffd866".to_string(),
                bright_blue: "#78dce8".to_string(),
                bright_magenta: "#ab9df2".to_string(),
                bright_cyan: "#78dce8".to_string(),
                bright_white: "#fcfcfa".to_string(),
            },
        },
        ui: UiTheme {
            border_radius: "6px".to_string(),
            shadows: true,
            animations: true,
            blur: true,
        },
        css_file: Some("styles.css".to_string()),
        path: None,
    }
}

const MONOKAI_CSS: &str = r#"/* Monokai Pro Theme for NeonShell */
:root {
  --surface-0: #2d2a2e;
  --surface-1: #353236;
  --surface-2: #403e41;
  --surface-3: #4a474c;
  --accent: #ffd866;
  --accent-muted: #c9a93e;
  --foreground: #fcfcfa;
  --foreground-muted: #939293;
  --border: #5b595c;
  --border-focus: #ffd866;
  --error: #ff6188;
  --warning: #fc9867;
  --success: #a9dc76;
}

.neon-glow, .btn-primary {
  box-shadow: 0 0 5px rgba(255, 216, 102, 0.3), 0 0 10px rgba(255, 216, 102, 0.15);
}

.btn-primary:hover {
  box-shadow: 0 0 10px rgba(255, 216, 102, 0.5), 0 0 20px rgba(255, 216, 102, 0.25);
}

.neon-text {
  background: linear-gradient(135deg, #ffd866 0%, #fc9867 50%, #ff6188 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.xterm-cursor-block {
  background-color: var(--accent) !important;
}

::selection {
  background: var(--accent);
  color: var(--surface-0);
}

.sidebar-active {
  border-left: 3px solid var(--accent);
  background: linear-gradient(90deg, rgba(255, 216, 102, 0.1) 0%, transparent 100%);
}

.btn-primary {
  background: linear-gradient(135deg, #ffd866 0%, #fc9867 100%);
  color: var(--surface-0);
  font-weight: 600;
}

.btn-primary:hover {
  background: linear-gradient(135deg, #ffe699 0%, #ffd866 100%);
}

.input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 5px rgba(255, 216, 102, 0.25);
}
"#;

// =============================================================================
// Nord Theme
// =============================================================================

fn create_nord_theme() -> Theme {
    Theme {
        id: "nord".to_string(),
        name: "Nord Aurora".to_string(),
        version: "1.0.0".to_string(),
        author: "NeonShell Team".to_string(),
        description: "A cool, arctic-inspired theme based on Nord".to_string(),
        colors: ThemeColors {
            background: "#2e3440".to_string(),
            foreground: "#eceff4".to_string(),
            accent: "#88c0d0".to_string(),
            accent_muted: "#5e81ac".to_string(),
            surface_0: "#2e3440".to_string(),
            surface_1: "#3b4252".to_string(),
            surface_2: "#434c5e".to_string(),
            surface_3: "#4c566a".to_string(),
            border: "#4c566a".to_string(),
            cursor: "#d8dee9".to_string(),
            selection: "#4c566a".to_string(),
            error: "#bf616a".to_string(),
            warning: "#ebcb8b".to_string(),
            success: "#a3be8c".to_string(),
        },
        terminal: TerminalTheme {
            font_family: "JetBrains Mono".to_string(),
            font_size: 14,
            ansi_colors: AnsiColors {
                black: "#3b4252".to_string(),
                red: "#bf616a".to_string(),
                green: "#a3be8c".to_string(),
                yellow: "#ebcb8b".to_string(),
                blue: "#81a1c1".to_string(),
                magenta: "#b48ead".to_string(),
                cyan: "#88c0d0".to_string(),
                white: "#e5e9f0".to_string(),
                bright_black: "#4c566a".to_string(),
                bright_red: "#bf616a".to_string(),
                bright_green: "#a3be8c".to_string(),
                bright_yellow: "#ebcb8b".to_string(),
                bright_blue: "#81a1c1".to_string(),
                bright_magenta: "#b48ead".to_string(),
                bright_cyan: "#8fbcbb".to_string(),
                bright_white: "#eceff4".to_string(),
            },
        },
        ui: UiTheme {
            border_radius: "10px".to_string(),
            shadows: true,
            animations: true,
            blur: true,
        },
        css_file: Some("styles.css".to_string()),
        path: None,
    }
}

const NORD_CSS: &str = r#"/* Nord Aurora Theme for NeonShell */
:root {
  --surface-0: #2e3440;
  --surface-1: #3b4252;
  --surface-2: #434c5e;
  --surface-3: #4c566a;
  --accent: #88c0d0;
  --accent-muted: #5e81ac;
  --foreground: #eceff4;
  --foreground-muted: #d8dee9;
  --border: #4c566a;
  --border-focus: #88c0d0;
  --error: #bf616a;
  --warning: #ebcb8b;
  --success: #a3be8c;
}

body {
  background: linear-gradient(180deg, #2e3440 0%, #272c36 100%);
}

.neon-glow, .btn-primary {
  box-shadow: 0 0 10px rgba(136, 192, 208, 0.2), 0 0 20px rgba(136, 192, 208, 0.1);
}

.btn-primary:hover {
  box-shadow: 0 0 15px rgba(136, 192, 208, 0.3), 0 0 30px rgba(136, 192, 208, 0.15);
}

.neon-text {
  background: linear-gradient(135deg, #bf616a 0%, #d08770 25%, #ebcb8b 50%, #a3be8c 75%, #b48ead 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: aurora-shift 8s ease infinite;
  background-size: 200% 200%;
}

@keyframes aurora-shift {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}

::selection {
  background: var(--accent);
  color: var(--surface-0);
}

.sidebar-active {
  border-left: 3px solid var(--accent);
  background: linear-gradient(90deg, rgba(136, 192, 208, 0.1) 0%, transparent 100%);
}

.btn {
  border-radius: 10px;
}

.btn:hover {
  transform: translateY(-1px);
}

.btn-primary {
  background: linear-gradient(135deg, #88c0d0 0%, #81a1c1 100%);
  color: var(--surface-0);
}

.btn-primary:hover {
  background: linear-gradient(135deg, #8fbcbb 0%, #88c0d0 100%);
}

.input {
  border-radius: 10px;
}

.input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 10px rgba(136, 192, 208, 0.15);
}

.panel {
  background: rgba(59, 66, 82, 0.9);
  backdrop-filter: blur(10px);
  border-radius: 10px;
}

::-webkit-scrollbar-thumb {
  border-radius: 5px;
}

.status-connected {
  color: var(--success);
  text-shadow: 0 0 8px rgba(163, 190, 140, 0.5);
}

.status-disconnected {
  color: var(--error);
  text-shadow: 0 0 8px rgba(191, 97, 106, 0.5);
}

.command-palette {
  border: 1px solid var(--accent);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), 0 0 20px rgba(136, 192, 208, 0.1);
  backdrop-filter: blur(12px);
  border-radius: 12px;
}
"#;

/// Export pack format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeonPack {
    pub version: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub theme: Option<Theme>,
    #[serde(default)]
    pub layout: Option<serde_json::Value>,
    #[serde(default)]
    pub hotkeys: Option<HashMap<String, String>>,
    #[serde(default)]
    pub snippets: Option<Vec<Snippet>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub variables: Vec<SnippetVariable>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetVariable {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub default: String,
}

