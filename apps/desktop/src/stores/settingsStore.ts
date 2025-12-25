import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface TerminalSettings {
  font_family: string;
  font_size: number;
  cursor_style: 'block' | 'underline' | 'bar';
  cursor_blink: boolean;
  scrollback: number;
  copy_on_select: boolean;
  bell_sound: boolean;
  bell_visual: boolean;
}

export interface GeneralSettings {
  theme: string;
  language: string;
  check_updates: boolean;
  start_minimized: boolean;
  restore_sessions: boolean;
}

export interface SshSettings {
  default_port: number;
  keepalive_interval: number;
  strict_host_checking: boolean;
  agent_forwarding: boolean;
  compression: boolean;
}

export interface SecuritySettings {
  store_passwords: string;
  auto_lock_timeout: number;
  clear_clipboard: boolean;
  clipboard_timeout: number;
}

export interface AppSettings {
  general: GeneralSettings;
  terminal: TerminalSettings;
  ssh: SshSettings;
  security: SecuritySettings;
}

interface SettingsStoreState {
  settings: AppSettings | null;
  loading: boolean;
  
  loadSettings: () => Promise<void>;
  updateSettings: (settings: AppSettings) => Promise<void>;
}

const defaultSettings: AppSettings = {
  general: {
    theme: 'cyberpunk',
    language: 'en',
    check_updates: true,
    start_minimized: false,
    restore_sessions: true,
  },
  terminal: {
    font_family: 'JetBrains Mono',
    font_size: 14,
    cursor_style: 'block',
    cursor_blink: true,
    scrollback: 10000,
    copy_on_select: true,
    bell_sound: false,
    bell_visual: false,
  },
  ssh: {
    default_port: 22,
    keepalive_interval: 60,
    strict_host_checking: true,
    agent_forwarding: false,
    compression: false,
  },
  security: {
    store_passwords: 'keychain',
    auto_lock_timeout: 300,
    clear_clipboard: true,
    clipboard_timeout: 30,
  },
};

export const useSettingsStore = create<SettingsStoreState>((set, get) => ({
  settings: null,
  loading: false,

  loadSettings: async () => {
    if (get().loading) return;
    
    set({ loading: true });
    try {
      const settings = await invoke<AppSettings>('get_settings');
      set({ settings, loading: false });
    } catch (error) {
      console.error('Failed to load settings:', error);
      // Use defaults if loading fails
      set({ settings: defaultSettings, loading: false });
    }
  },

  updateSettings: async (settings: AppSettings) => {
    try {
      await invoke('save_settings', { settings });
      set({ settings });
    } catch (error) {
      console.error('Failed to save settings:', error);
      throw error;
    }
  },
}));

