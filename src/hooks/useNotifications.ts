import { clearNotifications, focusPane, selectWorkspace } from "../lib/tauri-bridge";
import { storeNotification, timeAgo } from "../lib/notifications";
import type { AppNotification } from "../types";
import { useAppStore } from "../stores/appStore";
import { useUiStore } from "../stores/uiStore";

export function useNotificationSink(workspaceId: string, paneId: string, ptyId: string) {
  const push = useUiStore((s) => s.pushNotification);
  return (notification: AppNotification) => {
    push(storeNotification(notification, workspaceId, paneId, ptyId));
  };
}

export function useNotificationActions() {
  const state = useAppStore((s) => s.state);
  const items = useUiStore((s) => s.notifications);
  const clearLocal = useUiStore((s) => s.clearNotifications);
  const setNotificationsOpen = useUiStore((s) => s.setNotificationsOpen);

  const total = state?.workspaces.reduce((sum, w) => sum + w.notifications, 0) ?? 0;

  const openPane = (workspaceId: string, paneId: string) => {
    void selectWorkspace(workspaceId).then(() => focusPane(workspaceId, paneId));
  };

  const clearAll = () => {
    clearLocal();
    for (const ws of state?.workspaces ?? []) {
      if (ws.notifications > 0) {
        void clearNotifications(ws.id);
      }
    }
  };

  return { items, total, openPane, clearAll, setNotificationsOpen, timeAgo };
}