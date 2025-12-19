import { useState, useEffect, useCallback } from 'react';
import { 
  FolderOpen, 
  File, 
  FileText,
  Image,
  FileCode,
  FileArchive,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  RefreshCw,
  Upload,
  FolderPlus,
  Trash2,
  Download,
  Loader2,
  Home,
  Link2,
} from 'lucide-react';
import { useSftpStore, SftpEntry } from '@/stores/sftpStore';
import { useSessionStore } from '@/stores/sessionStore';
import { clsx } from 'clsx';

interface SftpBrowserProps {
  profileId: string | null;
}

export function SftpBrowser({ profileId }: SftpBrowserProps) {
  const { 
    activeProfileId,
    setActiveProfile,
    currentPath,
    entries,
    selectedEntries,
    loading,
    error,
    history,
    historyIndex,
    navigateTo,
    navigateUp,
    navigateBack,
    navigateForward,
    refresh,
    selectEntry,
    clearSelection,
    uploadFile,
    createFolder,
    deleteSelected,
    openEntry,
    downloadFile,
  } = useSftpStore();

  const { profiles } = useSessionStore();
  const [showNewFolderDialog, setShowNewFolderDialog] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; entry: SftpEntry } | null>(null);

  // Initialize with profile
  useEffect(() => {
    if (profileId !== activeProfileId) {
      setActiveProfile(profileId);
    }
  }, [profileId, activeProfileId, setActiveProfile]);

  // Close context menu on click outside
  useEffect(() => {
    const handleClick = () => setContextMenu(null);
    window.addEventListener('click', handleClick);
    return () => window.removeEventListener('click', handleClick);
  }, []);

  const handleCreateFolder = useCallback(async () => {
    if (!newFolderName.trim()) return;
    await createFolder(newFolderName.trim());
    setNewFolderName('');
    setShowNewFolderDialog(false);
  }, [newFolderName, createFolder]);

  const handleContextMenu = useCallback((e: React.MouseEvent, entry: SftpEntry) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY, entry });
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Delete' && selectedEntries.size > 0) {
      deleteSelected();
    } else if (e.key === 'a' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      useSftpStore.getState().selectAll();
    } else if (e.key === 'Escape') {
      clearSelection();
    }
  }, [selectedEntries, deleteSelected, clearSelection]);

  if (!profileId) {
    return (
      <div className="h-full flex flex-col items-center justify-center text-foreground-muted p-4">
        <FolderOpen className="w-12 h-12 mb-3 opacity-50" />
        <p className="text-sm">Select a profile to browse files</p>
        <p className="text-xs mt-1">Choose a saved connection from the list</p>
      </div>
    );
  }

  const profile = profiles.find(p => p.id === profileId);

  return (
    <div className="h-full flex flex-col bg-surface-0" onKeyDown={handleKeyDown} tabIndex={0}>
      {/* Header */}
      <div className="flex items-center gap-1 px-2 py-1.5 border-b border-border bg-surface-1">
        {/* Navigation buttons */}
        <button
          onClick={navigateBack}
          disabled={historyIndex <= 0 || loading}
          className="p-1.5 rounded hover:bg-surface-2 disabled:opacity-30 disabled:cursor-not-allowed"
          title="Back"
        >
          <ChevronLeft className="w-4 h-4" />
        </button>
        <button
          onClick={navigateForward}
          disabled={historyIndex >= history.length - 1 || loading}
          className="p-1.5 rounded hover:bg-surface-2 disabled:opacity-30 disabled:cursor-not-allowed"
          title="Forward"
        >
          <ChevronRight className="w-4 h-4" />
        </button>
        <button
          onClick={navigateUp}
          disabled={!currentPath || currentPath === '/' || loading}
          className="p-1.5 rounded hover:bg-surface-2 disabled:opacity-30 disabled:cursor-not-allowed"
          title="Up"
        >
          <ChevronUp className="w-4 h-4" />
        </button>
        <button
          onClick={() => navigateTo('')}
          disabled={loading}
          className="p-1.5 rounded hover:bg-surface-2 disabled:opacity-30"
          title="Home"
        >
          <Home className="w-4 h-4" />
        </button>
        <button
          onClick={refresh}
          disabled={loading}
          className="p-1.5 rounded hover:bg-surface-2 disabled:opacity-30"
          title="Refresh"
        >
          <RefreshCw className={clsx("w-4 h-4", loading && "animate-spin")} />
        </button>

        {/* Path breadcrumb */}
        <div className="flex-1 mx-2 px-2 py-1 bg-surface-0 rounded text-xs text-foreground-muted truncate font-mono">
          {profile?.username}@{profile?.host}:{currentPath || '~'}
        </div>

        {/* Action buttons */}
        <button
          onClick={uploadFile}
          disabled={loading}
          className="p-1.5 rounded hover:bg-surface-2 disabled:opacity-30"
          title="Upload file"
        >
          <Upload className="w-4 h-4" />
        </button>
        <button
          onClick={() => setShowNewFolderDialog(true)}
          disabled={loading}
          className="p-1.5 rounded hover:bg-surface-2 disabled:opacity-30"
          title="New folder"
        >
          <FolderPlus className="w-4 h-4" />
        </button>
        {selectedEntries.size > 0 && (
          <button
            onClick={deleteSelected}
            disabled={loading}
            className="p-1.5 rounded hover:bg-surface-2 text-error disabled:opacity-30"
            title={`Delete ${selectedEntries.size} item(s)`}
          >
            <Trash2 className="w-4 h-4" />
          </button>
        )}
      </div>

      {/* File list */}
      <div className="flex-1 overflow-y-auto">
        {loading && entries.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="w-8 h-8 animate-spin text-accent" />
          </div>
        ) : error ? (
          <div className="flex flex-col items-center justify-center h-full text-error p-4">
            <p className="text-sm">{error}</p>
            <button
              onClick={refresh}
              className="mt-2 btn btn-sm btn-primary"
            >
              Retry
            </button>
          </div>
        ) : entries.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-foreground-muted">
            <FolderOpen className="w-12 h-12 mb-2 opacity-50" />
            <p className="text-sm">Empty directory</p>
          </div>
        ) : (
          <div className="divide-y divide-border/50">
            {entries.map((entry) => (
              <FileRow
                key={entry.path}
                entry={entry}
                selected={selectedEntries.has(entry.path)}
                onSelect={(multi) => selectEntry(entry.path, multi)}
                onOpen={() => openEntry(entry)}
                onContextMenu={(e) => handleContextMenu(e, entry)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Status bar */}
      <div className="px-2 py-1 border-t border-border text-xs text-foreground-muted bg-surface-1">
        {entries.length} items
        {selectedEntries.size > 0 && ` • ${selectedEntries.size} selected`}
        {loading && ' • Loading...'}
      </div>

      {/* New folder dialog */}
      {showNewFolderDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-surface-1 rounded-lg p-4 w-80 border border-border shadow-xl">
            <h3 className="text-sm font-semibold mb-3">New Folder</h3>
            <input
              type="text"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleCreateFolder()}
              placeholder="Folder name"
              className="w-full px-3 py-2 bg-surface-0 border border-border rounded text-sm focus:outline-none focus:border-accent"
              autoFocus
            />
            <div className="flex justify-end gap-2 mt-4">
              <button
                onClick={() => setShowNewFolderDialog(false)}
                className="btn btn-sm"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateFolder}
                disabled={!newFolderName.trim()}
                className="btn btn-sm btn-primary"
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Context menu */}
      {contextMenu && (
        <div
          className="fixed z-50 bg-surface-2 rounded-lg border border-border shadow-xl py-1 min-w-[150px]"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onClick={(e) => e.stopPropagation()}
        >
          {!contextMenu.entry.is_dir && (
            <button
              onClick={() => {
                downloadFile(contextMenu.entry);
                setContextMenu(null);
              }}
              className="w-full px-3 py-2 text-left text-sm hover:bg-surface-3 flex items-center gap-2"
            >
              <Download className="w-4 h-4" />
              Download
            </button>
          )}
          <button
            onClick={() => {
              selectEntry(contextMenu.entry.path);
              deleteSelected();
              setContextMenu(null);
            }}
            className="w-full px-3 py-2 text-left text-sm hover:bg-surface-3 flex items-center gap-2 text-error"
          >
            <Trash2 className="w-4 h-4" />
            Delete
          </button>
        </div>
      )}
    </div>
  );
}

interface FileRowProps {
  entry: SftpEntry;
  selected: boolean;
  onSelect: (multi: boolean) => void;
  onOpen: () => void;
  onContextMenu: (e: React.MouseEvent) => void;
}

function FileRow({ entry, selected, onSelect, onOpen, onContextMenu }: FileRowProps) {
  const Icon = getFileIcon(entry);
  
  return (
    <div
      className={clsx(
        'flex items-center gap-2 px-2 py-1.5 cursor-pointer transition-colors',
        selected ? 'bg-accent/20' : 'hover:bg-surface-2'
      )}
      onClick={(e) => onSelect(e.ctrlKey || e.metaKey || e.shiftKey)}
      onDoubleClick={onOpen}
      onContextMenu={onContextMenu}
    >
      <Icon className={clsx(
        'w-4 h-4 flex-shrink-0',
        entry.is_dir ? 'text-accent' : 'text-foreground-muted'
      )} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1">
          <span className="text-sm truncate">{entry.name}</span>
          {entry.is_symlink && (
            <Link2 className="w-3 h-3 text-foreground-muted" />
          )}
        </div>
      </div>
      <span className="text-xs text-foreground-muted font-mono">
        {entry.permissions}
      </span>
      {!entry.is_dir && (
        <span className="text-xs text-foreground-muted w-16 text-right">
          {formatSize(entry.size)}
        </span>
      )}
    </div>
  );
}

function getFileIcon(entry: SftpEntry) {
  if (entry.is_dir) return FolderOpen;
  
  const ext = entry.name.split('.').pop()?.toLowerCase() || '';
  
  // Images
  if (['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'ico'].includes(ext)) {
    return Image;
  }
  
  // Code
  if (['js', 'ts', 'jsx', 'tsx', 'py', 'rb', 'go', 'rs', 'c', 'cpp', 'h', 'java', 'sh', 'bash', 'zsh', 'fish'].includes(ext)) {
    return FileCode;
  }
  
  // Archives
  if (['zip', 'tar', 'gz', 'bz2', 'xz', 'rar', '7z'].includes(ext)) {
    return FileArchive;
  }
  
  // Text
  if (['txt', 'md', 'markdown', 'json', 'yaml', 'yml', 'toml', 'xml', 'csv', 'log', 'conf', 'cfg', 'ini'].includes(ext)) {
    return FileText;
  }
  
  return File;
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

