import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export interface SessionConfig {
  host: string;
  port: number;
  username: string;
  auth_method: AuthMethod;
  jump_hosts?: JumpHost[];
  keepalive_interval?: number;
  agent_forwarding?: boolean;
  profile_id?: string;
}

export type AuthMethod =
  | { type: 'password' }
  | { type: 'key' }
  | { type: 'agent' }
  | { type: 'interactive' };

export interface JumpHost {
  host: string;
  port: number;
  username: string;
  auth_method: AuthMethod;
}

export type SessionState = 
  | 'Created' 
  | 'Connecting' 
  | 'WaitingForHostKey' 
  | 'Connected' 
  | 'Disconnected' 
  | 'Error';

export interface Session {
  id: string;
  host: string;
  port: number;
  username: string;
  state: SessionState;
  profile_id?: string;
  connected_at?: number;
  disconnected_at?: number;
  disconnect_reason?: string;
  error_message?: string;
  reconnect_attempts?: number;
}

export interface Profile {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  auth_method: AuthMethod;
  jump_hosts: JumpHost[];
  options: ProfileOptions;
  theme?: string;
  tags: string[];
  notes: string;
  created_at: number;
  updated_at: number;
}

export interface ProfileOptions {
  keepalive_interval: number;
  agent_forwarding: boolean;
  startup_commands: string[];
  environment: Record<string, string>;
}

interface SshDataEvent {
  session_id: string;
  data: number[];
}

interface SshClosedEvent {
  session_id: string;
  reason: string;
}

interface SshErrorEvent {
  session_id: string;
  message: string;
}

interface SessionStoreState {
  sessions: Map<string, Session>;
  profiles: Profile[];
  activeSessionId: string | null;
  
  // Event handlers - set by terminal components
  dataHandlers: Map<string, (data: Uint8Array) => void>;
  
  // Reconnection callbacks - terminal component can register to be notified
  reconnectCallbacks: Map<string, () => void>;
  
  // Track if listeners are already set up (singleton pattern)
  _listenersInitialized: boolean;
  
  // Session actions
  setActiveSession: (sessionId: string | null) => void;
  registerDataHandler: (sessionId: string, handler: (data: Uint8Array) => void) => void;
  unregisterDataHandler: (sessionId: string) => void;
  registerReconnectCallback: (sessionId: string, callback: () => void) => void;
  unregisterReconnectCallback: (sessionId: string) => void;
  
  // SSH actions
  disconnect: (sessionId: string) => Promise<void>;
  sendData: (sessionId: string, data: Uint8Array) => Promise<void>;
  resizePty: (sessionId: string, cols: number, rows: number) => Promise<void>;
  reconnect: (sessionId: string) => Promise<string | null>;
  
  // Profile actions
  loadProfiles: () => Promise<void>;
  
  // Internal state updates
  updateSession: (session: Session) => void;
  removeSession: (sessionId: string) => void;
  
  // Event listeners
  setupListeners: () => () => void;
}

export const useSessionStore = create<SessionStoreState>((set, get) => ({
  sessions: new Map(),
  profiles: [],
  activeSessionId: null,
  dataHandlers: new Map(),
  reconnectCallbacks: new Map(),
  _listenersInitialized: false,

  setActiveSession: (sessionId) => {
    set({ activeSessionId: sessionId });
  },

  registerDataHandler: (sessionId, handler) => {
    set((state) => {
      const handlers = new Map(state.dataHandlers);
      handlers.set(sessionId, handler);
      return { dataHandlers: handlers };
    });
  },

  unregisterDataHandler: (sessionId) => {
    set((state) => {
      const handlers = new Map(state.dataHandlers);
      handlers.delete(sessionId);
      return { dataHandlers: handlers };
    });
  },

  registerReconnectCallback: (sessionId, callback) => {
    set((state) => {
      const callbacks = new Map(state.reconnectCallbacks);
      callbacks.set(sessionId, callback);
      return { reconnectCallbacks: callbacks };
    });
  },

  unregisterReconnectCallback: (sessionId) => {
    set((state) => {
      const callbacks = new Map(state.reconnectCallbacks);
      callbacks.delete(sessionId);
      return { reconnectCallbacks: callbacks };
    });
  },

  disconnect: async (sessionId) => {
    try {
      await invoke('ssh_disconnect', { sessionId });
    } catch (error) {
      console.error('Failed to disconnect:', error);
    }
  },

  sendData: async (sessionId, data) => {
    try {
      // Send as array of bytes
      await invoke('send_data', { sessionId, data: Array.from(data) });
    } catch (error) {
      console.error('Failed to send data:', error);
    }
  },

  resizePty: async (sessionId, cols, rows) => {
    try {
      await invoke('ssh_resize', { sessionId, cols, rows });
    } catch (error) {
      console.error('Failed to resize PTY:', error);
    }
  },

  reconnect: async (sessionId) => {
    const session = get().sessions.get(sessionId);
    if (!session) {
      console.error('[Reconnect] Session not found:', sessionId);
      return null;
    }

    // Must have a profile_id to reconnect
    if (!session.profile_id) {
      console.error('[Reconnect] No profile_id for session:', sessionId);
      return null;
    }

    // Update session state to Connecting
    get().updateSession({
      ...session,
      state: 'Connecting',
      reconnect_attempts: (session.reconnect_attempts || 0) + 1,
    });

    try {
      console.log('[Reconnect] Attempting reconnection for profile:', session.profile_id);
      
      // Use connect_profile to reconnect using saved credentials
      const result = await invoke<{ session_id: string; profile_id?: string }>('connect_profile', {
        profileId: session.profile_id,
      });

      console.log('[Reconnect] Success, new session:', result.session_id);
      
      // Notify the terminal to switch to the new session
      const callback = get().reconnectCallbacks.get(sessionId);
      if (callback) {
        callback();
      }

      return result.session_id;
    } catch (error) {
      console.error('[Reconnect] Failed:', error);
      
      // Update session state to reflect failure
      get().updateSession({
        ...session,
        state: 'Disconnected',
        disconnect_reason: `Reconnection failed: ${error}`,
      });
      
      return null;
    }
  },

  loadProfiles: async () => {
    try {
      const profiles = await invoke<Profile[]>('list_profiles');
      set({ profiles });
    } catch (error) {
      console.error('Failed to load profiles:', error);
    }
  },

  updateSession: (session) => {
    set((state) => {
      const sessions = new Map(state.sessions);
      sessions.set(session.id, session);
      return { sessions };
    });
  },

  removeSession: (sessionId) => {
    set((state) => {
      const sessions = new Map(state.sessions);
      sessions.delete(sessionId);
      const handlers = new Map(state.dataHandlers);
      handlers.delete(sessionId);
      return { 
        sessions,
        dataHandlers: handlers,
        activeSessionId: state.activeSessionId === sessionId ? null : state.activeSessionId,
      };
    });
  },

  setupListeners: () => {
    // CRITICAL: Use synchronous check to prevent duplicate listeners
    // This is the root cause of double input/output - multiple ssh:data listeners
    if (get()._listenersInitialized) {
      console.log('[SSH] Listeners already initialized, skipping duplicate registration');
      return () => {}; // Return no-op cleanup
    }
    
    // Mark as initialized IMMEDIATELY (synchronously) before any async operations
    set({ _listenersInitialized: true });
    console.log('[SSH] Setting up event listeners (first time only)');

    const unlisteners: UnlistenFn[] = [];

    // Listen for session state changes
    listen<Session>('ssh:sessions', (event) => {
      console.log('[SSH] Session update:', event.payload);
      get().updateSession(event.payload);
    }).then((fn) => unlisteners.push(fn));

    // Listen for connection events
    listen<Session>('ssh:connected', (event) => {
      console.log('[SSH] Connected:', event.payload);
      get().updateSession(event.payload);
    }).then((fn) => unlisteners.push(fn));

    listen<Session>('ssh:disconnected', (event) => {
      console.log('[SSH] Disconnected:', event.payload);
      get().updateSession(event.payload);
    }).then((fn) => unlisteners.push(fn));

    // Listen for SSH data - route to appropriate handler
    // This listener must only exist ONCE or data will be duplicated
    listen<SshDataEvent>('ssh:data', (event) => {
      const { session_id, data } = event.payload;
      const handler = get().dataHandlers.get(session_id);
      if (handler) {
        // Convert number array to Uint8Array
        handler(new Uint8Array(data));
      }
    }).then((fn) => unlisteners.push(fn));

    // Listen for connection closed
    listen<SshClosedEvent>('ssh:closed', (event) => {
      console.log('[SSH] Closed:', event.payload);
      // Update session state with disconnect info
      const sessions = get().sessions;
      const session = sessions.get(event.payload.session_id);
      if (session) {
        get().updateSession({ 
          ...session, 
          state: 'Disconnected',
          disconnected_at: Date.now(),
          disconnect_reason: event.payload.reason || 'Connection closed',
        });
      }
    }).then((fn) => unlisteners.push(fn));

    // Listen for errors
    listen<SshErrorEvent>('ssh:error', (event) => {
      console.error('[SSH] Error:', event.payload);
      // Could show toast notification here
      const sessions = get().sessions;
      const session = sessions.get(event.payload.session_id);
      if (session) {
        get().updateSession({ 
          ...session, 
          state: 'Error',
          error_message: event.payload.message,
        });
      }
    }).then((fn) => unlisteners.push(fn));

    // Cleanup should NOT reset _listenersInitialized in normal operation
    // The listeners should persist for the lifetime of the app
    // Only in dev with HMR would we want to reset, but that causes the race condition
    const cleanup = () => {
      console.log('[SSH] Cleaning up listeners (app shutdown)');
      unlisteners.forEach((fn) => fn());
      // Don't reset _listenersInitialized - prevents race conditions during HMR
    };

    return cleanup;
  },
}));
