import { invoke } from '@tauri-apps/api/core';

// SSH API
export const ssh = {
  createSession: (config: unknown) => invoke<string>('create_session', { config }),
  connect: (sessionId: string, password?: string, privateKey?: string) =>
    invoke<{ success: boolean; error?: string }>('connect', { sessionId, password, privateKey }),
  disconnect: (sessionId: string) => invoke('disconnect', { sessionId }),
  sendData: (sessionId: string, data: number[]) => invoke('send_data', { sessionId, data }),
  resizePty: (sessionId: string, cols: number, rows: number) =>
    invoke('resize_pty', { sessionId, cols, rows }),
  listSessions: () => invoke('list_sessions'),
};

// Profile API
export const profiles = {
  list: () => invoke('list_profiles'),
  get: (id: string) => invoke('get_profile', { id }),
  save: (profile: unknown, isNew: boolean) => invoke('save_profile', { profile, isNew }),
  delete: (id: string) => invoke('delete_profile', { id }),
  importSshConfig: (content: string) => invoke('import_ssh_config', { content }),
  exportSshConfig: () => invoke<string>('export_ssh_config'),
};

// Settings API
export const settings = {
  get: () => invoke('get_settings'),
  save: (settings: unknown) => invoke('save_settings', { settings }),
};

// Keychain API
export const keychain = {
  store: (key: string, secret: string) => invoke('store_secret', { key, secret }),
  get: (key: string) => invoke<string | null>('get_secret', { key }),
  delete: (key: string) => invoke('delete_secret', { key }),
  has: (key: string) => invoke<boolean>('has_secret', { key }),
};

// Plugin API
export const plugins = {
  list: () => invoke('list_plugins'),
  get: (id: string) => invoke('get_plugin', { id }),
  enable: (id: string, permissions: string[]) => invoke('enable_plugin', { id, permissions }),
  disable: (id: string) => invoke('disable_plugin', { id }),
  install: (path: string) => invoke<string>('install_plugin', { path }),
};

// Script API
export const scripts = {
  list: () => invoke('list_scripts'),
  run: (id: string, fn: string, args: unknown) => invoke('run_script', { id, function: fn, args }),
  enable: (id: string) => invoke('enable_script', { id }),
  disable: (id: string) => invoke('disable_script', { id }),
};

// Theme API
export const themes = {
  list: () => invoke('list_themes'),
  get: (id: string) => invoke('get_theme', { id }),
  set: (id: string) => invoke('set_theme', { id }),
  exportPack: (
    name: string,
    includeTheme: boolean,
    includeLayout: boolean,
    includeHotkeys: boolean,
    includeSnippets: boolean
  ) =>
    invoke<number[]>('export_pack', {
      name,
      includeTheme,
      includeLayout,
      includeHotkeys,
      includeSnippets,
    }),
  importPack: (data: number[]) => invoke('import_pack', { data }),
};




