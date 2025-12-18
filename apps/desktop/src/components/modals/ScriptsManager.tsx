import { useState, useEffect } from 'react';
import { X, RefreshCw, FileCode, ToggleLeft, ToggleRight, Play } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useUIStore } from '@/stores/uiStore';
import { useAppStore } from '@/stores/appStore';

interface Script {
  metadata: {
    id: string;
    name: string;
    description: string;
    author: string;
    version: string;
    hooks: string[];
    commands: Array<{ id: string; name: string; description: string }>;
  };
  state: 'Disabled' | 'Enabled' | 'Running' | 'Error';
  path: string;
  error?: string;
}

export function ScriptsManager() {
  const { closeModal, addToast } = useUIStore();
  const { tabs, activeTabId } = useAppStore();
  const [scripts, setScripts] = useState<Script[]>([]);
  const [loading, setLoading] = useState(true);
  const [toggling, setToggling] = useState<string | null>(null);
  const [running, setRunning] = useState<string | null>(null);

  const activeTab = tabs.find(t => t.id === activeTabId);
  const hasActiveSession = activeTab?.sessionId && activeTab?.connected;

  useEffect(() => {
    loadScripts();
  }, []);

  const loadScripts = async () => {
    setLoading(true);
    try {
      const data = await invoke<Script[]>('list_scripts');
      setScripts(data);
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Failed to load scripts',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  const toggleScript = async (scriptId: string, currentState: string) => {
    setToggling(scriptId);
    try {
      if (currentState === 'Enabled') {
        await invoke('disable_script', { id: scriptId });
        addToast({
          type: 'success',
          title: 'Script disabled',
        });
      } else {
        await invoke('enable_script', { id: scriptId });
        addToast({
          type: 'success',
          title: 'Script enabled',
        });
      }
      await loadScripts();
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Failed to toggle script',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setToggling(null);
    }
  };

  const runScript = async (scriptId: string) => {
    if (!hasActiveSession) {
      addToast({
        type: 'warning',
        title: 'No active session',
        message: 'Connect to a host before running scripts',
      });
      return;
    }

    setRunning(scriptId);
    try {
      await invoke('run_script', { 
        id: scriptId, 
        sessionId: activeTab?.sessionId,
        function: 'on_manual_run',
        args: {}
      });
      addToast({
        type: 'success',
        title: 'Script executed',
      });
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Script execution failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setRunning(null);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-2xl bg-surface-1 rounded-xl border border-border shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <div className="flex items-center gap-3">
            <FileCode className="w-5 h-5 text-accent" />
            <h2 className="text-lg font-semibold text-foreground">Python Scripts</h2>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={loadScripts}
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
            <div className="text-center py-8 text-foreground-muted">Loading scripts...</div>
          ) : scripts.length === 0 ? (
            <div className="text-center py-8">
              <FileCode className="w-12 h-12 mx-auto mb-3 text-foreground-muted opacity-50" />
              <p className="text-foreground-muted">No scripts found</p>
              <p className="text-xs text-foreground-muted mt-1">
                Add Python scripts (.py) to the scripts folder in your config directory
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {scripts.map((script) => (
                <div
                  key={script.metadata.id}
                  className={`p-4 rounded-lg border ${
                    script.state === 'Error' 
                      ? 'border-error/50 bg-error/5'
                      : 'border-border bg-surface-2'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <h3 className="font-medium text-foreground">{script.metadata.name}</h3>
                        <span className="text-xs text-foreground-muted">v{script.metadata.version}</span>
                      </div>
                      {script.metadata.description && (
                        <p className="mt-1 text-sm text-foreground-muted">{script.metadata.description}</p>
                      )}
                      {script.metadata.author && (
                        <p className="mt-1 text-xs text-foreground-muted">by {script.metadata.author}</p>
                      )}
                      {script.metadata.hooks.length > 0 && (
                        <div className="mt-2 flex flex-wrap gap-1">
                          {script.metadata.hooks.map((hook) => (
                            <span
                              key={hook}
                              className="px-2 py-0.5 text-xs bg-surface-3 rounded text-foreground-muted"
                            >
                              {hook}
                            </span>
                          ))}
                        </div>
                      )}
                      {script.error && (
                        <p className="mt-2 text-xs text-error">{script.error}</p>
                      )}
                    </div>
                    <div className="ml-4 flex items-center gap-2 flex-shrink-0">
                      {/* Run button */}
                      <button
                        onClick={() => runScript(script.metadata.id)}
                        disabled={running === script.metadata.id || script.state !== 'Enabled'}
                        className="p-2 rounded hover:bg-surface-3 text-foreground-muted hover:text-accent disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
                        title={hasActiveSession ? 'Run script' : 'No active session'}
                      >
                        <Play className={`w-4 h-4 ${running === script.metadata.id ? 'animate-pulse' : ''}`} />
                      </button>
                      {/* Toggle button */}
                      <button
                        onClick={() => toggleScript(script.metadata.id, script.state)}
                        disabled={toggling === script.metadata.id}
                      >
                        {script.state === 'Enabled' ? (
                          <ToggleRight className="w-8 h-8 text-accent" />
                        ) : (
                          <ToggleLeft className="w-8 h-8 text-foreground-muted" />
                        )}
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-border text-xs text-foreground-muted">
          Scripts can automate tasks and hook into SSH session events. Use @hook decorators to define triggers.
        </div>
      </div>
    </div>
  );
}

