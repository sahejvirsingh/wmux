export interface ThemeColors {
  background: string;
  foreground: string;
  cursor: string;
  cursorAccent: string;
  selectionBackground: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  brightBlack: string;
  brightRed: string;
  brightGreen: string;
  brightYellow: string;
  brightBlue: string;
  brightMagenta: string;
  brightCyan: string;
  brightWhite: string;
}

export interface ThemeUi {
  accent: string;
  sidebar: string;
  sidebarActive: string;
  statusBar: string;
  border: string;
  panel: string;
  textMuted: string;
  danger: string;
}

export interface Theme {
  id: string;
  name: string;
  dark: boolean;
  colors: ThemeColors;
  ui: ThemeUi;
}

export function toXtermTheme(theme: Theme): ThemeColors {
  return theme.colors;
}