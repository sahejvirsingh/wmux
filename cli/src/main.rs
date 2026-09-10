use std::io::{Read, Write};
use std::time::Duration;

use serde_json::{json, Value};

const PIPE_NAME: &str = r"\\.\pipe\wmux";
const CONNECT_TIMEOUT_MS: u32 = 3000;
const READ_TIMEOUT: Duration = Duration::from_secs(10);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (method, params, json_out) = match parse_args(&args) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("error: {msg}");
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };

    match request(&method, params) {
        Ok(response) => {
            if response.get("ok").and_then(Value::as_bool).unwrap_or(false) {
                let result = response.get("result").cloned().unwrap_or(Value::Null);
                if json_out {
                    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
                } else {
                    print_human(&result);
                }
            } else {
                let error = response
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown error");
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("is the wmux app running?");
            std::process::exit(1);
        }
    }
}

fn parse_args(args: &[String]) -> Result<(String, Value, bool), String> {
    let json_out = args.iter().any(|a| a == "--json");
    let args: Vec<&String> = args.iter().filter(|a| a.as_str() != "--json").collect();
    let Some(command) = args.first().map(|s| s.as_str()) else {
        return Err("no command given".into());
    };

    let get_flag = |name: &str| -> Option<String> {
        args.windows(2)
            .find(|w| w[0].as_str() == name)
            .map(|w| w[1].clone())
    };

    let mut params = json!({});
    let mut hooks_method: Option<&str> = None;

    match command {
        "ping" | "list-workspaces" | "restore-session" | "save-session" => {}
        "new-workspace" => {
            if let Some(name) = get_flag("--name") {
                params["name"] = json!(name);
            }
        }
        "select-workspace" => {
            params["workspace"] = json!(get_flag("--workspace").ok_or("--workspace <id> is required")?);
        }
        "new-split" => {
            let direction = args.get(1).map(|s| s.as_str()).ok_or("usage: wmux new-split <right|down>")?;
            if direction != "right" && direction != "down" {
                return Err(format!("direction must be right or down, got {direction}"));
            }
            params["direction"] = json!(direction);
            if let Some(cmd) = get_flag("--command") {
                params["command"] = json!(cmd);
            }
            if let Some(ws) = get_flag("--workspace") {
                params["workspace"] = json!(ws);
            }
            if let Some(cwd) = get_flag("--cwd") {
                params["cwd"] = json!(cwd);
            }
        }
        "open-browser" => {
            let url = args.get(1).map(|s| s.as_str()).ok_or("usage: wmux open-browser <url>")?;
            params["url"] = json!(url);
            if let Some(ws) = get_flag("--workspace") {
                params["workspace"] = json!(ws);
            }
        }
        "send-text" => {
            let text = args.get(1).ok_or("usage: wmux send-text <text>")?;
            params["data"] = json!(text);
            if let Some(surface) = get_flag("--surface") {
                params["surface"] = json!(surface);
            }
        }
        "read-screen" => {
            if let Some(surface) = get_flag("--surface") {
                params["surface"] = json!(surface);
            }
            if let Some(lines) = get_flag("--lines") {
                params["lines"] = json!(lines.parse::<u64>().map_err(|_| "--lines must be a number")?);
            }
        }
        "notify" => {
            let message = args.get(1).ok_or("usage: wmux notify <message>")?;
            params["message"] = json!(message);
            if let Some(title) = get_flag("--title") {
                params["title"] = json!(title);
            }
        }
        "hooks" => {
            let sub = args.get(1).map(|s| s.as_str()).ok_or("usage: wmux hooks <setup|status>")?;
            match sub {
                "setup" => {
                    if let Some(agent) = args.get(2).map(|s| s.to_string()).or_else(|| get_flag("--agent")) {
                        params["agent"] = json!(agent);
                    }
                    hooks_method = Some("hooks.setup");
                }
                "status" => {
                    hooks_method = Some("hooks.status");
                }
                _ => return Err(format!("unknown hooks subcommand: {sub}")),
            }
        }
        _ => return Err(format!("unknown command: {command}")),
    }

    let method = match command {
        "ping" => "ping",
        "list-workspaces" => "workspace.list",
        "new-workspace" => "workspace.new",
        "select-workspace" => "workspace.select",
        "new-split" => "split.new",
        "open-browser" => "browser.open",
        "send-text" => "terminal.write",
        "read-screen" => "surface.read",
        "notify" => "notify",
        "hooks" => hooks_method.unwrap_or("hooks.status"),
        "restore-session" => "session.restore",
        "save-session" => "session.save",
        _ => unreachable!(),
    };

    Ok((method.to_string(), params, json_out))
}

fn request(method: &str, params: Value) -> Result<Value, String> {
    let mut client = connect()?;
    let req = json!({ "id": "cli-1", "method": method, "params": params });
    let mut line = serde_json::to_string(&req).map_err(|e| e.to_string())?;
    line.push('\n');
    client
        .write_all(line.as_bytes())
        .map_err(|e| format!("write failed: {e}"))?;
    client.flush().map_err(|e| format!("flush failed: {e}"))?;

    client.set_read_timeout(Some(READ_TIMEOUT));
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match client.read(&mut byte) {
            Ok(0) => return Err("no response from wmux".into()),
            Ok(_) => {
                if byte[0] == b'\n' {
                    break;
                }
                buf.push(byte[0]);
            }
            Err(e) => return Err(format!("read failed: {e}")),
        }
    }
    serde_json::from_slice(&buf).map_err(|e| format!("bad response: {e}"))
}

fn connect() -> Result<named_pipe::PipeClient, String> {
    match named_pipe::PipeClient::connect_ms(PIPE_NAME, CONNECT_TIMEOUT_MS) {
        Ok(client) => Ok(client),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::TimedOut {
                Err("timed out waiting for the wmux app".into())
            } else {
                Err(format!("connect failed: {e}"))
            }
        }
    }
}

fn print_human(result: &Value) {
    if result.is_null() || result == &json!({}) {
        return;
    }
    if let Some(workspaces) = result.get("workspaces").and_then(Value::as_array) {
        if workspaces.is_empty() {
            println!("(no workspaces)");
            return;
        }
        for ws in workspaces {
            let id = ws.get("id").and_then(Value::as_str).unwrap_or("?");
            let name = ws.get("name").and_then(Value::as_str).unwrap_or("?");
            let branch = ws.get("branch").and_then(Value::as_str).unwrap_or("");
            let notifications = ws.get("notifications").and_then(Value::as_u64).unwrap_or(0);
            let badge = if notifications > 0 {
                format!(" [{} notifications]", notifications)
            } else {
                String::new()
            };
            println!("{id}\t{name}\t{branch}{badge}");
        }
        return;
    }
    if let Some(id) = result.get("id").and_then(Value::as_str) {
        println!("{id}");
        return;
    }
    if let Some(text) = result.get("text").and_then(Value::as_str) {
        print!("{text}");
        if !text.ends_with('\n') {
            println!();
        }
        return;
    }
    if let Some(restored) = result.get("restored").and_then(Value::as_u64) {
        println!("restored {restored} workspaces");
        return;
    }
    if let Some(reports) = result.as_array() {
        for report in reports {
            let name = report.get("name").and_then(Value::as_str).unwrap_or("?");
            let ok = report.get("ok").and_then(Value::as_bool).unwrap_or(false);
            let message = report.get("message").and_then(Value::as_str).unwrap_or("");
            println!("{name}: {} {message}", if ok { "ok" } else { "failed" });
        }
        return;
    }
    if let Some(pong) = result.get("pong") {
        println!("pong: {pong}");
        return;
    }
    println!("{}", serde_json::to_string_pretty(result).unwrap_or_default());
}

fn usage() -> &'static str {
    "usage: wmux <command> [args]\n\
     \n\
     workspaces:\n\
       wmux list-workspaces [--json]\n\
       wmux new-workspace [--name <name>]\n\
       wmux select-workspace --workspace <id>\n\
       wmux restore-session\n\
       wmux save-session\n\
     \n\
     panes:\n\
       wmux new-split <right|down> [--command <cmd>] [--workspace <id>] [--cwd <dir>]\n\
       wmux open-browser <url> [--workspace <id>]\n\
     \n\
     surfaces:\n\
       wmux send-text <text> [--surface <pty-id>]\n\
       wmux read-screen [--surface <pty-id>] [--lines <n>] [--json]\n\
     \n\
     agent hooks:\n\
       wmux hooks setup [agent]\n\
       wmux hooks status\n\
       wmux notify <message> [--title <title>]\n\
     \n\
     misc:\n\
       wmux ping\n\
       all commands support --json"
}