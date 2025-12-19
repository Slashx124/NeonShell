<p align="center">
  <img src="docs/logo.png" alt="NeonShell Logo" width="200"/>
</p>

<h1 align="center">🌈 NeonShell</h1>

<p align="center">
  <strong>A heavily customizable terminal experience with AI Baked In.</strong>
</p>

<p align="center">
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"/></a>
  <a href="https://github.com/Slashx124/NeonShell/actions"><img src="https://github.com/Slashx124/NeonShell/actions/workflows/build.yml/badge.svg" alt="Build"/></a>
  <a href="https://github.com/Slashx124/NeonShell/releases"><img src="https://img.shields.io/github/v/release/Slashx124/NeonShell?include_prereleases" alt="Release"/></a>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-themes">Themes</a> •
  <a href="#-plugins">Plugins</a> •
  <a href="#-roadmap">Roadmap</a>
</p>

---

<p align="center">
  <img src="docs/screenshot.png" alt="NeonShell Screenshot" width="800"/>
</p>

## 🚀 Why NeonShell?

Tired of boring, cookie-cutter terminals? **NeonShell** brings the aesthetic energy of the early internet customization era to modern SSH workflows. Think **Myspace for SSH nerds** – but with rock-solid security, blazing performance, and AI-powered assistance.

```
╔══════════════════════════════════════════════════════════════╗
║  🔐 Security-First  │  🎨 Insane Theming  │  🤖 AI-Powered   ║
║  🔌 Plugin System   │  🐍 Python Scripts  │  ⚡ Rust Core    ║
╚══════════════════════════════════════════════════════════════╝
```

## ✨ Features

### 🔐 Security-First Design
- **OS Keychain Integration** – Passwords and keys stored in macOS Keychain, Windows Credential Manager, or Linux Secret Service
- **Zero Secret Logging** – Secrets are NEVER written to logs, debug output, or crash reports
- **Strict Host Key Checking** – TOFU (Trust On First Use) with clear fingerprint UI and mismatch warnings
- **Sanitized Debug Exports** – Share debug bundles without fear of leaking credentials

### 🎨 Extreme Theming
- **CSS Theme Packs** – Full control over colors, fonts, spacing, animations
- **Per-Host Themes** – Different aesthetic for production vs staging
- **Bundled Themes** – Ships with Dracula, Monokai Pro, and Nord Aurora
- **Import/Export Packs** – Share your entire setup (theme + layout + hotkeys + snippets)
- **Live Preview** – See changes instantly as you customize

### 🔌 Powerful Plugin System
- **Versioned API** – Plugins declare compatibility, graceful degradation
- **Sandboxed Permissions** – Plugins request only what they need
- **Event Hooks** – `onConnect`, `onDisconnect`, `onData`, `onCommand`, `onError`
- **UI Extensions** – Add panels, commands, context menus, status bar widgets
- **Local or Registry** – Install from folder or community registry

### 🐍 Python Automation
- **Embedded Scripting** – Write Python scripts that hook into terminal events
- **Pre-Connect Checks** – Validate VPN status, check credentials, verify network
- **Auto-Commands** – Run setup commands on connect (tmux attach, load dotfiles)
- **Output Parsing** – Parse terminal output and trigger actions
- **Custom Widgets** – Build dashboard widgets with live data

### ⚡ Power User Workflow
- **Command Palette** – `Ctrl+K` for instant access to any command
- **Hotkey Editor** – Customize every keyboard shortcut
- **Snippet Manager** – Save and expand command snippets with variables
- **Session Recipes** – Saved multi-tab layouts with pre-configured hosts
- **Scrollback Search** – Find anything in your terminal history
- **OpenSSH Config Import** – Bring your existing `~/.ssh/config` profiles

### 🛠️ Debug Console
- **Real-Time Logs** – Watch SSH events stream live (`Ctrl+\``)
- **Filtering** – Filter by level, subsystem, session, or search term
- **Export Debug Bundle** – One-click export of sanitized logs for support
- **Privacy-First** – All exports automatically redact secrets

### 🖥️ Cross-Platform
- **Windows** – Native `.exe` and `.msi` installers
- **macOS** – Universal binary (Intel + Apple Silicon) `.dmg`
- **Linux** – `.deb`, `.rpm`, and `.AppImage`

## 📦 Installation

### Download

**[📥 Download Latest Release](https://github.com/Slashx124/NeonShell/releases/latest)**

| Platform | File |
|----------|------|
| **Windows** | `NeonShell_x.x.x_x64-setup.exe` or `.msi` |
| **macOS (Intel)** | `NeonShell_x.x.x_x64.dmg` |
| **macOS (Apple Silicon)** | `NeonShell_x.x.x_aarch64.dmg` |
| **Linux (Debian/Ubuntu)** | `NeonShell_x.x.x_amd64.deb` |
| **Linux (AppImage)** | `NeonShell_x.x.x_amd64.AppImage` |
| **Linux (RPM)** | `NeonShell-x.x.x-1.x86_64.rpm` |

### Package Managers

```bash
# macOS (Homebrew) - Coming Soon
brew install --cask neonshell

# Windows (Winget) - Coming Soon
winget install NeonShell

# Linux (Snap) - Coming Soon
snap install neonshell
```

### Build from Source

```bash
# Prerequisites: Rust 1.75+, Node.js 20+, pnpm 8+

git clone https://github.com/yourorg/neonshell.git
cd neonshell
pnpm install
pnpm tauri build
```

<details>
<summary><b>Platform-specific dependencies</b></summary>

**Windows:**
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libsecret-1-dev
```
</details>

## 🏁 Quick Start

### 1. Create Your First Connection

Press `Ctrl+N` (or `Cmd+N` on macOS) to open the New Connection dialog:

- Enter your host, username, and port
- Choose authentication: **SSH Agent**, **Password**, or **Private Key**
- Check "Save as Profile" to remember it
- Click **Connect**

### 2. Customize Your Theme

Press `Ctrl+K` → type "theme" → select **Change Theme**

Or import a theme pack: `Ctrl+K` → **Import Pack**

### 3. Set Up Keyboard Shortcuts

Press `Ctrl+K` → **Keyboard Shortcuts** to see all hotkeys

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Command Palette |
| `Ctrl+N` | New Connection |
| `Ctrl+,` | Settings |
| `Ctrl+\`` | Debug Console |
| `Ctrl+B` | Toggle Sidebar |

### 4. Enable Python Scripts

Drop `.py` files into `~/.neonshell/scripts/` and they'll appear in the Scripts Manager.

## 🎨 Themes

### Bundled Themes

NeonShell ships with three beautiful themes:

| Theme | Preview |
|-------|---------|
| **Dracula** | Deep purple darkness with vibrant accents |
| **Monokai Pro** | Warm, professional tones inspired by Sublime Text |
| **Nord Aurora** | Cool Arctic blues with aurora-inspired highlights |

### Create Your Own

Themes are JSON + CSS files in `~/.neonshell/themes/`:

```json
{
  "id": "my-theme",
  "name": "My Awesome Theme",
  "version": "1.0.0",
  "colors": {
    "background": "#1a1a2e",
    "foreground": "#eaeaea",
    "accent": "#e94560",
    "surface0": "#16213e",
    "surface1": "#1a1a2e",
    "surface2": "#0f3460"
  },
  "terminal": {
    "cursorColor": "#e94560",
    "selectionBackground": "#e9456044"
  }
}
```

### Share Your Setup

Export everything as a `.zip` pack:
- Theme + custom CSS
- Keyboard shortcuts
- Snippets
- Layout preferences

**Does NOT include:** Passwords, private keys, or any secrets.

## 🔌 Plugins

### Installing Plugins

1. Download a plugin folder
2. Place in `~/.neonshell/plugins/`
3. Go to `Ctrl+K` → **Manage Plugins**
4. Enable the plugin and grant permissions

### Plugin Capabilities

| Permission | What It Allows |
|------------|----------------|
| `network` | Make HTTP/HTTPS requests |
| `filesystem` | Read/write plugin's data directory |
| `clipboard` | Access system clipboard |
| `notifications` | Show desktop notifications |
| `terminal` | Read terminal output stream |

### Example Plugin

```javascript
// plugins/hello-world/index.js
export default {
  activate(api) {
    api.commands.register('hello.greet', () => {
      api.ui.showToast('Hello from NeonShell! 👋');
    });
    
    api.hooks.onConnect((session) => {
      api.log(`Connected to ${session.host}`);
    });
  }
};
```

## 🐍 Python Scripts

### Example: Auto-Attach to tmux

```python
# ~/.neonshell/scripts/auto_tmux.py
from neonshell import hook, run_command

@hook("on_connect")
def attach_tmux(session):
    """Automatically attach to tmux or create new session."""
    run_command(session, "tmux attach || tmux new-session")
```

### Example: Health Check Before Connect

```python
# ~/.neonshell/scripts/vpn_check.py
from neonshell import hook, abort, prompt
import subprocess

@hook("pre_connect")
def check_vpn(session):
    """Ensure VPN is connected before allowing SSH."""
    result = subprocess.run(["pgrep", "openvpn"], capture_output=True)
    if result.returncode != 0:
        if not prompt("VPN not detected. Connect anyway?"):
            abort("Connection cancelled - VPN required")
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       NeonShell                              │
├──────────────────────┬──────────────────────────────────────┤
│   Frontend (React)   │         Backend (Tauri/Rust)         │
├──────────────────────┼──────────────────────────────────────┤
│ • xterm.js terminal  │ • SSH engine (libssh2)               │
│ • Zustand state      │ • OS keychain integration            │
│ • Tailwind CSS       │ • Plugin sandbox                     │
│ • Command palette    │ • Python script runner               │
│ • Theme engine       │ • Config management (TOML)           │
└──────────────────────┴──────────────────────────────────────┘
```

| Component | Technology | Why |
|-----------|------------|-----|
| Desktop Framework | Tauri v2 | Smaller than Electron, native performance, Rust security |
| Terminal | xterm.js | Industry standard, excellent performance |
| SSH | libssh2 (ssh2 crate) | Mature, cross-platform, full feature support |
| Frontend | React + TypeScript | Type safety, excellent tooling |
| Styling | Tailwind CSS | Rapid iteration, consistent design |
| State | Zustand | Simple, performant, TypeScript-first |
| Config | TOML | Human-readable, comments, schema validation |

## 🔒 Security

### Threat Model

| Threat | Mitigation |
|--------|------------|
| Private key theft | Keys encrypted in OS keychain, never on disk |
| Credential logging | Regex sanitization on ALL log output |
| Malicious plugins | Sandboxed with explicit permission grants |
| MITM attacks | Strict host key verification, clear warnings |
| Debug data leaks | Automatic redaction of secrets in exports |

### What We NEVER Do

- ❌ Store passwords in plaintext
- ❌ Log private keys or passphrases
- ❌ Send telemetry without consent
- ❌ Auto-update without permission
- ❌ Run plugins with full system access

### Security Advisories

Found a vulnerability? Please email **security@neonshell.dev** (do not open public issues for security bugs).

## 🗺️ Roadmap

### 🤖 AI Features (Coming Soon)
- [ ] **AI Command Suggestions** – Context-aware command completions powered by local LLM
- [ ] **Natural Language to Command** – Type "show disk usage" → `df -h`
- [ ] **Error Explanation** – AI explains cryptic error messages
- [ ] **Smart Autocomplete** – Learn from your command history
- [ ] **Chat with Terminal** – Ask questions about output, get actionable answers
- [ ] **AI-Powered Search** – "Find that command I ran last week to restart nginx"
- [ ] **Anomaly Detection** – Alert on unusual output patterns
- [ ] **Local-First AI** – Run models locally, no data leaves your machine

### 🌐 Website & Community
- [ ] **Official Website** – neonshell.dev with docs, downloads, showcase
- [ ] **Theme Gallery** – Browse and install community themes
- [ ] **Plugin Marketplace** – Discover and install plugins
- [ ] **Script Library** – Share Python automation scripts
- [ ] **User Showcase** – Show off your customized setups

### 🎨 Theming & Customization
- [ ] **Theme Editor** – Visual theme builder with live preview
- [ ] **Background Images** – Custom terminal backgrounds with blur
- [ ] **Custom Fonts** – Upload and use any font
- [ ] **Animation Presets** – Cursor effects, transitions, glows
- [ ] **Sound Themes** – Notification sounds, typing sounds

### 🔌 Plugins & Extensions
- [ ] **VSCode Extension** – Open NeonShell terminals in VSCode
- [ ] **Raycast Extension** – Quick connect from Raycast
- [ ] **Alfred Workflow** – macOS quick actions
- [ ] **Browser Extension** – SSH links handler

### 📱 Platform Expansion
- [ ] **iOS App** – Connect from iPhone/iPad
- [ ] **Android App** – Mobile SSH client
- [ ] **Web Version** – Browser-based NeonShell
- [ ] **Raspberry Pi** – Optimized ARM builds

### ⚡ Power Features
- [ ] **Split Panes** – Horizontal and vertical terminal splits
- [ ] **Session Recording** – Record and replay terminal sessions
- [ ] **Broadcast Input** – Type to multiple sessions at once
- [ ] **SFTP Browser** – Visual file browser with drag-and-drop
- [ ] **Port Forward Manager** – GUI for SSH tunnels
- [ ] **Jump Host Chains** – Multi-hop SSH with visual builder
- [ ] **Connection Sharing** – Multiplexed SSH connections

### 🔐 Security Enhancements
- [ ] **Hardware Key Support** – YubiKey, Titan Key for SSH
- [ ] **2FA Integration** – TOTP for SSH servers that support it
- [ ] **Audit Log** – Track all connections and commands
- [ ] **Session Timeout** – Auto-lock after inactivity
- [ ] **Biometric Unlock** – Face ID / Touch ID / Windows Hello

### 🧪 Developer Experience
- [ ] **Plugin SDK** – TypeScript types, testing utilities
- [ ] **Theme SDK** – CLI tool to scaffold and validate themes
- [ ] **API Documentation** – Interactive API explorer
- [ ] **Example Plugins** – Rich collection of reference implementations

### 🌍 Internationalization
- [ ] **Multi-Language UI** – Spanish, French, German, Japanese, Chinese, Korean
- [ ] **RTL Support** – Arabic, Hebrew layout support
- [ ] **Localized Documentation** – Translated docs

## 🤝 Contributing

We love contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Start for Contributors

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/neonshell.git
cd neonshell

# Install dependencies
pnpm install

# Run in development
pnpm dev

# Run tests
pnpm test

# Build
pnpm build
```

### Ways to Contribute

- 🐛 **Report Bugs** – Open an issue with reproduction steps
- 💡 **Suggest Features** – We love hearing ideas
- 🎨 **Create Themes** – Share your color schemes
- 🔌 **Build Plugins** – Extend functionality
- 📖 **Improve Docs** – Fix typos, add examples
- 🌍 **Translate** – Help us reach more users

## 📝 License

MIT License – see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) – For making Rust-powered desktop apps possible
- [xterm.js](https://xtermjs.org/) – The best terminal emulator for the web
- [libssh2](https://www.libssh2.org/) – Reliable SSH implementation
- [Dracula Theme](https://draculatheme.com/) – Color palette inspiration
- Everyone who believes terminals can be beautiful ✨

---

<p align="center">
  <strong>Built with 💜 by the NeonShell community</strong>
</p>

<p align="center">
  <a href="https://neonshell.dev">Website</a> •
  <a href="https://docs.neonshell.dev">Documentation</a> •
  <a href="https://discord.gg/neonshell">Discord</a> •
  <a href="https://twitter.com/neonshell">Twitter</a>
</p>
