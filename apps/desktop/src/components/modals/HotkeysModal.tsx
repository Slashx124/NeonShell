import { X, Keyboard } from 'lucide-react';
import { useUIStore } from '@/stores/uiStore';
import { formatShortcut } from '@/lib/commandRegistry';

interface Hotkey {
  id: string;
  name: string;
  shortcut: string;
  category: string;
}

const HOTKEYS: Hotkey[] = [
  // General
  { id: 'command-palette', name: 'Open Command Palette', shortcut: 'Ctrl+K', category: 'General' },
  { id: 'new-connection', name: 'New Connection', shortcut: 'Ctrl+N', category: 'General' },
  { id: 'settings', name: 'Open Settings', shortcut: 'Ctrl+,', category: 'General' },
  { id: 'toggle-sidebar', name: 'Toggle Sidebar', shortcut: 'Ctrl+B', category: 'General' },
  { id: 'debug-console', name: 'Debug Console', shortcut: 'Ctrl+`', category: 'General' },
  
  // Tabs
  { id: 'new-tab', name: 'New Tab', shortcut: 'Ctrl+T', category: 'Tabs' },
  { id: 'close-tab', name: 'Close Tab', shortcut: 'Ctrl+W', category: 'Tabs' },
  { id: 'next-tab', name: 'Next Tab', shortcut: 'Ctrl+Tab', category: 'Tabs' },
  { id: 'prev-tab', name: 'Previous Tab', shortcut: 'Ctrl+Shift+Tab', category: 'Tabs' },
  
  // Terminal
  { id: 'copy', name: 'Copy', shortcut: 'Ctrl+Shift+C', category: 'Terminal' },
  { id: 'paste', name: 'Paste', shortcut: 'Ctrl+Shift+V', category: 'Terminal' },
  { id: 'clear', name: 'Clear Terminal', shortcut: 'Ctrl+L', category: 'Terminal' },
  { id: 'search', name: 'Search in Terminal', shortcut: 'Ctrl+Shift+F', category: 'Terminal' },
  
  // Session
  { id: 'disconnect', name: 'Disconnect', shortcut: 'Ctrl+D', category: 'Session' },
  { id: 'reconnect', name: 'Reconnect', shortcut: 'Ctrl+R', category: 'Session' },
];

export function HotkeysModal() {
  const { closeModal } = useUIStore();

  // Group hotkeys by category
  const categories = HOTKEYS.reduce((acc, hotkey) => {
    if (!acc[hotkey.category]) {
      acc[hotkey.category] = [];
    }
    acc[hotkey.category].push(hotkey);
    return acc;
  }, {} as Record<string, Hotkey[]>);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-lg bg-surface-1 rounded-xl border border-border shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <div className="flex items-center gap-3">
            <Keyboard className="w-5 h-5 text-accent" />
            <h2 className="text-lg font-semibold text-foreground">Keyboard Shortcuts</h2>
          </div>
          <button
            onClick={closeModal}
            className="p-1 rounded hover:bg-surface-2 text-foreground-muted hover:text-foreground transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 max-h-[60vh] overflow-y-auto">
          <div className="space-y-6">
            {Object.entries(categories).map(([category, hotkeys]) => (
              <div key={category}>
                <h3 className="text-xs font-semibold text-foreground-muted uppercase tracking-wider mb-2">
                  {category}
                </h3>
                <div className="space-y-1">
                  {hotkeys.map((hotkey) => (
                    <div
                      key={hotkey.id}
                      className="flex items-center justify-between py-2 px-3 rounded hover:bg-surface-2"
                    >
                      <span className="text-sm text-foreground">{hotkey.name}</span>
                      <kbd className="px-2 py-1 text-xs bg-surface-3 rounded text-foreground-muted font-mono">
                        {formatShortcut(hotkey.shortcut)}
                      </kbd>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-border text-xs text-foreground-muted">
          Keyboard shortcuts can be customized in Settings → Keyboard.
        </div>
      </div>
    </div>
  );
}

