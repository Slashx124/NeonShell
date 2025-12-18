import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface Theme {
  id: string;
  name: string;
  colors: {
    background: string;
    foreground: string;
    accent: string;
    accent_muted: string;
    surface_0: string;
    surface_1: string;
    surface_2: string;
    surface_3: string;
    border: string;
    cursor: string;
    selection: string;
    error: string;
    warning: string;
    success: string;
  };
  terminal: {
    font_family: string;
    font_size: number;
    ansi_colors: {
      black: string;
      red: string;
      green: string;
      yellow: string;
      blue: string;
      magenta: string;
      cyan: string;
      white: string;
      bright_black: string;
      bright_red: string;
      bright_green: string;
      bright_yellow: string;
      bright_blue: string;
      bright_magenta: string;
      bright_cyan: string;
      bright_white: string;
    };
  };
}

interface ThemeState {
  currentTheme: Theme | null;
  themes: Theme[];
  loadTheme: () => Promise<void>;
  setTheme: (themeId: string) => Promise<void>;
  listThemes: () => Promise<void>;
}

const DEFAULT_THEME: Theme = {
  id: 'neon-default',
  name: 'Neon Default',
  colors: {
    background: '#0a0a0f',
    foreground: '#e0e0e0',
    accent: '#ff0080',
    accent_muted: '#aa0055',
    surface_0: '#0a0a0f',
    surface_1: '#12121a',
    surface_2: '#1a1a24',
    surface_3: '#22222e',
    border: '#333344',
    cursor: '#ff0080',
    selection: '#ff008044',
    error: '#ff0055',
    warning: '#ffaa00',
    success: '#00ff9f',
  },
  terminal: {
    font_family: 'JetBrains Mono',
    font_size: 14,
    ansi_colors: {
      black: '#0a0a0f',
      red: '#ff0055',
      green: '#00ff9f',
      yellow: '#ffff00',
      blue: '#00aaff',
      magenta: '#ff00ff',
      cyan: '#00ffff',
      white: '#ffffff',
      bright_black: '#333344',
      bright_red: '#ff5588',
      bright_green: '#55ffbb',
      bright_yellow: '#ffff55',
      bright_blue: '#55bbff',
      bright_magenta: '#ff55ff',
      bright_cyan: '#55ffff',
      bright_white: '#ffffff',
    },
  },
};

export const useThemeStore = create<ThemeState>((set) => ({
  currentTheme: DEFAULT_THEME,
  themes: [DEFAULT_THEME],

  loadTheme: async () => {
    try {
      const settings = await invoke<{ general: { theme: string } }>('get_settings');
      const theme = await invoke<Theme>('get_theme', { id: settings.general.theme });
      
      if (theme) {
        set({ currentTheme: theme });
        applyTheme(theme);
      }
    } catch (error) {
      console.error('Failed to load theme:', error);
      // Use default theme
      applyTheme(DEFAULT_THEME);
    }
  },

  setTheme: async (themeId: string) => {
    try {
      await invoke('set_theme', { id: themeId });
      const theme = await invoke<Theme>('get_theme', { id: themeId });
      
      if (theme) {
        set({ currentTheme: theme });
        applyTheme(theme);
      }
    } catch (error) {
      console.error('Failed to set theme:', error);
    }
  },

  listThemes: async () => {
    try {
      const themes = await invoke<Theme[]>('list_themes');
      set({ themes });
    } catch (error) {
      console.error('Failed to list themes:', error);
    }
  },
}));

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  
  // Apply CSS variables
  root.style.setProperty('--surface-0', theme.colors.surface_0);
  root.style.setProperty('--surface-1', theme.colors.surface_1);
  root.style.setProperty('--surface-2', theme.colors.surface_2);
  root.style.setProperty('--surface-3', theme.colors.surface_3);
  root.style.setProperty('--accent', theme.colors.accent);
  root.style.setProperty('--accent-muted', theme.colors.accent_muted);
  root.style.setProperty('--foreground', theme.colors.foreground);
  root.style.setProperty('--foreground-muted', theme.colors.foreground + '88');
  root.style.setProperty('--border', theme.colors.border);
  root.style.setProperty('--border-focus', theme.colors.accent);
  root.style.setProperty('--error', theme.colors.error);
  root.style.setProperty('--warning', theme.colors.warning);
  root.style.setProperty('--success', theme.colors.success);
}

