import { create } from 'zustand';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';
export type LogSubsystem = 'ssh' | 'config' | 'plugins' | 'python' | 'keychain' | 'app' | 'unknown';

export interface LogLine {
  timestamp: number;
  level: LogLevel;
  subsystem: LogSubsystem;
  session_id?: string;
  message: string;
  details?: Record<string, unknown>;
}

export interface LogFilter {
  session_id?: string;
  level?: LogLevel;
  subsystem?: LogSubsystem;
  search?: string;
  since?: number;
}

interface LogStoreState {
  // Log entries in memory
  entries: LogLine[];
  
  // Filter state
  filter: LogFilter;
  
  // UI state
  isPaused: boolean;
  
  // Max entries to keep
  maxEntries: number;
  
  // Actions
  addEntry: (entry: LogLine) => void;
  clearEntries: () => void;
  setFilter: (filter: Partial<LogFilter>) => void;
  clearFilter: () => void;
  setPaused: (paused: boolean) => void;
  
  // Backend interaction
  loadRecentLogs: (maxLines?: number) => Promise<void>;
  exportBundle: (path: string, options?: ExportOptions) => Promise<string>;
  
  // Filtered entries getter
  getFilteredEntries: () => LogLine[];
  
  // Event listeners
  setupListeners: () => () => void;
}

export interface ExportOptions {
  max_lines?: number;
  include_config?: boolean;
  include_sessions?: boolean;
  include_plugins?: boolean;
  redact_hostnames?: boolean;
}

const MAX_ENTRIES = 5000;

export const useLogStore = create<LogStoreState>((set, get) => ({
  entries: [],
  filter: {},
  isPaused: false,
  maxEntries: MAX_ENTRIES,

  addEntry: (entry) => {
    if (get().isPaused) return;
    
    set((state) => {
      const newEntries = [...state.entries, entry];
      // Trim if over limit
      if (newEntries.length > state.maxEntries) {
        return { entries: newEntries.slice(-state.maxEntries) };
      }
      return { entries: newEntries };
    });
  },

  clearEntries: () => {
    set({ entries: [] });
  },

  setFilter: (filter) => {
    set((state) => ({
      filter: { ...state.filter, ...filter },
    }));
  },

  clearFilter: () => {
    set({ filter: {} });
  },

  setPaused: (paused) => {
    set({ isPaused: paused });
  },

  loadRecentLogs: async (maxLines = 1000) => {
    try {
      const logs = await invoke<LogLine[]>('get_recent_logs', {
        maxLines,
        filter: null,
      });
      set({ entries: logs });
    } catch (error) {
      console.error('Failed to load recent logs:', error);
    }
  },

  exportBundle: async (path, options) => {
    const result = await invoke<string>('export_debug_bundle', {
      path,
      options: options || null,
    });
    return result;
  },

  getFilteredEntries: () => {
    const { entries, filter } = get();
    
    return entries.filter((entry) => {
      // Filter by session_id
      if (filter.session_id && entry.session_id !== filter.session_id) {
        return false;
      }
      
      // Filter by level
      if (filter.level && entry.level !== filter.level) {
        return false;
      }
      
      // Filter by subsystem
      if (filter.subsystem && entry.subsystem !== filter.subsystem) {
        return false;
      }
      
      // Filter by search term
      if (filter.search) {
        const searchLower = filter.search.toLowerCase();
        if (!entry.message.toLowerCase().includes(searchLower)) {
          return false;
        }
      }
      
      // Filter by timestamp
      if (filter.since && entry.timestamp < filter.since) {
        return false;
      }
      
      return true;
    });
  },

  setupListeners: () => {
    const unlisteners: UnlistenFn[] = [];

    // Listen for ssh:debug events
    listen<{
      session_id: string;
      stage: string;
      details: Record<string, unknown>;
    }>('ssh:debug', (event) => {
      const { session_id, stage, details } = event.payload;
      get().addEntry({
        timestamp: Date.now(),
        level: 'debug',
        subsystem: 'ssh',
        session_id,
        message: `[${stage}] ${JSON.stringify(details)}`,
        details,
      });
    }).then((fn) => unlisteners.push(fn));

    // Listen for ssh:error events
    listen<{
      session_id: string;
      message: string;
    }>('ssh:error', (event) => {
      get().addEntry({
        timestamp: Date.now(),
        level: 'error',
        subsystem: 'ssh',
        session_id: event.payload.session_id,
        message: event.payload.message,
      });
    }).then((fn) => unlisteners.push(fn));

    // Listen for ssh:closed events
    listen<{
      session_id: string;
      reason?: string;
    }>('ssh:closed', (event) => {
      get().addEntry({
        timestamp: Date.now(),
        level: 'info',
        subsystem: 'ssh',
        session_id: event.payload.session_id,
        message: `Session closed: ${event.payload.reason || 'unknown reason'}`,
      });
    }).then((fn) => unlisteners.push(fn));

    // Listen for ssh:connected events
    listen<{
      id: string;
      host: string;
      username: string;
    }>('ssh:connected', (event) => {
      get().addEntry({
        timestamp: Date.now(),
        level: 'info',
        subsystem: 'ssh',
        session_id: event.payload.id,
        message: `Connected to ${event.payload.username}@${event.payload.host}`,
      });
    }).then((fn) => unlisteners.push(fn));

    // Listen for ssh:sessions state updates
    listen<{
      id: string;
      state: string;
    }>('ssh:sessions', (event) => {
      get().addEntry({
        timestamp: Date.now(),
        level: 'debug',
        subsystem: 'ssh',
        session_id: event.payload.id,
        message: `Session state: ${event.payload.state}`,
      });
    }).then((fn) => unlisteners.push(fn));

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  },
}));




