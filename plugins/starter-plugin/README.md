# Starter Plugin

A minimal example plugin demonstrating the NeonShell plugin API.

## Installation

1. Copy this folder to `~/.neonshell/plugins/`
2. Restart NeonShell or reload plugins
3. Enable the plugin in Settings → Plugins

## Features

- **Hello Command**: Shows a greeting notification
- **Connection Info**: Displays current connection details
- **Connection Hooks**: Notifies on connect/disconnect
- **Status Bar Widget**: Shows plugin status
- **Context Menu**: Adds "Copy Last Command" option

## API Usage

This plugin demonstrates:

- `api.commands.register()` - Add commands to palette
- `api.hooks.onConnect()` - React to connection events
- `api.ui.showNotification()` - Display notifications
- `api.ui.addStatusBarItem()` - Add status bar widgets
- `api.terminal.addContextMenuItem()` - Extend context menu

## Building Your Own Plugin

1. Create a new folder in `~/.neonshell/plugins/`
2. Add a `manifest.json` with required fields
3. Create your `index.js` with `activate()` and `deactivate()` exports
4. Request only the permissions you need

## Permissions

This plugin uses:

- `notifications` - To show notification popups
- `terminal` - To add context menu items

