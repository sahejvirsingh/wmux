import { create } from "zustand";
import type { StoredNotification } from "../lib/notifications";

interface UiStore {
  sidebarVisible: boolean;
  setSidebarVisible: (visible: boolean) => void;
  toggleSidebar: () => void;
  paletteOpen: boolean;
  setPaletteOpen: (open: boolean) => void;
  settingsOpen: boolean;
  setSettingsOpen: (open: boolean) => void;
  sshOpen: boolean;
  setSshOpen: (open: boolean) => void;
  notificationsOpen: boolean;
  setNotificationsOpen: (open: boolean) => void;
  pendingChord: string[] | null;
  setPendingChord: (chord: string[] | null) => void;
  searchPaneId: string | null;
  setSearchPaneId: (paneId: string | null) => void;
  notifications: StoredNotification[];
  pushNotification: (n: StoredNotification) => void;
  clearNotifications: () => void;
}

export const useUiStore = create<UiStore>((set) => ({
  sidebarVisible: true,
  setSidebarVisible: (visible) => set({ sidebarVisible: visible }),
  toggleSidebar: () => set((s) => ({ sidebarVisible: !s.sidebarVisible })),
  paletteOpen: false,
  setPaletteOpen: (open) => set({ paletteOpen: open }),
  settingsOpen: false,
  setSettingsOpen: (open) => set({ settingsOpen: open }),
  sshOpen: false,
  setSshOpen: (open) => set({ sshOpen: open }),
  notificationsOpen: false,
  setNotificationsOpen: (open) => set({ notificationsOpen: open }),
  pendingChord: null,
  setPendingChord: (chord) => set({ pendingChord: chord }),
  searchPaneId: null,
  setSearchPaneId: (paneId) => set({ searchPaneId: paneId }),
  notifications: [],
  pushNotification: (n) => set((s) => ({ notifications: [n, ...s.notifications].slice(0, 100) })),
  clearNotifications: () => set({ notifications: [] }),
}));