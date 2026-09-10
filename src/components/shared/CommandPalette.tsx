import { Command } from "cmdk";
import {
  Columns2,
  Globe,
  PanelLeft,
  Rows2,
  Settings,
  SquareSplitHorizontal,
  Trash2,
  Undo2,
  X,
} from "lucide-react";
import { useEffect, useState } from "react";
import { ACTIONS } from "../../lib/keybindings";
import { allThemes } from "../../themes";
import { useAppStore } from "../../stores/appStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useUiStore } from "../../stores/uiStore";
import {
  browserSplit,
  clearNotifications,
  closePane,
  createWorkspace,
  navigatePanes,
  selectWorkspace,
  sessionRestore,
  sessionSave,
  splitPane,
  zoomPane,
} from "../../lib/tauri-bridge";
import { activeWorkspace } from "../../stores/appStore";
import "./CommandPalette.css";

export function CommandPalette() {
  const open = useUiStore((s) => s.paletteOpen);
  const setOpen = useUiStore((s) => s.setPaletteOpen);
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const setSettingsOpen = useUiStore((s) => s.setSettingsOpen);
  const setNotificationsOpen = useUiStore((s) => s.setNotificationsOpen);
  const appState = useAppStore((s) => s.state);
  const settings = useSettingsStore((s) => s.settings);
  const updateSettings = useSettingsStore((s) => s.update);
  const [search, setSearch] = useState("");

  useEffect(() => {
    if (!open) {
      setSearch("");
    }
  }, [open]);

  if (!open) {
    return null;
  }

  const workspace = activeWorkspace();

  const run = (fn: () => void) => {
    setOpen(false);
    fn();
  };

  return (
    <div className="palette-backdrop" onMouseDown={() => setOpen(false)}>
      <div className="palette" onMouseDown={(e) => e.stopPropagation()}>
        <Command label="command palette" shouldFilter={!search.startsWith(">")}>
          <div className="palette-header">
            <Command.Input
              autoFocus
              value={search}
              onValueChange={setSearch}
              placeholder="Type a command or search workspaces…"
            />
            <button className="icon-button" onClick={() => setOpen(false)} title="Close">
              <X size={14} />
            </button>
          </div>
          <Command.List className="palette-list">
            <Command.Empty>No results</Command.Empty>

            <Command.Group heading="Workspaces" className="palette-group">
              {(appState?.workspaces ?? []).map((ws) => (
                <Command.Item
                  key={ws.id}
                  value={`workspace ${ws.name} ${ws.cwd}`}
                  onSelect={() => run(() => void selectWorkspace(ws.id))}
                >
                  <span className="palette-item-label">{ws.name}</span>
                  <span className="palette-item-hint">{ws.cwd}</span>
                </Command.Item>
              ))}
              <Command.Item
                value="new workspace"
                onSelect={() => run(() => void createWorkspace())}
              >
                <SquareSplitHorizontal size={13} />
                <span className="palette-item-label">New workspace</span>
                <span className="palette-item-hint">
                  {ACTIONS.find((a) => a.id === "newWorkspace")?.label}
                </span>
              </Command.Item>
            </Command.Group>

            <Command.Group heading="Panes" className="palette-group">
              <Command.Item
                value="split right"
                onSelect={() =>
                  run(() => workspace && void splitPane(workspace.id, "vertical"))
                }
              >
                <Columns2 size={13} />
                <span className="palette-item-label">Split right</span>
              </Command.Item>
              <Command.Item
                value="split down"
                onSelect={() =>
                  run(() => workspace && void splitPane(workspace.id, "horizontal"))
                }
              >
                <Rows2 size={13} />
                <span className="palette-item-label">Split down</span>
              </Command.Item>
              <Command.Item
                value="open browser pane"
                onSelect={() =>
                  run(() => workspace && void browserSplit(workspace.id, "https://www.google.com"))
                }
              >
                <Globe size={13} />
                <span className="palette-item-label">Open browser pane</span>
              </Command.Item>
              <Command.Item
                value="zoom pane"
                onSelect={() =>
                  run(() => workspace && void zoomPane(workspace.id, workspace.activePane ?? undefined))
                }
              >
                <SquareSplitHorizontal size={13} />
                <span className="palette-item-label">Zoom pane</span>
              </Command.Item>
              <Command.Item
                value="next pane"
                onSelect={() => run(() => workspace && void navigatePanes(workspace.id, "next"))}
              >
                <span className="palette-item-label">Next pane</span>
              </Command.Item>
              <Command.Item
                value="close pane"
                onSelect={() =>
                  run(() => workspace && workspace.activePane && void closePane(workspace.id, workspace.activePane))
                }
              >
                <Trash2 size={13} />
                <span className="palette-item-label">Close pane</span>
              </Command.Item>
            </Command.Group>

            <Command.Group heading="Theme" className="palette-group">
              {allThemes().map((theme) => (
                <Command.Item
                  key={theme.id}
                  value={`theme ${theme.name}`}
                  onSelect={() => run(() => void updateSettings({ themeId: theme.id }))}
                >
                  <span
                    className="palette-swatch"
                    style={{ background: theme.colors.background, borderColor: theme.ui.accent }}
                  />
                  <span className="palette-item-label">{theme.name}</span>
                  {settings.themeId === theme.id && (
                    <span className="palette-item-hint">active</span>
                  )}
                </Command.Item>
              ))}
            </Command.Group>

            <Command.Group heading="Session & app" className="palette-group">
              <Command.Item
                value="reopen previous session"
                onSelect={() => run(() => void sessionRestore())}
              >
                <Undo2 size={13} />
                <span className="palette-item-label">Reopen previous session</span>
              </Command.Item>
              <Command.Item value="save session" onSelect={() => run(() => void sessionSave())}>
                <span className="palette-item-label">Save session</span>
              </Command.Item>
              <Command.Item
                value="clear notifications"
                onSelect={() =>
                  run(() => {
                    useUiStore.getState().clearNotifications();
                    for (const ws of appState?.workspaces ?? []) {
                      if (ws.notifications > 0) {
                        void clearNotifications(ws.id);
                      }
                    }
                  })
                }
              >
                <span className="palette-item-label">Clear all notifications</span>
              </Command.Item>
              <Command.Item value="toggle sidebar" onSelect={() => run(toggleSidebar)}>
                <PanelLeft size={13} />
                <span className="palette-item-label">Toggle sidebar</span>
              </Command.Item>
              <Command.Item
                value="notifications"
                onSelect={() => run(() => setNotificationsOpen(true))}
              >
                <span className="palette-item-label">Show notifications</span>
              </Command.Item>
              <Command.Item value="settings" onSelect={() => run(() => setSettingsOpen(true))}>
                <Settings size={13} />
                <span className="palette-item-label">Settings</span>
              </Command.Item>
            </Command.Group>
          </Command.List>
        </Command>
      </div>
    </div>
  );
}
