import { useEffect } from "react";
import { advanceChord, eventToCombo, type ActionId } from "../lib/keybindings";
import { useAppStore } from "../stores/appStore";
import { bindings } from "../stores/settingsStore";
import { useUiStore } from "../stores/uiStore";
import {
  closePane,
  createWorkspace,
  navigatePanes,
  selectWorkspace,
  sessionRestore,
  splitPane,
  zoomPane,
} from "../lib/tauri-bridge";
import { activeWorkspace } from "../stores/appStore";

export function useKeybindings(): void {
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        return;
      }
      const combo = eventToCombo(event);
      const ui = useUiStore.getState();
      const result = advanceChord(bindings(), { pending: ui.pendingChord }, combo);
      if (ui.pendingChord !== result.state.pending) {
        ui.setPendingChord(result.state.pending);
      }
      const action = result.action as ActionId | null;
      if (!action) {
        return;
      }

      const workspace = activeWorkspace();
      switch (action) {
        case "commandPalette":
        case "workspaceSwitcher":
          useUiStore.getState().setPaletteOpen(true);
          break;
        case "newWorkspace":
          void createWorkspace();
          break;
        case "toggleSidebar":
          useUiStore.getState().toggleSidebar();
          break;
        case "closePane":
          if (workspace && workspace.activePane) {
            void closePane(workspace.id, workspace.activePane);
          }
          break;
        case "newSurface":
        case "splitRight":
          if (workspace) {
            void splitPane(workspace.id, "vertical");
          }
          break;
        case "splitDown":
          if (workspace) {
            void splitPane(workspace.id, "horizontal");
          }
          break;
        case "nextPane":
          if (workspace) {
            void navigatePanes(workspace.id, "next");
          }
          break;
        case "prevPane":
          if (workspace) {
            void navigatePanes(workspace.id, "prev");
          }
          break;
        case "nextWorkspace": {
          const state = useAppStore.getState().state;
          if (state && state.workspaces.length > 0) {
            const idx = Math.max(
              0,
              state.workspaces.findIndex((w) => w.id === state.activeWorkspace),
            );
            const next = state.workspaces[(idx + 1) % state.workspaces.length];
            if (next && next.id !== state.activeWorkspace) {
              void selectWorkspace(next.id);
            }
          }
          break;
        }
        case "prevWorkspace": {
          const state = useAppStore.getState().state;
          if (state && state.workspaces.length > 0) {
            const idx = Math.max(
              0,
              state.workspaces.findIndex((w) => w.id === state.activeWorkspace),
            );
            const prev = state.workspaces[(idx + state.workspaces.length - 1) % state.workspaces.length];
            if (prev && prev.id !== state.activeWorkspace) {
              void selectWorkspace(prev.id);
            }
          }
          break;
        }
        case "zoomPane":
          if (workspace) {
            void zoomPane(workspace.id, workspace.activePane ?? undefined);
          }
          break;
        case "findInTerminal":
          useUiStore.getState().setSearchPaneId(workspace?.activePane ?? null);
          break;
        case "showNotifications":
          useUiStore.getState().setNotificationsOpen(!useUiStore.getState().notificationsOpen);
          break;
        case "restoreSession":
          void sessionRestore();
          break;
        case "openSettings":
          useUiStore.getState().setSettingsOpen(true);
          break;
        default:
          break;
      }
      event.preventDefault();
      event.stopPropagation();
    };
    window.addEventListener("keydown", handler, { capture: true });
    return () => window.removeEventListener("keydown", handler, { capture: true });
  }, []);
}