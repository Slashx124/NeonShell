import { useEffect, useCallback } from 'react';
import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';
import { Sidebar } from '@/components/Sidebar';
import { TabBar } from '@/components/TabBar';
import { TerminalPane } from '@/components/TerminalPane';
import { StatusBar } from '@/components/StatusBar';
import { CommandPalette } from '@/components/CommandPalette';
import { ConnectionDialog } from '@/components/ConnectionDialog';
import { HostKeyModal } from '@/components/HostKeyModal';
import { ModalHost } from '@/components/ModalHost';
import { ToastContainer } from '@/components/ToastContainer';
import { useAppStore } from '@/stores/appStore';
import { useUIStore } from '@/stores/uiStore';
import { useThemeStore } from '@/stores/themeStore';
import { useSessionStore } from '@/stores/sessionStore';
import { useLogStore } from '@/stores/logStore';
import { getCommands, matchesShortcut, type CommandContext } from '@/lib/commandRegistry';

export function App() {
  const { 
    showSidebar, 
    showCommandPalette,
    showConnectionDialog,
    setShowCommandPalette,
    setShowConnectionDialog,
    activeTabId,
    tabs,
    toggleSidebar
  } = useAppStore();
  const { openModal, closeModal, addToast, activeModal } = useUIStore();
  const { loadTheme, listThemes } = useThemeStore();
  const { disconnect, loadProfiles, setupListeners } = useSessionStore();
  const { setupListeners: setupLogListeners } = useLogStore();

  // Build command context for shortcuts
  const getCommandContext = useCallback((): CommandContext => ({
    setShowCommandPalette,
    setShowConnectionDialog,
    activeTabId,
    tabs,
    openModal,
    closeModal,
    addToast,
    disconnect,
    listThemes,
  }), [
    setShowCommandPalette,
    setShowConnectionDialog,
    activeTabId,
    tabs,
    openModal,
    closeModal,
    addToast,
    disconnect,
    listThemes,
  ]);

  useEffect(() => {
    // Load theme and profiles on mount
    loadTheme();
    loadProfiles();

    // Setup SSH event listeners
    const cleanupSession = setupListeners();
    
    // Setup log event listeners
    const cleanupLog = setupLogListeners();

    return () => {
      cleanupSession();
      cleanupLog();
    };
  }, [loadTheme, loadProfiles, setupListeners, setupLogListeners]);

  useEffect(() => {
    // Register keyboard shortcuts
    const handleKeyDown = (e: KeyboardEvent) => {
      // Command palette: Ctrl/Cmd + K
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setShowCommandPalette(!showCommandPalette);
        return;
      }

      // Toggle sidebar: Ctrl/Cmd + B
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault();
        toggleSidebar();
        return;
      }

      // Debug Console: Ctrl/Cmd + ` (backtick)
      if ((e.ctrlKey || e.metaKey) && e.key === '`') {
        e.preventDefault();
        // Toggle debug console modal
        if (activeModal === 'debugConsole') {
          closeModal();
        } else {
          openModal('debugConsole');
        }
        return;
      }

      // Don't process shortcuts when command palette is open (it handles its own)
      if (showCommandPalette) return;

      // Check registered command shortcuts
      const commands = getCommands();
      const ctx = getCommandContext();

      for (const cmd of commands) {
        if (cmd.shortcut && matchesShortcut(e, cmd.shortcut)) {
          e.preventDefault();
          // Check if command is available
          if (!cmd.when || cmd.when(ctx)) {
            cmd.run(ctx);
          }
          return;
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [showCommandPalette, setShowCommandPalette, toggleSidebar, getCommandContext, activeModal, openModal, closeModal]);

  return (
    <div className="h-full w-full flex flex-col bg-surface-0">
      {/* Tab bar */}
      <TabBar />

      {/* Main content area */}
      <div className="flex-1 flex overflow-hidden">
        <PanelGroup direction="horizontal" autoSaveId="neonshell-layout">
          {/* Sidebar */}
          {showSidebar && (
            <>
              <Panel defaultSize={20} minSize={15} maxSize={40}>
                <Sidebar />
              </Panel>
              <PanelResizeHandle className="w-1 bg-border hover:bg-accent transition-colors" />
            </>
          )}

          {/* Terminal area */}
          <Panel defaultSize={80}>
            <TerminalPane />
          </Panel>
        </PanelGroup>
      </div>

      {/* Status bar */}
      <StatusBar />

      {/* Command Palette */}
      {showCommandPalette && <CommandPalette />}

      {/* Connection Dialog */}
      {showConnectionDialog && <ConnectionDialog />}

      {/* Host Key Verification Modal */}
      <HostKeyModal />

      {/* Modal Host - renders active modal */}
      <ModalHost />

      {/* Toast notifications */}
      <ToastContainer />
    </div>
  );
}
