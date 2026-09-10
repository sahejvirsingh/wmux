import { describe, expect, it } from "vitest";
import {
  advanceChord,
  comboMatches,
  DEFAULT_BINDINGS,
  eventToCombo,
  formatCombo,
  parseCombo,
} from "./keybindings";

function fakeEvent(init: {
  key: string;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  metaKey?: boolean;
}): KeyboardEvent {
  return {
    key: init.key,
    ctrlKey: init.ctrlKey ?? false,
    shiftKey: init.shiftKey ?? false,
    altKey: init.altKey ?? false,
    metaKey: init.metaKey ?? false,
  } as KeyboardEvent;
}

describe("parseCombo", () => {
  it("parses simple key", () => {
    const parsed = parseCombo("c");
    expect(parsed?.key).toBe("c");
    expect(parsed?.modifiers.size).toBe(0);
  });

  it("parses modifiers", () => {
    const parsed = parseCombo("ctrl+shift+d");
    expect(parsed?.key).toBe("d");
    expect([...(parsed?.modifiers ?? [])].sort()).toEqual(["ctrl", "shift"]);
  });

  it("rejects unknown modifiers", () => {
    expect(parseCombo("hyper+x")).toBeNull();
  });

  it("normalizes case", () => {
    expect(parseCombo("Ctrl+Shift+P")?.key).toBe("p");
  });
});

describe("eventToCombo", () => {
  it("builds combo from event", () => {
    expect(eventToCombo(fakeEvent({ key: "D", ctrlKey: true, shiftKey: true }))).toBe("ctrl+shift+d");
  });

  it("plain key", () => {
    expect(eventToCombo(fakeEvent({ key: "b" }))).toBe("b");
  });
});

describe("comboMatches", () => {
  it("matches equivalent combos", () => {
    expect(comboMatches("ctrl+shift+d", "ctrl+shift+d")).toBe(true);
    expect(comboMatches("ctrl+d", "ctrl+shift+d")).toBe(false);
    expect(comboMatches("ctrl+d", "d")).toBe(false);
  });

  it("order does not matter", () => {
    expect(comboMatches("shift+ctrl+z", "ctrl+shift+z")).toBe(true);
  });
});

describe("formatCombo", () => {
  it("pretty prints", () => {
    expect(formatCombo("ctrl+shift+p")).toBe("Ctrl+Shift+P");
  });

  it("pretty prints arrows", () => {
    expect(formatCombo("ctrl+shift+arrowright")).toBe("Ctrl+Shift+→");
  });
});

describe("advanceChord", () => {
  it("fires single-step binding", () => {
    const result = advanceChord(DEFAULT_BINDINGS, { pending: null }, "ctrl+shift+p");
    expect(result.action).toBe("commandPalette");
    expect(result.state.pending).toBeNull();
  });

  it("arms two-step chord then fires", () => {
    const bindings = { newSurface: ["ctrl+b", "c"] };
    const armed = advanceChord(bindings, { pending: null }, "ctrl+b");
    expect(armed.action).toBeNull();
    expect(armed.state.pending).toEqual(["ctrl+b", "c"]);
    const fired = advanceChord(bindings, armed.state, "c");
    expect(fired.action).toBe("newSurface");
    expect(fired.state.pending).toBeNull();
  });

  it("resets chord on wrong second key", () => {
    const bindings = { newSurface: ["ctrl+b", "c"] };
    const armed = advanceChord(bindings, { pending: null }, "ctrl+b");
    const missed = advanceChord(bindings, armed.state, "x");
    expect(missed.action).toBeNull();
    expect(missed.state.pending).toBeNull();
  });

  it("unrelated key does nothing", () => {
    const result = advanceChord(DEFAULT_BINDINGS, { pending: null }, "q");
    expect(result.action).toBeNull();
  });
});
