use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::config::settings::session_path;
use crate::model::{PaneNode, Workspace, WorkspaceManager};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub version: u32,
    pub windows: Vec<WindowSnapshot>,
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowSnapshot {
    pub position: [i32; 2],
    pub size: [u32; 2],
    pub workspaces: Vec<WorkspaceSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub id: String,
    pub name: String,
    pub cwd: String,
    pub pane_layout: PaneNode,
    pub panes: Vec<PaneSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaneSnapshot {
    pub id: String,
    pub pty_id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub shell: String,
    pub cwd: String,
    pub url: Option<String>,
    pub scroll_position: usize,
    pub agent_resume: Option<AgentResume>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResume {
    pub agent: String,
    pub session_id: String,
    pub command: String,
}

pub fn save_session(state: &AppState) -> Result<(), String> {
    let workspaces = state.workspaces.read();
    let snapshot = build_snapshot(&workspaces, state)?;

    let path = session_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir failed: {e}"))?;
    }
    let json = serde_json::to_string_pretty(&snapshot).map_err(|e| format!("serialize failed: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write {} failed: {e}", path.display()))?;

    for ws in workspaces.workspaces() {
        for pty_id in ws.pty_ids() {
            if let Some(lines) = state.pty.scrollback(&pty_id) {
                let _ = state.db.lock().save(&pty_id, &lines);
            }
        }
    }
    log::info!("session saved to {}", path.display());
    Ok(())
}

pub fn restore_session(state: &AppState) -> Result<usize, String> {
    let path = session_path();
    let json = std::fs::read_to_string(&path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    let snapshot: SessionSnapshot =
        serde_json::from_str(&json).map_err(|e| format!("parse {} failed: {e}", path.display()))?;

    for pane_pty in state.pty.alive_pty_ids() {
        let _ = state.pty.kill(&pane_pty);
    }
    if let Some(window) = state.app.get_window("main") {
        for webview in window.webviews() {
            if webview.label() != "main" {
                let _ = webview.close();
            }
        }
    }

    let mut manager = WorkspaceManager::new();
    let mut count = 0;
    for window in &snapshot.windows {
        for ws_snap in &window.workspaces {
            let mut ws = Workspace {
                id: ws_snap.id.clone(),
                name: ws_snap.name.clone(),
                cwd: ws_snap.cwd.clone(),
                branch: None,
                ports: Vec::new(),
                notifications: 0,
                layout: ws_snap.pane_layout.clone(),
                active_pane: ws_snap
                    .pane_layout
                    .first_terminal_id()
                    .or_else(|| ws_snap.panes.first().map(|p| p.id.clone())),
            };
            spawn_restored_ptys(state, &mut ws, &ws_snap.panes)?;
            manager.restore(ws);
            count += 1;
        }
    }
    if manager.workspaces().is_empty() {
        return Ok(0);
    }

    {
        let mut lock = state.workspaces.write();
        *lock = manager;
    }
    Ok(count)
}

fn spawn_restored_ptys(state: &AppState, ws: &mut Workspace, panes: &[PaneSnapshot]) -> Result<(), String> {
    let mut pending: Vec<(String, String, String, String, Option<String>)> = Vec::new();
    collect_pending(&ws.layout, &mut pending);

    for (pane_id, pty_id, shell, cwd, command) in pending {
        let pane = panes.iter().find(|p| p.id == pane_id);
        let shell = pane.map(|p| p.shell.clone()).filter(|s| !s.is_empty()).unwrap_or(shell);
        let cwd = pane.map(|p| p.cwd.clone()).filter(|c| !c.is_empty()).unwrap_or(cwd);

        if pty_id.is_empty() {
            continue;
        }
        let channel = tauri::ipc::Channel::new(|_| Ok(()));
        let spec = crate::pty::PtySpec {
            pty_id: pty_id.clone(),
            label: pane_id.clone(),
            shell: Some(shell),
            command,
            cwd: Some(cwd),
            cols: 120,
            rows: 40,
        };
        if let Err(e) = state.pty.spawn(spec, channel) {
            log::warn!("restore: spawn {pane_id} failed: {e}");
            ws.layout.set_terminal_pty(&pane_id, String::new());
            continue;
        }
        if let Ok(Some(lines)) = state.db.lock().load(&pty_id) {
            state.pty.restore_scrollback(&pty_id, lines);
        }
    }
    Ok(())
}

fn collect_pending(node: &PaneNode, out: &mut Vec<(String, String, String, String, Option<String>)>) {
    match node {
        PaneNode::Terminal { id, pty_id, shell, cwd, command, .. } => {
            out.push((id.clone(), pty_id.clone(), shell.clone(), cwd.clone(), command.clone()));
        }
        PaneNode::Browser { .. } => {}
        PaneNode::Split { children, .. } => {
            for c in children {
                collect_pending(c, out);
            }
        }
    }
}

fn build_snapshot(workspaces: &WorkspaceManager, state: &AppState) -> Result<SessionSnapshot, String> {
    let (position, size) = window_geometry(&state.app);
    let workspaces: Vec<WorkspaceSnapshot> = workspaces
        .workspaces()
        .iter()
        .map(|ws| {
            let mut panes = Vec::new();
            collect_panes(&ws.layout, &mut panes);
            WorkspaceSnapshot {
                id: ws.id.clone(),
                name: ws.name.clone(),
                cwd: ws.cwd.clone(),
                pane_layout: ws.layout.clone(),
                panes,
            }
        })
        .collect();

    let saved_at = chrono_now();
    Ok(SessionSnapshot {
        version: 1,
        windows: vec![WindowSnapshot {
            position,
            size,
            workspaces,
        }],
        saved_at,
    })
}

fn collect_panes(node: &PaneNode, out: &mut Vec<PaneSnapshot>) {
    match node {
        PaneNode::Terminal { id, pty_id, shell, cwd, .. } => {
            out.push(PaneSnapshot {
                id: id.clone(),
                pty_id: pty_id.clone(),
                kind: "local".into(),
                shell: shell.clone(),
                cwd: cwd.clone(),
                url: None,
                scroll_position: 0,
                agent_resume: None,
            });
        }
        PaneNode::Browser { id, url, .. } => {
            out.push(PaneSnapshot {
                id: id.clone(),
                pty_id: String::new(),
                kind: "browser".into(),
                shell: String::new(),
                cwd: String::new(),
                url: Some(url.clone()),
                scroll_position: 0,
                agent_resume: None,
            });
        }
        PaneNode::Split { children, .. } => {
            for c in children {
                collect_panes(c, out);
            }
        }
    }
}

fn window_geometry(app: &tauri::AppHandle) -> ([i32; 2], [u32; 2]) {
    let default = ([100, 100], [1440, 900]);
    let Some(window) = app.get_webview_window("main") else {
        return default;
    };
    let pos = window.outer_position().ok();
    let size = window.outer_size().ok();
    (
        pos.map(|p| [p.x, p.y]).unwrap_or(default.0),
        size.map(|s| [s.width, s.height]).unwrap_or(default.1),
    )
}

fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

pub fn session_exists() -> bool {
    session_path().exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_roundtrip_parse() {
        let json = r#"{
            "version": 1,
            "windows": [{
                "position": [100, 100],
                "size": [1920, 1080],
                "workspaces": [{
                    "id": "ws-1",
                    "name": "myproject",
                    "cwd": "C:\\projects\\myproject",
                    "paneLayout": {
                        "type": "split",
                        "id": "split-1",
                        "direction": "vertical",
                        "ratio": 0.5,
                        "children": [
                            {"type": "terminal", "id": "pane-1", "ptyId": "p1", "shell": "pwsh.exe", "cwd": "C:\\projects\\myproject", "title": "t", "zoomed": false, "attention": false},
                            {"type": "terminal", "id": "pane-2", "ptyId": "p2", "shell": "pwsh.exe", "cwd": "C:\\projects\\myproject", "title": "t", "zoomed": false, "attention": false}
                        ]
                    },
                    "panes": []
                }]
            }],
            "savedAt": "0"
        }"#;
        let snapshot: SessionSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.windows.len(), 1);
        let ws = &snapshot.windows[0].workspaces[0];
        assert_eq!(ws.name, "myproject");
        let mut ids = Vec::new();
        ws.pane_layout.terminal_ids(&mut ids);
        assert_eq!(ids, vec!["pane-1", "pane-2"]);
    }

    #[test]
    fn agent_resume_parse() {
        let json = r#"{"agent": "claude", "sessionId": "sess_abc123", "command": "claude --resume sess_abc123"}"#;
        let resume: AgentResume = serde_json::from_str(json).unwrap();
        assert_eq!(resume.agent, "claude");
        assert_eq!(resume.command, "claude --resume sess_abc123");
    }
}