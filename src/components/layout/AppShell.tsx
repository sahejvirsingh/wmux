import { useEffect } from "react";
import { useAppStore } from "../../stores/appStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useUiStore } from "../../stores/uiStore";
import { useTheme } from "../../hooks/useTheme";
import { browserSplit, closePane, createWorkspace, splitPane, zoomPane } from "../../lib/tauri-bridge";
import { PaneContainer } from "../terminal/PaneContainer";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";
import { TitleBar } from "./TitleBar";
import { CommandPalette } from "../shared/CommandPalette";
import { ContextMenu, type ContextMenuEntry, useContextMenu } from "../shared/ContextMenu";
import { NotificationPanel } from "../shared/NotificationPanel";
import { SettingsDialog } from "../settings/SettingsDialog";
import { SSHDialog } from "../ssh/SSHDialog";
import "./AppShell.css";

export function AppShell() {
  const appState = useAppStore((s) => s.state);
  const sidebarVisible = useUiStore((s) => s.sidebarVisible);
  const setSidebarVisible = useUiStore((s) => s.setSidebarVisible);
  const searchPaneId = useUiStore((s) => s.searchPaneId);
  const setSearchPaneId = useUiStore((s) => s.setSearchPaneId);
  const settings = useSettingsStore((s) => s.settings);
  const { xtermTheme } = useTheme();
  const menu = useContextMenu();

  useEffect(() => {
    setSidebarVisible(settings.sidebarVisible);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (appState && appState.workspaces.length === 0) {
      void createWorkspace();
    }
  }, [appState]);

  const active = appState?.workspaces.find((w) => w.id === appState.activeWorkspace) ?? null;

  const openPaneMenu = (e: React.MouseEvent, paneId: string) => {
    e.preventDefault();
    const entries: ContextMenuEntry[] = [
      { id: "split-right", label: "Split right" },
      { id: "split-down", label: "Split down" },
      { id: "open-browser", label: "Open browser pane" },
      { id: "zoom", label: "Zoom pane" },
      { id: "sep-1", label: "", separator: true },
      { id: "close", label: "Close pane", danger: true },
    ];
    menu.open(entries, e.clientX, e.clientY, (id) => {
      if (!active) {
        return;
      }
      if (id === "split-right") {
        void splitPane(active.id, "vertical");
      } else if (id === "split-down") {
        void splitPane(active.id, "horizontal");
      } else if (id === "open-browser") {
        void browserSplit(active.id, "https://www.google.com");
      } else if (id === "zoom") {
        void zoomPane(active.id, paneId);
      } else if (id === "close") {
        void closePane(active.id, paneId);
      }
    });
  };

  return (
    <div
      className="app-shell"
      onContextMenu={(e) => {
        const pane = (e.target as HTMLElement).closest(".terminal-pane");
        if (pane && active) {
          const paneId = pane.getAttribute("data-pane-id");
          if (paneId) {
            openPaneMenu(e, paneId);
          }
        }
      }}
    >
      <TitleBar />
      <div className="app-body">
        {sidebarVisible && <Sidebar />}
        <main className="app-main">
          {active ? (
            <PaneContainer
              workspace={active}
              xtermTheme={xtermTheme}
              fontSize={settings.fontSize}
              fontFamily={settings.fontFamily}
              searchPaneId={searchPaneId}
              onCloseSearch={() => setSearchPaneId(null)}
            />
          ) : (
            <div className="app-empty">
              <p>Loading…</p>
            </div>
          )}
        </main>
      </div>
      <StatusBar />
      <CommandPalette />
      <NotificationPanel />
      <SettingsDialog />
      <SSHDialog />
      {menu.visible && <ContextMenu menu={menu} />}
    </div>
  );
}
