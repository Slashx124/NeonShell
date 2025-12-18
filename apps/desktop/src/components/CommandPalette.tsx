import { useEffect, useRef, useState, useMemo } from 'react';
import { Command } from 'cmdk';
import { Search } from 'lucide-react';
import { useAppStore } from '@/stores/appStore';
import { useUIStore } from '@/stores/uiStore';
import { useSessionStore } from '@/stores/sessionStore';
import { useThemeStore } from '@/stores/themeStore';
import { getCommands, formatShortcut, type CommandContext } from '@/lib/commandRegistry';

export function CommandPalette() {
  const { 
    setShowCommandPalette, 
    setShowConnectionDialog,
    activeTabId,
    tabs 
  } = useAppStore();
  const { openModal, closeModal, addToast } = useUIStore();
  const { disconnect } = useSessionStore();
  const { listThemes } = useThemeStore();
  
  const [search, setSearch] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  // Build command context
  const ctx: CommandContext = useMemo(() => ({
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

  // Get commands filtered by their "when" predicate
  const commands = useMemo(() => {
    const allCommands = getCommands();
    return allCommands.filter(cmd => !cmd.when || cmd.when(ctx));
  }, [ctx]);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Close on escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setShowCommandPalette(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [setShowCommandPalette]);

  const handleSelect = async (commandId: string) => {
    const command = commands.find(c => c.id === commandId);
    if (command) {
      try {
        await command.run(ctx);
      } catch (error) {
        addToast({
          type: 'error',
          title: 'Command failed',
          message: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }
  };

  return (
    <div 
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/50 backdrop-blur-sm"
      onClick={() => setShowCommandPalette(false)}
    >
      <div 
        className="w-full max-w-lg bg-surface-1 rounded-xl border border-border shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <Command className="flex flex-col" loop>
          {/* Search input */}
          <div className="flex items-center gap-3 px-4 border-b border-border">
            <Search className="w-5 h-5 text-foreground-muted" />
            <Command.Input
              ref={inputRef}
              value={search}
              onValueChange={setSearch}
              placeholder="Type a command or search..."
              className="flex-1 py-4 bg-transparent text-foreground placeholder:text-foreground-muted focus:outline-none"
            />
          </div>

          {/* Command list */}
          <Command.List className="max-h-[400px] overflow-y-auto p-2">
            <Command.Empty className="py-6 text-center text-foreground-muted">
              No commands found
            </Command.Empty>

            <Command.Group heading="Commands" className="text-xs text-foreground-muted px-2 py-1">
              {commands.map((cmd) => (
                <Command.Item
                  key={cmd.id}
                  value={cmd.title}
                  onSelect={() => handleSelect(cmd.id)}
                  className="flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer data-[selected=true]:bg-surface-2 transition-colors"
                >
                  <cmd.icon className="w-5 h-5 text-accent" />
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-sm text-foreground">{cmd.title}</div>
                    {cmd.subtitle && (
                      <div className="text-xs text-foreground-muted truncate">{cmd.subtitle}</div>
                    )}
                  </div>
                  {cmd.shortcut && (
                    <kbd className="px-2 py-1 text-xs bg-surface-3 rounded text-foreground-muted">
                      {formatShortcut(cmd.shortcut)}
                    </kbd>
                  )}
                </Command.Item>
              ))}
            </Command.Group>
          </Command.List>

          {/* Footer hint */}
          <div className="px-4 py-2 border-t border-border text-xs text-foreground-muted flex items-center gap-4">
            <span>
              <kbd className="px-1.5 py-0.5 bg-surface-2 rounded">↑↓</kbd> Navigate
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-surface-2 rounded">↵</kbd> Select
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-surface-2 rounded">esc</kbd> Close
            </span>
          </div>
        </Command>
      </div>
    </div>
  );
}
