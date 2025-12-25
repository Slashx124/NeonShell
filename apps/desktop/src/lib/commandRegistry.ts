import { 
  Plus, 
  Settings, 
  Palette, 
  Puzzle, 
  FileCode,
  LogOut,
  Keyboard,
  Download,
  Upload,
  Terminal,
  Cpu,
  type LucideIcon
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';
import type { ModalType } from '@/stores/uiStore';

export interface CommandItem {
  id: string;
  title: string;
  subtitle?: string;
  shortcut?: string;
  icon: LucideIcon;
  when?: (ctx: CommandContext) => boolean;
  run: (ctx: CommandContext) => void | Promise<void>;
}

export interface CommandContext {
  // App store actions
  setShowCommandPalette: (show: boolean) => void;
  setShowConnectionDialog: (show: boolean) => void;
  activeTabId: string | null;
  tabs: Array<{ id: string; sessionId?: string; connected: boolean }>;
  
  // UI store actions
  openModal: (modal: ModalType) => void;
  closeModal: () => void;
  addToast: (toast: { type: 'success' | 'error' | 'warning' | 'info'; title: string; message?: string }) => void;
  
  // Session store actions
  disconnect: (sessionId: string) => Promise<void>;
  
  // Theme store actions
  listThemes: () => Promise<void>;
}

export function getCommands(): CommandItem[] {
  return [
    {
      id: 'new-connection',
      title: 'New Connection',
      subtitle: 'Connect to a new SSH host',
      icon: Plus,
      shortcut: 'Ctrl+N',
      run: (ctx) => {
        ctx.setShowCommandPalette(false);
        ctx.setShowConnectionDialog(true);
      },
    },
    {
      id: 'disconnect',
      title: 'Disconnect',
      subtitle: 'Close current connection',
      icon: LogOut,
      when: (ctx) => {
        // Only show if there's an active connected tab
        const activeTab = ctx.tabs.find(t => t.id === ctx.activeTabId);
        return activeTab?.connected ?? false;
      },
      run: async (ctx) => {
        ctx.setShowCommandPalette(false);
        
        const activeTab = ctx.tabs.find(t => t.id === ctx.activeTabId);
        if (!activeTab?.sessionId) {
          ctx.addToast({
            type: 'warning',
            title: 'No active connection',
            message: 'There is no active SSH session to disconnect.',
          });
          return;
        }
        
        try {
          await ctx.disconnect(activeTab.sessionId);
          ctx.addToast({
            type: 'success',
            title: 'Disconnected',
            message: 'SSH session closed successfully.',
          });
        } catch (error) {
          ctx.addToast({
            type: 'error',
            title: 'Disconnect failed',
            message: error instanceof Error ? error.message : 'Unknown error',
          });
        }
      },
    },
    {
      id: 'settings',
      title: 'Open Settings',
      subtitle: 'Configure NeonShell',
      icon: Settings,
      shortcut: 'Ctrl+,',
      run: (ctx) => {
        ctx.setShowCommandPalette(false);
        ctx.openModal('settings');
      },
    },
    {
      id: 'themes',
      title: 'Change Theme',
      subtitle: 'Switch color theme',
      icon: Palette,
      run: async (ctx) => {
        ctx.setShowCommandPalette(false);
        await ctx.listThemes();
        ctx.openModal('themePicker');
      },
    },
    {
      id: 'plugins',
      title: 'Manage Plugins',
      subtitle: 'Install and configure plugins',
      icon: Puzzle,
      run: (ctx) => {
        ctx.setShowCommandPalette(false);
        ctx.openModal('plugins');
      },
    },
    {
      id: 'scripts',
      title: 'Python Scripts',
      subtitle: 'Manage automation scripts',
      icon: FileCode,
      run: (ctx) => {
        ctx.setShowCommandPalette(false);
        ctx.openModal('scripts');
      },
    },
    {
      id: 'hotkeys',
      title: 'Keyboard Shortcuts',
      subtitle: 'View and edit hotkeys',
      icon: Keyboard,
      run: (ctx) => {
        ctx.setShowCommandPalette(false);
        ctx.openModal('hotkeys');
      },
    },
    {
      id: 'export-pack',
      title: 'Export Pack',
      subtitle: 'Export theme and settings as a pack',
      icon: Download,
      run: async (ctx) => {
        ctx.setShowCommandPalette(false);
        
        try {
          const path = await save({
            title: 'Export NeonShell Pack',
            defaultPath: 'neonshell-pack.zip',
            filters: [{ name: 'NeonShell Pack', extensions: ['zip'] }],
          });
          
          if (!path) return; // User cancelled
          
          await invoke('export_pack', { path });
          
          ctx.addToast({
            type: 'success',
            title: 'Pack exported',
            message: `Saved to ${path}`,
          });
        } catch (error) {
          ctx.addToast({
            type: 'error',
            title: 'Export failed',
            message: error instanceof Error ? error.message : 'Unknown error',
          });
        }
      },
    },
    {
      id: 'import-pack',
      title: 'Import Pack',
      subtitle: 'Import a NeonShell pack',
      icon: Upload,
      run: async (ctx) => {
        ctx.setShowCommandPalette(false);
        
        try {
          const path = await open({
            title: 'Import NeonShell Pack',
            filters: [{ name: 'NeonShell Pack', extensions: ['zip'] }],
            multiple: false,
          });
          
          if (!path) return; // User cancelled
          
          await invoke('import_pack', { path });
          
          ctx.addToast({
            type: 'success',
            title: 'Pack imported',
            message: 'Settings and theme imported. Reloading...',
          });
          
          // Reload the UI state
          setTimeout(() => {
            window.location.reload();
          }, 1500);
        } catch (error) {
          ctx.addToast({
            type: 'error',
            title: 'Import failed',
            message: error instanceof Error ? error.message : 'Unknown error',
          });
        }
      },
    },
    {
      id: 'debug-console',
      title: 'Debug Console',
      subtitle: 'View debug logs and export reports',
      icon: Terminal,
      shortcut: 'Ctrl+`',
      run: (ctx) => {
        ctx.setShowCommandPalette(false);
        ctx.openModal('debugConsole');
      },
    },
    {
      id: 'ai-settings',
      title: 'AI Model Settings',
      subtitle: 'Configure AI models, gateway, and local providers',
      icon: Cpu,
      run: (ctx) => {
        ctx.setShowCommandPalette(false);
        ctx.openModal('aiSettings');
      },
    },
  ];
}

// Get keyboard shortcut in platform-specific format
export function formatShortcut(shortcut: string): string {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  
  if (isMac) {
    return shortcut
      .replace('Ctrl+', '⌘')
      .replace('Alt+', '⌥')
      .replace('Shift+', '⇧');
  }
  
  return shortcut;
}

// Parse shortcut for event matching
export function matchesShortcut(e: KeyboardEvent, shortcut: string): boolean {
  const parts = shortcut.toLowerCase().split('+');
  const key = parts[parts.length - 1];
  const hasCtrl = parts.includes('ctrl');
  const hasAlt = parts.includes('alt');
  const hasShift = parts.includes('shift');
  
  const ctrlOrMeta = e.ctrlKey || e.metaKey;
  
  return (
    ctrlOrMeta === hasCtrl &&
    e.altKey === hasAlt &&
    e.shiftKey === hasShift &&
    e.key.toLowerCase() === key
  );
}

