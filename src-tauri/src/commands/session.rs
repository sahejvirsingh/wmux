use tauri::{AppHandle, State};

use crate::persistence::snapshot::{restore_session, save_session, session_exists};
use crate::AppState;

pub fn save(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let _ = app;
    save_session(state)
}

pub fn restore(app: &AppHandle, state: &AppState) -> Result<usize, String> {
    let count = restore_session(state)?;
    crate::commands::emit_state(app, &state.workspaces);
    Ok(count)
}

pub fn exists() -> bool {
    session_exists()
}

#[tauri::command]
pub fn session_save(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    save(&app, &state)
}

#[tauri::command]
pub fn session_restore(app: AppHandle, state: State<AppState>) -> Result<usize, String> {
    restore(&app, &state)
}

#[tauri::command]
pub fn session_exists_cmd() -> bool {
    exists()
}