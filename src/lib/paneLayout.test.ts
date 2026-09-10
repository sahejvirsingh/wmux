import { describe, expect, it } from "vitest";
import { collectTerminals, cwdBasename, directionToFlex, findNode, findTerminalByPty, terminalCount, zoomedTerminal } from "./paneLayout";
import type { PaneNode, TerminalPaneNode } from "../types";

function terminalNode(id: string, ptyId = "", extra: Partial<PaneNode> = {}): PaneNode {
  return {
    type: "terminal",
    id,
    ptyId,
    shell: "pwsh.exe",
    cwd: "C:\\work",
    command: null,
    title: "terminal",
    zoomed: false,
    attention: false,
    ...extra,
  } as PaneNode;
}

const tree: PaneNode = {
  type: "split",
  id: "split-1",
  direction: "vertical",
  ratio: 0.5,
  children: [
    terminalNode("pane-1", "pty-1"),
    {
      type: "split",
      id: "split-2",
      direction: "horizontal",
      ratio: 0.5,
      children: [terminalNode("pane-2", "pty-2"), terminalNode("pane-3", "pty-3")],
    },
  ],
};

describe("collectTerminals", () => {
  it("collects all terminals in DFS order", () => {
    const terminals = collectTerminals(tree);
    expect(terminals.map((t) => t.id)).toEqual(["pane-1", "pane-2", "pane-3"]);
  });
});

describe("findNode", () => {
  it("finds nested nodes", () => {
    expect(findNode(tree, "split-2")?.id).toBe("split-2");
    expect(findNode(tree, "pane-3")?.id).toBe("pane-3");
    expect(findNode(tree, "missing")).toBeNull();
  });
});

describe("findTerminalByPty", () => {
  it("finds by pty id", () => {
    expect(findTerminalByPty(tree, "pty-2")?.id).toBe("pane-2");
    expect(findTerminalByPty(tree, "pty-none")).toBeNull();
  });
});

describe("zoomedTerminal", () => {
  it("returns zoomed node only", () => {
    expect(zoomedTerminal(tree)).toBeNull();
    const zoomedPane = { ...(collectTerminals(tree)[0] as TerminalPaneNode), zoomed: true };
    const withZoom: PaneNode = {
      ...tree,
      children: [zoomedPane, tree.children[1]],
    };
    expect(zoomedTerminal(withZoom)?.id).toBe("pane-1");
  });
});

describe("directionToFlex", () => {
  it("maps directions", () => {
    expect(directionToFlex("vertical")).toBe("row");
    expect(directionToFlex("horizontal")).toBe("column");
  });
});

describe("terminalCount", () => {
  it("counts terminals", () => {
    expect(terminalCount(tree)).toBe(3);
    expect(terminalCount(terminalNode("solo"))).toBe(1);
  });
});

describe("cwdBasename", () => {
  it("extracts windows basename", () => {
    expect(cwdBasename("C:\\projects\\myproject")).toBe("myproject");
  });

  it("handles posix paths and empty", () => {
    expect(cwdBasename("/home/dev/project")).toBe("project");
    expect(cwdBasename("")).toBe("");
  });

  it("trims trailing slashes", () => {
    expect(cwdBasename("C:\\work\\")).toBe("work");
  });
});
