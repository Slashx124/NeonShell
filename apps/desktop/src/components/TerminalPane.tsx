import { useEffect, useRef, useCallback, useState } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';
import { SerializeAddon } from '@xterm/addon-serialize';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '@/stores/appStore';
import { useSessionStore } from '@/stores/sessionStore';
import { useThemeStore } from '@/stores/themeStore';
import { useUIStore } from '@/stores/uiStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { WifiOff, RefreshCw, Loader2 } from 'lucide-react';
import '@xterm/xterm/css/xterm.css';

// Global registry to track active terminals per session (prevents duplicates during HMR)
// Key: sessionId, Value: { terminal, dispose function }
const activeTerminals = new Map<string, { 
  terminal: Terminal; 
  dispose: () => void;
  initialized: boolean;
}>();

// Track which session IDs have had their onData handler set up
// This prevents duplicate keystroke handlers
const initializedSessions = new Set<string>();

export function TerminalPane() {
  const { tabs, activeTabId } = useAppStore();
  const { sessions } = useSessionStore();
  const activeTab = tabs.find((t) => t.id === activeTabId);

  if (!activeTab) {
    return <EmptyState />;
  }

  // Check session state
  const session = activeTab.sessionId ? sessions.get(activeTab.sessionId) : null;
  const isConnecting = session?.state === 'Connecting' || session?.state === 'WaitingForHostKey';
  const isDisconnected = session?.state === 'Disconnected';
  const isError = session?.state === 'Error';

  if (!activeTab.sessionId) {
    return <ConnectingState tab={activeTab} status="Initializing..." />;
  }

  // For initial connection (before we have any session data)
  if (isConnecting && !session?.connected_at) {
    return <ConnectingState tab={activeTab} status={session?.state === 'WaitingForHostKey' ? 'Waiting for host key verification...' : 'Connecting...'} />;
  }

  // For errors during initial connection (never connected)
  if (isError && !session?.connected_at) {
    return <ErrorState tab={activeTab} errorMessage={session?.error_message} />;
  }

  // Show terminal with overlay for disconnected/reconnecting states
  // This preserves the terminal content while showing the reconnect UI
  return (
    <div className="h-full w-full bg-surface-0 relative">
      <TerminalView 
        key={activeTab.sessionId}
        sessionId={activeTab.sessionId} 
        isDisconnected={isDisconnected || isError}
        isReconnecting={isConnecting && !!session?.connected_at}
      />
    </div>
  );
}

function EmptyState() {
  const { setShowConnectionDialog } = useAppStore();

  return (
    <div className="h-full w-full flex flex-col items-center justify-center bg-surface-0 text-foreground-muted">
      <div className="text-center max-w-md">
        {/* Neon logo effect */}
        <div className="mb-8 relative">
          <h1 className="text-6xl font-bold font-mono neon-text bg-gradient-to-r from-neon-pink via-neon-cyan to-neon-purple bg-clip-text text-transparent animate-pulse-neon">
            N$
          </h1>
          <div className="absolute inset-0 blur-xl bg-gradient-to-r from-neon-pink via-neon-cyan to-neon-purple opacity-30" />
        </div>
        
        <h2 className="text-2xl font-bold mb-2 text-foreground">Welcome to NeonShell</h2>
        <p className="text-sm mb-6">
          The SSH terminal for power users who want extreme theming and automation
        </p>
        
        <button
          onClick={() => setShowConnectionDialog(true)}
          className="btn btn-primary text-lg px-8 py-3 animate-glow"
        >
          New Connection
        </button>
        
        <div className="mt-8 text-xs space-y-1">
          <p>
            <kbd className="px-2 py-1 bg-surface-2 rounded text-foreground-muted">Ctrl+K</kbd>
            {' '}Command Palette
          </p>
          <p>
            <kbd className="px-2 py-1 bg-surface-2 rounded text-foreground-muted">Ctrl+Shift+N</kbd>
            {' '}New Connection
          </p>
        </div>
      </div>
    </div>
  );
}

function ConnectingState({ tab, status }: { tab: { title: string }; status: string }) {
  return (
    <div className="h-full w-full flex flex-col items-center justify-center bg-surface-0">
      <div className="animate-pulse">
        <div className="w-12 h-12 border-4 border-accent border-t-transparent rounded-full animate-spin" />
      </div>
      <p className="mt-4 text-foreground-muted">Connecting to {tab.title}...</p>
      <p className="mt-2 text-sm text-foreground-muted/70">{status}</p>
    </div>
  );
}

function ErrorState({ tab, errorMessage }: { tab: { title: string }; errorMessage?: string }) {
  const { setShowConnectionDialog } = useAppStore();

  return (
    <div className="h-full w-full flex flex-col items-center justify-center bg-surface-0">
      <div className="text-center max-w-md">
        <div className="text-6xl mb-4">⚠️</div>
        <h2 className="text-xl font-bold mb-2 text-error">Connection Failed</h2>
        <p className="text-sm text-foreground-muted mb-2">
          Failed to connect to {tab.title}
        </p>
        {errorMessage && (
          <p className="text-xs text-foreground-muted/80 mb-4 px-4 py-2 bg-surface-1 rounded-lg border border-border">
            {errorMessage}
          </p>
        )}
        <button
          onClick={() => setShowConnectionDialog(true)}
          className="btn btn-primary"
        >
          Try Again
        </button>
      </div>
    </div>
  );
}

function TerminalView({ 
  sessionId, 
  isDisconnected = false,
  isReconnecting = false,
}: { 
  sessionId: string;
  isDisconnected?: boolean;
  isReconnecting?: boolean;
}) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const serializeAddonRef = useRef<SerializeAddon | null>(null);
  const initIdRef = useRef<string | null>(null);
  const [reconnectPending, setReconnectPending] = useState(false);
  const { 
    sendData, 
    resizePty, 
    registerDataHandler, 
    unregisterDataHandler, 
    sessions,
    reconnect,
  } = useSessionStore();
  const { tabs, updateTab } = useAppStore();
  const { currentTheme } = useThemeStore();
  const { addToast } = useUIStore();
  const { settings } = useSettingsStore();
  
  // Get profile ID from the tab or session
  const tab = tabs.find(t => t.sessionId === sessionId);
  const session = sessions.get(sessionId);
  const profileId = tab?.profileId || session?.profile_id;
  
  // Get terminal settings with defaults
  const terminalSettings = settings?.terminal ?? {
    font_family: 'JetBrains Mono',
    font_size: 14,
    cursor_style: 'block' as const,
    cursor_blink: true,
    scrollback: 10000,
    copy_on_select: true,
    bell_sound: false,
    bell_visual: false,
  };

  // Handle reconnection
  const handleReconnect = useCallback(async () => {
    if (reconnectPending || isReconnecting) return;
    
    if (!profileId) {
      addToast({
        type: 'error',
        title: 'Cannot reconnect',
        message: 'This session was not saved. Please create a new connection.',
      });
      return;
    }

    setReconnectPending(true);
    
    // Write reconnecting message to terminal
    if (xtermRef.current) {
      xtermRef.current.write('\r\n\x1b[33m⟳ Reconnecting...\x1b[0m\r\n');
    }

    try {
      const newSessionId = await reconnect(sessionId);
      
      if (newSessionId && tab) {
        // Update the tab to use the new session
        // Note: Don't register data handler here - the useEffect will handle it
        // when the component re-renders with the new sessionId
        updateTab(tab.id, { sessionId: newSessionId });
        
        addToast({
          type: 'success',
          title: 'Reconnected',
          message: `Connected to ${session?.host}`,
        });
        
        // Write success message
        if (xtermRef.current) {
          xtermRef.current.write('\r\n\x1b[32m✓ Reconnected successfully\x1b[0m\r\n\r\n');
        }
      }
    } catch (error) {
      console.error('Reconnection failed:', error);
      addToast({
        type: 'error',
        title: 'Reconnection failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
      
      if (xtermRef.current) {
        xtermRef.current.write(`\r\n\x1b[31m✗ Reconnection failed: ${error}\x1b[0m\r\n`);
      }
    } finally {
      setReconnectPending(false);
    }
  }, [sessionId, profileId, reconnect, tab, updateTab, addToast, session, reconnectPending, isReconnecting]);

  // Data handler callback
  const handleData = useCallback((data: Uint8Array) => {
    if (xtermRef.current) {
      xtermRef.current.write(data);
    }
  }, []);

  // Save history when component unmounts or session disconnects
  const saveHistory = useCallback(async () => {
    if (!serializeAddonRef.current || !profileId) return;
    
    try {
      const serialized = serializeAddonRef.current.serialize();
      const encoder = new TextEncoder();
      const data = encoder.encode(serialized);
      
      await invoke('save_terminal_history', {
        profileId,
        data: Array.from(data),
      });
      console.log('[Terminal] Saved history for profile:', profileId);
    } catch (error) {
      console.error('[Terminal] Failed to save history:', error);
    }
  }, [profileId]);

  // Load history when terminal initializes
  const loadHistory = useCallback(async (terminal: Terminal) => {
    if (!profileId) return;
    
    try {
      const data = await invoke<number[] | null>('load_terminal_history', {
        profileId,
      });
      
      if (data && data.length > 0) {
        const decoder = new TextDecoder();
        const content = decoder.decode(new Uint8Array(data));
        terminal.write(content);
        terminal.write('\r\n\x1b[90m--- Previous session history ---\x1b[0m\r\n');
        console.log('[Terminal] Loaded history for profile:', profileId);
      }
    } catch (error) {
      console.error('[Terminal] Failed to load history:', error);
    }
  }, [profileId]);

  // Initialize terminal
  useEffect(() => {
    if (!terminalRef.current) return;
    
    // CRITICAL: Check if this session already has an active terminal
    // This prevents duplicate terminals and duplicate keystroke handlers
    const existing = activeTerminals.get(sessionId);
    if (existing && existing.initialized) {
      console.log('[Terminal] Session already has active terminal, reusing:', sessionId);
      xtermRef.current = existing.terminal;
      // Re-attach to DOM if needed
      if (terminalRef.current && !terminalRef.current.querySelector('.xterm')) {
        existing.terminal.open(terminalRef.current);
      }
      return; // Don't initialize again
    }
    
    // If there's an existing but not fully initialized, clean it up
    if (existing) {
      console.log('[Terminal] Cleaning up partially initialized terminal for session:', sessionId);
      existing.dispose();
      activeTerminals.delete(sessionId);
    }
    
    // Also clean up local ref
    if (xtermRef.current) {
      console.log('[Terminal] Cleaning up local terminal ref');
      xtermRef.current.dispose();
      xtermRef.current = null;
      fitAddonRef.current = null;
      serializeAddonRef.current = null;
    }
    
    // Clean up from previous sessions for this ID
    initializedSessions.delete(sessionId);
    
    // Generate unique init ID for this effect run
    const initId = `${sessionId}-${Date.now()}`;
    
    // Mark this initialization
    initIdRef.current = initId;

    console.log('[Terminal] Initializing NEW terminal for session:', sessionId, 'initId:', initId);

    const themeTerminal = currentTheme?.terminal;
    const colors = currentTheme?.colors;

    // Map cursor style from settings to xterm format
    const cursorStyleMap: Record<string, 'block' | 'underline' | 'bar'> = {
      'block': 'block',
      'underline': 'underline', 
      'bar': 'bar',
    };

    const terminal = new Terminal({
      // Use settings, falling back to theme, then defaults
      fontFamily: terminalSettings.font_family || themeTerminal?.font_family || 'JetBrains Mono, Consolas, monospace',
      fontSize: terminalSettings.font_size || themeTerminal?.font_size || 14,
      cursorBlink: terminalSettings.cursor_blink,
      cursorStyle: cursorStyleMap[terminalSettings.cursor_style] || 'block',
      allowTransparency: true,
      scrollback: terminalSettings.scrollback || 10000,
      // Note: xterm.js v5 removed bellSound/bellStyle - bell is controlled via CSS/theme
      theme: {
        background: colors?.surface_0 || '#0a0a0f',
        foreground: colors?.foreground || '#e0e0e0',
        cursor: colors?.accent || '#ff0080',
        selectionBackground: colors?.selection || '#ff008044',
        black: themeTerminal?.ansi_colors?.black || '#0a0a0f',
        red: themeTerminal?.ansi_colors?.red || '#ff0055',
        green: themeTerminal?.ansi_colors?.green || '#00ff9f',
        yellow: themeTerminal?.ansi_colors?.yellow || '#ffff00',
        blue: themeTerminal?.ansi_colors?.blue || '#00aaff',
        magenta: themeTerminal?.ansi_colors?.magenta || '#ff00ff',
        cyan: themeTerminal?.ansi_colors?.cyan || '#00ffff',
        white: themeTerminal?.ansi_colors?.white || '#ffffff',
        brightBlack: themeTerminal?.ansi_colors?.bright_black || '#333344',
        brightRed: themeTerminal?.ansi_colors?.bright_red || '#ff5588',
        brightGreen: themeTerminal?.ansi_colors?.bright_green || '#55ffbb',
        brightYellow: themeTerminal?.ansi_colors?.bright_yellow || '#ffff55',
        brightBlue: themeTerminal?.ansi_colors?.bright_blue || '#55bbff',
        brightMagenta: themeTerminal?.ansi_colors?.bright_magenta || '#ff55ff',
        brightCyan: themeTerminal?.ansi_colors?.bright_cyan || '#55ffff',
        brightWhite: themeTerminal?.ansi_colors?.bright_white || '#ffffff',
      },
    });

    // Add addons
    const fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon();
    const searchAddon = new SearchAddon();
    const serializeAddon = new SerializeAddon();

    terminal.loadAddon(fitAddon);
    terminal.loadAddon(webLinksAddon);
    terminal.loadAddon(searchAddon);
    terminal.loadAddon(serializeAddon);

    // Open terminal
    terminal.open(terminalRef.current);
    
    // Delay fit to ensure DOM is ready
    setTimeout(() => {
      fitAddon.fit();
    }, 50);

    xtermRef.current = terminal;
    fitAddonRef.current = fitAddon;
    serializeAddonRef.current = serializeAddon;

    // Load previous history if available
    loadHistory(terminal);

    // Store disposables for cleanup
    const disposables: { dispose: () => void }[] = [];

    // CRITICAL: Check if onData handler already set up for this session
    // This prevents duplicate keystroke handlers
    if (!initializedSessions.has(sessionId)) {
      initializedSessions.add(sessionId);
      
      // Handle input - send keystrokes to SSH session
      // If disconnected, trigger auto-reconnect on first keystroke
      const dataDisposable = terminal.onData((data) => {
        const currentSession = useSessionStore.getState().sessions.get(sessionId);
        
        if (currentSession?.state === 'Disconnected' || currentSession?.state === 'Error') {
          // Auto-reconnect on user input
          console.log('[Terminal] User input while disconnected, triggering reconnect');
          handleReconnect();
          return; // Don't send data to dead session
        }
        
        const encoder = new TextEncoder();
        sendData(sessionId, encoder.encode(data));
      });
      disposables.push(dataDisposable);
      console.log('[Terminal] Registered onData handler for session:', sessionId);
    } else {
      console.log('[Terminal] onData handler already exists for session:', sessionId);
    }

    // Handle resize (these are safe to register multiple times as they just update state)
    const resizeDisposable = terminal.onResize(({ cols, rows }) => {
      resizePty(sessionId, cols, rows);
    });
    disposables.push(resizeDisposable);

    // Handle copy-on-select
    const selectionDisposable = terminal.onSelectionChange(() => {
      const currentSettings = useSettingsStore.getState().settings;
      if (currentSettings?.terminal.copy_on_select) {
        const selection = terminal.getSelection();
        if (selection && selection.length > 0) {
          navigator.clipboard.writeText(selection).catch((err) => {
            console.error('[Terminal] Failed to copy selection:', err);
          });
        }
      }
    });
    disposables.push(selectionDisposable);

    // Initial resize notification (after a short delay for fit)
    setTimeout(() => {
      resizePty(sessionId, terminal.cols, terminal.rows);
    }, 100);

    // Register data handler with the store
    registerDataHandler(sessionId, handleData);

    // Create cleanup function
    const cleanup = () => {
      console.log('[Terminal] Running cleanup for initId:', initId);
      
      // Save history before cleanup
      saveHistory();
      
      // Dispose all event handlers first
      disposables.forEach(d => d.dispose());
      
      // Cleanup
      unregisterDataHandler(sessionId);
      terminal.dispose();
      xtermRef.current = null;
      fitAddonRef.current = null;
      serializeAddonRef.current = null;
      initIdRef.current = null;
      activeTerminals.delete(sessionId);
      initializedSessions.delete(sessionId);
    };

    // Register in global registry for HMR cleanup - mark as fully initialized
    activeTerminals.set(sessionId, { terminal, dispose: cleanup, initialized: true });

    return () => {
      console.log('[Terminal] Effect cleanup for initId:', initId);
      
      // Only cleanup if this is still the active initialization
      if (initIdRef.current === initId) {
        cleanup();
      }
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId, currentTheme]);
  // Note: We intentionally limit dependencies to prevent re-registering handlers
  // The callbacks use refs and store.getState() to access current values

  // Handle window/panel resize
  useEffect(() => {
    const handleResize = () => {
      if (fitAddonRef.current) {
        fitAddonRef.current.fit();
      }
    };

    window.addEventListener('resize', handleResize);
    
    // Also handle panel resizes
    const observer = new ResizeObserver(handleResize);
    if (terminalRef.current) {
      observer.observe(terminalRef.current);
    }

    return () => {
      window.removeEventListener('resize', handleResize);
      observer.disconnect();
    };
  }, []);

  const showOverlay = isDisconnected || isReconnecting || reconnectPending;

  return (
    <div className="h-full w-full relative">
      {/* Terminal container */}
      <div 
        ref={terminalRef} 
        className={`h-full w-full ${showOverlay ? 'opacity-50' : ''}`}
        style={{ padding: '8px', backgroundColor: 'var(--surface-0, #0a0a0f)' }}
      />
      
      {/* Disconnection overlay */}
      {showOverlay && (
        <DisconnectedOverlay
          session={session}
          isReconnecting={isReconnecting || reconnectPending}
          canReconnect={!!profileId}
          onReconnect={handleReconnect}
        />
      )}
    </div>
  );
}

function DisconnectedOverlay({
  session,
  isReconnecting,
  canReconnect,
  onReconnect,
}: {
  session?: { host: string; port: number; username: string; disconnect_reason?: string };
  isReconnecting: boolean;
  canReconnect: boolean;
  onReconnect: () => void;
}) {
  return (
    <div className="absolute inset-0 flex items-center justify-center bg-black/40 backdrop-blur-sm">
      <div className="bg-surface-1 rounded-xl border border-border p-6 shadow-2xl max-w-md text-center">
        {isReconnecting ? (
          <>
            <Loader2 className="w-12 h-12 text-accent mx-auto mb-4 animate-spin" />
            <h3 className="text-lg font-semibold text-foreground mb-2">Reconnecting...</h3>
            <p className="text-sm text-foreground-muted">
              Reconnecting to {session?.username}@{session?.host}:{session?.port}
            </p>
          </>
        ) : (
          <>
            <WifiOff className="w-12 h-12 text-warning mx-auto mb-4" />
            <h3 className="text-lg font-semibold text-foreground mb-2">Connection Lost</h3>
            <p className="text-sm text-foreground-muted mb-1">
              Disconnected from {session?.username}@{session?.host}:{session?.port}
            </p>
            {session?.disconnect_reason && (
              <p className="text-xs text-foreground-muted/70 mb-4">
                {session.disconnect_reason}
              </p>
            )}
            
            {canReconnect ? (
              <div className="space-y-3 mt-4">
                <button
                  onClick={onReconnect}
                  className="btn btn-primary w-full flex items-center justify-center gap-2"
                >
                  <RefreshCw className="w-4 h-4" />
                  Reconnect
                </button>
                <p className="text-xs text-foreground-muted">
                  Or start typing to auto-reconnect
                </p>
              </div>
            ) : (
              <div className="mt-4">
                <p className="text-xs text-foreground-muted mb-3">
                  This session wasn't saved. Please create a new connection to reconnect.
                </p>
                <button
                  onClick={() => useAppStore.getState().setShowConnectionDialog(true)}
                  className="btn btn-primary"
                >
                  New Connection
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
