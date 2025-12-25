/**
 * NeonShell Plugin API
 * 
 * This package provides TypeScript types for building NeonShell plugins.
 */

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  api_version: string;
  description?: string;
  author?: string;
  homepage?: string;
  main: string;
  permissions: PluginPermission[];
  signed?: boolean;
}

export type PluginPermission =
  | 'network'
  | 'filesystem'
  | 'clipboard'
  | 'notifications'
  | 'terminal'
  | 'shell';

export interface Plugin {
  activate(api: PluginAPI): void | Promise<void>;
  deactivate?(): void | Promise<void>;
}

export interface PluginAPI {
  commands: CommandsAPI;
  hooks: HooksAPI;
  sessions: SessionsAPI;
  ui: UIAPI;
  terminal: TerminalAPI;
  clipboard: ClipboardAPI;
  storage: StorageAPI;
}

// Commands API
export interface CommandsAPI {
  register(id: string, definition: CommandDefinition): void;
  unregister(id: string): void;
  execute(id: string): Promise<void>;
}

export interface CommandDefinition {
  name: string;
  description?: string;
  shortcut?: string;
  execute: () => void | Promise<void>;
}

// Hooks API
export interface HooksAPI {
  onConnect(handler: (session: Session) => void): () => void;
  onDisconnect(handler: (session: Session) => void): () => void;
  onData(handler: (session: Session, data: string) => void): () => void;
  onCommand(handler: (session: Session, command: string) => void): () => void;
  onError(handler: (session: Session, error: Error) => void): () => void;
}

// Sessions API
export interface SessionsAPI {
  getActive(): Session | null;
  getAll(): Session[];
  getById(id: string): Session | null;
}

export interface Session {
  id: string;
  host: string;
  port: number;
  username: string;
  state: 'Created' | 'Connecting' | 'Connected' | 'Disconnected' | 'Error';
  profileId?: string;
  connectedAt?: number;
}

// UI API
export interface UIAPI {
  showNotification(options: NotificationOptions): void;
  showQuickPick<T extends string>(items: T[], options?: QuickPickOptions): Promise<T | undefined>;
  showInput(options?: InputOptions): Promise<string | undefined>;
  showPanel(panel: PanelDefinition): string;
  hidePanel(id: string): void;
  addStatusBarItem(item: StatusBarItem): string;
  removeStatusBarItem(id: string): void;
}

export interface NotificationOptions {
  title?: string;
  body: string;
  type?: 'info' | 'success' | 'warning' | 'error';
}

export interface QuickPickOptions {
  placeholder?: string;
  canPickMany?: boolean;
}

export interface InputOptions {
  placeholder?: string;
  prompt?: string;
  password?: boolean;
  value?: string;
}

export interface PanelDefinition {
  id: string;
  title: string;
  content: string | HTMLElement;
  position?: 'left' | 'right' | 'bottom';
}

export interface StatusBarItem {
  id: string;
  text: string;
  tooltip?: string;
  position?: 'left' | 'right';
  priority?: number;
}

// Terminal API
export interface TerminalAPI {
  addContextMenuItem(item: ContextMenuItem): string;
  removeContextMenuItem(id: string): void;
  write(sessionId: string, data: string): void;
  getSelection(sessionId: string): string | null;
  search(sessionId: string, query: string): void;
}

export interface ContextMenuItem {
  id: string;
  label: string;
  shortcut?: string;
  execute: (context: ContextMenuContext) => void;
}

export interface ContextMenuContext {
  sessionId: string;
  selectedText?: string;
  x: number;
  y: number;
}

// Clipboard API
export interface ClipboardAPI {
  read(): Promise<string>;
  write(text: string): Promise<void>;
}

// Storage API
export interface StorageAPI {
  get<T>(key: string, defaultValue?: T): Promise<T | undefined>;
  set<T>(key: string, value: T): Promise<void>;
  delete(key: string): Promise<void>;
  keys(): Promise<string[]>;
}




