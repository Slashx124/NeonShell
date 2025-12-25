import { useEffect, useState } from 'react';
import { 
  X, 
  Cpu, 
  Cloud, 
  Key, 
  Server, 
  Check, 
  AlertTriangle,
  ExternalLink,
  Trash2,
  Plus,
  RefreshCw
} from 'lucide-react';
import { useAIStore, type ModelProvider } from '@/stores/aiStore';
import { useUIStore } from '@/stores/uiStore';
import { clsx } from 'clsx';

export function AISettingsModal() {
  const { closeModal, addToast } = useUIStore();
  const {
    settings,
    models,
    isGatewayAuthenticated,
    ollamaAvailable,
    deviceLinkCode,
    loadSettings,
    loadModels,
    checkGatewayAuth,
    startGatewayAuth,
    pollGatewayAuth,
    gatewayLogout,
    addPersonalKey,
    removePersonalKey,
    checkOllama,
  } = useAIStore();

  const [activeTab, setActiveTab] = useState<'gateway' | 'local' | 'personal'>('gateway');
  const [polling, setPolling] = useState(false);
  const [showAddKey, setShowAddKey] = useState(false);
  const [newKeyProvider, setNewKeyProvider] = useState<ModelProvider>('openai');
  const [newKeyName, setNewKeyName] = useState('');
  const [newKeyValue, setNewKeyValue] = useState('');

  useEffect(() => {
    loadSettings();
    loadModels();
    checkGatewayAuth();
    checkOllama();
  }, []);

  // Poll for gateway auth
  useEffect(() => {
    if (!polling || !deviceLinkCode) return;

    const pollInterval = setInterval(async () => {
      const deviceCode = localStorage.getItem('gateway_device_code');
      if (!deviceCode) {
        setPolling(false);
        return;
      }

      const success = await pollGatewayAuth(deviceCode);
      if (success) {
        setPolling(false);
        localStorage.removeItem('gateway_device_code');
        addToast({
          type: 'success',
          title: 'Connected!',
          message: 'Successfully linked to NeonShell gateway',
        });
      }
    }, 5000);

    return () => clearInterval(pollInterval);
  }, [polling, deviceLinkCode, pollGatewayAuth, addToast]);

  const handleStartAuth = async () => {
    try {
      const deviceCode = await startGatewayAuth();
      localStorage.setItem('gateway_device_code', deviceCode);
      setPolling(true);
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Auth failed',
        message: String(error),
      });
    }
  };

  const handleAddPersonalKey = async () => {
    if (!newKeyName || !newKeyValue) {
      addToast({ type: 'error', title: 'Missing fields', message: 'Please fill all fields' });
      return;
    }

    try {
      await addPersonalKey(newKeyProvider, newKeyName, newKeyValue);
      setShowAddKey(false);
      setNewKeyName('');
      setNewKeyValue('');
      addToast({ type: 'success', title: 'Key added', message: 'Personal API key stored securely' });
    } catch (error) {
      addToast({ type: 'error', title: 'Failed', message: String(error) });
    }
  };

  const hostedModels = models.filter((m) => m.source === 'hosted' || m.source === 'org');
  const localModels = models.filter((m) => m.source === 'local');
  const personalModels = models.filter((m) => m.source === 'personal');

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-surface-1 rounded-xl border border-border w-full max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-border">
          <div className="flex items-center gap-3">
            <Cpu className="w-6 h-6 text-accent" />
            <h2 className="text-xl font-semibold">AI Model Settings</h2>
          </div>
          <button onClick={() => closeModal()} className="p-2 hover:bg-surface-2 rounded-lg">
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-border">
          {[
            { id: 'gateway', label: 'Gateway', icon: Cloud, count: hostedModels.length },
            { id: 'local', label: 'Local', icon: Server, count: localModels.length },
            { id: 'personal', label: 'Personal', icon: Key, count: personalModels.length },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={clsx(
                'flex-1 flex items-center justify-center gap-2 p-3 transition-colors',
                activeTab === tab.id
                  ? 'border-b-2 border-accent text-accent'
                  : 'text-foreground-muted hover:text-foreground'
              )}
            >
              <tab.icon className="w-4 h-4" />
              <span>{tab.label}</span>
              {tab.count > 0 && (
                <span className="px-1.5 py-0.5 bg-surface-2 rounded text-xs">{tab.count}</span>
              )}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {activeTab === 'gateway' && (
            <div className="space-y-4">
              {/* Gateway Status */}
              <div className={clsx(
                'p-4 rounded-lg border',
                isGatewayAuthenticated
                  ? 'bg-green-500/10 border-green-500/30'
                  : 'bg-surface-2 border-border'
              )}>
                {isGatewayAuthenticated ? (
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <Check className="w-5 h-5 text-green-400" />
                      <div>
                        <p className="font-medium text-green-400">Connected to NeonShell Gateway</p>
                        <p className="text-sm text-foreground-muted">Access to hosted & org models</p>
                      </div>
                    </div>
                    <button
                      onClick={gatewayLogout}
                      className="px-3 py-1.5 text-sm text-red-400 hover:bg-red-500/10 rounded"
                    >
                      Disconnect
                    </button>
                  </div>
                ) : deviceLinkCode ? (
                  <div className="text-center">
                    <p className="text-foreground-muted mb-2">Enter this code at</p>
                    <a
                      href="https://neonshell.dev/link"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-accent hover:underline flex items-center justify-center gap-1"
                    >
                      neonshell.dev/link <ExternalLink className="w-4 h-4" />
                    </a>
                    <div className="mt-4 text-3xl font-mono font-bold tracking-widest text-accent">
                      {deviceLinkCode}
                    </div>
                    <p className="mt-4 text-sm text-foreground-muted">
                      {polling ? 'Waiting for approval...' : 'Code will expire in 15 minutes'}
                    </p>
                  </div>
                ) : (
                  <div className="text-center">
                    <Cloud className="w-12 h-12 text-foreground-muted mx-auto mb-3" />
                    <p className="text-foreground-muted mb-4">
                      Connect to access hosted AI models without managing your own API keys
                    </p>
                    <button
                      onClick={handleStartAuth}
                      className="btn btn-primary"
                    >
                      Connect to Gateway
                    </button>
                  </div>
                )}
              </div>

              {/* Hosted Models List */}
              {hostedModels.length > 0 && (
                <div>
                  <h3 className="font-medium mb-2">Available Models</h3>
                  <div className="space-y-2">
                    {hostedModels.map((model) => (
                      <div key={model.id} className="flex items-center gap-3 p-3 bg-surface-2 rounded-lg">
                        <Cpu className="w-5 h-5 text-accent" />
                        <div className="flex-1">
                          <div className="flex items-center gap-2">
                            <span className="font-medium">{model.name}</span>
                            {model.badge && (
                              <span className="text-xs px-1.5 py-0.5 bg-accent/20 text-accent rounded">
                                {model.badge}
                              </span>
                            )}
                            <span className={clsx(
                              'text-xs px-1.5 py-0.5 rounded',
                              model.source === 'org' ? 'bg-purple-500/20 text-purple-400' : 'bg-cyan-500/20 text-cyan-400'
                            )}>
                              {model.source}
                            </span>
                          </div>
                          <p className="text-sm text-foreground-muted">{model.description}</p>
                        </div>
                        {model.pricing && (
                          <span className="text-xs text-foreground-muted">
                            ${model.pricing.input_per_1m_tokens}/1M in
                          </span>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === 'local' && (
            <div className="space-y-4">
              {/* Ollama Status */}
              <div className={clsx(
                'p-4 rounded-lg border',
                ollamaAvailable
                  ? 'bg-green-500/10 border-green-500/30'
                  : 'bg-orange-500/10 border-orange-500/30'
              )}>
                <div className="flex items-center gap-3">
                  {ollamaAvailable ? (
                    <>
                      <Check className="w-5 h-5 text-green-400" />
                      <div>
                        <p className="font-medium text-green-400">Ollama is running</p>
                        <p className="text-sm text-foreground-muted">Local models available</p>
                      </div>
                    </>
                  ) : (
                    <>
                      <AlertTriangle className="w-5 h-5 text-orange-400" />
                      <div>
                        <p className="font-medium text-orange-400">Ollama not detected</p>
                        <p className="text-sm text-foreground-muted">
                          Install from <a href="https://ollama.ai" target="_blank" rel="noopener noreferrer" className="text-accent hover:underline">ollama.ai</a>
                        </p>
                      </div>
                    </>
                  )}
                  <button
                    onClick={checkOllama}
                    className="ml-auto p-2 hover:bg-surface-2 rounded"
                  >
                    <RefreshCw className="w-4 h-4" />
                  </button>
                </div>
              </div>

              {/* Local Models List */}
              {localModels.length > 0 ? (
                <div className="space-y-2">
                  {localModels.map((model) => (
                    <div key={model.id} className="flex items-center gap-3 p-3 bg-surface-2 rounded-lg">
                      <Server className="w-5 h-5 text-green-400" />
                      <div className="flex-1">
                        <span className="font-medium">{model.name}</span>
                        <p className="text-sm text-foreground-muted">{model.endpoint}</p>
                      </div>
                      <span className="text-xs px-1.5 py-0.5 bg-green-500/20 text-green-400 rounded">
                        local
                      </span>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-center text-foreground-muted py-8">
                  {ollamaAvailable ? 'No models found. Run `ollama pull llama3` to get started.' : 'Start Ollama to see local models.'}
                </p>
              )}
            </div>
          )}

          {activeTab === 'personal' && (
            <div className="space-y-4">
              {/* Warning */}
              <div className="p-3 bg-orange-500/10 border border-orange-500/30 rounded-lg flex items-start gap-2">
                <AlertTriangle className="w-5 h-5 text-orange-400 flex-shrink-0" />
                <p className="text-sm text-orange-300">
                  Personal API keys are stored encrypted in your OS keychain. You pay the provider directly.
                </p>
              </div>

              {/* Add Key Form */}
              {showAddKey ? (
                <div className="p-4 bg-surface-2 rounded-lg space-y-3">
                  <select
                    value={newKeyProvider}
                    onChange={(e) => setNewKeyProvider(e.target.value as ModelProvider)}
                    className="w-full px-3 py-2 bg-surface-0 border border-border rounded-lg"
                  >
                    <option value="openai">OpenAI</option>
                    <option value="anthropic">Anthropic</option>
                  </select>
                  <input
                    type="text"
                    value={newKeyName}
                    onChange={(e) => setNewKeyName(e.target.value)}
                    placeholder="Key name (e.g., Personal GPT-4)"
                    className="w-full px-3 py-2 bg-surface-0 border border-border rounded-lg"
                  />
                  <input
                    type="password"
                    value={newKeyValue}
                    onChange={(e) => setNewKeyValue(e.target.value)}
                    placeholder="API key (sk-...)"
                    className="w-full px-3 py-2 bg-surface-0 border border-border rounded-lg font-mono"
                  />
                  <div className="flex gap-2">
                    <button
                      onClick={() => setShowAddKey(false)}
                      className="flex-1 btn btn-secondary"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleAddPersonalKey}
                      className="flex-1 btn btn-primary"
                    >
                      Add Key
                    </button>
                  </div>
                </div>
              ) : (
                <button
                  onClick={() => setShowAddKey(true)}
                  className="w-full p-3 border border-dashed border-border rounded-lg text-foreground-muted hover:border-accent hover:text-accent transition-colors flex items-center justify-center gap-2"
                >
                  <Plus className="w-4 h-4" />
                  Add Personal API Key
                </button>
              )}

              {/* Personal Keys List */}
              {settings?.personal_keys.map((key) => (
                <div key={key.id} className="flex items-center gap-3 p-3 bg-surface-2 rounded-lg">
                  <Key className="w-5 h-5 text-purple-400" />
                  <div className="flex-1">
                    <span className="font-medium">{key.name}</span>
                    <p className="text-sm text-foreground-muted capitalize">{key.provider}</p>
                  </div>
                  <button
                    onClick={() => removePersonalKey(key.id)}
                    className="p-2 text-red-400 hover:bg-red-500/10 rounded"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-border flex justify-between items-center">
          <button
            onClick={() => loadModels(true)}
            className="btn btn-secondary flex items-center gap-2"
          >
            <RefreshCw className="w-4 h-4" />
            Refresh Models
          </button>
          <button onClick={() => closeModal()} className="btn btn-primary">
            Done
          </button>
        </div>
      </div>
    </div>
  );
}




