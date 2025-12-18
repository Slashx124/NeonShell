import { useEffect, useRef, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';
import { SerializeAddon } from '@xterm/addon-serialize';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '@/stores/appStore';
import { useSessionStore } from '@/stores/sessionStore';
import { useThemeStore } from '@/stores/themeStore';
import '@xterm/xterm/css/xterm.css';

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
  const isError = session?.state === 'Error';

  if (!activeTab.sessionId) {
    return <ConnectingState tab={activeTab} status="Initializing..." />;
  }

  if (isConnecting) {
    return <ConnectingState tab={activeTab} status={session?.state === 'WaitingForHostKey' ? 'Waiting for host key verification...' : 'Connecting...'} />;
  }

  if (isError) {
    return <ErrorState tab={activeTab} />;
  }

  // Show terminal even if state is not Connected yet - we want to see output as it comes
  return (
    <div className="h-full w-full bg-surface-0">
      <TerminalView sessionId={activeTab.sessionId} />
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

function ErrorState({ tab }: { tab: { title: string } }) {
  const { setShowConnectionDialog } = useAppStore();

  return (
    <div className="h-full w-full flex flex-col items-center justify-center bg-surface-0">
      <div className="text-center max-w-md">
        <div className="text-6xl mb-4">⚠️</div>
        <h2 className="text-xl font-bold mb-2 text-error">Connection Failed</h2>
        <p className="text-sm text-foreground-muted mb-6">
          Failed to connect to {tab.title}
        </p>
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

function TerminalView({ sessionId }: { sessionId: string }) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const serializeAddonRef = useRef<SerializeAddon | null>(null);
  const { sendData, resizePty, registerDataHandler, unregisterDataHandler, sessions } = useSessionStore();
  const { tabs } = useAppStore();
  const { currentTheme } = useThemeStore();
  
  // Get profile ID from the tab or session
  const tab = tabs.find(t => t.sessionId === sessionId);
  const session = sessions.get(sessionId);
  const profileId = tab?.profileId || session?.profile_id;

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
    if (!terminalRef.current || xtermRef.current) return;

    const theme = currentTheme?.terminal;
    const colors = currentTheme?.colors;

    const terminal = new Terminal({
      fontFamily: theme?.font_family || 'JetBrains Mono, Consolas, monospace',
      fontSize: theme?.font_size || 14,
      cursorBlink: true,
      cursorStyle: 'block',
      allowTransparency: true,
      scrollback: 10000, // Keep 10000 lines of scrollback
      theme: {
        background: colors?.surface_0 || '#0a0a0f',
        foreground: colors?.foreground || '#e0e0e0',
        cursor: colors?.accent || '#ff0080',
        selectionBackground: colors?.selection || '#ff008044',
        black: theme?.ansi_colors?.black || '#0a0a0f',
        red: theme?.ansi_colors?.red || '#ff0055',
        green: theme?.ansi_colors?.green || '#00ff9f',
        yellow: theme?.ansi_colors?.yellow || '#ffff00',
        blue: theme?.ansi_colors?.blue || '#00aaff',
        magenta: theme?.ansi_colors?.magenta || '#ff00ff',
        cyan: theme?.ansi_colors?.cyan || '#00ffff',
        white: theme?.ansi_colors?.white || '#ffffff',
        brightBlack: theme?.ansi_colors?.bright_black || '#333344',
        brightRed: theme?.ansi_colors?.bright_red || '#ff5588',
        brightGreen: theme?.ansi_colors?.bright_green || '#55ffbb',
        brightYellow: theme?.ansi_colors?.bright_yellow || '#ffff55',
        brightBlue: theme?.ansi_colors?.bright_blue || '#55bbff',
        brightMagenta: theme?.ansi_colors?.bright_magenta || '#ff55ff',
        brightCyan: theme?.ansi_colors?.bright_cyan || '#55ffff',
        brightWhite: theme?.ansi_colors?.bright_white || '#ffffff',
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

    // Handle input - send keystrokes to SSH session
    terminal.onData((data) => {
      const encoder = new TextEncoder();
      sendData(sessionId, encoder.encode(data));
    });

    // Handle resize
    terminal.onResize(({ cols, rows }) => {
      resizePty(sessionId, cols, rows);
    });

    // Initial resize notification (after a short delay for fit)
    setTimeout(() => {
      resizePty(sessionId, terminal.cols, terminal.rows);
    }, 100);

    // Register data handler with the store
    registerDataHandler(sessionId, handleData);

    return () => {
      // Save history before cleanup
      saveHistory();
      
      // Cleanup
      unregisterDataHandler(sessionId);
      terminal.dispose();
      xtermRef.current = null;
      fitAddonRef.current = null;
      serializeAddonRef.current = null;
    };
  }, [sessionId, sendData, resizePty, currentTheme, registerDataHandler, unregisterDataHandler, handleData, loadHistory, saveHistory]);

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

  return (
    <div 
      ref={terminalRef} 
      className="h-full w-full" 
      style={{ padding: '8px', backgroundColor: 'var(--surface-0, #0a0a0f)' }}
    />
  );
}
