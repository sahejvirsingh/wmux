import { allThemes } from "../../themes";
import { useSettingsStore } from "../../stores/settingsStore";
import "./ThemeEditor.css";

export function ThemeEditor() {
  const settings = useSettingsStore((s) => s.settings);
  const update = useSettingsStore((s) => s.update);
  const customThemes = useSettingsStore((s) => s.customThemes);

  const customParsed = customThemes.filter((t) => t.content !== null);
  const themes = allThemes(
    customParsed.map((t) => t.content!).map((t, i) => ({ ...t, id: customParsed[i].id, name: customParsed[i].name })),
  );

  return (
    <div className="theme-editor">
      {themes.map((theme) => (
        <button
          key={theme.id}
          className={`theme-card ${settings.themeId === theme.id ? "active" : ""}`}
          onClick={() => void update({ themeId: theme.id })}
        >
          <div
            className="theme-preview"
            style={{
              background: theme.colors.background,
              color: theme.colors.foreground,
              borderColor: theme.ui.border,
            }}
          >
            <span style={{ color: theme.colors.green }}>$</span>
            <span> echo hello</span>
            <span style={{ color: theme.colors.cyan }}> wmux</span>
          </div>
          <div className="theme-name">
            {theme.name}
            {settings.themeId === theme.id && <span className="theme-active-tag">active</span>}
          </div>
          <div className="theme-swatches">
            {[theme.colors.red, theme.colors.green, theme.colors.yellow, theme.colors.blue, theme.colors.magenta, theme.colors.cyan].map(
              (c) => (
                <span key={c} className="theme-swatch" style={{ background: c }} />
              ),
            )}
          </div>
        </button>
      ))}
    </div>
  );
}
