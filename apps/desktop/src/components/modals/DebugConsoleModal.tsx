import { useEffect, useRef, useState } from 'react';
import { 
  X, 
  Terminal, 
  Download, 
  Trash2, 
  Pause, 
  Play, 
  Search, 
  Copy, 
  ExternalLink,
  Filter,
  ChevronDown
} from 'lucide-react';
import { useUIStore } from '@/stores/uiStore';
import { useLogStore, LogLevel, LogSubsystem } from '@/stores/logStore';
import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

const LEVEL_COLORS: Record<LogLevel, string> = {
  debug: 'text-foreground-muted',
  info: 'text-blue-400',
  warn: 'text-yellow-400',
  error: 'text-red-400',
};

const SUBSYSTEM_COLORS: Record<LogSubsystem, string> = {
  ssh: 'text-green-400',
  config: 'text-purple-400',
  plugins: 'text-orange-400',
  python: 'text-cyan-400',
  keychain: 'text-pink-400',
  app: 'text-foreground',
  unknown: 'text-foreground-muted',
};

function formatTimestamp(ts: number): string {
  const date = new Date(ts);
  return date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3,
  });
}

export function DebugConsoleModal() {
  const { closeModal, addToast } = useUIStore();
  const { 
    filter, 
    isPaused, 
    setFilter, 
    clearFilter,
    clearEntries, 
    setPaused,
    getFilteredEntries,
    loadRecentLogs,
    exportBundle,
  } = useLogStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [showFilters, setShowFilters] = useState(false);
  const [autoScroll, setAutoScroll] = useState(true);
  const [selectedLines, setSelectedLines] = useState<Set<number>>(new Set());
  const [exporting, setExporting] = useState(false);
  
  const logContainerRef = useRef<HTMLDivElement>(null);
  const filteredEntries = getFilteredEntries();

  // Load recent logs on mount
  useEffect(() => {
    loadRecentLogs(1000);
  }, [loadRecentLogs]);

  // Auto-scroll to bottom when new entries arrive
  useEffect(() => {
    if (autoScroll && !isPaused && logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [filteredEntries.length, autoScroll, isPaused]);

  // Handle scroll to detect if user scrolled up
  const handleScroll = () => {
    if (logContainerRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = logContainerRef.current;
      const isAtBottom = scrollHeight - scrollTop - clientHeight < 50;
      setAutoScroll(isAtBottom);
    }
  };

  // Apply search filter
  useEffect(() => {
    if (searchQuery) {
      setFilter({ search: searchQuery });
    } else {
      setFilter({ search: undefined });
    }
  }, [searchQuery, setFilter]);

  // Copy selected or all (sanitized)
  const handleCopy = async (all: boolean) => {
    try {
      let text: string;
      if (all) {
        text = filteredEntries
          .map((e) => `[${formatTimestamp(e.timestamp)}] [${e.level.toUpperCase()}] [${e.subsystem}] ${e.message}`)
          .join('\n');
      } else {
        text = filteredEntries
          .filter((_, i) => selectedLines.has(i))
          .map((e) => `[${formatTimestamp(e.timestamp)}] [${e.level.toUpperCase()}] [${e.subsystem}] ${e.message}`)
          .join('\n');
      }
      await navigator.clipboard.writeText(text);
      addToast({
        type: 'success',
        title: 'Copied to clipboard',
        message: all ? 'All visible logs copied' : 'Selected logs copied',
      });
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Copy failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  // Export debug bundle
  const handleExport = async () => {
    setExporting(true);
    try {
      const path = await save({
        title: 'Export Debug Bundle',
        defaultPath: `neonshell-debug-${Date.now()}.zip`,
        filters: [{ name: 'Debug Bundle', extensions: ['zip'] }],
      });

      if (!path) {
        setExporting(false);
        return; // User cancelled
      }

      const exportPath = await exportBundle(path, {
        max_lines: 10000,
        include_config: true,
        include_sessions: true,
        include_plugins: true,
        redact_hostnames: false,
      });

      addToast({
        type: 'success',
        title: 'Debug bundle exported',
        message: `Saved to ${exportPath}`,
      });

      // Offer to reveal in explorer
      try {
        await invoke('reveal_in_explorer', { path: exportPath });
      } catch {
        // Ignore if reveal fails
      }
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Export failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setExporting(false);
    }
  };

  // Clear view
  const handleClear = () => {
    clearEntries();
    setSelectedLines(new Set());
    addToast({
      type: 'info',
      title: 'Log view cleared',
      message: 'In-memory logs cleared. File logs preserved.',
    });
  };

  // Toggle line selection
  const toggleLineSelection = (index: number) => {
    setSelectedLines((prev) => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-5xl h-[80vh] bg-surface-1 rounded-xl border border-border shadow-2xl overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-border bg-surface-0">
          <div className="flex items-center gap-3">
            <Terminal className="w-5 h-5 text-accent" />
            <h2 className="text-lg font-semibold text-foreground">Debug Console</h2>
            <span className="text-xs text-foreground-muted bg-surface-2 px-2 py-0.5 rounded">
              {filteredEntries.length} entries
            </span>
          </div>
          <button
            onClick={closeModal}
            className="p-1 rounded hover:bg-surface-2 text-foreground-muted hover:text-foreground transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Toolbar */}
        <div className="flex items-center gap-2 px-4 py-2 border-b border-border bg-surface-0">
          {/* Search */}
          <div className="relative flex-1 max-w-xs">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground-muted" />
            <input
              type="text"
              placeholder="Search logs..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-9 pr-3 py-1.5 bg-surface-2 border border-border rounded text-sm text-foreground placeholder:text-foreground-muted focus:outline-none focus:ring-1 focus:ring-accent"
            />
          </div>

          {/* Filter button */}
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`flex items-center gap-1 px-3 py-1.5 text-sm rounded border transition-colors ${
              showFilters || Object.keys(filter).filter(k => k !== 'search' && filter[k as keyof typeof filter]).length > 0
                ? 'bg-accent/20 border-accent text-accent'
                : 'bg-surface-2 border-border text-foreground-muted hover:text-foreground'
            }`}
          >
            <Filter className="w-4 h-4" />
            Filters
            <ChevronDown className={`w-3 h-3 transition-transform ${showFilters ? 'rotate-180' : ''}`} />
          </button>

          <div className="w-px h-6 bg-border" />

          {/* Pause/Resume */}
          <button
            onClick={() => setPaused(!isPaused)}
            className={`flex items-center gap-1 px-3 py-1.5 text-sm rounded border transition-colors ${
              isPaused
                ? 'bg-yellow-500/20 border-yellow-500 text-yellow-400'
                : 'bg-surface-2 border-border text-foreground-muted hover:text-foreground'
            }`}
            title={isPaused ? 'Resume' : 'Pause'}
          >
            {isPaused ? <Play className="w-4 h-4" /> : <Pause className="w-4 h-4" />}
            {isPaused ? 'Resume' : 'Pause'}
          </button>

          {/* Copy */}
          <button
            onClick={() => handleCopy(selectedLines.size === 0)}
            className="flex items-center gap-1 px-3 py-1.5 text-sm bg-surface-2 border border-border rounded text-foreground-muted hover:text-foreground transition-colors"
            title={selectedLines.size > 0 ? 'Copy selected' : 'Copy all'}
          >
            <Copy className="w-4 h-4" />
            {selectedLines.size > 0 ? `Copy (${selectedLines.size})` : 'Copy All'}
          </button>

          {/* Clear */}
          <button
            onClick={handleClear}
            className="flex items-center gap-1 px-3 py-1.5 text-sm bg-surface-2 border border-border rounded text-foreground-muted hover:text-foreground transition-colors"
            title="Clear view"
          >
            <Trash2 className="w-4 h-4" />
            Clear
          </button>

          <div className="w-px h-6 bg-border" />

          {/* Export */}
          <button
            onClick={handleExport}
            disabled={exporting}
            className="flex items-center gap-1 px-3 py-1.5 text-sm bg-accent text-white rounded hover:bg-accent/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <Download className="w-4 h-4" />
            {exporting ? 'Exporting...' : 'Export Bundle'}
          </button>
        </div>

        {/* Filter panel */}
        {showFilters && (
          <div className="flex items-center gap-4 px-4 py-2 border-b border-border bg-surface-0">
            <div className="flex items-center gap-2">
              <span className="text-xs text-foreground-muted">Level:</span>
              <select
                value={filter.level || ''}
                onChange={(e) => setFilter({ level: e.target.value as LogLevel || undefined })}
                className="px-2 py-1 bg-surface-2 border border-border rounded text-xs text-foreground"
              >
                <option value="">All</option>
                <option value="debug">Debug</option>
                <option value="info">Info</option>
                <option value="warn">Warning</option>
                <option value="error">Error</option>
              </select>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-foreground-muted">Subsystem:</span>
              <select
                value={filter.subsystem || ''}
                onChange={(e) => setFilter({ subsystem: e.target.value as LogSubsystem || undefined })}
                className="px-2 py-1 bg-surface-2 border border-border rounded text-xs text-foreground"
              >
                <option value="">All</option>
                <option value="ssh">SSH</option>
                <option value="config">Config</option>
                <option value="plugins">Plugins</option>
                <option value="python">Python</option>
                <option value="keychain">Keychain</option>
                <option value="app">App</option>
              </select>
            </div>
            {Object.keys(filter).filter(k => filter[k as keyof typeof filter]).length > 0 && (
              <button
                onClick={clearFilter}
                className="text-xs text-accent hover:underline"
              >
                Clear filters
              </button>
            )}
          </div>
        )}

        {/* Log entries */}
        <div
          ref={logContainerRef}
          onScroll={handleScroll}
          className="flex-1 overflow-y-auto font-mono text-xs bg-surface-0"
        >
          {filteredEntries.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-foreground-muted">
              <Terminal className="w-12 h-12 mb-3 opacity-50" />
              <p>No log entries</p>
              <p className="text-xs mt-1">Debug events will appear here in real-time</p>
            </div>
          ) : (
            <table className="w-full">
              <tbody>
                {filteredEntries.map((entry, index) => (
                  <tr
                    key={`${entry.timestamp}-${index}`}
                    onClick={() => toggleLineSelection(index)}
                    className={`hover:bg-surface-2 cursor-pointer border-b border-border/50 ${
                      selectedLines.has(index) ? 'bg-accent/10' : ''
                    }`}
                  >
                    <td className="px-2 py-1 text-foreground-muted whitespace-nowrap align-top w-24">
                      {formatTimestamp(entry.timestamp)}
                    </td>
                    <td className={`px-2 py-1 whitespace-nowrap align-top w-16 font-semibold ${LEVEL_COLORS[entry.level]}`}>
                      {entry.level.toUpperCase()}
                    </td>
                    <td className={`px-2 py-1 whitespace-nowrap align-top w-20 ${SUBSYSTEM_COLORS[entry.subsystem]}`}>
                      [{entry.subsystem}]
                    </td>
                    {entry.session_id && (
                      <td className="px-2 py-1 whitespace-nowrap align-top w-24 text-foreground-muted">
                        {entry.session_id.slice(0, 8)}
                      </td>
                    )}
                    <td className="px-2 py-1 text-foreground break-all">
                      {entry.message}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-2 border-t border-border bg-surface-0 text-xs text-foreground-muted">
          <div className="flex items-center gap-4">
            <span>
              {isPaused && <span className="text-yellow-400 mr-2">⏸ Paused</span>}
              {autoScroll ? 'Auto-scrolling' : 'Scroll paused'}
            </span>
          </div>
          <div className="flex items-center gap-4">
            <a
              href="https://github.com/yourorg/neonshell/issues/new"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1 text-accent hover:underline"
            >
              <ExternalLink className="w-3 h-3" />
              Report Issue
            </a>
            <span>Press <kbd className="px-1 py-0.5 bg-surface-2 rounded">Ctrl+`</kbd> to toggle</span>
          </div>
        </div>
      </div>
    </div>
  );
}

