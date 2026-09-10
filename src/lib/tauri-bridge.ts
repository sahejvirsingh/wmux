import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AgentStatus,
  AppStateJson,
  AppNotification,
  InstallReport,
  PtyEvent,
  PtySpawnSpec,
  SavedSshConnection,
  Settings,
  SshTarget,
  SshTestResult,
  ThemeMeta,
} from "../types";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function getState(): Promise<AppStateJson> {
  return invoke<AppStateJson>("get_state");
}

export async function createWorkspace(name?: string): Promise<string> {
  return invoke<string>("workspace_create", { name });
}

export async function deleteWorkspace(id: string): Promise<void> {
  return invoke("workspace_delete", { id });
}

export async function renameWorkspace(id: string, name: string): Promise<void> {
  return invoke("workspace_rename", { id, name });
}

export async function selectWorkspace(id: string): Promise<void> {
  return invoke("workspace_select", { id });
}

export async function reorderWorkspaces(ids: string[]): Promise<void> {
  return invoke("workspace_reorder", { ids });
}

export async function splitPane(
  workspaceId: string,
  direction: "horizontal" | "vertical",
  command?: string,
  cwd?: string,
): Promise<void> {
  return invoke("pane_split", { workspaceId, direction, command, cwd });
}

export function browserSplit(workspaceId: string, url: string): Promise<string> {
  return invoke<string>("browser_split", { workspaceId, url });
}

export function browserWebviewCreate(
  paneId: string,
  url: string,
  x: number,
  y: number,
  width: number,
  height: number,
): Promise<void> {
  return invoke("browser_webview_create", { paneId, url, x, y, width, height });
}

export function browserWebviewMove(
  paneId: string,
  x: number,
  y: number,
  width: number,
  height: number,
): Promise<void> {
  return invoke("browser_webview_move", { paneId, x, y, width, height });
}

export function browserWebviewHide(paneId: string): Promise<void> {
  return invoke("browser_webview_hide", { paneId });
}

export function browserWebviewShow(paneId: string): Promise<void> {
  return invoke("browser_webview_show", { paneId });
}

export function browserWebviewEval(paneId: string, js: string): Promise<void> {
  return invoke("browser_webview_eval", { paneId, js });
}

export function browserNavigate(workspaceId: string, paneId: string, url: string): Promise<void> {
  return invoke("browser_navigate", { workspaceId, paneId, url });
}

export async function closePane(workspaceId: string, paneId: string): Promise<void> {
  return invoke("pane_close", { workspaceId, paneId });
}

export async function focusPane(workspaceId: string, paneId: string): Promise<void> {
  return invoke("pane_focus", { workspaceId, paneId });
}

export async function navigatePanes(workspaceId: string, direction: string): Promise<void> {
  return invoke("pane_navigate", { workspaceId, direction });
}

export async function zoomPane(workspaceId: string, paneId?: string): Promise<void> {
  return invoke("pane_zoom", { workspaceId, paneId });
}

export async function resizePane(workspaceId: string, splitId: string, ratio: number): Promise<void> {
  return invoke("pane_resize", { workspaceId, splitId, ratio });
}

export async function clearNotifications(workspaceId: string, paneId?: string): Promise<void> {
  return invoke("workspace_clear_notifications", { workspaceId, paneId });
}

export function spawnPty(spec: PtySpawnSpec, channel: Channel<PtyEvent>): Promise<void> {
  return invoke("spawn_pty_cmd", { spec, channel });
}

export function writePty(ptyId: string, data: string): Promise<void> {
  return invoke("write_pty_cmd", { ptyId, data });
}

export function resizePty(ptyId: string, cols: number, rows: number): Promise<void> {
  return invoke("resize_pty_cmd", { ptyId, cols, rows });
}

export function killPty(ptyId: string): Promise<void> {
  return invoke("kill_pty_cmd", { ptyId });
}

export function detachPty(ptyId: string): Promise<void> {
  return invoke("detach_pty_cmd", { ptyId });
}

export function registerPty(workspaceId: string, paneId: string, ptyId: string): Promise<void> {
  return invoke("workspace_register_pty", { workspaceId, paneId, ptyId });
}

export function attachPty(
  workspaceId: string,
  paneId: string,
  ptyId: string,
  channel: Channel<PtyEvent>,
): Promise<void> {
  return invoke("workspace_attach_pty", { workspaceId, paneId, ptyId, channel });
}

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings_cmd");
}

export function setSettings(settings: Settings): Promise<void> {
  return invoke("set_settings_cmd", { settings });
}

export function listThemes(): Promise<ThemeMeta[]> {
  return invoke<ThemeMeta[]>("list_themes_cmd");
}

export function hooksStatus(): Promise<AgentStatus[]> {
  return invoke<AgentStatus[]>("hooks_status_cmd");
}

export function hooksInstall(agent?: string): Promise<InstallReport[]> {
  return invoke<InstallReport[]>("hooks_install_cmd", { agent });
}

export function sshTest(target: SshTarget, auth: SavedSshConnection["auth"]): Promise<SshTestResult> {
  return invoke<SshTestResult>("ssh_test_cmd", { target, auth });
}

export function sshConnect(target: SshTarget, workspaceName?: string): Promise<string> {
  return invoke<string>("ssh_connect_cmd", { target, workspaceName });
}

export function sshSave(connection: SavedSshConnection): Promise<void> {
  return invoke("ssh_save_cmd", { connection });
}

export function sshList(): Promise<SavedSshConnection[]> {
  return invoke<SavedSshConnection[]>("ssh_list_cmd");
}

export function sshDelete(id: string): Promise<void> {
  return invoke("ssh_delete_cmd", { id });
}

export function sessionSave(): Promise<void> {
  return invoke("session_save");
}

export function sessionRestore(): Promise<number> {
  return invoke<number>("session_restore");
}

export function onStateChange(handler: (state: AppStateJson) => void): Promise<() => void> {
  return listen<AppStateJson>("wmux:state", (event) => handler(event.payload));
}

export function makeChannel(handler: (event: PtyEvent) => void): Channel<PtyEvent> {
  const channel = new Channel<PtyEvent>();
  channel.onmessage = handler;
  return channel;
}

export function newPtyId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `pty-${crypto.randomUUID()}`;
  }
  return `pty-${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

export type { AppNotification };