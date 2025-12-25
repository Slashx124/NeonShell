import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { produce } from 'immer';

// =============================================================================
// Types (mirror Rust types)
// =============================================================================

export type ModelSource = 'hosted' | 'org' | 'local' | 'personal';
export type ModelProvider = 'openai' | 'anthropic' | 'google' | 'azure' | 'ollama' | 'openai_compatible' | 'custom';

export interface ModelCapabilities {
  chat: boolean;
  completion: boolean;
  embeddings: boolean;
  vision: boolean;
  function_calling: boolean;
  streaming: boolean;
}

export interface ModelPricing {
  input_per_1m_tokens: number;
  output_per_1m_tokens: number;
  currency: string;
}

export interface Model {
  id: string;
  name: string;
  provider: ModelProvider;
  source: ModelSource;
  model_id: string;
  description?: string;
  context_window: number;
  max_output_tokens?: number;
  capabilities: ModelCapabilities;
  pricing?: ModelPricing;
  enabled: boolean;
  badge?: string;
  endpoint?: string;
}

export interface LocalModelConfig {
  id: string;
  name: string;
  provider: ModelProvider;
  model_id: string;
  endpoint: string;
  enabled: boolean;
}

export interface PersonalKeyConfig {
  id: string;
  provider: ModelProvider;
  name: string;
  key_id: string;
  enabled: boolean;
}

export interface AISettings {
  enable_gateway: boolean;
  gateway_url: string;
  local_models: LocalModelConfig[];
  personal_keys: PersonalKeyConfig[];
  default_model?: string;
  require_tool_approval: boolean;
  auto_mode: boolean;
}

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  name?: string;
  tool_calls?: ToolCall[];
  tool_call_id?: string;
}

export interface ToolCall {
  id: string;
  type: string;
  function: {
    name: string;
    arguments: string;
  };
}

export interface ChatRequest {
  model_id: string;
  messages: ChatMessage[];
  tools?: any[];
  temperature?: number;
  max_tokens?: number;
  stream?: boolean;
}

export interface ChatResponse {
  id: string;
  model: string;
  created: number;
  choices: {
    index: number;
    message: ChatMessage;
    finish_reason: string;
  }[];
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

// =============================================================================
// Store
// =============================================================================

interface AIStoreState {
  // Settings
  settings: AISettings | null;
  settingsLoading: boolean;
  
  // Models
  models: Model[];
  modelsLoading: boolean;
  selectedModel: string | null;
  
  // Gateway auth
  isGatewayAuthenticated: boolean;
  gatewayAuthLoading: boolean;
  deviceLinkCode: string | null;
  
  // Ollama
  ollamaAvailable: boolean;
  
  // Actions
  loadSettings: () => Promise<void>;
  saveSettings: (settings: AISettings) => Promise<void>;
  loadModels: (forceRefresh?: boolean) => Promise<void>;
  selectModel: (modelId: string) => void;
  
  // Gateway auth
  checkGatewayAuth: () => Promise<void>;
  startGatewayAuth: () => Promise<string>;
  pollGatewayAuth: (deviceCode: string) => Promise<boolean>;
  gatewayLogout: () => Promise<void>;
  
  // Personal keys
  addPersonalKey: (provider: ModelProvider, name: string, apiKey: string) => Promise<void>;
  removePersonalKey: (keyId: string) => Promise<void>;
  
  // Ollama
  checkOllama: () => Promise<void>;
  
  // Chat
  chat: (request: ChatRequest) => Promise<ChatResponse>;
}

export const useAIStore = create<AIStoreState>((set, get) => ({
  settings: null,
  settingsLoading: false,
  models: [],
  modelsLoading: false,
  selectedModel: null,
  isGatewayAuthenticated: false,
  gatewayAuthLoading: false,
  deviceLinkCode: null,
  ollamaAvailable: false,

  loadSettings: async () => {
    set({ settingsLoading: true });
    try {
      const settings = await invoke<AISettings>('get_ai_settings');
      set({ settings, selectedModel: settings.default_model ?? null });
    } catch (error) {
      console.error('Failed to load AI settings:', error);
    } finally {
      set({ settingsLoading: false });
    }
  },

  saveSettings: async (settings) => {
    try {
      await invoke('save_ai_settings', { settings });
      set({ settings });
    } catch (error) {
      console.error('Failed to save AI settings:', error);
      throw error;
    }
  },

  loadModels: async (forceRefresh = false) => {
    set({ modelsLoading: true });
    try {
      const models = await invoke<Model[]>('get_models', { forceRefresh });
      set({ models });
      
      // Auto-select first model if none selected
      const { selectedModel } = get();
      if (!selectedModel && models.length > 0) {
        set({ selectedModel: models[0].id });
      }
    } catch (error) {
      console.error('Failed to load models:', error);
    } finally {
      set({ modelsLoading: false });
    }
  },

  selectModel: (modelId) => {
    set({ selectedModel: modelId });
  },

  checkGatewayAuth: async () => {
    try {
      const isAuth = await invoke<boolean>('is_gateway_authenticated');
      set({ isGatewayAuthenticated: isAuth });
    } catch (error) {
      console.error('Failed to check gateway auth:', error);
      set({ isGatewayAuthenticated: false });
    }
  },

  startGatewayAuth: async () => {
    set({ gatewayAuthLoading: true });
    try {
      const response = await invoke<{ user_code: string; device_code: string }>('gateway_auth_start', {
        deviceName: 'NeonShell Desktop',
        platform: navigator.platform.includes('Win') ? 'windows' : 
                  navigator.platform.includes('Mac') ? 'macos' : 'linux',
      });
      set({ deviceLinkCode: response.user_code });
      return response.device_code;
    } catch (error) {
      console.error('Failed to start gateway auth:', error);
      throw error;
    } finally {
      set({ gatewayAuthLoading: false });
    }
  },

  pollGatewayAuth: async (deviceCode) => {
    try {
      const response = await invoke<{ access_token?: string; error?: string }>('gateway_auth_poll', {
        deviceCode,
      });
      
      if (response.access_token) {
        set({ isGatewayAuthenticated: true, deviceLinkCode: null });
        // Refresh models after auth
        get().loadModels(true);
        return true;
      }
      
      return false; // Still pending
    } catch (error) {
      console.error('Gateway auth poll error:', error);
      return false;
    }
  },

  gatewayLogout: async () => {
    try {
      await invoke('gateway_logout');
      set({ isGatewayAuthenticated: false });
      get().loadModels(true);
    } catch (error) {
      console.error('Failed to logout from gateway:', error);
    }
  },

  addPersonalKey: async (provider, name, apiKey) => {
    try {
      const keyId = await invoke<string>('store_personal_key', {
        provider,
        name,
        apiKey,
      });
      
      // Update settings with new key
      const { settings } = get();
      if (settings) {
        const newSettings = produce(settings, (draft) => {
          draft.personal_keys.push({
            id: keyId,
            provider,
            name,
            key_id: `personal:key:${keyId}`,
            enabled: true,
          });
        });
        await get().saveSettings(newSettings);
      }
      
      // Refresh models
      get().loadModels(true);
    } catch (error) {
      console.error('Failed to add personal key:', error);
      throw error;
    }
  },

  removePersonalKey: async (keyId) => {
    try {
      await invoke('delete_personal_key', { keyId });
      
      // Update settings
      const { settings } = get();
      if (settings) {
        const newSettings = produce(settings, (draft) => {
          draft.personal_keys = draft.personal_keys.filter((k) => k.id !== keyId);
        });
        await get().saveSettings(newSettings);
      }
      
      // Refresh models
      get().loadModels(true);
    } catch (error) {
      console.error('Failed to remove personal key:', error);
      throw error;
    }
  },

  checkOllama: async () => {
    try {
      const available = await invoke<boolean>('check_ollama');
      set({ ollamaAvailable: available });
    } catch {
      set({ ollamaAvailable: false });
    }
  },

  chat: async (request) => {
    try {
      return await invoke<ChatResponse>('ai_chat', { request });
    } catch (error) {
      console.error('Chat error:', error);
      throw error;
    }
  },
}));




