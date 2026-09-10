use tauri::State;
use tauri::ipc::Channel;

use crate::pty::{PtyEvent, PtyInfo, PtySpec};
use crate::AppState;

pub fn spawn_pty(state: &AppState, spec: PtySpec, channel: Channel<PtyEvent>) -> Result<(), String> {
    state.pty.spawn(spec, channel)
}

pub fn write_pty(state: &AppState, pty_id: &str, data: &str) -> Result<(), String> {
    state.pty.write(pty_id, data)
}

pub fn resize_pty(state: &AppState, pty_id: &str, cols: u16, rows: u16) -> Result<(), String> {
    state.pty.resize(pty_id, cols, rows)
}

pub fn kill_pty(state: &AppState, pty_id: &str) -> Result<(), String> {
    state.pty.kill(pty_id)
}

pub fn detach_pty(state: &AppState, pty_id: &str) {
    state.pty.detach(pty_id);
}

pub fn list_ptys(state: &AppState) -> Vec<PtyInfo> {
    state.pty.list()
}

#[tauri::command]
pub fn spawn_pty_cmd(state: State<AppState>, spec: PtySpec, channel: Channel<PtyEvent>) -> Result<(), String> {
    spawn_pty(&state, spec, channel)
}

#[tauri::command]
pub fn write_pty_cmd(state: State<AppState>, pty_id: String, data: String) -> Result<(), String> {
    write_pty(&state, &pty_id, &data)
}

#[tauri::command]
pub fn resize_pty_cmd(state: State<AppState>, pty_id: String, cols: u16, rows: u16) -> Result<(), String> {
    resize_pty(&state, &pty_id, cols, rows)
}

#[tauri::command]
pub fn kill_pty_cmd(state: State<AppState>, pty_id: String) -> Result<(), String> {
    kill_pty(&state, &pty_id)
}

#[tauri::command]
pub fn detach_pty_cmd(state: State<AppState>, pty_id: String) {
    detach_pty(&state, &pty_id)
}

#[tauri::command]
pub fn list_ptys_cmd(state: State<AppState>) -> Vec<PtyInfo> {
    list_ptys(&state)
}