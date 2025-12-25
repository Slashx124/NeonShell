import { useState, useEffect } from 'react';
import { X, RefreshCw, Puzzle, ToggleLeft, ToggleRight, AlertTriangle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useUIStore } from '@/stores/uiStore';

interface Plugin {
  manifest: {
    id: string;
    name: string;
    version: string;
    api_version: string;
    description: string;
    author: string;
    permissions: string[];
    signed: boolean;
  };
  state: 'Disabled' | 'Enabled' | 'Error';
  path: string;
  granted_permissions: string[];
  error?: string;
}

export function PluginManager() {
  const { closeModal, addToast } = useUIStore();
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [loading, setLoading] = useState(true);
  const [toggling, setToggling] = useState<string | null>(null);

  useEffect(() => {
    loadPlugins();
  }, []);

  const loadPlugins = async () => {
    setLoading(true);
    try {
      const data = await invoke<Plugin[]>('list_plugins');
      setPlugins(data);
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Failed to load plugins',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  const togglePlugin = async (pluginId: string, currentState: string) => {
    setToggling(pluginId);
    try {
      if (currentState === 'Enabled') {
        await invoke('disable_plugin', { id: pluginId });
        addToast({
          type: 'success',
          title: 'Plugin disabled',
        });
      } else {
        // For enabling, grant all requested permissions
        const plugin = plugins.find(p => p.manifest.id === pluginId);
        const permissions = plugin?.manifest.permissions || [];
        await invoke('enable_plugin', { id: pluginId, permissions });
        addToast({
          type: 'success',
          title: 'Plugin enabled',
        });
      }
      await loadPlugins();
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Failed to toggle plugin',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setToggling(null);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-2xl bg-surface-1 rounded-xl border border-border shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <div className="flex items-center gap-3">
            <Puzzle className="w-5 h-5 text-accent" />
            <h2 className="text-lg font-semibold text-foreground">Plugin Manager</h2>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={loadPlugins}
              disabled={loading}
              className="p-2 rounded hover:bg-surface-2 text-foreground-muted hover:text-foreground transition-colors disabled:opacity-50"
              title="Refresh"
            >
              <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
            </button>
            <button
              onClick={closeModal}
              className="p-1 rounded hover:bg-surface-2 text-foreground-muted hover:text-foreground transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="p-4 max-h-[60vh] overflow-y-auto">
          {loading ? (
            <div className="text-center py-8 text-foreground-muted">Loading plugins...</div>
          ) : plugins.length === 0 ? (
            <div className="text-center py-8">
              <Puzzle className="w-12 h-12 mx-auto mb-3 text-foreground-muted opacity-50" />
              <p className="text-foreground-muted">No plugins installed</p>
              <p className="text-xs text-foreground-muted mt-1">
                Add plugins to the plugins folder in your config directory
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {plugins.map((plugin) => (
                <div
                  key={plugin.manifest.id}
                  className={`p-4 rounded-lg border ${
                    plugin.state === 'Error' 
                      ? 'border-error/50 bg-error/5'
                      : 'border-border bg-surface-2'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <h3 className="font-medium text-foreground">{plugin.manifest.name}</h3>
                        <span className="text-xs text-foreground-muted">v{plugin.manifest.version}</span>
                        {!plugin.manifest.signed && (
                          <span className="flex items-center gap-1 text-xs text-warning">
                            <AlertTriangle className="w-3 h-3" />
                            Unsigned
                          </span>
                        )}
                      </div>
                      {plugin.manifest.description && (
                        <p className="mt-1 text-sm text-foreground-muted">{plugin.manifest.description}</p>
                      )}
                      {plugin.manifest.author && (
                        <p className="mt-1 text-xs text-foreground-muted">by {plugin.manifest.author}</p>
                      )}
                      {plugin.manifest.permissions.length > 0 && (
                        <div className="mt-2 flex flex-wrap gap-1">
                          {plugin.manifest.permissions.map((perm) => (
                            <span
                              key={perm}
                              className="px-2 py-0.5 text-xs bg-surface-3 rounded text-foreground-muted"
                            >
                              {perm}
                            </span>
                          ))}
                        </div>
                      )}
                      {plugin.error && (
                        <p className="mt-2 text-xs text-error">{plugin.error}</p>
                      )}
                    </div>
                    <button
                      onClick={() => togglePlugin(plugin.manifest.id, plugin.state)}
                      disabled={toggling === plugin.manifest.id}
                      className="ml-4 flex-shrink-0"
                    >
                      {plugin.state === 'Enabled' ? (
                        <ToggleRight className="w-8 h-8 text-accent" />
                      ) : (
                        <ToggleLeft className="w-8 h-8 text-foreground-muted" />
                      )}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-border text-xs text-foreground-muted">
          Plugins extend NeonShell functionality. Review permissions before enabling.
        </div>
      </div>
    </div>
  );
}




