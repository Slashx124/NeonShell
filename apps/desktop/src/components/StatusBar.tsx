import { 
  Wifi, 
  WifiOff, 
  Clock, 
  Cpu,
  Settings
} from 'lucide-react';
import { useAppStore } from '@/stores/appStore';
import { useSessionStore } from '@/stores/sessionStore';
import { useUIStore } from '@/stores/uiStore';
import { useEffect, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';

export function StatusBar() {
  const { tabs, activeTabId } = useAppStore();
  const { sessions } = useSessionStore();
  const { openModal } = useUIStore();
  const [time, setTime] = useState(new Date());
  const [appVersion, setAppVersion] = useState('');

  // Get app version from Tauri
  useEffect(() => {
    getVersion().then((version) => {
      setAppVersion(`v${version}`);
    }).catch(() => {
      setAppVersion('v0.2.2'); // Fallback
    });
  }, []);

  const activeTab = tabs.find((t) => t.id === activeTabId);
  const activeSession = activeTab?.sessionId 
    ? sessions.get(activeTab.sessionId) 
    : null;

  // Detect platform for shortcut display
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  const settingsShortcut = isMac ? '⌘,' : 'Ctrl+,';

  // Update time every minute
  useEffect(() => {
    const interval = setInterval(() => {
      setTime(new Date());
    }, 60000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="h-6 flex items-center px-3 bg-surface-1 border-t border-border text-xs text-foreground-muted select-none">
      {/* Left side - Version */}
      <div className="flex items-center gap-4 flex-1">
        <span className="font-medium">NeonShell {appVersion}</span>
        
        {/* Connection status */}
        {activeSession ? (
          <div className="flex items-center gap-1.5">
            {activeSession.state === 'Connected' ? (
              <>
                <Wifi className="w-3 h-3 text-success" />
                <span className="text-success">Connected</span>
              </>
            ) : activeSession.state === 'Connecting' ? (
              <>
                <Wifi className="w-3 h-3 text-warning animate-pulse" />
                <span className="text-warning">Connecting...</span>
              </>
            ) : (
              <>
                <WifiOff className="w-3 h-3 text-error" />
                <span className="text-error">Disconnected</span>
              </>
            )}
          </div>
        ) : (
          <div className="flex items-center gap-1.5">
            <WifiOff className="w-3 h-3" />
            <span>No connection</span>
          </div>
        )}

        {/* Host info */}
        {activeSession && (
          <div className="flex items-center gap-1.5">
            <Cpu className="w-3 h-3" />
            <span>{activeSession.username}@{activeSession.host}:{activeSession.port}</span>
          </div>
        )}
      </div>

      {/* Center - Settings shortcut */}
      <button
        onClick={() => openModal('settings')}
        className="flex items-center gap-1.5 px-2 py-0.5 rounded hover:bg-surface-2 hover:text-foreground transition-colors"
        title={`Open Settings (${settingsShortcut})`}
      >
        <Settings className="w-3 h-3" />
        <span>Settings</span>
        <kbd className="px-1 py-0.5 text-[10px] bg-surface-2 rounded font-mono">{settingsShortcut}</kbd>
      </button>

      {/* Right side */}
      <div className="flex items-center gap-4 flex-1 justify-end">
        {/* Time */}
        <div className="flex items-center gap-1.5">
          <Clock className="w-3 h-3" />
          <span>{time.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
        </div>
      </div>
    </div>
  );
}

