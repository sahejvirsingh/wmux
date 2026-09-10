import { useEffect, useMemo } from "react";
import type { ITheme } from "@xterm/xterm";
import { findTheme, type Theme } from "../themes";
import { useSettingsStore } from "../stores/settingsStore";

export interface ThemeHandle {
  theme: Theme;
  xtermTheme: ITheme;
  isDark: boolean;
}

export function useTheme(): ThemeHandle {
  const settings = useSettingsStore((s) => s.settings);
  const customThemes = useSettingsStore((s) => s.customThemes);

  const parsedCustom = useMemo<Theme[]>(() => {
    return customThemes
      .map((meta) => {
        if (!meta.content) {
          return null;
        }
        try {
          const theme = meta.content as unknown as Theme;
          if (!theme.colors || !theme.colors.background || !theme.ui) {
            return null;
          }
          return { ...theme, id: meta.id };
        } catch {
          return null;
        }
      })
      .filter((t): t is Theme => t !== null);
  }, [customThemes]);

  const theme = useMemo(() => findTheme(settings.themeId, parsedCustom), [settings.themeId, parsedCustom]);

  const xtermTheme = useMemo<ITheme>(
    () => ({
      background: theme.colors.background,
      foreground: theme.colors.foreground,
      cursor: theme.colors.cursor,
      cursorAccent: theme.colors.cursorAccent,
      selectionBackground: theme.colors.selectionBackground,
      black: theme.colors.black,
      red: theme.colors.red,
      green: theme.colors.green,
      yellow: theme.colors.yellow,
      blue: theme.colors.blue,
      magenta: theme.colors.magenta,
      cyan: theme.colors.cyan,
      white: theme.colors.white,
      brightBlack: theme.colors.brightBlack,
      brightRed: theme.colors.brightRed,
      brightGreen: theme.colors.brightGreen,
      brightYellow: theme.colors.brightYellow,
      brightBlue: theme.colors.brightBlue,
      brightMagenta: theme.colors.brightMagenta,
      brightCyan: theme.colors.brightCyan,
      brightWhite: theme.colors.brightWhite,
    }),
    [theme],
  );

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("theme-dark", theme.dark);
    root.classList.toggle("theme-light", !theme.dark);
    root.style.setProperty("--accent", theme.ui.accent);
    root.style.setProperty("--sidebar", theme.ui.sidebar);
    root.style.setProperty("--sidebar-active", theme.ui.sidebarActive);
    root.style.setProperty("--statusbar", theme.ui.statusBar);
    root.style.setProperty("--border", theme.ui.border);
    root.style.setProperty("--panel", theme.ui.panel);
    root.style.setProperty("--muted", theme.ui.textMuted);
    root.style.setProperty("--danger", theme.ui.danger);
    root.style.setProperty("--background", theme.colors.background);
    root.style.setProperty("--foreground", theme.colors.foreground);
    root.style.setProperty("--font-mono", settings.fontFamily);
  }, [theme, settings.fontFamily]);

  return { theme, xtermTheme, isDark: theme.dark };
}