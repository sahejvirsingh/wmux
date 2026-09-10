import { KeyRound, Palette, Settings2, Terminal, Webhook } from "lucide-react";
import { useEffect, useState } from "react";
import type { AgentStatus, InstallReport } from "../../types";
import { hooksInstall, hooksStatus } from "../../lib/tauri-bridge";
import { useSettingsStore } from "../../stores/settingsStore";
import { useUiStore } from "../../stores/uiStore";
import { KeybindingEditor } from "./KeybindingEditor";
import { ThemeEditor } from "./ThemeEditor";
import "./SettingsDialog.css";

type Tab = "general" | "appearance" | "notifications" | "keybindings" | "agents";

const TABS: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: "general", label: "General", icon: <Settings2 size={13} /> },
  { id: "appearance", label: "Appearance", icon: <Palette size={13} /> },
  { id: "notifications", label: "Notifications", icon: <Terminal size={13} /> },
  { id: "keybindings", label: "Keybindings", icon: <KeyRound size={13} /> },
  { id: "agents", label: "Agent hooks", icon: <Webhook size={13} /> },
];

export function SettingsDialog() {
  const open = useUiStore((s) => s.settingsOpen);
  const setOpen = useUiStore((s) => s.setSettingsOpen);
  const settings = useSettingsStore((s) => s.settings);
  const update = useSettingsStore((s) => s.update);
  const [tab, setTab] = useState<Tab>("general");
  const [agents, setAgents] = useState<AgentStatus[] | null>(null);
  const [reports, setReports] = useState<InstallReport[]>([]);

  useEffect(() => {
    if (open && agents === null) {
      hooksStatus()
        .then(setAgents)
        .catch(() => setAgents([]));
    }
  }, [open, agents]);

  if (!open) {
    return null;
  }

  const install = (agent?: string) => {
    hooksInstall(agent)
      .then(setReports)
      .catch(() => setReports([]));
    hooksStatus()
      .then(setAgents)
      .catch(() => {});
  };

  return (
    <div className="panel-backdrop settings-backdrop" onMouseDown={() => setOpen(false)}>
      <div className="settings-dialog" onMouseDown={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <span>Settings</span>
          <button className="icon-button" onClick={() => setOpen(false)}>
            ✕
          </button>
        </div>
        <div className="settings-body">
          <nav className="settings-nav">
            {TABS.map((t) => (
              <button
                key={t.id}
                className={`settings-nav-item ${tab === t.id ? "active" : ""}`}
                onClick={() => setTab(t.id)}
              >
                {t.icon}
                {t.label}
              </button>
            ))}
          </nav>
          <div className="settings-content">
            {tab === "general" && (
              <div className="settings-fields">
                <label className="settings-field">
                  <span>Shell (leave empty to auto-detect pwsh → powershell → cmd)</span>
                  <input
                    value={settings.shell ?? ""}
                    placeholder="C:\Program Files\PowerShell\7\pwsh.exe"
                    onChange={(e) => void update({ shell: e.target.value || null })}
                  />
                </label>
                <label className="settings-field">
                  <span>Font size</span>
                  <input
                    type="number"
                    min={8}
                    max={32}
                    value={settings.fontSize}
                    onChange={(e) => void update({ fontSize: Number(e.target.value) || 13 })}
                  />
                </label>
                <label className="settings-field">
                  <span>Font family</span>
                  <input
                    value={settings.fontFamily}
                    onChange={(e) => void update({ fontFamily: e.target.value })}
                  />
                </label>
                <label className="settings-toggle">
                  <input
                    type="checkbox"
                    checked={settings.autoRestore}
                    onChange={(e) => void update({ autoRestore: e.target.checked })}
                  />
                  <span>Restore previous session on launch</span>
                </label>
                <label className="settings-toggle">
                  <input
                    type="checkbox"
                    checked={settings.agentAutoResume}
                    onChange={(e) => void update({ agentAutoResume: e.target.checked })}
                  />
                  <span>Auto-resume detected agent sessions on restore</span>
                </label>
              </div>
            )}
            {tab === "appearance" && (
              <div>
                <ThemeEditor />
              </div>
            )}
            {tab === "notifications" && (
              <div className="settings-fields">
                <label className="settings-toggle">
                  <input
                    type="checkbox"
                    checked={settings.notifications.rings}
                    onChange={(e) =>
                      void update({ notifications: { ...settings.notifications, rings: e.target.checked } })
                    }
                  />
                  <span>Notification rings on panes</span>
                </label>
                <label className="settings-toggle">
                  <input
                    type="checkbox"
                    checked={settings.notifications.badges}
                    onChange={(e) =>
                      void update({ notifications: { ...settings.notifications, badges: e.target.checked } })
                    }
                  />
                  <span>Unread badges on workspace tabs</span>
                </label>
                <label className="settings-toggle">
                  <input
                    type="checkbox"
                    checked={settings.notifications.toasts}
                    onChange={(e) =>
                      void update({ notifications: { ...settings.notifications, toasts: e.target.checked } })
                    }
                  />
                  <span>Windows toast notifications</span>
                </label>
              </div>
            )}
            {tab === "keybindings" && <KeybindingEditor />}
            {tab === "agents" && (
              <div className="agents-panel">
                <p className="settings-note">
                  Detected coding agents. Hooks run <code>wmux notify</code> so panes light up when
                  the agent needs attention.
                </p>
                {(agents ?? []).map((agent) => (
                  <div key={agent.name} className="agent-row">
                    <div className="agent-info">
                      <span className="agent-name">{agent.name}</span>
                      <span className="agent-status">
                        {agent.installed ? (agent.binaryPath ?? "installed") : "not installed"}
                      </span>
                      {agent.installed && (
                        <span className={`agent-hooks ${agent.hooksInstalled ? "ok" : ""}`}>
                          {agent.hooksSupported
                            ? agent.hooksInstalled
                              ? "hooks installed"
                              : "hooks not installed"
                            : "hooks unsupported"}
                        </span>
                      )}
                    </div>
                    <code className="agent-resume">{agent.resumeCommand}</code>
                    {agent.installed && agent.hooksSupported && (
                      <button className="settings-small-button" onClick={() => install(agent.name)}>
                        {agent.hooksInstalled ? "reinstall" : "install hooks"}
                      </button>
                    )}
                  </div>
                ))}
                {reports.length > 0 && (
                  <div className="agent-reports">
                    {reports.map((r) => (
                      <div key={r.name} className={r.ok ? "report-ok" : "report-fail"}>
                        {r.name}: {r.message}
                      </div>
                    ))}
                  </div>
                )}
                <button className="settings-small-button" onClick={() => install()}>
                  install hooks for all agents
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
