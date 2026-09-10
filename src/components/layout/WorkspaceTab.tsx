import { GitBranch, Globe, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { cwdBasename } from "../../lib/paneLayout";
import type { Workspace } from "../../types";
import "./WorkspaceTab.css";

export interface WorkspaceTabProps {
  workspace: Workspace;
  active: boolean;
  onSelect: () => void;
  onRename: (name: string) => void;
  onDelete: () => void;
  onDragStart: () => void;
  onDragOver: (e: React.DragEvent) => void;
  onDrop: (e: React.DragEvent) => void;
  dragOver: boolean;
}

export function WorkspaceTab({
  workspace,
  active,
  onSelect,
  onRename,
  onDelete,
  onDragStart,
  onDragOver,
  onDrop,
  dragOver,
}: WorkspaceTabProps) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(workspace.name);
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [editing]);

  const commit = () => {
    setEditing(false);
    const name = draft.trim();
    if (name && name !== workspace.name) {
      onRename(name);
    } else {
      setDraft(workspace.name);
    }
  };

  return (
    <div
      className={`workspace-tab ${active ? "active" : ""} ${dragOver ? "drag-over" : ""}`}
      onClick={onSelect}
      onDoubleClick={() => setEditing(true)}
      draggable={!editing}
      onDragStart={onDragStart}
      onDragOver={(e) => {
        e.preventDefault();
        onDragOver(e);
      }}
      onDrop={onDrop}
      title={workspace.cwd}
    >
      {editing ? (
        <input
          ref={inputRef}
          className="workspace-tab-edit"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              commit();
            } else if (e.key === "Escape") {
              setDraft(workspace.name);
              setEditing(false);
            }
            e.stopPropagation();
          }}
          onClick={(e) => e.stopPropagation()}
        />
      ) : (
        <>
          <div className="workspace-tab-name">{workspace.name}</div>
          {cwdBasename(workspace.cwd) && (
            <div className="workspace-tab-cwd">{cwdBasename(workspace.cwd)}</div>
          )}
          <div className="workspace-tab-meta">
            {workspace.branch && (
              <span className="workspace-tab-branch">
                <GitBranch size={10} />
                {workspace.branch}
              </span>
            )}
            {workspace.ports.length > 0 && (
              <span className="workspace-tab-ports" title={workspace.ports.join(", ")}>
                <Globe size={10} />
                {workspace.ports.length}
              </span>
            )}
          </div>
          <div className="workspace-tab-footer">
            {workspace.notifications > 0 && (
              <span className="workspace-tab-badge">{workspace.notifications}</span>
            )}
            <button
              className="workspace-tab-delete"
              title="Delete workspace"
              onClick={(e) => {
                e.stopPropagation();
                onDelete();
              }}
            >
              <X size={11} />
            </button>
          </div>
        </>
      )}
    </div>
  );
}
