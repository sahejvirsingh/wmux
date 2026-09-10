import { useRef } from "react";
import type { ITheme } from "@xterm/xterm";
import { directionToFlex, zoomedTerminal } from "../../lib/paneLayout";
import type { PaneNode, Workspace } from "../../types";
import { BrowserPane } from "./BrowserPane";
import { PaneDivider } from "./PaneDivider";
import { TerminalPane } from "./TerminalPane";
import { zoomPane } from "../../lib/tauri-bridge";
import "./PaneContainer.css";

export interface PaneContainerProps {
  workspace: Workspace;
  xtermTheme: ITheme;
  fontSize: number;
  fontFamily: string;
  searchPaneId: string | null;
  onCloseSearch: () => void;
}

export function PaneContainer({
  workspace,
  xtermTheme,
  fontSize,
  fontFamily,
  searchPaneId,
  onCloseSearch,
}: PaneContainerProps) {
  const rootRef = useRef<HTMLDivElement | null>(null);
  const zoomed = zoomedTerminal(workspace.layout);

  if (zoomed) {
    return (
      <div className="pane-container" ref={rootRef}>
        {zoomed.type === "browser" ? (
          <BrowserPane
            node={zoomed}
            workspaceId={workspace.id}
            focused={true}
            onZoom={() => void zoomPane(workspace.id, zoomed.id)}
          />
        ) : (
          <TerminalPane
            node={zoomed}
            workspace={workspace}
            focused={true}
            xtermTheme={xtermTheme}
            fontSize={fontSize}
            fontFamily={fontFamily}
            searchVisible={searchPaneId === zoomed.id}
            onCloseSearch={onCloseSearch}
            onZoom={() => void zoomPane(workspace.id, zoomed.id)}
          />
        )}
      </div>
    );
  }

  return (
    <div className="pane-container" ref={rootRef}>
      {renderNode(workspace.layout, workspace, {
        xtermTheme,
        fontSize,
        fontFamily,
        searchPaneId,
        onCloseSearch,
        containerRef: rootRef,
      })}
    </div>
  );
}

interface RenderContext {
  xtermTheme: ITheme;
  fontSize: number;
  fontFamily: string;
  searchPaneId: string | null;
  onCloseSearch: () => void;
  containerRef: React.RefObject<HTMLDivElement | null>;
}

function renderNode(
  node: PaneNode,
  workspace: Workspace,
  ctx: RenderContext,
): React.ReactNode {
  if (node.type === "terminal") {
    return (
      <TerminalPane
        key={node.id}
        node={node}
        workspace={workspace}
        focused={workspace.activePane === node.id}
        xtermTheme={ctx.xtermTheme}
        fontSize={ctx.fontSize}
        fontFamily={ctx.fontFamily}
        searchVisible={ctx.searchPaneId === node.id}
        onCloseSearch={ctx.onCloseSearch}
        onZoom={() => void zoomPane(workspace.id, node.id)}
      />
    );
  }

  if (node.type === "browser") {
    return (
      <BrowserPane
        key={node.id}
        node={node}
        workspaceId={workspace.id}
        focused={workspace.activePane === node.id}
        onZoom={() => void zoomPane(workspace.id, node.id)}
      />
    );
  }

  const first = node.children[0];
  const second = node.children[1];
  if (!first || !second) {
    return null;
  }

  return (
    <div key={node.id} className={`pane-split ${directionToFlex(node.direction)}`}>
      <div className="pane-split-child" style={{ flex: `${node.ratio} 1 0%` }}>
        {renderNode(first, workspace, ctx)}
      </div>
      <PaneDivider
        workspaceId={workspace.id}
        splitId={node.id}
        direction={node.direction}
        ratio={node.ratio}
        containerRef={ctx.containerRef}
      />
      <div className="pane-split-child" style={{ flex: `${1 - node.ratio} 1 0%` }}>
        {renderNode(second, workspace, ctx)}
      </div>
    </div>
  );
}
