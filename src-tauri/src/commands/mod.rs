pub mod browser;
pub mod session;
pub mod settings;
pub mod ssh;
pub mod terminal;
pub mod workspace;

use std::sync::mpsc;
use std::time::Duration;

use parking_lot::RwLock;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::model::WorkspaceManager;

pub fn emit_state(app: &AppHandle, workspaces: &RwLock<WorkspaceManager>) {
    let json = workspaces.read().state_json();
    let _ = app.emit("wmux:state", json);
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    pub port: u32,
    pub process: String,
}

pub fn git_branch(cwd: &str) -> Option<String> {
    let (tx, rx) = mpsc::channel::<std::io::Result<std::process::Output>>();
    let cwd = cwd.to_string();
    std::thread::spawn(move || {
        let result = std::process::Command::new("git")
            .args(["-C", &cwd, "branch", "--show-current"])
            .output();
        let _ = tx.send(result);
    });
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(out)) if out.status.success() => {
            let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
            (!branch.is_empty()).then_some(branch)
        }
        _ => None,
    }
}

pub fn listening_ports() -> Vec<PortInfo> {
    let mut by_pid: std::collections::HashMap<String, Vec<u32>> = std::collections::HashMap::new();
    if let Ok(out) = std::process::Command::new("netstat.exe").args(["-ano"]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5
                && (parts[0] == "TCP" || parts[0] == "UDP")
                && parts[3] == "LISTENING"
            {
                let addr = parts[1];
                if let Some((_, port)) = addr.rsplit_once(':') {
                    if let (Ok(port), Ok(_)) = (port.parse::<u32>(), parts[4].parse::<u32>()) {
                        by_pid.entry(parts[4].to_string()).or_default().push(port);
                    }
                }
            }
        }
    }
    if by_pid.is_empty() {
        return Vec::new();
    }

    let mut names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if let Ok(out) = std::process::Command::new("tasklist.exe")
        .args(["/FO", "CSV", "/NH"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let mut csv = line.split(',');
            let name = csv.next().unwrap_or("").trim_matches('"').to_string();
            let pid = csv.next().unwrap_or("").trim_matches('"').to_string();
            if !name.is_empty() && !pid.is_empty() {
                names.insert(pid, name);
            }
        }
    }

    let mut ports: Vec<PortInfo> = by_pid
        .into_iter()
        .flat_map(|(pid, ps)| {
            let process = names.get(&pid).cloned().unwrap_or_else(|| "unknown".into());
            ps.into_iter().map(move |port| PortInfo {
                port,
                process: process.clone(),
            })
        })
        .collect();
    ports.sort_by_key(|p| p.port);
    ports.dedup_by_key(|p| p.port);
    ports
}