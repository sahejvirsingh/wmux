import { create } from "zustand";
import type { AppStateJson } from "../types";
import { getState, isTauri, onStateChange } from "../lib/tauri-bridge";

interface AppStore {
  state: AppStateJson | null;
  subscribed: boolean;
  setState: (state: AppStateJson) => void;
  init: () => Promise<void>;
}

export const useAppStore = create<AppStore>((set, get) => ({
  state: null,
  subscribed: false,
  setState: (state) => set({ state }),
  init: async () => {
    if (!isTauri() || get().subscribed) {
      return;
    }
    set({ subscribed: true });
    try {
      const state = await getState();
      set({ state });
    } catch (e) {
      console.error("failed to load initial state", e);
    }
    onStateChange((state) => set({ state })).catch((e) =>
      console.error("failed to subscribe to state changes", e),
    );
  },
}));

export function activeWorkspace(): AppStateJson["workspaces"][number] | null {
  const state = useAppStore.getState().state;
  if (!state || !state.activeWorkspace) {
    return null;
  }
  return state.workspaces.find((w) => w.id === state.activeWorkspace) ?? null;
}