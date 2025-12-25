import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { nanoid } from 'nanoid';

export interface Tab {
  id: string;
  title: string;
  sessionId?: string;
  profileId?: string;
  connected: boolean;
}

export interface SplitPane {
  id: string;
  tabId: string;
  direction: 'horizontal' | 'vertical';
}

interface AppState {
  // UI State
  showSidebar: boolean;
  showCommandPalette: boolean;
  showConnectionDialog: boolean;
  editingProfileId: string | null;
  sidebarTab: 'connections' | 'snippets' | 'files' | 'plugins';
  
  // Tabs
  tabs: Tab[];
  activeTabId: string | null;
  
  // Actions
  toggleSidebar: () => void;
  setShowCommandPalette: (show: boolean) => void;
  setShowConnectionDialog: (show: boolean, editingProfileId?: string | null) => void;
  setSidebarTab: (tab: 'connections' | 'snippets' | 'files' | 'plugins') => void;
  
  // Tab actions
  addTab: (tab?: Partial<Tab>) => string;
  removeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTab: (id: string, updates: Partial<Tab>) => void;
}

export const useAppStore = create<AppState>()(
  immer((set) => ({
    // Initial state
    showSidebar: true,
    showCommandPalette: false,
    showConnectionDialog: false,
    editingProfileId: null,
    sidebarTab: 'connections',
    tabs: [],
    activeTabId: null,

    // UI Actions
    toggleSidebar: () => set((state) => { 
      state.showSidebar = !state.showSidebar; 
    }),
    
    setShowCommandPalette: (show) => set((state) => { 
      state.showCommandPalette = show; 
    }),
    
    setShowConnectionDialog: (show, editingProfileId = null) => set((state) => { 
      state.showConnectionDialog = show;
      state.editingProfileId = show ? editingProfileId : null;
    }),
    
    setSidebarTab: (tab) => set((state) => { 
      state.sidebarTab = tab; 
    }),

    // Tab Actions
    addTab: (tab) => {
      const id = nanoid();
      const newTab: Tab = {
        id,
        title: tab?.title || 'New Tab',
        sessionId: tab?.sessionId,
        profileId: tab?.profileId,
        connected: tab?.connected ?? false,
      };
      
      set((state) => {
        state.tabs.push(newTab);
        state.activeTabId = id;
      });
      
      return id;
    },
    
    removeTab: (id) => set((state) => {
      const index = state.tabs.findIndex((t) => t.id === id);
      if (index !== -1) {
        state.tabs.splice(index, 1);
        
        // Set new active tab if needed
        if (state.activeTabId === id) {
          if (state.tabs.length > 0) {
            const newIndex = Math.min(index, state.tabs.length - 1);
            state.activeTabId = state.tabs[newIndex].id;
          } else {
            state.activeTabId = null;
          }
        }
      }
    }),
    
    setActiveTab: (id) => set((state) => { 
      state.activeTabId = id; 
    }),
    
    updateTab: (id, updates) => set((state) => {
      const tab = state.tabs.find((t) => t.id === id);
      if (tab) {
        Object.assign(tab, updates);
      }
    }),
  }))
);

