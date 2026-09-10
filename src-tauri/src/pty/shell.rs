use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Shell {
    pub path: PathBuf,
    pub name: String,
}

const CANDIDATES: [(&str, &str); 3] = [
    ("pwsh.exe", "PowerShell 7"),
    ("powershell.exe", "Windows PowerShell"),
    ("cmd.exe", "Command Prompt"),
];

impl Shell {
    pub fn detect() -> Option<Shell> {
        for (binary, name) in CANDIDATES {
            if let Some(path) = find_on_path(binary) {
                return Some(Shell { path, name: name.to_string() });
            }
        }
        None
    }

    pub fn from_override(shell: &str) -> Option<Shell> {
        let path = PathBuf::from(shell);
        if path.exists() {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| shell.to_string());
            return Some(Shell { path, name });
        }
        find_on_path(shell).map(|p| Shell {
            name: shell.to_string(),
            path: p,
        })
    }

    pub fn is_cmd(&self) -> bool {
        self.path
            .file_name()
            .map(|f| f.eq_ignore_ascii_case("cmd.exe"))
            .unwrap_or(false)
    }
}

fn find_on_path(binary: &str) -> Option<PathBuf> {
    let output = Command::new("where.exe").arg(binary).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    Some(PathBuf::from(line))
}

pub fn shell_args_for(shell: &Shell, command: Option<&str>) -> Vec<String> {
    let mut args = Vec::new();
    if shell.is_cmd() {
        if let Some(cmd) = command {
            args.push("/c".to_string());
            args.push(cmd.to_string());
        }
    } else {
        if shell.name.eq_ignore_ascii_case("PowerShell 7") {
            args.push("-NoLogo".to_string());
        }
        if let Some(cmd) = command {
            args.push("-NoExit".to_string());
            args.push("-Command".to_string());
            args.push(cmd.to_string());
        }
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_a_shell_on_windows() {
        let shell = Shell::detect();
        assert!(shell.is_some(), "expected a shell on Windows");
        assert!(shell.unwrap().path.exists());
    }

    #[test]
    fn builds_cmd_args() {
        let shell = Shell {
            path: PathBuf::from("cmd.exe"),
            name: "Command Prompt".into(),
        };
        assert_eq!(shell_args_for(&shell, None), Vec::<String>::new());
        assert_eq!(shell_args_for(&shell, Some("npm run dev")), vec!["/c", "npm run dev"]);
    }

    #[test]
    fn builds_pwsh_args() {
        let shell = Shell {
            path: PathBuf::from("pwsh.exe"),
            name: "PowerShell 7".into(),
        };
        assert_eq!(shell_args_for(&shell, Some("npm run dev")), vec!["-NoLogo", "-NoExit", "-Command", "npm run dev"]);
    }
}