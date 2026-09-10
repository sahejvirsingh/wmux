import { useCallback, useRef } from "react";
import { resizePane } from "../../lib/tauri-bridge";
import type { SplitDirection } from "../../types";
import "./PaneDivider.css";

export interface PaneDividerProps {
  workspaceId: string;
  splitId: string;
  direction: SplitDirection;
  ratio: number;
  containerRef: React.RefObject<HTMLDivElement | null>;
}

export function PaneDivider({ workspaceId, splitId, direction, ratio, containerRef }: PaneDividerProps) {
  const draggingRef = useRef(false);
  const lastSentRef = useRef(0);

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      e.preventDefault();
      draggingRef.current = true;
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
    },
    [],
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!draggingRef.current || !containerRef.current) {
        return;
      }
      const rect = containerRef.current.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) {
        return;
      }
      let next: number;
      if (direction === "vertical") {
        next = (e.clientX - rect.left) / rect.width;
      } else {
        next = (e.clientY - rect.top) / rect.height;
      }
      next = Math.min(0.9, Math.max(0.1, next));
      const now = performance.now();
      if (now - lastSentRef.current > 40) {
        lastSentRef.current = now;
        void resizePane(workspaceId, splitId, next);
      }
    },
    [containerRef, direction, splitId, workspaceId],
  );

  const onPointerUp = useCallback((e: React.PointerEvent) => {
    draggingRef.current = false;
    try {
      (e.target as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {
      /* pointer already released */
    }
  }, []);

  return (
    <div
      className={`pane-divider ${direction === "vertical" ? "vertical" : "horizontal"}`}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      data-ratio={ratio}
    />
  );
}
