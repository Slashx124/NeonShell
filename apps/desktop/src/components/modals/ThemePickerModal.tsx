import { useState } from 'react';
import { X, Check, Upload, FolderOpen, AlertTriangle } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useUIStore } from '@/stores/uiStore';
import { useThemeStore } from '@/stores/themeStore';

interface ThemeImportResult {
  success: boolean;
  theme_id?: string;
  theme_name?: string;
  error?: string;
}

export function ThemePickerModal() {
  const { closeModal, addToast } = useUIStore();
  const { themes, currentTheme, setTheme, listThemes, loadTheme } = useThemeStore();
  const [importing, setImporting] = useState(false);

  const handleSelectTheme = async (themeId: string) => {
    try {
      await setTheme(themeId);
      addToast({
        type: 'success',
        title: 'Theme changed',
      });
      closeModal();
    } catch (error) {
      addToast({
        type: 'error',
        title: 'Failed to change theme',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const handleImportTheme = async () => {
    try {
      // Open file dialog to select a ZIP file
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Theme Package',
          extensions: ['zip']
        }],
        title: 'Select Theme ZIP File',
      });

      if (!selected) {
        return; // User cancelled
      }

      setImporting(true);

      // Call the backend to import and validate the theme
      const result = await invoke<ThemeImportResult>('import_theme_zip', {
        path: selected,
      });

      if (result.success && result.theme_name) {
        addToast({
          type: 'success',
          title: 'Theme imported',
          message: `"${result.theme_name}" has been installed and activated.`,
        });
        
        // Refresh theme list and apply the new theme
        await listThemes();
        await loadTheme();
        closeModal();
      } else {
        addToast({
          type: 'error',
          title: 'Import failed',
          message: result.error || 'Unknown error occurred',
        });
      }
    } catch (error) {
      console.error('Theme import error:', error);
      
      // Parse the error message
      let errorMessage = 'Failed to import theme';
      if (typeof error === 'object' && error !== null) {
        const errObj = error as { message?: string };
        if (errObj.message) {
          errorMessage = errObj.message;
        }
      } else if (typeof error === 'string') {
        errorMessage = error;
      }
      
      addToast({
        type: 'error',
        title: 'Import failed',
        message: errorMessage,
      });
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-md bg-surface-1 rounded-xl border border-border shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 className="text-lg font-semibold text-foreground">Choose Theme</h2>
          <button
            onClick={closeModal}
            className="p-1 rounded hover:bg-surface-2 text-foreground-muted hover:text-foreground transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Theme list */}
        <div className="p-4 max-h-[50vh] overflow-y-auto">
          <div className="space-y-2">
            {themes.map((theme) => (
              <button
                key={theme.id}
                onClick={() => handleSelectTheme(theme.id)}
                className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-colors ${
                  currentTheme?.id === theme.id
                    ? 'border-accent bg-accent/10'
                    : 'border-border hover:bg-surface-2'
                }`}
              >
                {/* Color preview */}
                <div className="flex gap-1">
                  <div
                    className="w-4 h-4 rounded"
                    style={{ backgroundColor: theme.colors.accent }}
                  />
                  <div
                    className="w-4 h-4 rounded"
                    style={{ backgroundColor: theme.colors.background }}
                  />
                  <div
                    className="w-4 h-4 rounded"
                    style={{ backgroundColor: theme.colors.surface_2 }}
                  />
                </div>
                
                <span className="flex-1 text-left text-sm font-medium text-foreground">
                  {theme.name}
                </span>
                
                {currentTheme?.id === theme.id && (
                  <Check className="w-4 h-4 text-accent" />
                )}
              </button>
            ))}
          </div>

          {themes.length === 0 && (
            <div className="text-center py-8 text-foreground-muted">
              No themes available
            </div>
          )}
        </div>

        {/* Import section */}
        <div className="px-4 pb-4">
          <div className="p-3 rounded-lg bg-surface-2/50 border border-border">
            <div className="flex items-start gap-3">
              <Upload className="w-5 h-5 text-accent mt-0.5" />
              <div className="flex-1">
                <h3 className="text-sm font-medium text-foreground mb-1">
                  Import Theme
                </h3>
                <p className="text-xs text-foreground-muted mb-3">
                  Import a theme from a .zip file containing theme.json and optional CSS.
                </p>
                <button
                  onClick={handleImportTheme}
                  disabled={importing}
                  className="btn btn-primary text-sm flex items-center gap-2"
                >
                  {importing ? (
                    <>
                      <span className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                      Importing...
                    </>
                  ) : (
                    <>
                      <FolderOpen className="w-4 h-4" />
                      Select ZIP File
                    </>
                  )}
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Security notice */}
        <div className="px-6 py-3 border-t border-border bg-surface-0/50">
          <div className="flex items-start gap-2 text-xs text-foreground-muted">
            <AlertTriangle className="w-3.5 h-3.5 mt-0.5 flex-shrink-0 text-warning" />
            <span>
              Only import themes from trusted sources. Theme files are validated before installation.
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
