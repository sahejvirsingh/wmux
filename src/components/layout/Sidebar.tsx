import { Plug, Plus, Settings } from "lucide-react";
import { useState } from "react";
import { useAppStore } from "../../stores/appStore";
import { useUiStore } from "../../stores/uiStore";
import { createWorkspace, deleteWorkspace, renameWorkspace, reorderWorkspaces, selectWorkspace } from "../../lib/tauri-bridge";
import { WorkspaceTab } from "./WorkspaceTab";
import "./Sidebar.css";

export function Sidebar() {
  const appState = useAppStore((s) => s.state);
  const setSettingsOpen = useUiStore((s) => s.setSettingsOpen);
  const setSshOpen = useUiStore((s) => s.setSshOpen);
  const [dragId, setDragId] = useState<string | null>(null);
  const [dragOverId, setDragOverId] = useState<string | null>(null);

  const workspaces = appState?.workspaces ?? [];
  const activeId = appState?.activeWorkspace ?? null;

  const onDrop = (targetId: string) => {
    if (!dragId || dragId === targetId) {
      setDragId(null);
      setDragOverId(null);
      return;
    }
    const ids = workspaces.map((w) => w.id);
    const from = ids.indexOf(dragId);
    const to = ids.indexOf(targetId);
    if (from === -1 || to === -1) {
      setDragId(null);
      setDragOverId(null);
      return;
    }
    ids.splice(to, 0, ids.splice(from, 1)[0]);
    void reorderWorkspaces(ids);
    setDragId(null);
    setDragOverId(null);
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-header">workspaces</div>
      <div className="sidebar-list">
        {workspaces.map((ws) => (
          <WorkspaceTab
            key={ws.id}
            workspace={ws}
            active={ws.id === activeId}
            onSelect={() => void selectWorkspace(ws.id)}
            onRename={(name) => void renameWorkspace(ws.id, name)}
            onDelete={() => void deleteWorkspace(ws.id)}
            onDragStart={() => setDragId(ws.id)}
            onDragOver={() => setDragOverId(ws.id)}
            onDrop={() => onDrop(ws.id)}
            dragOver={dragOverId === ws.id && dragId !== ws.id}
          />
        ))}
      </div>
      <div className="sidebar-footer">
        <button className="sidebar-button" onClick={() => void createWorkspace()} title="New workspace (Ctrl+N)">
          <Plus size={14} />
          <span>New workspace</span>
        </button>
        <button className="sidebar-button" onClick={() => setSshOpen(true)} title="Connect via SSH">
          <Plug size={14} />
          <span>SSH</span>
        </button>
        <button className="sidebar-button" onClick={() => setSettingsOpen(true)} title="Settings (Ctrl+,)">
          <Settings size={14} />
          <span>Settings</span>
        </button>
      </div>
    </aside>
  );
}
