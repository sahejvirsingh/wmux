import type { BrowserPaneNode, PaneNode, SplitDirection, TerminalPaneNode } from "../types";

export function collectTerminals(node: PaneNode): TerminalPaneNode[] {
  if (node.type === "terminal") {
    return [node];
  }
  if (node.type === "browser") {
    return [];
  }
  return node.children.flatMap(collectTerminals);
}

export function findNode(node: PaneNode, id: string): PaneNode | null {
  if (node.id === id) {
    return node;
  }
  if (node.type === "split") {
    for (const child of node.children) {
      const found = findNode(child, id);
      if (found) {
        return found;
      }
    }
  }
  return null;
}

export function findTerminalByPty(node: PaneNode, ptyId: string): TerminalPaneNode | null {
  if (node.type === "terminal") {
    return node.ptyId === ptyId ? node : null;
  }
  if (node.type === "browser") {
    return null;
  }
  for (const child of node.children) {
    const found = findTerminalByPty(child, ptyId);
    if (found) {
      return found;
    }
  }
  return null;
}

export function zoomedTerminal(node: PaneNode): TerminalPaneNode | BrowserPaneNode | null {
  if (node.type === "terminal" || node.type === "browser") {
    return node.zoomed ? node : null;
  }
  for (const child of node.children) {
    const found = zoomedTerminal(child);
    if (found) {
      return found;
    }
  }
  return null;
}

export function directionToFlex(direction: SplitDirection): "row" | "column" {
  return direction === "vertical" ? "row" : "column";
}

export function terminalCount(node: PaneNode): number {
  return collectTerminals(node).length;
}

export function cwdBasename(cwd: string): string {
  if (!cwd) {
    return "";
  }
  const normalized = cwd.replace(/\\/g, "/").replace(/\/+$/, "");
  const idx = normalized.lastIndexOf("/");
  return idx === -1 ? normalized : normalized.slice(idx + 1);
}