import { 
  X, 
  Plus, 
  PanelLeftClose, 
  PanelLeft,
  Terminal,
  Circle
} from 'lucide-react';
import { useAppStore, Tab } from '@/stores/appStore';
import { clsx } from 'clsx';

export function TabBar() {
  const { 
    tabs, 
    activeTabId, 
    showSidebar,
    removeTab, 
    setActiveTab,
    toggleSidebar,
    setShowConnectionDialog
  } = useAppStore();

  const handleNewTab = () => {
    // Show connection dialog for new connections
    setShowConnectionDialog(true);
  };

  return (
    <div className="flex items-center bg-surface-1 border-b border-border h-10 select-none">
      {/* Sidebar toggle */}
      <button
        onClick={toggleSidebar}
        className="p-2 text-foreground-muted hover:text-foreground hover:bg-surface-2 transition-colors"
        title={showSidebar ? 'Hide sidebar' : 'Show sidebar'}
      >
        {showSidebar ? (
          <PanelLeftClose className="w-4 h-4" />
        ) : (
          <PanelLeft className="w-4 h-4" />
        )}
      </button>

      {/* Tabs */}
      <div className="flex-1 flex items-center overflow-x-auto">
        {tabs.map((tab) => (
          <TabItem
            key={tab.id}
            tab={tab}
            isActive={tab.id === activeTabId}
            onSelect={() => setActiveTab(tab.id)}
            onClose={() => removeTab(tab.id)}
          />
        ))}

        {/* New tab button */}
        <button
          onClick={handleNewTab}
          className="p-2 text-foreground-muted hover:text-foreground hover:bg-surface-2 transition-colors rounded"
          title="New tab"
        >
          <Plus className="w-4 h-4" />
        </button>
      </div>

      {/* Window controls placeholder for custom titlebar if needed */}
      <div className="w-2" />
    </div>
  );
}

function TabItem({
  tab,
  isActive,
  onSelect,
  onClose,
}: {
  tab: Tab;
  isActive: boolean;
  onSelect: () => void;
  onClose: () => void;
}) {
  return (
    <div
      className={clsx(
        'flex items-center gap-2 px-3 py-2 border-r border-border cursor-pointer transition-colors min-w-[120px] max-w-[200px]',
        isActive
          ? 'bg-surface-0 text-foreground'
          : 'bg-surface-1 text-foreground-muted hover:bg-surface-2 hover:text-foreground'
      )}
      onClick={onSelect}
    >
      {/* Status indicator */}
      <div className="flex-shrink-0">
        {tab.connected ? (
          <Circle className="w-2 h-2 fill-success text-success" />
        ) : (
          <Terminal className="w-3 h-3 text-foreground-muted" />
        )}
      </div>

      {/* Title */}
      <span className="flex-1 truncate text-sm">{tab.title}</span>

      {/* Close button */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          onClose();
        }}
        className="flex-shrink-0 p-0.5 rounded hover:bg-surface-3 transition-colors"
      >
        <X className="w-3 h-3" />
      </button>
    </div>
  );
}

