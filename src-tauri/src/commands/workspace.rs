use parking_lot::RwLock;
use tauri::{AppHandle, State};
use tauri::ipc::Channel;

use crate::commands::{git_branch, listening_ports};
use crate::model::{AppStateJson, NavDirection, PaneNode, SplitDirection, WorkspaceManager};
use crate::pty::PtyEvent;
use crate::AppState;

pub fn state_json(workspaces: &RwLock<WorkspaceManager>) -> AppStateJson {
    workspaces.read().state_json()
}

pub fn create_workspace(app: &AppHandle, state: &AppState, name: Option<String>) -> String {
    let home = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "C:\\".into());
    let cwd = state
        .workspaces
        .read()
        .active_workspace()
        .map(|w| w.cwd.clone())
        .filter(|c| !c.is_empty())
        .unwrap_or(home);

    let shell = detect_default_shell(state);
    let id = {
        let mut workspaces = state.workspaces.write();
        let name = name.unwrap_or_else(|| format!("workspace {}", workspaces.workspaces().len() + 1));
        workspaces.create(name, shell, cwd).id.clone()
    };
    crate::commands::emit_state(app, &state.workspaces);
    id
}

pub fn delete_workspace(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String> {
    let (pty_ids, browser_ids) = {
        let workspaces = state.workspaces.read();
        let Some(ws) = workspaces.get(id) else {
            return Err("workspace not found".into());
        };
        (ws.pty_ids(), ws.browser_ids())
    };
    for pty_id in pty_ids {
        let _ = state.pty.kill(&pty_id);
    }
    let removed = state.workspaces.write().delete(id);
    if removed.is_none() {
        return Err("workspace not found".into());
    }
    for pane_id in browser_ids {
        crate::commands::browser::close_webview(app, &pane_id);
    }
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn rename_workspace(app: &AppHandle, state: &AppState, id: &str, name: &str) -> Result<(), String> {
    let mut workspaces = state.workspaces.write();
    if workspaces.get(id).is_none() {
        return Err("workspace not found".into());
    }
    workspaces.rename(id, name.to_string());
    drop(workspaces);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn select_workspace(app: &AppHandle, state: &AppState, id: &str) -> Result<(), String> {
    let mut workspaces = state.workspaces.write();
    if !workspaces.select(id) {
        return Err("workspace not found".into());
    }
    drop(workspaces);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn reorder_workspaces(app: &AppHandle, state: &AppState, ids: Vec<String>) -> Result<(), String> {
    state.workspaces.write().reorder(ids);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn split_pane(
    app: &AppHandle,
    state: &AppState,
    workspace_id: &str,
    direction: SplitDirection,
    command: Option<String>,
    cwd: Option<String>,
) -> Result<(), String> {
    let shell = detect_default_shell(state);
    let fallback_cwd = state
        .workspaces
        .read()
        .get(workspace_id)
        .map(|ws| ws.cwd.clone())
        .unwrap_or_default();
    let cwd = cwd.unwrap_or(fallback_cwd);
    let mut workspaces = state.workspaces.write();
    workspaces.split(workspace_id, direction, shell, cwd, command)?;
    drop(workspaces);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn split_browser_pane(
    app: &AppHandle,
    state: &AppState,
    workspace_id: &str,
    url: &str,
) -> Result<String, String> {
    let mut workspaces = state.workspaces.write();
    let outcome = workspaces.split_browser(workspace_id, url.to_string())?;
    drop(workspaces);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(outcome.new_node_id)
}

pub fn close_pane(app: &AppHandle, state: &AppState, workspace_id: &str, pane_id: &str) -> Result<(), String> {
    let mut workspaces = state.workspaces.write();
    let to_kill = workspaces.close_pane(workspace_id, pane_id)?;
    drop(workspaces);
    for pty_id in to_kill {
        if !pty_id.is_empty() {
            let _ = state.pty.kill(&pty_id);
        }
    }
    crate::commands::browser::close_webview(app, pane_id);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn focus_pane(app: &AppHandle, state: &AppState, workspace_id: &str, pane_id: &str) -> Result<(), String> {
    let mut workspaces = state.workspaces.write();
    if !workspaces.focus_pane(workspace_id, pane_id) {
        return Err("pane not found".into());
    }
    drop(workspaces);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn navigate_panes(app: &AppHandle, state: &AppState, workspace_id: &str, direction: &str) -> Result<(), String> {
    let dir = match direction {
        "next" | "right" | "down" => NavDirection::Next,
        "prev" | "left" | "up" => NavDirection::Prev,
        _ => return Err("direction must be next|prev|left|right|up|down".into()),
    };
    state.workspaces.write().navigate(workspace_id, dir);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn zoom_pane(app: &AppHandle, state: &AppState, workspace_id: &str, pane_id: Option<String>) -> Result<(), String> {
    state
        .workspaces
        .write()
        .zoom_toggle(workspace_id, pane_id.as_deref());
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn resize_pane(
    app: &AppHandle,
    state: &AppState,
    workspace_id: &str,
    split_id: &str,
    ratio: f64,
) -> Result<(), String> {
    let mut workspaces = state.workspaces.write();
    if !workspaces.set_ratio(workspace_id, split_id, ratio) {
        return Err("split not found".into());
    }
    drop(workspaces);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn attach_pty(
    app: &AppHandle,
    state: &AppState,
    workspace_id: &str,
    pane_id: &str,
    pty_id: &str,
    channel: Option<Channel<PtyEvent>>,
    replay: bool,
) -> Result<(), String> {
    state.pty.attach(pty_id, channel, replay)?;
    let mut workspaces = state.workspaces.write();
    if !workspaces.attach_pty(workspace_id, pane_id, pty_id.to_string()) {
        return Err("pane not found".into());
    }
    drop(workspaces);
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn clear_notifications(
    app: &AppHandle,
    state: &AppState,
    workspace_id: &str,
    pane_id: Option<String>,
) -> Result<(), String> {
    state
        .workspaces
        .write()
        .clear_notifications(workspace_id, pane_id.as_deref());
    crate::commands::emit_state(app, &state.workspaces);
    Ok(())
}

pub fn refresh_metadata(app: &AppHandle, state: &AppState) {
    let ids: Vec<String> = state
        .workspaces
        .read()
        .workspaces()
        .iter()
        .map(|w| w.id.clone())
        .collect();

    let mut changed = false;
    {
        let mut workspaces = state.workspaces.write();
        for id in ids {
            let Some(ws) = workspaces.get_mut(&id) else {
                continue;
            };
            let active_cwd = ws
                .active_pane
                .clone()
                .and_then(|p| match ws.layout.find(&p) {
                    Some(PaneNode::Terminal { pty_id, .. }) if !pty_id.is_empty() => {
                        state.pty.cwd(pty_id)
                    }
                    _ => None,
                })
                .filter(|c| !c.is_empty());
            let cwd = active_cwd.unwrap_or_else(|| ws.cwd.clone());
            if cwd != ws.cwd {
                ws.cwd = cwd.clone();
                changed = true;
            }
            let branch = git_branch(&ws.cwd);
            if branch != ws.branch {
                ws.branch = branch;
                changed = true;
            }
        }
    }
    if changed {
        crate::commands::emit_state(app, &state.workspaces);
    }

    let ports: Vec<u32> = listening_ports().into_iter().map(|p| p.port).collect();
    {
        let mut workspaces = state.workspaces.write();
        if let Some(active) = workspaces.active_workspace_mut() {
            if active.ports != ports {
                active.ports = ports;
                drop(workspaces);
                crate::commands::emit_state(app, &state.workspaces);
                return;
            }
        }
    }
}

fn detect_default_shell(state: &AppState) -> String {
    state
        .settings
        .read()
        .shell
        .clone()
        .or_else(|| crate::pty::shell::Shell::detect().map(|s| s.path.display().to_string()))
        .unwrap_or_else(|| "pwsh.exe".into())
}

#[tauri::command]
pub fn get_state(state: State<AppState>) -> AppStateJson {
    state_json(&state.workspaces)
}

#[tauri::command]
pub fn workspace_create(app: AppHandle, state: State<AppState>, name: Option<String>) -> String {
    create_workspace(&app, &state, name)
}

#[tauri::command]
pub fn workspace_delete(app: AppHandle, state: State<AppState>, id: String) -> Result<(), String> {
    delete_workspace(&app, &state, &id)
}

#[tauri::command]
pub fn workspace_rename(app: AppHandle, state: State<AppState>, id: String, name: String) -> Result<(), String> {
    rename_workspace(&app, &state, &id, &name)
}

#[tauri::command]
pub fn workspace_select(app: AppHandle, state: State<AppState>, id: String) -> Result<(), String> {
    select_workspace(&app, &state, &id)
}

#[tauri::command]
pub fn workspace_reorder(app: AppHandle, state: State<AppState>, ids: Vec<String>) -> Result<(), String> {
    reorder_workspaces(&app, &state, ids)
}

#[tauri::command]
pub fn pane_split(
    app: AppHandle,
    state: State<AppState>,
    workspace_id: String,
    direction: SplitDirection,
    command: Option<String>,
    cwd: Option<String>,
) -> Result<(), String> {
    split_pane(&app, &state, &workspace_id, direction, command, cwd)
}

#[tauri::command]
pub fn browser_split(
    app: AppHandle,
    state: State<AppState>,
    workspace_id: String,
    url: String,
) -> Result<String, String> {
    split_browser_pane(&app, &state, &workspace_id, &url)
}

#[tauri::command]
pub fn pane_close(app: AppHandle, state: State<AppState>, workspace_id: String, pane_id: String) -> Result<(), String> {
    close_pane(&app, &state, &workspace_id, &pane_id)
}

#[tauri::command]
pub fn pane_focus(app: AppHandle, state: State<AppState>, workspace_id: String, pane_id: String) -> Result<(), String> {
    focus_pane(&app, &state, &workspace_id, &pane_id)
}

#[tauri::command]
pub fn pane_navigate(app: AppHandle, state: State<AppState>, workspace_id: String, direction: String) -> Result<(), String> {
    navigate_panes(&app, &state, &workspace_id, &direction)
}

#[tauri::command]
pub fn pane_zoom(app: AppHandle, state: State<AppState>, workspace_id: String, pane_id: Option<String>) -> Result<(), String> {
    zoom_pane(&app, &state, &workspace_id, pane_id)
}

#[tauri::command]
pub fn pane_resize(
    app: AppHandle,
    state: State<AppState>,
    workspace_id: String,
    split_id: String,
    ratio: f64,
) -> Result<(), String> {
    resize_pane(&app, &state, &workspace_id, &split_id, ratio)
}

#[tauri::command]
pub fn workspace_register_pty(
    app: AppHandle,
    state: State<AppState>,
    workspace_id: String,
    pane_id: String,
    pty_id: String,
) -> Result<(), String> {
    attach_pty(&app, &state, &workspace_id, &pane_id, &pty_id, None, false)
}

#[tauri::command]
pub fn workspace_attach_pty(
    app: AppHandle,
    state: State<AppState>,
    workspace_id: String,
    pane_id: String,
    pty_id: String,
    channel: Channel<PtyEvent>,
) -> Result<(), String> {
    attach_pty(&app, &state, &workspace_id, &pane_id, &pty_id, Some(channel), true)
}

#[tauri::command]
pub fn workspace_clear_notifications(
    app: AppHandle,
    state: State<AppState>,
    workspace_id: String,
    pane_id: Option<String>,
) -> Result<(), String> {
    clear_notifications(&app, &state, &workspace_id, pane_id)
}