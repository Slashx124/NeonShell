import { useUIStore } from '@/stores/uiStore';
import { 
  SettingsModal, 
  ThemePickerModal, 
  PluginManager, 
  ScriptsManager, 
  HotkeysModal,
  DebugConsoleModal,
  AISettingsModal,
} from './modals';

export function ModalHost() {
  const { activeModal } = useUIStore();

  if (!activeModal) return null;

  switch (activeModal) {
    case 'settings':
      return <SettingsModal />;
    case 'themePicker':
      return <ThemePickerModal />;
    case 'plugins':
      return <PluginManager />;
    case 'scripts':
      return <ScriptsManager />;
    case 'hotkeys':
      return <HotkeysModal />;
    case 'debugConsole':
      return <DebugConsoleModal />;
    case 'aiSettings':
      return <AISettingsModal />;
    default:
      return null;
  }
}

