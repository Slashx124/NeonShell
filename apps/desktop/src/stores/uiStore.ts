import { create } from 'zustand';
import { nanoid } from 'nanoid';

export type ModalType = 
  | 'settings' 
  | 'themePicker' 
  | 'plugins' 
  | 'scripts' 
  | 'hotkeys'
  | 'debugConsole'
  | null;

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
}

interface UIState {
  // Modal state
  activeModal: ModalType;
  openModal: (modal: ModalType) => void;
  closeModal: () => void;
  
  // Toast state
  toasts: Toast[];
  addToast: (toast: Omit<Toast, 'id'>) => string;
  removeToast: (id: string) => void;
}

export const useUIStore = create<UIState>((set, get) => ({
  // Modal state
  activeModal: null,
  
  openModal: (modal) => set({ activeModal: modal }),
  closeModal: () => set({ activeModal: null }),
  
  // Toast state
  toasts: [],
  
  addToast: (toast) => {
    const id = nanoid();
    const newToast: Toast = {
      id,
      duration: 4000,
      ...toast,
    };
    
    set((state) => ({
      toasts: [...state.toasts, newToast],
    }));
    
    // Auto-remove after duration
    if (newToast.duration && newToast.duration > 0) {
      setTimeout(() => {
        get().removeToast(id);
      }, newToast.duration);
    }
    
    return id;
  },
  
  removeToast: (id) => {
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    }));
  },
}));

