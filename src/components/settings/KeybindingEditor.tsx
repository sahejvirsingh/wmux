import { useEffect, useState } from "react";
import { ACTIONS, eventToCombo, formatCombo } from "../../lib/keybindings";
import { useSettingsStore } from "../../stores/settingsStore";
import "./KeybindingEditor.css";

export function KeybindingEditor() {
  const settings = useSettingsStore((s) => s.settings);
  const update = useSettingsStore((s) => s.update);
  const resetBinding = useSettingsStore((s) => s.resetBinding);
  const [capturing, setCapturing] = useState<string | null>(null);

  useEffect(() => {
    if (!capturing) {
      return;
    }
    const onKey = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.key === "Escape") {
        setCapturing(null);
        return;
      }
      if (event.key === "Control" || event.key === "Shift" || event.key === "Alt" || event.key === "Meta") {
        return;
      }
      const combo = eventToCombo(event);
      const next = { ...settings.keybindings };
      next[capturing] = [combo];
      void update({ keybindings: next });
      setCapturing(null);
    };
    window.addEventListener("keydown", onKey, { capture: true });
    return () => window.removeEventListener("keydown", onKey, { capture: true });
  }, [capturing, settings.keybindings, update]);

  return (
    <div className="keybinding-editor">
      {ACTIONS.map((action) => {
        const binding = settings.keybindings[action.id] ?? [];
        return (
          <div key={action.id} className="keybinding-row">
            <span className="keybinding-label">{action.label}</span>
            <div className="keybinding-value">
              <button
                className={`keybinding-combo ${capturing === action.id ? "capturing" : ""}`}
                onClick={() => setCapturing(action.id)}
              >
                {capturing === action.id
                  ? "press keys…"
                  : binding.map(formatCombo).join(" then ") || "unbound"}
              </button>
              <button
                className="icon-button"
                title="Reset to default"
                onClick={() => void resetBinding(action.id)}
              >
                ↺
              </button>
            </div>
          </div>
        );
      })}
      <p className="keybinding-hint">
        Two-step chords can be configured in <code>~/.wmux/wmux.json</code> as{" "}
        <code>["ctrl+b", "c"]</code>-style arrays.
      </p>
    </div>
  );
}
