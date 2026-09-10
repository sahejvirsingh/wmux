pub mod named_pipe;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use crate::model::SplitDirection;
use crate::notifications::AppNotification;
use crate::AppState;

pub fn start_pipe_server(app: AppHandle) {
    named_pipe::serve(app);
}

pub fn dispatch(app: &AppHandle, method: &str, params: &Value) -> Result<Value, String> {
    let state = app.state::<AppState>();
    match method {
        "ping" => Ok(json!({ "pong": true })),

        "workspace.list" => Ok(json!(state.workspaces.read().state_json())),

        "workspace.new" => {
            let name = params.get("name").and_then(Value::as_str).map(String::from);
            let id = crate::commands::workspace::create_workspace(app, &state, name);
            Ok(json!({ "id": id }))
        }

        "workspace.select" => {
            let id = str_param(params, "workspace")?;
            crate::commands::workspace::select_workspace(app, &state, id)?;
            Ok(json!({}))
        }

        "workspace.delete" => {
            let id = str_param(params, "workspace")?;
            crate::commands::workspace::delete_workspace(app, &state, id)?;
            Ok(json!({}))
        }

        "workspace.rename" => {
            let id = str_param(params, "workspace")?;
            let name = str_param(params, "name")?;
            crate::commands::workspace::rename_workspace(app, &state, id, name)?;
            Ok(json!({}))
        }

        "split.new" => {
            let direction = match str_param(params, "direction")? {
                "right" => SplitDirection::Vertical,
                "down" => SplitDirection::Horizontal,
                other => return Err(format!("direction must be right|down, got {other}")),
            };
            let workspace = params.get("workspace").and_then(Value::as_str).map(String::from);
            let command = params.get("command").and_then(Value::as_str).map(String::from);
            let cwd = params.get("cwd").and_then(Value::as_str).map(String::from);
            let ws_id = match workspace {
                Some(id) => id,
                None => state
                    .workspaces
                    .read()
                    .active_id()
                    .ok_or_else(|| "no active workspace".to_string())?
                    .to_string(),
            };
            crate::commands::workspace::split_pane(app, &state, &ws_id, direction, command, cwd)?;
            Ok(json!({ "workspace": ws_id }))
        }

        "browser.open" => {
            let url = str_param(params, "url")?;
            let ws_id = match params.get("workspace").and_then(Value::as_str) {
                Some(id) => id.to_string(),
                None => state
                    .workspaces
                    .read()
                    .active_id()
                    .ok_or_else(|| "no active workspace".to_string())?
                    .to_string(),
            };
            let pane_id = crate::commands::workspace::split_browser_pane(app, &state, &ws_id, url)?;
            Ok(json!({ "workspace": ws_id, "pane": pane_id }))
        }

        "terminal.write" => {
            let id = params
                .get("surface")
                .and_then(Value::as_str)
                .or_else(|| params.get("ptyId").and_then(Value::as_str))
                .ok_or_else(|| "surface is required".to_string())?;
            let data = str_param(params, "data")?;
            crate::commands::terminal::write_pty(&state, id, data)?;
            Ok(json!({}))
        }

        "surface.read" => {
            let pty_id = resolve_surface(&state, params)?;
            let lines = params
                .get("lines")
                .and_then(Value::as_u64)
                .unwrap_or(100)
                .min(1000) as usize;
            let scrollback = state.pty.scrollback(&pty_id).unwrap_or_default();
            let tail = scrollback.iter().rev().take(lines).rev().cloned().collect::<Vec<_>>();
            Ok(json!({
                "surface": pty_id,
                "text": tail.join("\n"),
                "lines": tail
            }))
        }

        "hooks.setup" => {
            let agent = params.get("agent").and_then(Value::as_str).map(String::from);
            Ok(json!(crate::commands::settings::hooks_install(agent)))
        }

        "hooks.status" => Ok(json!(crate::commands::settings::hooks_status())),

        "session.save" => {
            crate::commands::session::save(app, &state)?;
            Ok(json!({}))
        }

        "session.restore" => {
            let count = crate::commands::session::restore(app, &state)?;
            Ok(json!({ "restored": count }))
        }

        "notify" => {
            let title = params.get("title").and_then(Value::as_str).unwrap_or("wmux").to_string();
            let message = str_param(params, "message")?;
            let notification = AppNotification::rich(title, message);
            let settings = state.settings.read().clone();
            let active_id = state.workspaces.read().active_id().map(String::from);
            if let Some(id) = &active_id {
                state.workspaces.write().mark_attention(id, "");
                crate::commands::emit_state(app, &state.workspaces);
            }
            if settings.notifications.toasts {
                use tauri_plugin_notification::NotificationExt;
                let _ = app
                    .notification()
                    .builder()
                    .title(notification.title)
                    .body(notification.message)
                    .show();
            }
            Ok(json!({ "workspace": active_id }))
        }

        "state.get" => Ok(json!(state.workspaces.read().state_json())),

        _ => Err(format!("unknown method: {method}")),
    }
}

fn resolve_surface(state: &AppState, params: &Value) -> Result<String, String> {
    if let Some(surface) = params.get("surface").and_then(Value::as_str) {
        if !surface.is_empty() {
            return Ok(surface.to_string());
        }
    }
    let workspaces = state.workspaces.read();
    let ws = workspaces
        .active_workspace()
        .ok_or_else(|| "no active workspace".to_string())?;
    let pane = ws
        .active_pane
        .clone()
        .or_else(|| ws.layout.first_terminal_id())
        .ok_or_else(|| "no active pane".to_string())?;
    let pty_id = match ws.layout.find_ref(&pane) {
        Some(crate::model::PaneNode::Terminal { pty_id, .. }) => pty_id.clone(),
        _ => String::new(),
    };
    if pty_id.is_empty() {
        return Err("active pane has no pty".into());
    }
    Ok(pty_id)
}

fn str_param<'a>(params: &'a Value, key: &str) -> Result<&'a str, String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string param: {key}"))
}