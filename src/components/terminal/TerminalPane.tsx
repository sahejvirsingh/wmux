import { SearchAddon } from "@xterm/addon-search";
import { X, ZoomIn } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { focusPane } from "../../lib/tauri-bridge";
import { useNotificationSink } from "../../hooks/useNotifications";
import { useTerminal } from "../../hooks/useTerminal";
import type { ITheme } from "@xterm/xterm";
import type { TerminalPaneNode, Workspace } from "../../types";
import { NotificationRing } from "./NotificationRing";
import "./TerminalPane.css";

export interface TerminalPaneProps {
  node: TerminalPaneNode;
  workspace: Workspace;
  focused: boolean;
  xtermTheme: ITheme;
  fontSize: number;
  fontFamily: string;
  searchVisible: boolean;
  onCloseSearch: () => void;
  onZoom: () => void;
}

export function TerminalPane({
  node,
  workspace,
  focused,
  xtermTheme,
  fontSize,
  fontFamily,
  searchVisible,
  onCloseSearch,
  onZoom,
}: TerminalPaneProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const notify = useNotificationSink(workspace.id, node.id, node.ptyId);
  const handle = useTerminal({
    containerRef,
    node,
    workspaceId: workspace.id,
    workspaceCwd: workspace.cwd,
    theme: xtermTheme,
    fontSize,
    fontFamily,
    onNotification: notify,
    onExit: () => {},
  });
  const [searchQuery, setSearchQuery] = useState("");

  useEffect(() => {
    if (focused && handle.ready) {
      handle.focus();
    }
  }, [focused, handle.ready, handle]);

  const runSearch = (query: string, forward = true) => {
    const addon: SearchAddon | null = handle.searchAddon;
    if (!addon || !query) {
      return;
    }
    if (forward) {
      addon.findNext(query);
    } else {
      addon.findPrevious(query);
    }
  };

  return (
    <div
      className={`terminal-pane ${focused ? "focused" : ""}`}
      data-pane-id={node.id}
      onMouseDown={() => {
        if (!focused) {
          void focusPane(workspace.id, node.id);
        }
        handle.focus();
      }}
    >
      <div className="terminal-pane-header">
        <span className="terminal-pane-title">{node.title || "terminal"}</span>
        <div className="terminal-pane-actions">
          {node.attention && <span className="attention-dot" title="unread notification" />}
          <button className="icon-button" title="Zoom pane" onClick={(e) => { e.stopPropagation(); onZoom(); }}>
            <ZoomIn size={13} />
          </button>
        </div>
      </div>
      <div className="terminal-pane-body">
        <NotificationRing attention={node.attention} />
        <div ref={containerRef} className="terminal-xterm-container" />
        {handle.exited && (
          <div className="terminal-exited-overlay">
            <span>process exited{handle.exitCode !== null ? ` (code ${handle.exitCode})` : ""}</span>
          </div>
        )}
        {searchVisible && (
          <div className="terminal-search">
            <input
              autoFocus
              placeholder="Find in terminal…"
              value={searchQuery}
              onChange={(e) => {
                setSearchQuery(e.target.value);
                runSearch(e.target.value);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  runSearch(searchQuery, !e.shiftKey);
                } else if (e.key === "Escape") {
                  handle.searchAddon?.clearDecorations();
                  onCloseSearch();
                }
                e.stopPropagation();
              }}
            />
            <button
              className="icon-button"
              title="Close search"
              onClick={() => {
                handle.searchAddon?.clearDecorations();
                onCloseSearch();
              }}
            >
              <X size={13} />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
