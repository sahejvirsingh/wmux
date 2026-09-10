import { useCallback } from "react";
import {
  clearNotifications,
  closePane,
  createWorkspace,
  deleteWorkspace,
  focusPane,
  navigatePanes,
  renameWorkspace,
  reorderWorkspaces,
  selectWorkspace,
  splitPane,
  zoomPane,
} from "../lib/tauri-bridge";
import { useAppStore } from "../stores/appStore";

export function useWorkspace() {
  const state = useAppStore((s) => s.state);
  const workspaces = state?.workspaces ?? [];
  const activeId = state?.activeWorkspace ?? null;
  const active = workspaces.find((w) => w.id === activeId) ?? null;

  const create = useCallback((name?: string) => createWorkspace(name), []);
  const remove = useCallback((id: string) => deleteWorkspace(id), []);
  const rename = useCallback((id: string, name: string) => renameWorkspace(id, name), []);
  const select = useCallback((id: string) => selectWorkspace(id), []);
  const reorder = useCallback((ids: string[]) => reorderWorkspaces(ids), []);
  const clear = useCallback(
    (workspaceId: string, paneId?: string) => clearNotifications(workspaceId, paneId),
    [],
  );

  return { workspaces, activeId, active, create, remove, rename, select, reorder, clear };
}

export function usePaneLayout(workspaceId: string | null) {
  const split = useCallback(
    (direction: "horizontal" | "vertical", command?: string, cwd?: string) =>
      workspaceId ? splitPane(workspaceId, direction, command, cwd) : Promise.resolve(),
    [workspaceId],
  );
  const close = useCallback(
    (paneId: string) => (workspaceId ? closePane(workspaceId, paneId) : Promise.resolve()),
    [workspaceId],
  );
  const focus = useCallback(
    (paneId: string) => (workspaceId ? focusPane(workspaceId, paneId) : Promise.resolve()),
    [workspaceId],
  );
  const navigate = useCallback(
    (direction: "next" | "prev") =>
      workspaceId ? navigatePanes(workspaceId, direction) : Promise.resolve(),
    [workspaceId],
  );
  const zoom = useCallback(
    (paneId?: string) => (workspaceId ? zoomPane(workspaceId, paneId) : Promise.resolve()),
    [workspaceId],
  );

  return { split, close, focus, navigate, zoom };
}