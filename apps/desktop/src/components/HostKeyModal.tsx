import { useEffect, useState } from 'react';
import { ShieldAlert, ShieldCheck, ShieldX, Key, AlertTriangle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface HostKeyInfo {
  session_id: string;
  host: string;
  port: number;
  key_type: string;
  fingerprint_sha256: string;
}

export function HostKeyModal() {
  const [hostKeyRequest, setHostKeyRequest] = useState<HostKeyInfo | null>(null);
  const [deciding, setDeciding] = useState(false);

  useEffect(() => {
    // Listen for host key verification requests
    const unlisten = listen<HostKeyInfo>('ssh:hostkey_request', (event) => {
      console.log('Host key verification request:', event.payload);
      setHostKeyRequest(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleDecision = async (decision: 'once' | 'always' | 'reject') => {
    if (!hostKeyRequest) return;

    setDeciding(true);
    try {
      await invoke('ssh_hostkey_decision', {
        sessionId: hostKeyRequest.session_id,
        decision,
      });
      setHostKeyRequest(null);
    } catch (err) {
      console.error('Failed to send host key decision:', err);
    } finally {
      setDeciding(false);
    }
  };

  if (!hostKeyRequest) return null;

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div 
        className="w-full max-w-lg bg-surface-1 rounded-xl border border-border shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center gap-3 px-6 py-4 border-b border-border bg-warning/10">
          <ShieldAlert className="w-6 h-6 text-warning" />
          <div>
            <h2 className="text-lg font-semibold text-warning">Unknown Host Key</h2>
            <p className="text-sm text-foreground-muted">Verify the server's identity</p>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          <div className="flex items-start gap-3 p-4 rounded-lg bg-surface-2">
            <AlertTriangle className="w-5 h-5 text-warning flex-shrink-0 mt-0.5" />
            <p className="text-sm">
              The authenticity of host <strong className="text-foreground">{hostKeyRequest.host}:{hostKeyRequest.port}</strong> can't be established.
              This is the first time you're connecting to this server.
            </p>
          </div>

          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm text-foreground-muted">
              <Key className="w-4 h-4" />
              <span>Key type: <strong className="text-foreground">{hostKeyRequest.key_type}</strong></span>
            </div>
            <div className="p-3 rounded-lg bg-surface-0 border border-border">
              <p className="text-xs text-foreground-muted mb-1">SHA256 Fingerprint:</p>
              <code className="text-sm font-mono text-accent break-all">
                {hostKeyRequest.fingerprint_sha256}
              </code>
            </div>
          </div>

          <div className="p-3 rounded-lg bg-info/10 border border-info/20 text-sm">
            <p className="text-info">
              <strong>Security Tip:</strong> Verify this fingerprint matches what your server administrator expects.
              If you're unsure, contact your administrator before proceeding.
            </p>
          </div>
        </div>

        {/* Footer */}
        <div className="flex flex-col gap-2 px-6 py-4 border-t border-border bg-surface-0/50">
          <div className="flex gap-2">
            <button
              onClick={() => handleDecision('always')}
              disabled={deciding}
              className="flex-1 btn btn-primary flex items-center justify-center gap-2"
            >
              <ShieldCheck className="w-4 h-4" />
              Trust & Save
            </button>
            <button
              onClick={() => handleDecision('once')}
              disabled={deciding}
              className="flex-1 btn flex items-center justify-center gap-2"
            >
              <ShieldAlert className="w-4 h-4" />
              Trust Once
            </button>
          </div>
          <button
            onClick={() => handleDecision('reject')}
            disabled={deciding}
            className="btn bg-error/10 text-error hover:bg-error/20 border-error/30 flex items-center justify-center gap-2"
          >
            <ShieldX className="w-4 h-4" />
            Reject Connection
          </button>
          <p className="text-xs text-center text-foreground-muted mt-2">
            "Trust & Save" adds this key to your known_hosts file for future connections.
          </p>
        </div>
      </div>
    </div>
  );
}




