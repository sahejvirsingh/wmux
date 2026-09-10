export type SplitDirection = "horizontal" | "vertical";

export interface TerminalPaneNode {
  type: "terminal";
  id: string;
  ptyId: string;
  shell: string;
  cwd: string;
  command: string | null;
  title: string;
  zoomed: boolean;
  attention: boolean;
}

export interface SplitPaneNode {
  type: "split";
  id: string;
  direction: SplitDirection;
  ratio: number;
  children: PaneNode[];
}

export interface BrowserPaneNode {
  type: "browser";
  id: string;
  url: string;
  title: string;
  zoomed: boolean;
  attention: boolean;
}

export type PaneNode = TerminalPaneNode | SplitPaneNode | BrowserPaneNode;

export interface Workspace {
  id: string;
  name: string;
  cwd: string;
  branch: string | null;
  ports: number[];
  notifications: number;
  layout: PaneNode;
  activePane: string | null;
}

export interface AppStateJson {
  activeWorkspace: string | null;
  workspaces: Workspace[];
}

export type NotificationKind = "desktop" | "progress" | "rich";

export interface AppNotification {
  title: string;
  message: string;
  kind: NotificationKind;
}

export type PtyEvent =
  | { type: "output"; data: string }
  | { type: "notification"; notification: AppNotification }
  | { type: "title"; title: string }
  | { type: "cwd"; cwd: string }
  | { type: "scrollback"; data: string }
  | { type: "exit"; code: number };

export interface PtySpawnSpec {
  ptyId: string;
  label: string;
  shell: string | null;
  command: string | null;
  cwd: string | null;
  cols: number;
  rows: number;
}

export interface Settings {
  themeId: string;
  fontSize: number;
  fontFamily: string;
  shell: string | null;
  autoRestore: boolean;
  agentAutoResume: boolean;
  notifications: { rings: boolean; toasts: boolean; badges: boolean };
  keybindings: Record<string, string[]>;
  sidebarVisible: boolean;
}

export interface AgentStatus {
  name: string;
  installed: boolean;
  binaryPath: string | null;
  hooksInstalled: boolean;
  hooksSupported: boolean;
  resumeCommand: string;
}

export interface InstallReport {
  name: string;
  ok: boolean;
  message: string;
}

export interface ThemeMeta {
  id: string;
  name: string;
  content: import("../themes/types").Theme | null;
}

export type SshAuth =
  | { kind: "password"; password: string }
  | { kind: "key"; keyPath: string; passphrase: string | null };

export interface SshTarget {
  host: string;
  port: number;
  user: string;
}

export interface SshTestResult {
  ok: boolean;
  banner: string | null;
  error: string | null;
}

export interface SavedSshConnection {
  id: string;
  name: string;
  host: string;
  port: number;
  user: string;
  auth: SshAuth;
}