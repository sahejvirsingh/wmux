import { ArrowLeft, ArrowRight, RotateCw, ZoomIn } from "lucide-react";
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import {
  browserNavigate,
  browserWebviewCreate,
  browserWebviewEval,
  browserWebviewHide,
  browserWebviewMove,
  browserWebviewShow,
  focusPane,
} from "../../lib/tauri-bridge";
import type { BrowserPaneNode } from "../../types";
import "./BrowserPane.css";

export interface BrowserPaneProps {
  node: BrowserPaneNode;
  workspaceId: string;
  focused: boolean;
  onZoom: () => void;
}

export function BrowserPane({ node, workspaceId, focused, onZoom }: BrowserPaneProps) {
  const areaRef = useRef<HTMLDivElement | null>(null);
  const frameRef = useRef<number | null>(null);
  const [input, setInput] = useState(node.url);

  useEffect(() => {
    setInput(node.url);
  }, [node.url]);

  const syncBounds = useCallback(() => {
    if (frameRef.current !== null) {
      return;
    }
    frameRef.current = requestAnimationFrame(() => {
      frameRef.current = null;
      const el = areaRef.current;
      if (!el) {
        return;
      }
      const rect = el.getBoundingClientRect();
      void browserWebviewMove(
        node.id,
        rect.left,
        rect.top,
        Math.max(rect.width, 1),
        Math.max(rect.height, 1),
      ).catch(() => {});
    });
  }, [node.id]);

  useLayoutEffect(() => {
    const el = areaRef.current;
    if (!el) {
      return;
    }
    const rect = el.getBoundingClientRect();
    void browserWebviewCreate(
      node.id,
      node.url,
      rect.left,
      rect.top,
      Math.max(rect.width, 1),
      Math.max(rect.height, 1),
    )
      .then(() => browserWebviewShow(node.id))
      .catch((e) => console.error("failed to open browser pane", e));
    const observer = new ResizeObserver(syncBounds);
    observer.observe(el);
    window.addEventListener("resize", syncBounds);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", syncBounds);
      if (frameRef.current !== null) {
        cancelAnimationFrame(frameRef.current);
        frameRef.current = null;
      }
      void browserWebviewHide(node.id).catch(() => {});
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [node.id, syncBounds]);

  const submitUrl = () => {
    const value = input.trim();
    if (!value) {
      return;
    }
    void browserNavigate(workspaceId, node.id, value).catch(() => {});
  };

  return (
    <div className={`browser-pane terminal-pane ${focused ? "focused" : ""}`} data-pane-id={node.id}>
      <div
        className="terminal-pane-header"
        onMouseDown={() => void focusPane(workspaceId, node.id)}
      >
        <span className="terminal-pane-title">{node.title || "browser"}</span>
        <div className="terminal-pane-actions">
          {node.attention && <span className="attention-dot" title="unread notification" />}
          <button
            className="icon-button"
            title="Zoom pane"
            onClick={(e) => {
              e.stopPropagation();
              onZoom();
            }}
          >
            <ZoomIn size={13} />
          </button>
        </div>
      </div>
      <div className="browser-toolbar">
        <button
          className="icon-button"
          title="Back"
          onClick={() => void browserWebviewEval(node.id, "history.back()").catch(() => {})}
        >
          <ArrowLeft size={13} />
        </button>
        <button
          className="icon-button"
          title="Forward"
          onClick={() => void browserWebviewEval(node.id, "history.forward()").catch(() => {})}
        >
          <ArrowRight size={13} />
        </button>
        <button
          className="icon-button"
          title="Reload"
          onClick={() => void browserWebviewEval(node.id, "location.reload()").catch(() => {})}
        >
          <RotateCw size={13} />
        </button>
        <input
          className="browser-url-input"
          value={input}
          spellCheck={false}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              submitUrl();
            }
          }}
          placeholder="Enter a URL and press Enter"
        />
      </div>
      <div ref={areaRef} className="browser-area" />
    </div>
  );
}
