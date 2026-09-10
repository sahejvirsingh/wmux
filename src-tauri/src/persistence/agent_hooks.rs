use std::path::PathBuf;
use std::process::Command;

use serde::Serialize;

pub struct AgentDef {
    pub name: &'static str,
    pub binary: &'static str,
    pub resume_template: &'static str,
    pub hooks_dir: Option<&'static str>,
}

pub const AGENTS: [AgentDef; 5] = [
    AgentDef {
        name: "claude",
        binary: "claude.exe",
        resume_template: "claude --resume {id}",
        hooks_dir: Some("Claude\\hooks"),
    },
    AgentDef {
        name: "codex",
        binary: "codex.exe",
        resume_template: "codex resume {id}",
        hooks_dir: Some("codex\\hooks"),
    },
    AgentDef {
        name: "gemini",
        binary: "gemini.exe",
        resume_template: "gemini --resume {id}",
        hooks_dir: Some("gemini\\hooks"),
    },
    AgentDef {
        name: "agy",
        binary: "agy.exe",
        resume_template: "agy --conversation {id}",
        hooks_dir: Some("agy\\hooks"),
    },
    AgentDef {
        name: "opencode",
        binary: "opencode.exe",
        resume_template: "opencode --session {id}",
        hooks_dir: None,
    },
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    pub name: String,
    pub installed: bool,
    pub binary_path: Option<String>,
    pub hooks_installed: bool,
    pub hooks_supported: bool,
    pub resume_command: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallReport {
    pub name: String,
    pub ok: bool,
    pub message: String,
}

#[allow(dead_code)]
pub fn resume_command(agent: &str, session_id: &str) -> Option<String> {
    AGENTS
        .iter()
        .find(|a| a.name == agent)
        .map(|a| a.resume_template.replace("{id}", session_id))
}

pub fn detect_installed() -> Vec<AgentStatus> {
    AGENTS
        .iter()
        .map(|agent| {
            let binary_path = find_binary(agent.binary);
            let hooks_installed = agent
                .hooks_dir
                .map(|_| hooks_dir(agent).map(|d| d.join("wmux-hook.cmd").exists()).unwrap_or(false))
                .unwrap_or(false);
            AgentStatus {
                name: agent.name.to_string(),
                installed: binary_path.is_some(),
                binary_path,
                hooks_installed,
                hooks_supported: agent.hooks_dir.is_some(),
                resume_command: agent.resume_template.replace("{id}", "<session-id>"),
            }
        })
        .collect()
}

pub fn install_hooks(agent: &str) -> InstallReport {
    let Some(def) = AGENTS.iter().find(|a| a.name == agent) else {
        return InstallReport {
            name: agent.to_string(),
            ok: false,
            message: format!("unknown agent: {agent}"),
        };
    };
    let Some(_dir) = def.hooks_dir else {
        return InstallReport {
            name: agent.to_string(),
            ok: false,
            message: format!("{agent} does not support file-based hooks"),
        };
    };
    let Some(root) = hooks_dir(def) else {
        return InstallReport {
            name: agent.to_string(),
            ok: false,
            message: "could not resolve hooks directory".into(),
        };
    };
    let result = write_hooks(def, &root);
    match result {
        Ok(()) => InstallReport {
            name: agent.to_string(),
            ok: true,
            message: format!("hooks installed at {}", root.display()),
        },
        Err(e) => InstallReport {
            name: agent.to_string(),
            ok: false,
            message: e,
        },
    }
}

fn hooks_dir(def: &AgentDef) -> Option<PathBuf> {
    let dir = def.hooks_dir?;
    let base = std::env::var("APPDATA").ok()?;
    Some(PathBuf::from(base).join(dir))
}

fn cli_path() -> PathBuf {
    if let Ok(exe) = std::env::var("WMUX_EXE") {
        return PathBuf::from(exe);
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local).join("wmux").join("wmux.exe");
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("wmux.exe")
}

fn write_hooks(def: &AgentDef, root: &PathBuf) -> Result<(), String> {
    if def.name == "claude" {
        write_claude_hooks(root)?;
    } else {
        write_generic_hooks(def, root)?;
    }
    Ok(())
}

fn write_claude_hooks(root: &PathBuf) -> Result<(), String> {
    let cli = cli_path();
    let cmd_script = root.join("wmux-hook.cmd");
    let script = format!(
        "@echo off\r\n\"{}\" notify \"Claude Code: %*\"" ,
        cli.display()
    );
    std::fs::write(&cmd_script, script).map_err(|e| format!("write {} failed: {e}", cmd_script.display()))?;

    let hooks_json = format!(
        "{{\"matcher\":\"General\",\"hooks\":[{{\"type\":\"command\",\"command\":\"\\\"{}\\\" \\\"%*\\\"\"}}]}}",
        cmd_script.display().to_string().replace('\\', "\\\\")
    );

    for event in ["Stop", "Notification"] {
        let dir = root.join(event);
        std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {} failed: {e}", dir.display()))?;
        let file = dir.join("General.json");
        std::fs::write(&file, &hooks_json).map_err(|e| format!("write {} failed: {e}", file.display()))?;
    }
    Ok(())
}

fn write_generic_hooks(def: &AgentDef, root: &PathBuf) -> Result<(), String> {
    let cli = cli_path();
    let cmd_script = root.join("wmux-hook.cmd");
    let script = format!(
        "@echo off\r\n\"{}\" notify \"{}: %*\"" ,
        cli.display(),
        def.name
    );
    std::fs::write(&cmd_script, script).map_err(|e| format!("write {} failed: {e}", cmd_script.display()))
}

fn find_binary(binary: &str) -> Option<String> {
    let output = Command::new("where.exe").arg(binary).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().next().map(|l| l.trim().to_string()).filter(|l| !l.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_command_template() {
        assert_eq!(
            resume_command("claude", "sess_abc"),
            Some("claude --resume sess_abc".to_string())
        );
        assert_eq!(
            resume_command("codex", "sess_abc"),
            Some("codex resume sess_abc".to_string())
        );
        assert_eq!(resume_command("nope", "x"), None);
    }

    #[test]
    fn install_unknown_agent_fails() {
        let report = install_hooks("nope");
        assert!(!report.ok);
    }
}