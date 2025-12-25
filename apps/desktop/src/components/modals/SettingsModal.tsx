import { useState, useEffect } from 'react';
import { X, Save } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useUIStore } from '@/stores/uiStore';
import { useSettingsStore } from '@/stores/settingsStore';

interface Settings {
  general: {
    theme: string;
    language: string;
    check_updates: boolean;
    start_minimized: boolean;
    restore_sessions: boolean;
  };
  terminal: {
    font_family: string;
    font_size: number;
    cursor_style: string;
    cursor_blink: boolean;
    scrollback: number;
    copy_on_select: boolean;
    bell_sound: boolean;
    bell_visual: boolean;
  };
  ssh: {
    default_port: number;
    keepalive_interval: number;
    strict_host_checking: boolean;
    agent_forwarding: boolean;
    compression: boolean;
  };
  security: {
    store_passwords: string;
    auto_lock_timeout: number;
    clear_clipboard: boolean;
    clipboard_timeout: number;
  };
}

export function SettingsModal() {
  const { closeModal, addToast } = useUIStore();
  const { loadSettings: reloadGlobalSettings } = useSettingsStore();
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [activeTab, setActiveTab] = useState<'general' | 'terminal' | 'ssh' | 'security'>('general');

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const data = await invoke<Settings>('get_settings');
      setSettings(data);
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Failed to load settings',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async () => {
    if (!settings) return;
    
    setSaving(true);
    try {
      await invoke('save_settings', { settings });
      // Reload settings in the global store so they take effect immediately
      await reloadGlobalSettings();
      addToast({
        type: 'success',
        title: 'Settings saved',
        message: 'Some settings may require reconnecting to take effect.',
      });
      closeModal();
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Failed to save settings',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setSaving(false);
    }
  };

  const updateSetting = <K extends keyof Settings>(
    section: K,
    key: keyof Settings[K],
    value: Settings[K][keyof Settings[K]]
  ) => {
    if (!settings) return;
    setSettings({
      ...settings,
      [section]: {
        ...settings[section],
        [key]: value,
      },
    });
  };

  const tabs = [
    { id: 'general' as const, label: 'General' },
    { id: 'terminal' as const, label: 'Terminal' },
    { id: 'ssh' as const, label: 'SSH' },
    { id: 'security' as const, label: 'Security' },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-2xl bg-surface-1 rounded-xl border border-border shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 className="text-lg font-semibold text-foreground">Settings</h2>
          <button
            onClick={closeModal}
            className="p-1 rounded hover:bg-surface-2 text-foreground-muted hover:text-foreground transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-border px-4">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-3 text-sm font-medium border-b-2 transition-colors ${
                activeTab === tab.id
                  ? 'border-accent text-accent'
                  : 'border-transparent text-foreground-muted hover:text-foreground'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="p-6 max-h-[60vh] overflow-y-auto">
          {loading ? (
            <div className="text-center py-8 text-foreground-muted">Loading...</div>
          ) : !settings ? (
            <div className="text-center py-8 text-error">Failed to load settings</div>
          ) : (
            <>
              {activeTab === 'general' && (
                <div className="space-y-4">
                  <SettingRow label="Check for updates">
                    <Toggle
                      checked={settings.general.check_updates}
                      onChange={(v) => updateSetting('general', 'check_updates', v)}
                    />
                  </SettingRow>
                  <SettingRow label="Start minimized">
                    <Toggle
                      checked={settings.general.start_minimized}
                      onChange={(v) => updateSetting('general', 'start_minimized', v)}
                    />
                  </SettingRow>
                  <SettingRow label="Restore sessions on startup">
                    <Toggle
                      checked={settings.general.restore_sessions}
                      onChange={(v) => updateSetting('general', 'restore_sessions', v)}
                    />
                  </SettingRow>
                </div>
              )}

              {activeTab === 'terminal' && (
                <div className="space-y-4">
                  <SettingRow label="Font family">
                    <input
                      type="text"
                      value={settings.terminal.font_family}
                      onChange={(e) => updateSetting('terminal', 'font_family', e.target.value)}
                      className="w-48 px-3 py-1.5 bg-surface-2 border border-border rounded text-sm text-foreground"
                    />
                  </SettingRow>
                  <SettingRow label="Font size">
                    <input
                      type="number"
                      min={8}
                      max={32}
                      value={settings.terminal.font_size}
                      onChange={(e) => updateSetting('terminal', 'font_size', parseInt(e.target.value) || 14)}
                      className="w-20 px-3 py-1.5 bg-surface-2 border border-border rounded text-sm text-foreground"
                    />
                  </SettingRow>
                  <SettingRow label="Cursor style">
                    <select
                      value={settings.terminal.cursor_style}
                      onChange={(e) => updateSetting('terminal', 'cursor_style', e.target.value)}
                      className="w-32 px-3 py-1.5 bg-surface-2 border border-border rounded text-sm text-foreground"
                    >
                      <option value="block">Block</option>
                      <option value="underline">Underline</option>
                      <option value="bar">Bar</option>
                    </select>
                  </SettingRow>
                  <SettingRow label="Cursor blink">
                    <Toggle
                      checked={settings.terminal.cursor_blink}
                      onChange={(v) => updateSetting('terminal', 'cursor_blink', v)}
                    />
                  </SettingRow>
                  <SettingRow label="Scrollback lines">
                    <input
                      type="number"
                      min={100}
                      max={100000}
                      value={settings.terminal.scrollback}
                      onChange={(e) => updateSetting('terminal', 'scrollback', parseInt(e.target.value) || 10000)}
                      className="w-28 px-3 py-1.5 bg-surface-2 border border-border rounded text-sm text-foreground"
                    />
                  </SettingRow>
                  <SettingRow label="Copy on select">
                    <Toggle
                      checked={settings.terminal.copy_on_select}
                      onChange={(v) => updateSetting('terminal', 'copy_on_select', v)}
                    />
                  </SettingRow>
                  <SettingRow label="Bell sound">
                    <Toggle
                      checked={settings.terminal.bell_sound}
                      onChange={(v) => updateSetting('terminal', 'bell_sound', v)}
                    />
                  </SettingRow>
                </div>
              )}

              {activeTab === 'ssh' && (
                <div className="space-y-4">
                  <SettingRow label="Default port">
                    <input
                      type="number"
                      min={1}
                      max={65535}
                      value={settings.ssh.default_port}
                      onChange={(e) => updateSetting('ssh', 'default_port', parseInt(e.target.value) || 22)}
                      className="w-24 px-3 py-1.5 bg-surface-2 border border-border rounded text-sm text-foreground"
                    />
                  </SettingRow>
                  <SettingRow label="Keepalive interval (seconds)">
                    <input
                      type="number"
                      min={0}
                      max={3600}
                      value={settings.ssh.keepalive_interval}
                      onChange={(e) => updateSetting('ssh', 'keepalive_interval', parseInt(e.target.value) || 60)}
                      className="w-24 px-3 py-1.5 bg-surface-2 border border-border rounded text-sm text-foreground"
                    />
                  </SettingRow>
                  <SettingRow label="Strict host key checking">
                    <Toggle
                      checked={settings.ssh.strict_host_checking}
                      onChange={(v) => updateSetting('ssh', 'strict_host_checking', v)}
                    />
                  </SettingRow>
                  <SettingRow label="Agent forwarding">
                    <Toggle
                      checked={settings.ssh.agent_forwarding}
                      onChange={(v) => updateSetting('ssh', 'agent_forwarding', v)}
                    />
                  </SettingRow>
                  <SettingRow label="Compression">
                    <Toggle
                      checked={settings.ssh.compression}
                      onChange={(v) => updateSetting('ssh', 'compression', v)}
                    />
                  </SettingRow>
                </div>
              )}

              {activeTab === 'security' && (
                <div className="space-y-4">
                  <SettingRow label="Password storage">
                    <select
                      value={settings.security.store_passwords}
                      onChange={(e) => updateSetting('security', 'store_passwords', e.target.value)}
                      className="w-36 px-3 py-1.5 bg-surface-2 border border-border rounded text-sm text-foreground"
                    >
                      <option value="keychain">OS Keychain</option>
                      <option value="never">Never</option>
                    </select>
                  </SettingRow>
                  <SettingRow label="Auto-lock timeout (seconds)">
                    <input
                      type="number"
                      min={0}
                      max={3600}
                      value={settings.security.auto_lock_timeout}
                      onChange={(e) => updateSetting('security', 'auto_lock_timeout', parseInt(e.target.value) || 300)}
                      className="w-24 px-3 py-1.5 bg-surface-2 border border-border rounded text-sm text-foreground"
                    />
                  </SettingRow>
                  <SettingRow label="Clear clipboard after copy">
                    <Toggle
                      checked={settings.security.clear_clipboard}
                      onChange={(v) => updateSetting('security', 'clear_clipboard', v)}
                    />
                  </SettingRow>
                </div>
              )}
            </>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 px-6 py-4 border-t border-border">
          <button
            onClick={closeModal}
            className="px-4 py-2 text-sm text-foreground-muted hover:text-foreground transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={saveSettings}
            disabled={saving || !settings}
            className="flex items-center gap-2 px-4 py-2 bg-accent text-white rounded-lg hover:bg-accent/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <Save className="w-4 h-4" />
            {saving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  );
}

function SettingRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between py-2">
      <span className="text-sm text-foreground">{label}</span>
      {children}
    </div>
  );
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`w-10 h-5 rounded-full transition-colors ${
        checked ? 'bg-accent' : 'bg-surface-3'
      }`}
    >
      <div
        className={`w-4 h-4 rounded-full bg-white transition-transform ${
          checked ? 'translate-x-5' : 'translate-x-0.5'
        }`}
      />
    </button>
  );
}




