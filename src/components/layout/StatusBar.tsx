import { Bell, GitBranch, HardDrive, Save } from "lucide-react";
import { useAppStore } from "../../stores/appStore";
import { useNotificationActions } from "../../hooks/useNotifications";
import { sessionSave } from "../../lib/tauri-bridge";
import "./StatusBar.css";

export function StatusBar() {
  const appState = useAppStore((s) => s.state);
  const { total, setNotificationsOpen } = useNotificationActions();
  const active = appState?.workspaces.find((w) => w.id === appState.activeWorkspace);

  return (
    <footer className="statusbar">
      <div className="statusbar-left">
        {active ? (
          <>
            <span className="statusbar-item" title={active.cwd}>
              <HardDrive size={11} />
              {active.cwd || "—"}
            </span>
            {active.branch && (
              <span className="statusbar-item">
                <GitBranch size={11} />
                {active.branch}
              </span>
            )}
            {active.ports.length > 0 && (
              <span className="statusbar-item" title={active.ports.join(", ")}>
                ports: {active.ports.slice(0, 6).join(", ")}
                {active.ports.length > 6 ? "…" : ""}
              </span>
            )}
          </>
        ) : (
          <span className="statusbar-item muted">no active workspace</span>
        )}
      </div>
      <div className="statusbar-right">
        <button
          className={`statusbar-button ${total > 0 ? "has-notifications" : ""}`}
          onClick={() => setNotificationsOpen(true)}
          title="Notifications (Ctrl+Shift+I)"
        >
          <Bell size={12} />
          {total > 0 ? total : ""}
        </button>
        <button
          className="statusbar-button"
          onClick={() => void sessionSave()}
          title="Save session now"
        >
          <Save size={12} />
        </button>
      </div>
    </footer>
  );
}
