import { Bell, CheckCheck, X } from "lucide-react";
import { useNotificationActions } from "../../hooks/useNotifications";
import { useUiStore } from "../../stores/uiStore";
import "./NotificationPanel.css";

export function NotificationPanel() {
  const open = useUiStore((s) => s.notificationsOpen);
  const setOpen = useUiStore((s) => s.setNotificationsOpen);
  const { items, total, openPane, clearAll, timeAgo } = useNotificationActions();

  if (!open) {
    return null;
  }

  return (
    <div className="panel-backdrop" onMouseDown={() => setOpen(false)}>
      <div className="notification-panel" onMouseDown={(e) => e.stopPropagation()}>
        <div className="panel-header">
          <span className="panel-title">
            <Bell size={13} />
            Notifications {total > 0 && <span className="panel-count">{total}</span>}
          </span>
          <div className="panel-actions">
            <button className="icon-button" title="Mark all read" onClick={clearAll}>
              <CheckCheck size={14} />
            </button>
            <button className="icon-button" title="Close" onClick={() => setOpen(false)}>
              <X size={14} />
            </button>
          </div>
        </div>
        <div className="notification-list">
          {items.length === 0 && <div className="notification-empty">No notifications yet</div>}
          {items.map((n) => (
            <button
              key={n.key}
              className="notification-item"
              onClick={() => {
                openPane(n.workspaceId, n.paneId);
                setOpen(false);
              }}
            >
              <div className="notification-item-title">
                <span>{n.title}</span>
                <span className="notification-item-time">{timeAgo(n.at)}</span>
              </div>
              <div className="notification-item-message">{n.message}</div>
              <div className="notification-item-kind">{n.kind}</div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
