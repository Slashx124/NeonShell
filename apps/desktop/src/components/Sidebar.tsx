import { useEffect, useState } from 'react';
import { 
  Server, 
  Code, 
  FolderOpen, 
  Puzzle,
  Plus,
  Search,
  ChevronRight,
  Trash2,
  MoreVertical,
  Pencil
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '@/stores/appStore';
import { useSessionStore, Profile } from '@/stores/sessionStore';
import { useUIStore } from '@/stores/uiStore';
import { SftpBrowser } from './SftpBrowser';
import { clsx } from 'clsx';

const tabs = [
  { id: 'connections' as const, icon: Server, label: 'Connections' },
  { id: 'snippets' as const, icon: Code, label: 'Snippets' },
  { id: 'files' as const, icon: FolderOpen, label: 'Files' },
  { id: 'plugins' as const, icon: Puzzle, label: 'Plugins' },
];

export function Sidebar() {
  const { sidebarTab, setSidebarTab, setShowConnectionDialog } = useAppStore();
  const { profiles, loadProfiles } = useSessionStore();

  useEffect(() => {
    loadProfiles();
  }, [loadProfiles]);

  return (
    <div className="h-full flex flex-col bg-surface-1 border-r border-border">
      {/* Tab buttons */}
      <div className="flex border-b border-border">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setSidebarTab(tab.id)}
            className={clsx(
              'flex-1 p-3 flex items-center justify-center transition-colors',
              sidebarTab === tab.id
                ? 'bg-surface-2 text-accent'
                : 'text-foreground-muted hover:text-foreground hover:bg-surface-2'
            )}
            title={tab.label}
          >
            <tab.icon className="w-5 h-5" />
          </button>
        ))}
      </div>

      {/* Search */}
      <div className="p-3 border-b border-border">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground-muted" />
          <input
            type="text"
            placeholder="Search..."
            className="w-full pl-9 pr-3 py-2 bg-surface-0 rounded-lg text-sm border border-border focus:border-accent focus:outline-none"
          />
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-2">
        {sidebarTab === 'connections' && (
          <ConnectionsList profiles={profiles} />
        )}
        {sidebarTab === 'snippets' && <SnippetsList />}
        {sidebarTab === 'files' && <FilesList />}
        {sidebarTab === 'plugins' && <PluginsList />}
      </div>

      {/* Add button */}
      {sidebarTab === 'connections' && (
        <div className="p-3 border-t border-border">
          <button
            onClick={() => setShowConnectionDialog(true)}
            className="w-full btn btn-primary flex items-center justify-center gap-2"
          >
            <Plus className="w-4 h-4" />
            New Connection
          </button>
        </div>
      )}
    </div>
  );
}

interface ConnectionResult {
  success: boolean;
  session_id: string;
  host: string;
  connected_at?: number;
  error?: string;
  profile_id?: string;
}

function ConnectionsList({ profiles }: { profiles: Profile[] }) {
  const { addTab, updateTab, setShowConnectionDialog } = useAppStore();
  const { loadProfiles } = useSessionStore();
  const { addToast } = useUIStore();
  const [connecting, setConnecting] = useState<string | null>(null);
  const [menuOpen, setMenuOpen] = useState<string | null>(null);

  const handleEdit = (profile: Profile, e: React.MouseEvent) => {
    e.stopPropagation();
    setMenuOpen(null);
    setShowConnectionDialog(true, profile.id);
  };

  const handleConnect = async (profile: Profile) => {
    setConnecting(profile.id);
    
    try {
      // Create tab first (will show connecting state)
      const tabId = addTab({
        title: profile.name,
        connected: false,
      });

      // Connect using saved profile - backend will retrieve credentials from keychain
      const result = await invoke<ConnectionResult>('connect_profile', {
        profileId: profile.id,
      });

      if (result.success && result.session_id) {
        updateTab(tabId, {
          sessionId: result.session_id,
          profileId: profile.id,
          connected: false, // Will be true when connected event received
        });
        
        addToast({
          type: 'success',
          title: 'Connecting...',
          message: `Connecting to ${profile.name}`,
        });
      } else {
        addToast({
          type: 'error',
          title: 'Connection failed',
          message: result.error || 'Unknown error',
        });
      }
    } catch (error) {
      console.error('Connection error:', error);
      addToast({
        type: 'error',
        title: 'Connection failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setConnecting(null);
    }
  };

  const handleDelete = async (profile: Profile, e: React.MouseEvent) => {
    e.stopPropagation();
    setMenuOpen(null);
    
    try {
      await invoke('delete_profile', { id: profile.id });
      
      // Also delete stored credentials
      try {
        await invoke('delete_secret', { key: `password:${profile.id}` });
      } catch { /* Ignore if no password stored */ }
      try {
        await invoke('delete_secret', { key: `key:${profile.id}` });
      } catch { /* Ignore if no key stored */ }
      try {
        await invoke('delete_secret', { key: `passphrase:${profile.id}` });
      } catch { /* Ignore if no passphrase stored */ }
      
      // Clear history
      try {
        await invoke('clear_terminal_history', { profileId: profile.id });
      } catch { /* Ignore */ }
      
      await loadProfiles();
      
      addToast({
        type: 'success',
        title: 'Profile deleted',
        message: `${profile.name} has been removed`,
      });
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Failed to delete profile',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  if (profiles.length === 0) {
    return (
      <div className="text-center text-foreground-muted py-8">
        <Server className="w-12 h-12 mx-auto mb-3 opacity-50" />
        <p className="text-sm">No connections yet</p>
        <p className="text-xs mt-1">Create your first connection to get started</p>
      </div>
    );
  }

  return (
    <div className="space-y-1">
      {profiles.map((profile) => (
        <div
          key={profile.id}
          className="relative group"
        >
          <button
            onClick={() => handleConnect(profile)}
            disabled={connecting === profile.id}
            className={clsx(
              'w-full p-3 rounded-lg text-left transition-colors',
              connecting === profile.id
                ? 'bg-accent/10 cursor-wait'
                : 'hover:bg-surface-2'
            )}
          >
            <div className="flex items-center gap-3">
              <div className={clsx(
                'w-8 h-8 rounded-lg flex items-center justify-center',
                connecting === profile.id
                  ? 'bg-accent animate-pulse'
                  : 'bg-gradient-to-br from-accent to-neon-purple'
              )}>
                {connecting === profile.id ? (
                  <span className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                ) : (
                  <Server className="w-4 h-4 text-white" />
                )}
              </div>
              <div className="flex-1 min-w-0">
                <div className="font-medium text-sm truncate">{profile.name}</div>
                <div className="text-xs text-foreground-muted truncate">
                  {profile.username}@{profile.host}:{profile.port}
                </div>
              </div>
              <ChevronRight className={clsx(
                'w-4 h-4 text-foreground-muted transition-opacity',
                connecting === profile.id ? 'opacity-0' : 'opacity-0 group-hover:opacity-100'
              )} />
            </div>
          </button>
          
          {/* Context menu button */}
          <button
            onClick={(e) => {
              e.stopPropagation();
              setMenuOpen(menuOpen === profile.id ? null : profile.id);
            }}
            className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-surface-3 transition-all"
          >
            <MoreVertical className="w-4 h-4 text-foreground-muted" />
          </button>
          
          {/* Context menu */}
          {menuOpen === profile.id && (
            <div className="absolute right-2 top-full mt-1 z-10 bg-surface-2 rounded-lg border border-border shadow-lg py-1 min-w-[120px]">
              <button
                onClick={(e) => handleEdit(profile, e)}
                className="w-full px-3 py-2 text-left text-sm text-foreground hover:bg-surface-3 flex items-center gap-2"
              >
                <Pencil className="w-4 h-4" />
                Edit
              </button>
              <button
                onClick={(e) => handleDelete(profile, e)}
                className="w-full px-3 py-2 text-left text-sm text-error hover:bg-surface-3 flex items-center gap-2"
              >
                <Trash2 className="w-4 h-4" />
                Delete
              </button>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

function SnippetsList() {
  return (
    <div className="text-center text-foreground-muted py-8">
      <Code className="w-12 h-12 mx-auto mb-3 opacity-50" />
      <p className="text-sm">No snippets yet</p>
      <p className="text-xs mt-1">Create reusable command snippets</p>
    </div>
  );
}

function FilesList() {
  const { profiles } = useSessionStore();
  const [selectedProfileId, setSelectedProfileId] = useState<string | null>(null);
  
  if (profiles.length === 0) {
    return (
      <div className="text-center text-foreground-muted py-8">
        <FolderOpen className="w-12 h-12 mx-auto mb-3 opacity-50" />
        <p className="text-sm">No connections saved</p>
        <p className="text-xs mt-1">Save a connection to browse files via SFTP</p>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Profile selector */}
      <div className="p-2 border-b border-border">
        <select
          value={selectedProfileId || ''}
          onChange={(e) => setSelectedProfileId(e.target.value || null)}
          className="w-full px-2 py-1.5 bg-surface-0 border border-border rounded text-sm focus:outline-none focus:border-accent"
        >
          <option value="">Select a server...</option>
          {profiles.map((profile) => (
            <option key={profile.id} value={profile.id}>
              {profile.name} ({profile.username}@{profile.host})
            </option>
          ))}
        </select>
      </div>
      
      {/* SFTP Browser */}
      <div className="flex-1 overflow-hidden">
        <SftpBrowser profileId={selectedProfileId} />
      </div>
    </div>
  );
}

function PluginsList() {
  return (
    <div className="text-center text-foreground-muted py-8">
      <Puzzle className="w-12 h-12 mx-auto mb-3 opacity-50" />
      <p className="text-sm">No plugins installed</p>
      <p className="text-xs mt-1">Add plugins from the plugins folder</p>
    </div>
  );
}

