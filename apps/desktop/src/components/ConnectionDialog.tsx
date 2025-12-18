import { useState } from 'react';
import { X, Server, Key, User, Lock, AlertTriangle } from 'lucide-react';
import { useAppStore } from '@/stores/appStore';
import { useSessionStore } from '@/stores/sessionStore';
import { invoke } from '@tauri-apps/api/core';
import { clsx } from 'clsx';

type AuthType = 'agent' | 'password' | 'key';

interface ConnectRequest {
  host: string;
  port: number;
  username: string;
  auth: AuthRequest;
  name?: string;
  save_profile: boolean;
}

type AuthRequest =
  | { type: 'agent' }
  | { type: 'password'; password: string }
  | { type: 'private_key'; private_key: string; passphrase?: string };

interface ConnectionResult {
  success: boolean;
  session_id: string;
  host: string;
  connected_at?: number;
  error?: string;
  profile_id?: string;
}

export function ConnectionDialog() {
  const { setShowConnectionDialog, addTab, updateTab } = useAppStore();
  const { loadProfiles } = useSessionStore();

  const [host, setHost] = useState('');
  const [port, setPort] = useState('22');
  const [username, setUsername] = useState('');
  const [authType, setAuthType] = useState<AuthType>('password');
  const [password, setPassword] = useState('');
  const [privateKey, setPrivateKey] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [saveProfile, setSaveProfile] = useState(false);
  const [profileName, setProfileName] = useState('');
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleConnect = async () => {
    if (!host || !username) {
      setError('Host and username are required');
      return;
    }

    if (authType === 'password' && !password) {
      setError('Password is required');
      return;
    }

    if (authType === 'key' && !privateKey) {
      setError('Private key is required');
      return;
    }

    setError(null);
    setConnecting(true);

    try {
      // Build auth request - secrets sent directly in memory, not stored to keychain
      let auth: AuthRequest;
      switch (authType) {
        case 'password':
          auth = { type: 'password', password };
          break;
        case 'key':
          auth = { 
            type: 'private_key', 
            private_key: privateKey,
            passphrase: passphrase || undefined,
          };
          break;
        case 'agent':
        default:
          auth = { type: 'agent' };
      }

      const request: ConnectRequest = {
        host,
        port: parseInt(port, 10),
        username,
        auth,
        name: profileName || undefined,
        save_profile: saveProfile,
      };

      // Create tab first (will show connecting state)
      const tabTitle = profileName || `${username}@${host}`;
      const tabId = addTab({
        title: tabTitle,
        connected: false,
      });

      // Call the new ssh_connect command
      const result = await invoke<ConnectionResult>('ssh_connect', { request });

      if (result.success && result.session_id) {
        // Update tab with session ID and profile ID if saved
        updateTab(tabId, {
          sessionId: result.session_id,
          profileId: result.profile_id,
          connected: false, // Will be true when we receive ssh:connected event
        });
        
        // Reload profiles if we saved one
        if (saveProfile && result.profile_id) {
          await loadProfiles();
        }
        
        // Close dialog - connection will complete in background
        setShowConnectionDialog(false);
      } else {
        // Connection initiation failed
        setError(result.error || 'Connection failed');
      }
    } catch (err) {
      console.error('Connection error:', err);
      // Parse error message from Tauri
      let errorMsg = 'Connection failed';
      if (typeof err === 'object' && err !== null) {
        const errObj = err as { message?: string; code?: string };
        if (errObj.message) {
          errorMsg = errObj.message;
        }
      } else if (typeof err === 'string') {
        errorMsg = err;
      }
      setError(errorMsg);
    } finally {
      setConnecting(false);
    }
  };

  return (
    <div 
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={() => setShowConnectionDialog(false)}
    >
      <div 
        className="w-full max-w-md bg-surface-1 rounded-xl border border-border shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <Server className="w-5 h-5 text-accent" />
            New Connection
          </h2>
          <button
            onClick={() => setShowConnectionDialog(false)}
            className="p-1 rounded hover:bg-surface-2 transition-colors"
          >
            <X className="w-5 h-5 text-foreground-muted" />
          </button>
        </div>

        {/* Form */}
        <div className="p-6 space-y-4">
          {/* Connection name */}
          <div>
            <label className="block text-sm font-medium mb-1.5">Name (optional)</label>
            <input
              type="text"
              value={profileName}
              onChange={(e) => setProfileName(e.target.value)}
              placeholder="My Server"
              className="input w-full"
            />
          </div>

          {/* Host and port */}
          <div className="flex gap-3">
            <div className="flex-1">
              <label className="block text-sm font-medium mb-1.5">Host</label>
              <div className="relative">
                <Server className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground-muted" />
                <input
                  type="text"
                  value={host}
                  onChange={(e) => setHost(e.target.value)}
                  placeholder="hostname or IP"
                  className="input w-full pl-10"
                />
              </div>
            </div>
            <div className="w-24">
              <label className="block text-sm font-medium mb-1.5">Port</label>
              <input
                type="text"
                value={port}
                onChange={(e) => setPort(e.target.value)}
                placeholder="22"
                className="input w-full"
              />
            </div>
          </div>

          {/* Username */}
          <div>
            <label className="block text-sm font-medium mb-1.5">Username</label>
            <div className="relative">
              <User className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground-muted" />
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="username"
                className="input w-full pl-10"
              />
            </div>
          </div>

          {/* Authentication */}
          <div>
            <label className="block text-sm font-medium mb-1.5">Authentication</label>
            <div className="flex gap-2 mb-3">
              {(['agent', 'password', 'key'] as const).map((type) => (
                <button
                  key={type}
                  onClick={() => setAuthType(type)}
                  className={clsx(
                    'flex-1 py-2 px-3 rounded-lg text-sm font-medium transition-colors border',
                    authType === type
                      ? 'bg-accent text-white border-accent'
                      : 'bg-surface-2 text-foreground-muted border-border hover:border-accent'
                  )}
                >
                  {type === 'agent' && 'SSH Agent'}
                  {type === 'password' && 'Password'}
                  {type === 'key' && 'Private Key'}
                </button>
              ))}
            </div>

            {/* Password input */}
            {authType === 'password' && (
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground-muted" />
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Password"
                  className="input w-full pl-10"
                  autoComplete="off"
                />
              </div>
            )}

            {/* Private key input */}
            {authType === 'key' && (
              <div className="space-y-3">
                <div>
                  <div className="relative">
                    <Key className="absolute left-3 top-3 w-4 h-4 text-foreground-muted" />
                    <textarea
                      value={privateKey}
                      onChange={(e) => setPrivateKey(e.target.value)}
                      placeholder="Paste your private key here (PEM or OpenSSH format)..."
                      className="input w-full pl-10 h-24 resize-none font-mono text-xs"
                      autoComplete="off"
                      spellCheck={false}
                    />
                  </div>
                  <p className="text-xs text-foreground-muted mt-1">
                    Paste your private key content. It will be sent securely in memory.
                  </p>
                </div>
                <div className="relative">
                  <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground-muted" />
                  <input
                    type="password"
                    value={passphrase}
                    onChange={(e) => setPassphrase(e.target.value)}
                    placeholder="Passphrase (if key is encrypted)"
                    className="input w-full pl-10"
                    autoComplete="off"
                  />
                </div>
              </div>
            )}

            {/* SSH Agent info */}
            {authType === 'agent' && (
              <div className="p-3 rounded-lg bg-surface-2 text-sm text-foreground-muted">
                <p>
                  Using system SSH agent. Make sure ssh-agent is running and has keys loaded.
                </p>
                <p className="mt-1 text-xs">
                  On Linux/macOS: <code className="bg-surface-0 px-1 rounded">ssh-add ~/.ssh/id_rsa</code>
                </p>
              </div>
            )}
          </div>

          {/* Save profile */}
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={saveProfile}
              onChange={(e) => setSaveProfile(e.target.checked)}
              className="w-4 h-4 rounded border-border bg-surface-2 text-accent focus:ring-accent"
            />
            <span className="text-sm">Save as profile</span>
          </label>

          {/* Error message */}
          {error && (
            <div className="p-3 rounded-lg bg-error/10 border border-error/20 text-error text-sm flex items-start gap-2">
              <AlertTriangle className="w-4 h-4 mt-0.5 flex-shrink-0" />
              <span>{error}</span>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 px-6 py-4 border-t border-border bg-surface-0/50">
          <button
            onClick={() => setShowConnectionDialog(false)}
            className="btn"
            disabled={connecting}
          >
            Cancel
          </button>
          <button
            onClick={handleConnect}
            className="btn btn-primary"
            disabled={connecting || !host || !username}
          >
            {connecting ? (
              <>
                <span className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                Connecting...
              </>
            ) : (
              'Connect'
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
