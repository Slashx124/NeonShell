/**
 * NeonShell Starter Plugin
 * 
 * This is a minimal example demonstrating the plugin API.
 * Use this as a template to build your own plugins.
 */

export default {
  /**
   * Called when the plugin is activated
   * @param {PluginAPI} api - The NeonShell plugin API
   */
  activate(api) {
    console.log('Starter Plugin activated!');

    // Register a command in the command palette
    api.commands.register('starter-plugin.hello', {
      name: 'Hello from Plugin',
      description: 'Show a greeting notification',
      execute: () => {
        api.ui.showNotification({
          title: 'Hello!',
          body: 'Greetings from the Starter Plugin 👋',
          type: 'info',
        });
      },
    });

    // Register a command to show connection info
    api.commands.register('starter-plugin.connection-info', {
      name: 'Show Connection Info',
      description: 'Display current connection details',
      execute: () => {
        const session = api.sessions.getActive();
        if (session) {
          api.ui.showNotification({
            title: 'Connection Info',
            body: `Connected to ${session.username}@${session.host}:${session.port}`,
            type: 'info',
          });
        } else {
          api.ui.showNotification({
            title: 'No Connection',
            body: 'Not connected to any host',
            type: 'warning',
          });
        }
      },
    });

    // Hook into connection events
    api.hooks.onConnect((session) => {
      console.log(`[Starter Plugin] Connected to ${session.host}`);
      api.ui.showNotification({
        title: 'Connected',
        body: `Successfully connected to ${session.host}`,
        type: 'success',
      });
    });

    api.hooks.onDisconnect((session) => {
      console.log(`[Starter Plugin] Disconnected from ${session.host}`);
    });

    // Add a status bar widget
    api.ui.addStatusBarItem({
      id: 'starter-plugin.status',
      text: '🔌 Plugin Active',
      tooltip: 'Starter Plugin is running',
      position: 'right',
    });

    // Add context menu item to terminal
    api.terminal.addContextMenuItem({
      id: 'starter-plugin.copy-command',
      label: 'Copy Last Command',
      execute: (context) => {
        if (context.selectedText) {
          api.clipboard.write(context.selectedText);
          api.ui.showNotification({
            title: 'Copied',
            body: 'Text copied to clipboard',
            type: 'info',
          });
        }
      },
    });
  },

  /**
   * Called when the plugin is deactivated
   */
  deactivate() {
    console.log('Starter Plugin deactivated!');
    // Cleanup is automatic - registered commands and hooks are removed
  },
};

/**
 * Plugin API Type Definitions (for IDE support)
 * 
 * @typedef {Object} PluginAPI
 * @property {CommandsAPI} commands
 * @property {HooksAPI} hooks
 * @property {SessionsAPI} sessions
 * @property {UIAPI} ui
 * @property {TerminalAPI} terminal
 * @property {ClipboardAPI} clipboard
 * 
 * @typedef {Object} CommandsAPI
 * @property {function(string, CommandDefinition): void} register
 * 
 * @typedef {Object} CommandDefinition
 * @property {string} name
 * @property {string} [description]
 * @property {function(): void} execute
 * 
 * @typedef {Object} HooksAPI
 * @property {function(function(Session): void): void} onConnect
 * @property {function(function(Session): void): void} onDisconnect
 * @property {function(function(Session, string): void): void} onData
 * 
 * @typedef {Object} Session
 * @property {string} id
 * @property {string} host
 * @property {number} port
 * @property {string} username
 * @property {string} state
 */




