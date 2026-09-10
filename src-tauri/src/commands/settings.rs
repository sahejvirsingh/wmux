use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::config::settings::{save_settings, themes_dir, Settings};
use crate::persistence::agent_hooks::{detect_installed, install_hooks, AgentStatus, InstallReport};
use crate::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeMeta {
    pub id: String,
    pub name: String,
    pub content: Option<serde_json::Value>,
}

pub fn get_settings(state: &AppState) -> Settings {
    state.settings.read().clone()
}

pub fn set_settings(app: &AppHandle, state: &AppState, settings: Settings) -> Result<(), String> {
    save_settings(&settings)?;
    *state.settings.write() = settings;
    let _ = app.emit("wmux:settings", ());
    Ok(())
}

pub fn list_themes() -> Vec<ThemeMeta> {
    let mut themes = Vec::new();
    let dir = themes_dir();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let id = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let parsed = std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok());
                let name = parsed
                    .as_ref()
                    .and_then(|v| v.get("name").and_then(|n| n.as_str().map(|s| s.to_string())))
                    .unwrap_or_else(|| id.clone());
                themes.push(ThemeMeta {
                    id,
                    name,
                    content: parsed,
                });
            }
        }
    }
    themes.sort_by(|a, b| a.id.cmp(&b.id));
    themes
}

pub fn hooks_status() -> Vec<AgentStatus> {
    detect_installed()
}

pub fn hooks_install(agent: Option<String>) -> Vec<InstallReport> {
    match agent {
        Some(name) => vec![install_hooks(&name)],
        None => crate::persistence::agent_hooks::AGENTS
            .iter()
            .map(|a| install_hooks(a.name))
            .collect(),
    }
}

#[tauri::command]
pub fn get_settings_cmd(state: State<AppState>) -> Settings {
    get_settings(&state)
}

#[tauri::command]
pub fn set_settings_cmd(app: AppHandle, state: State<AppState>, settings: Settings) -> Result<(), String> {
    set_settings(&app, &state, settings)
}

#[tauri::command]
pub fn list_themes_cmd() -> Vec<ThemeMeta> {
    list_themes()
}

#[tauri::command]
pub fn hooks_status_cmd() -> Vec<AgentStatus> {
    hooks_status()
}

#[tauri::command]
pub fn hooks_install_cmd(agent: Option<String>) -> Vec<InstallReport> {
    hooks_install(agent)
}