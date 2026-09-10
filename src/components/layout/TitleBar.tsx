import { Minus, PanelLeft, Square, Terminal, X } from "lucide-react";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppStore } from "../../stores/appStore";
import { useUiStore } from "../../stores/uiStore";
import "./TitleBar.css";

export function TitleBar() {
  const appState = useAppStore((s) => s.state);
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const pendingChord = useUiStore((s) => s.pendingChord);
  const [maximized, setMaximized] = useState(false);
  const active = appState?.workspaces.find((w) => w.id === appState.activeWorkspace);

  useEffect(() => {
    let cancelled = false;
    const poll = async () => {
      try {
        const win = getCurrentWindow();
        const isMax = await win.isMaximized();
        if (!cancelled) {
          setMaximized(isMax);
        }
      } catch {
        /* not in tauri */
      }
    };
    void poll();
    const interval = window.setInterval(poll, 1000);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, []);

  const minimize = () => getCurrentWindow().minimize().catch(() => {});
  const toggleMaximize = () => {
    const win = getCurrentWindow();
    if (maximized) {
      win.unmaximize().catch(() => {});
    } else {
      win.maximize().catch(() => {});
    }
  };
  const close = () => getCurrentWindow().hide().catch(() => {});

  return (
    <div className="titlebar" data-tauri-drag-region>
      <button className="titlebar-icon-button" title="Toggle sidebar (Ctrl+B)" onClick={toggleSidebar}>
        <PanelLeft size={14} />
      </button>
      <div className="titlebar-brand" data-tauri-drag-region>
        <Terminal size={13} />
        <span>wmux</span>
      </div>
      <div className="titlebar-workspace" data-tauri-drag-region>
        {active ? active.name : "no workspace"}
      </div>
      {pendingChord && <div className="titlebar-chord">waiting for {pendingChord[1]}…</div>}
      <div className="titlebar-spacer" data-tauri-drag-region />
      <div className="titlebar-controls">
        <button className="titlebar-control" title="Minimize" onClick={minimize}>
          <Minus size={13} />
        </button>
        <button className="titlebar-control" title={maximized ? "Restore" : "Maximize"} onClick={toggleMaximize}>
          <Square size={11} />
        </button>
        <button className="titlebar-control close" title="Close" onClick={close}>
          <X size={13} />
        </button>
      </div>
    </div>
  );
}
