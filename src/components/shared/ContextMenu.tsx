import { useCallback, useEffect, useState } from "react";
import "./ContextMenu.css";

export interface ContextMenuEntry {
  id: string;
  label: string;
  separator?: boolean;
  danger?: boolean;
}

export interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  entries: ContextMenuEntry[];
  onAction: (id: string) => void;
  close: () => void;
  open: (entries: ContextMenuEntry[], x: number, y: number, onAction: (id: string) => void) => void;
}

export function useContextMenu(): ContextMenuState {
  const [visible, setVisible] = useState(false);
  const [x, setX] = useState(0);
  const [y, setY] = useState(0);
  const [entries, setEntries] = useState<ContextMenuEntry[]>([]);
  const [onAction, setOnAction] = useState<(id: string) => void>(() => {});

  const open = useCallback(
    (newEntries: ContextMenuEntry[], px: number, py: number, action: (id: string) => void) => {
      setEntries(newEntries);
      setX(px);
      setY(py);
      setOnAction(() => action);
      setVisible(true);
    },
    [],
  );

  const close = useCallback(() => setVisible(false), []);

  return { visible, x, y, entries, onAction, close, open };
}

export function ContextMenu({ menu }: { menu: ContextMenuState }) {
  useEffect(() => {
    if (!menu.visible) {
      return;
    }
    const dismiss = () => menu.close();
    window.addEventListener("mousedown", dismiss);
    window.addEventListener("blur", dismiss);
    return () => {
      window.removeEventListener("mousedown", dismiss);
      window.removeEventListener("blur", dismiss);
    };
  }, [menu]);

  if (!menu.visible) {
    return null;
  }

  const style: React.CSSProperties = {
    left: Math.min(menu.x, window.innerWidth - 200),
    top: Math.min(menu.y, window.innerHeight - 200),
  };

  return (
    <div className="context-menu" style={style} onContextMenu={(e) => e.preventDefault()}>
      {menu.entries.map((entry) =>
        entry.separator ? (
          <div key={entry.id} className="context-menu-separator" />
        ) : (
          <button
            key={entry.id}
            className={`context-menu-item ${entry.danger ? "danger" : ""}`}
            onMouseDown={(e) => {
              e.stopPropagation();
              menu.close();
              menu.onAction(entry.id);
            }}
          >
            {entry.label}
          </button>
        ),
      )}
    </div>
  );
}
