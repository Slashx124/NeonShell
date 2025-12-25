import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';
import { writeFile, readFile } from '@tauri-apps/plugin-fs';
import { useUIStore } from './uiStore';

export interface SftpEntry {
  name: string;
  path: string;
  is_dir: boolean;
  is_symlink: boolean;
  size: number;
  modified: number | null;
  permissions: string;
}

export interface SftpListResponse {
  entries: SftpEntry[];
  current_path: string;
}

interface SftpStoreState {
  // Current state
  activeProfileId: string | null;
  currentPath: string;
  entries: SftpEntry[];
  selectedEntries: Set<string>;
  loading: boolean;
  error: string | null;
  
  // Navigation history
  history: string[];
  historyIndex: number;
  
  // Actions
  setActiveProfile: (profileId: string | null) => void;
  navigateTo: (path: string) => Promise<void>;
  navigateUp: () => Promise<void>;
  navigateBack: () => Promise<void>;
  navigateForward: () => Promise<void>;
  refresh: () => Promise<void>;
  
  // Selection
  selectEntry: (path: string, multi?: boolean) => void;
  clearSelection: () => void;
  selectAll: () => void;
  
  // File operations
  downloadFile: (entry: SftpEntry) => Promise<void>;
  uploadFile: () => Promise<void>;
  createFolder: (name: string) => Promise<void>;
  deleteSelected: () => Promise<void>;
  renameEntry: (entry: SftpEntry, newName: string) => Promise<void>;
  
  // Double-click handling
  openEntry: (entry: SftpEntry) => Promise<void>;
}

export const useSftpStore = create<SftpStoreState>((set, get) => ({
  activeProfileId: null,
  currentPath: '',
  entries: [],
  selectedEntries: new Set(),
  loading: false,
  error: null,
  history: [],
  historyIndex: -1,

  setActiveProfile: (profileId) => {
    set({ 
      activeProfileId: profileId, 
      currentPath: '',
      entries: [],
      selectedEntries: new Set(),
      history: [],
      historyIndex: -1,
      error: null,
    });
    
    if (profileId) {
      // Automatically navigate to home
      get().navigateTo('');
    }
  },

  navigateTo: async (path) => {
    const { activeProfileId, history, historyIndex } = get();
    if (!activeProfileId) return;

    set({ loading: true, error: null });

    try {
      const response = await invoke<SftpListResponse>('sftp_list', {
        profileId: activeProfileId,
        path,
      });

      // Update history
      const newHistory = history.slice(0, historyIndex + 1);
      newHistory.push(response.current_path);

      set({
        currentPath: response.current_path,
        entries: response.entries,
        selectedEntries: new Set(),
        loading: false,
        history: newHistory,
        historyIndex: newHistory.length - 1,
      });
    } catch (error) {
      console.error('SFTP navigate error:', error);
      set({
        loading: false,
        error: error instanceof Error ? error.message : 'Failed to list directory',
      });
      useUIStore.getState().addToast({
        type: 'error',
        title: 'SFTP Error',
        message: error instanceof Error ? error.message : 'Failed to list directory',
      });
    }
  },

  navigateUp: async () => {
    const { currentPath } = get();
    if (!currentPath || currentPath === '/') return;
    
    // Go to parent directory
    const parts = currentPath.split('/').filter(Boolean);
    parts.pop();
    const parentPath = '/' + parts.join('/');
    
    await get().navigateTo(parentPath);
  },

  navigateBack: async () => {
    const { history, historyIndex, activeProfileId } = get();
    if (historyIndex <= 0 || !activeProfileId) return;

    const newIndex = historyIndex - 1;
    const path = history[newIndex];

    set({ loading: true, error: null });

    try {
      const response = await invoke<SftpListResponse>('sftp_list', {
        profileId: activeProfileId,
        path,
      });

      set({
        currentPath: response.current_path,
        entries: response.entries,
        selectedEntries: new Set(),
        loading: false,
        historyIndex: newIndex,
      });
    } catch (error) {
      set({ loading: false, error: String(error) });
    }
  },

  navigateForward: async () => {
    const { history, historyIndex, activeProfileId } = get();
    if (historyIndex >= history.length - 1 || !activeProfileId) return;

    const newIndex = historyIndex + 1;
    const path = history[newIndex];

    set({ loading: true, error: null });

    try {
      const response = await invoke<SftpListResponse>('sftp_list', {
        profileId: activeProfileId,
        path,
      });

      set({
        currentPath: response.current_path,
        entries: response.entries,
        selectedEntries: new Set(),
        loading: false,
        historyIndex: newIndex,
      });
    } catch (error) {
      set({ loading: false, error: String(error) });
    }
  },

  refresh: async () => {
    const { currentPath } = get();
    await get().navigateTo(currentPath);
  },

  selectEntry: (path, multi = false) => {
    set((state) => {
      const newSelection = multi ? new Set(state.selectedEntries) : new Set<string>();
      
      if (newSelection.has(path)) {
        newSelection.delete(path);
      } else {
        newSelection.add(path);
      }
      
      return { selectedEntries: newSelection };
    });
  },

  clearSelection: () => {
    set({ selectedEntries: new Set() });
  },

  selectAll: () => {
    set((state) => ({
      selectedEntries: new Set(state.entries.map(e => e.path)),
    }));
  },

  downloadFile: async (entry) => {
    const { activeProfileId } = get();
    if (!activeProfileId || entry.is_dir) return;

    try {
      // Show save dialog
      const savePath = await save({
        title: 'Save file',
        defaultPath: entry.name,
      });

      if (!savePath) return;

      useUIStore.getState().addToast({
        type: 'info',
        title: 'Downloading...',
        message: entry.name,
      });

      // Download file contents
      const contents = await invoke<number[]>('sftp_download', {
        profileId: activeProfileId,
        path: entry.path,
      });

      // Write to local file
      await writeFile(savePath, new Uint8Array(contents));

      useUIStore.getState().addToast({
        type: 'success',
        title: 'Download complete',
        message: entry.name,
      });
    } catch (error) {
      console.error('Download error:', error);
      useUIStore.getState().addToast({
        type: 'error',
        title: 'Download failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  },

  uploadFile: async () => {
    const { activeProfileId, currentPath, refresh } = get();
    if (!activeProfileId) return;

    try {
      // Show open dialog
      const files = await open({
        title: 'Select file to upload',
        multiple: false,
      });

      if (!files) return;

      const filePath = Array.isArray(files) ? files[0] : files;
      const fileName = filePath.split(/[/\\]/).pop() || 'uploaded_file';
      const remotePath = currentPath === '/' 
        ? `/${fileName}` 
        : `${currentPath}/${fileName}`;

      useUIStore.getState().addToast({
        type: 'info',
        title: 'Uploading...',
        message: fileName,
      });

      // Read local file
      const contents = await readFile(filePath);

      // Upload to remote
      await invoke('sftp_upload', {
        profileId: activeProfileId,
        remotePath,
        contents: Array.from(contents),
      });

      useUIStore.getState().addToast({
        type: 'success',
        title: 'Upload complete',
        message: fileName,
      });

      // Refresh the directory
      await refresh();
    } catch (error) {
      console.error('Upload error:', error);
      useUIStore.getState().addToast({
        type: 'error',
        title: 'Upload failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  },

  createFolder: async (name) => {
    const { activeProfileId, currentPath, refresh } = get();
    if (!activeProfileId || !name) return;

    const folderPath = currentPath === '/' 
      ? `/${name}` 
      : `${currentPath}/${name}`;

    try {
      await invoke('sftp_mkdir', {
        profileId: activeProfileId,
        path: folderPath,
      });

      useUIStore.getState().addToast({
        type: 'success',
        title: 'Folder created',
        message: name,
      });

      await refresh();
    } catch (error) {
      console.error('Create folder error:', error);
      useUIStore.getState().addToast({
        type: 'error',
        title: 'Failed to create folder',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  },

  deleteSelected: async () => {
    const { activeProfileId, selectedEntries, entries, refresh } = get();
    if (!activeProfileId || selectedEntries.size === 0) return;

    const toDelete = entries.filter(e => selectedEntries.has(e.path));

    try {
      for (const entry of toDelete) {
        await invoke('sftp_delete', {
          profileId: activeProfileId,
          path: entry.path,
          isDir: entry.is_dir,
        });
      }

      useUIStore.getState().addToast({
        type: 'success',
        title: 'Deleted',
        message: `${toDelete.length} item(s) deleted`,
      });

      set({ selectedEntries: new Set() });
      await refresh();
    } catch (error) {
      console.error('Delete error:', error);
      useUIStore.getState().addToast({
        type: 'error',
        title: 'Delete failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  },

  renameEntry: async (entry, newName) => {
    const { activeProfileId, currentPath, refresh } = get();
    if (!activeProfileId || !newName) return;

    const newPath = currentPath === '/' 
      ? `/${newName}` 
      : `${currentPath}/${newName}`;

    try {
      await invoke('sftp_rename', {
        profileId: activeProfileId,
        fromPath: entry.path,
        toPath: newPath,
      });

      useUIStore.getState().addToast({
        type: 'success',
        title: 'Renamed',
        message: `${entry.name} → ${newName}`,
      });

      await refresh();
    } catch (error) {
      console.error('Rename error:', error);
      useUIStore.getState().addToast({
        type: 'error',
        title: 'Rename failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  },

  openEntry: async (entry) => {
    if (entry.is_dir) {
      await get().navigateTo(entry.path);
    } else {
      // Download file on double-click
      await get().downloadFile(entry);
    }
  },
}));




