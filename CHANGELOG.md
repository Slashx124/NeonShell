# Changelog

All notable changes to NeonShell will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.3] - 2024-12-24

### Fixed

- App now displays the correct version number dynamically from Tauri config
- Fixed TypeScript build errors (removed invalid xterm.js options, fixed AuthMethod type)

## [0.2.2] - 2024-12-24

### Added

- **Connection Error Display**: Users now see detailed error messages when SSH connections fail, including specific reasons like authentication errors, network issues, or host key problems, instead of a generic "Connection Failed" message.
- **Edit Connection Profiles**: Click the three-dot menu next to a saved connection in the sidebar and select "Edit" to modify connection details (name, host, port, username, authentication method).
- **Settings Store**: New centralized settings store (`settingsStore.ts`) for reactive settings management across the application.
- **Copy on Select**: Terminal now supports copying selected text to clipboard automatically when the "Copy on select" setting is enabled.

### Fixed

- **SSH Algorithm Compatibility**: Configured preferred key exchange, host key, encryption (cipher), and MAC algorithms to maximize compatibility with different SSH servers. Resolves "Unable to exchange encryption keys" errors when connecting to servers with specific algorithm requirements.
- **Double Input/Output Bug**: Fixed critical issue where keystrokes and server responses appeared twice in the terminal. Root cause was multiple `ssh:data` event listeners being registered due to:
  - React Strict Mode calling `useEffect` twice in development
  - Async `listen()` promises not being resolved before re-registration
  - HMR (Hot Module Replacement) not properly cleaning up old listeners
  - Solution: Implemented synchronous `_listenersInitialized` flag and `initializedSessions` Set to prevent duplicate registrations.
- **Terminal Settings Application**: Terminal now properly applies user settings for font family, font size, cursor style, cursor blink, scrollback buffer, and bell preferences from the Settings modal.
- **Shell Fallback Mechanism**: Added fallback shell commands (`$SHELL -l`, `/bin/sh -l`, `bash -l`, `sh -l`) when the primary shell request fails, improving compatibility with various server configurations.

### Changed

- Improved terminal initialization with global registry to track active terminals per session, preventing duplicate terminal instances during component re-renders.
- Enhanced cleanup logic in terminal component to properly dispose all event handlers and prevent memory leaks.

## [Unreleased]

