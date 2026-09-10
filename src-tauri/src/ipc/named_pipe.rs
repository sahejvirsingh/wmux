use std::io::{Read, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Duration;

use named_pipe::{PipeOptions, PipeServer};
use serde_json::{json, Value};
use tauri::AppHandle;

const PIPE_NAME: &str = r"\\.\pipe\wmux";

pub fn serve(app: AppHandle) {
    std::thread::spawn(move || {
        let options = PipeOptions::new(PIPE_NAME);
        loop {
            let mut connecting = match options.single() {
                Ok(c) => c,
                Err(e) => {
                    let _ = writeln!(std::io::stderr(), "[wmux-pipe] create failed: {e}");
                    std::thread::sleep(Duration::from_millis(1000));
                    continue;
                }
            };
            loop {
                let mut server = match connecting.wait() {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = writeln!(std::io::stderr(), "[wmux-pipe] wait failed: {e}");
                        std::thread::sleep(Duration::from_millis(1000));
                        break;
                    }
                };
                let _ = writeln!(std::io::stderr(), "[wmux-pipe] client connected");
                let handled = catch_unwind(AssertUnwindSafe(|| handle_client(&mut server, &app)));
                if handled.is_err() {
                    let _ = writeln!(std::io::stderr(), "[wmux-pipe] request handler panicked");
                }
                match server.disconnect() {
                    Ok(connecting_server) => {
                        connecting = connecting_server;
                    }
                    Err(e) => {
                        let _ = writeln!(std::io::stderr(), "[wmux-pipe] disconnect failed: {e}");
                        std::thread::sleep(Duration::from_millis(100));
                        break;
                    }
                }
            }
        }
    });
}

fn handle_client(server: &mut PipeServer, app: &AppHandle) {
    loop {
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match server.read(&mut byte) {
                Ok(0) => return,
                Ok(_) => {
                    if byte[0] == b'\n' {
                        break;
                    }
                    buf.push(byte[0]);
                    if buf.len() > 1_000_000 {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
        let response = match serde_json::from_slice::<Value>(&buf) {
            Ok(req) => {
                let id = req.get("id").cloned().unwrap_or(Value::Null);
                let method = req.get("method").and_then(Value::as_str).unwrap_or("");
                let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
                match crate::ipc::dispatch(app, method, &params) {
                    Ok(result) => json!({ "id": id, "ok": true, "result": result }),
                    Err(error) => json!({ "id": id, "ok": false, "error": error }),
                }
            }
            Err(_) => json!({ "id": Value::Null, "ok": false, "error": "invalid request" }),
        };
        let mut out = serde_json::to_string(&response).unwrap_or_else(|_| "{}".into());
        out.push('\n');
        if server.write_all(out.as_bytes()).is_err() || server.flush().is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_name_is_stable() {
        assert_eq!(PIPE_NAME, r"\\.\pipe\wmux");
    }
}