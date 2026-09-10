export type ActionId =
  | "commandPalette"
  | "newWorkspace"
  | "workspaceSwitcher"
  | "toggleSidebar"
  | "newSurface"
  | "closePane"
  | "splitRight"
  | "splitDown"
  | "nextPane"
  | "prevPane"
  | "nextWorkspace"
  | "prevWorkspace"
  | "zoomPane"
  | "findInTerminal"
  | "showNotifications"
  | "restoreSession"
  | "openSettings";

export interface ActionDef {
  id: ActionId;
  label: string;
}

export const ACTIONS: ActionDef[] = [
  { id: "commandPalette", label: "Command Palette" },
  { id: "newWorkspace", label: "New Workspace" },
  { id: "workspaceSwitcher", label: "Go to Workspace" },
  { id: "toggleSidebar", label: "Toggle Sidebar" },
  { id: "newSurface", label: "New Surface (Tab)" },
  { id: "closePane", label: "Close Pane" },
  { id: "splitRight", label: "Split Right" },
  { id: "splitDown", label: "Split Down" },
  { id: "nextPane", label: "Next Pane" },
  { id: "prevPane", label: "Previous Pane" },
  { id: "nextWorkspace", label: "Next Workspace" },
  { id: "prevWorkspace", label: "Previous Workspace" },
  { id: "zoomPane", label: "Zoom Pane" },
  { id: "findInTerminal", label: "Find in Terminal" },
  { id: "showNotifications", label: "Show Notifications" },
  { id: "restoreSession", label: "Reopen Previous Session" },
  { id: "openSettings", label: "Open Settings" },
];

export const DEFAULT_BINDINGS: Record<ActionId, string[]> = {
  commandPalette: ["ctrl+shift+p"],
  newWorkspace: ["ctrl+n"],
  workspaceSwitcher: ["ctrl+p"],
  toggleSidebar: ["ctrl+b"],
  newSurface: ["ctrl+t"],
  closePane: ["ctrl+w"],
  splitRight: ["ctrl+shift+d"],
  splitDown: ["ctrl+shift+e"],
  nextPane: ["ctrl+shift+arrowright"],
  prevPane: ["ctrl+shift+arrowleft"],
  nextWorkspace: ["ctrl+shift+]"],
  prevWorkspace: ["ctrl+shift+["],
  zoomPane: ["ctrl+shift+z"],
  findInTerminal: ["ctrl+shift+f"],
  showNotifications: ["ctrl+shift+i"],
  restoreSession: ["ctrl+shift+o"],
  openSettings: ["ctrl+,"],
};

const MODIFIERS = new Set(["ctrl", "shift", "alt", "meta"]);

export function parseCombo(combo: string): { modifiers: Set<string>; key: string } | null {
  const parts = combo
    .split("+")
    .map((p) => p.trim().toLowerCase())
    .filter((p) => p.length > 0);
  if (parts.length === 0) {
    return null;
  }
  const key = parts[parts.length - 1];
  const modifiers = new Set(parts.slice(0, -1));
  for (const mod of modifiers) {
    if (!MODIFIERS.has(mod)) {
      return null;
    }
  }
  return { modifiers, key };
}

export function eventToCombo(event: KeyboardEvent): string {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("ctrl");
  if (event.shiftKey) parts.push("shift");
  if (event.altKey) parts.push("alt");
  if (event.metaKey) parts.push("meta");
  parts.push(event.key.toLowerCase());
  return parts.join("+");
}

export function comboMatches(binding: string, combo: string): boolean {
  const a = parseCombo(binding);
  const b = parseCombo(combo);
  if (!a || !b) {
    return false;
  }
  return a.key === b.key && a.modifiers.size === b.modifiers.size && [...a.modifiers].every((m) => b.modifiers.has(m));
}

export function formatCombo(combo: string): string {
  const parsed = parseCombo(combo);
  if (!parsed) {
    return combo;
  }
  const mods = ["ctrl", "alt", "shift", "meta"].filter((m) => parsed.modifiers.has(m));
  const key = parsed.key === "arrowleft" ? "←" : parsed.key === "arrowright" ? "→" : parsed.key === "arrowup" ? "↑" : parsed.key === "arrowdown" ? "↓" : parsed.key;
  return [...mods.map((m) => (m === "ctrl" ? "Ctrl" : m === "shift" ? "Shift" : m === "alt" ? "Alt" : "Win")), key.toUpperCase()]
    .join("+");
}

export interface ChordState {
  pending: string[] | null;
}

export function advanceChord(
  bindings: Record<string, string[]>,
  state: ChordState,
  combo: string,
): { state: ChordState; action: ActionId | null } {
  if (state.pending) {
    const second = state.pending[1];
    if (comboMatches(second, combo)) {
      const action = Object.keys(bindings).find(
        (k) => bindings[k].length === 2 && comboMatches(bindings[k][0], state.pending![0]) && bindings[k][1] === second,
      );
      return { state: { pending: null }, action: (action as ActionId) ?? null };
    }
    return { state: { pending: null }, action: null };
  }
  for (const [actionId, binding] of Object.entries(bindings)) {
    if (binding.length >= 1 && comboMatches(binding[0], combo)) {
      if (binding.length === 2) {
        return { state: { pending: binding }, action: null };
      }
      return { state: { pending: null }, action: actionId as ActionId };
    }
  }
  return { state: { pending: null }, action: null };
}