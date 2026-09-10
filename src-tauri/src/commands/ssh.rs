use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::ssh::{ssh_command, ssh_test, SshAuth, SshTarget, SshTestResult};
use crate::config::settings::wmux_dir;
use crate::pty::PtySpec;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSshConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub auth: SshAuth,
}

fn connections_path() -> std::path::PathBuf {
    wmux_dir().join("ssh_connections.json")
}

pub fn test_connection(target: &SshTarget, auth: &SshAuth) -> SshTestResult {
    ssh_test(target, auth)
}

pub fn connect_ssh(
    app: &AppHandle,
    state: &AppState,
    target: &SshTarget,
    workspace_name: Option<String>,
) -> Result<String, String> {
    let args = ssh_command(target);
    let command = format!("ssh {}", args.join(" "));
    let name = workspace_name.unwrap_or_else(|| format!("{}@{}", target.user, target.host));

    let shell = state
        .settings
        .read()
        .shell
        .clone()
        .or_else(|| crate::pty::shell::Shell::detect().map(|s| s.path.display().to_string()))
        .unwrap_or_else(|| "pwsh.exe".into());

    let id = {
        let mut workspaces = state.workspaces.write();
        workspaces.create(name, shell, String::new()).id.clone()
    };

    let channel = tauri::ipc::Channel::new(|_| Ok(()));
    let spec = PtySpec {
        pty_id: format!("ssh-{id}-1"),
        label: format!("ssh-{}", target.host),
        shell: None,
        command: Some(command),
        cwd: None,
        cols: 120,
        rows: 40,
    };
    state.pty.spawn(spec, channel)?;
    crate::commands::emit_state(app, &state.workspaces);
    Ok(id)
}

pub fn save_connection(connection: SavedSshConnection) -> Result<(), String> {
    let path = connections_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir failed: {e}"))?;
    }
    let mut connections = list_connections();
    connections.retain(|c| c.id != connection.id);
    connections.push(connection);
    let json = serde_json::to_string_pretty(&connections).map_err(|e| format!("serialize failed: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write failed: {e}"))
}

pub fn list_connections() -> Vec<SavedSshConnection> {
    let path = connections_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn delete_connection(id: &str) -> Result<(), String> {
    let mut connections = list_connections();
    connections.retain(|c| c.id != id);
    let json = serde_json::to_string_pretty(&connections).map_err(|e| format!("serialize failed: {e}"))?;
    std::fs::write(connections_path(), json).map_err(|e| format!("write failed: {e}"))
}

#[tauri::command]
pub fn ssh_test_cmd(target: SshTarget, auth: SshAuth) -> SshTestResult {
    test_connection(&target, &auth)
}

#[tauri::command]
pub fn ssh_connect_cmd(
    app: AppHandle,
    state: State<AppState>,
    target: SshTarget,
    workspace_name: Option<String>,
) -> Result<String, String> {
    connect_ssh(&app, &state, &target, workspace_name)
}

#[tauri::command]
pub fn ssh_save_cmd(connection: SavedSshConnection) -> Result<(), String> {
    save_connection(connection)
}

#[tauri::command]
pub fn ssh_list_cmd() -> Vec<SavedSshConnection> {
    list_connections()
}

#[tauri::command]
pub fn ssh_delete_cmd(id: String) -> Result<(), String> {
    delete_connection(&id)
}