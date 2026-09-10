import { create } from "zustand";
import type { Settings, ThemeMeta } from "../types";
import { DEFAULT_BINDINGS, type ActionId } from "../lib/keybindings";
import { getSettings, isTauri, listThemes, setSettings } from "../lib/tauri-bridge";

export const defaultSettings: Settings = {
  themeId: "midnight",
  fontSize: 13,
  fontFamily: "Cascadia Code, Cascadia Mono, Consolas, monospace",
  shell: null,
  autoRestore: true,
  agentAutoResume: true,
  notifications: { rings: true, toasts: true, badges: true },
  keybindings: { ...DEFAULT_BINDINGS },
  sidebarVisible: true,
};

interface SettingsStore {
  settings: Settings;
  customThemes: ThemeMeta[];
  loaded: boolean;
  load: () => Promise<void>;
  update: (patch: Partial<Settings>) => Promise<void>;
  resetBinding: (action: ActionId) => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: defaultSettings,
  customThemes: [],
  loaded: false,
  load: async () => {
    if (!isTauri() || get().loaded) {
      return;
    }
    set({ loaded: true });
    try {
      const [settings, themes] = await Promise.all([getSettings(), listThemes()]);
      set({
        settings: { ...defaultSettings, ...settings, keybindings: { ...DEFAULT_BINDINGS, ...settings.keybindings } },
        customThemes: themes,
      });
    } catch (e) {
      console.error("failed to load settings", e);
    }
  },
  update: async (patch) => {
    const next = { ...get().settings, ...patch };
    set({ settings: next });
    if (isTauri()) {
      try {
        await setSettings(next);
      } catch (e) {
        console.error("failed to save settings", e);
      }
    }
  },
  resetBinding: async (action) => {
    const current = { ...get().settings.keybindings };
    current[action] = DEFAULT_BINDINGS[action];
    await get().update({ keybindings: current });
  },
}));

export function bindings(): Record<ActionId, string[]> {
  return useSettingsStore.getState().settings.keybindings as Record<ActionId, string[]>;
}