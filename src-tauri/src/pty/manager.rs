use std::collections::VecDeque;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

use crate::notifications::{AppNotification, OscEvent, OscParser};
use crate::pty::shell::{shell_args_for, Shell};

const BATCH_INTERVAL: Duration = Duration::from_millis(16);
const MAX_LINE_LEN: usize = 4096;
const RING_CAPACITY: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PtyEvent {
    Output { data: String },
    Notification { notification: AppNotification },
    Title { title: String },
    Cwd { cwd: String },
    Scrollback { data: String },
    Exit { code: i32 },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtySpec {
    pub pty_id: String,
    pub label: String,
    pub shell: Option<String>,
    pub command: Option<String>,
    pub cwd: Option<String>,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyInfo {
    pub id: String,
    pub label: String,
    pub alive: bool,
    pub cwd: Option<String>,
    pub title: Option<String>,
}

pub type Sink = dyn Fn(&str, PtySinkEvent) + Send + Sync;

#[derive(Debug, Clone)]
pub enum PtySinkEvent {
    Notification(AppNotification),
    Title(String),
    Cwd(String),
    Exited(i32),
}

struct Entry {
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    child: Arc<Mutex<Option<Box<dyn Child + Send + Sync>>>>,
    channel: Mutex<Option<Channel<PtyEvent>>>,
    ring: Mutex<RingBuffer>,
    cwd: Mutex<Option<String>>,
    title: Mutex<Option<String>>,
    stop: Arc<AtomicBool>,
    alive: AtomicBool,
    label: String,
}

enum FlushMsg {
    Data(String, Vec<OscEvent>),
    Exit(i32),
}

#[derive(Clone)]
pub struct PtyManager {
    entries: Arc<Mutex<std::collections::HashMap<String, Arc<Entry>>>>,
    sink: Option<Arc<Sink>>,
}

pub struct RingBuffer {
    lines: VecDeque<String>,
    partial: String,
    max: usize,
}

impl RingBuffer {
    pub fn new(max: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            partial: String::new(),
            max,
        }
    }

    pub fn push(&mut self, data: &str) {
        for ch in data.chars() {
            if ch == '\n' {
                let partial = std::mem::take(&mut self.partial);
                self.push_line(partial);
            } else if ch != '\r' {
                self.partial.push(ch);
                if self.partial.chars().count() > MAX_LINE_LEN {
                    let partial = std::mem::take(&mut self.partial);
                    self.push_line(partial);
                }
            }
        }
    }

    fn push_line(&mut self, line: String) {
        if self.lines.len() >= self.max {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn flush_partial(&mut self) {
        if !self.partial.is_empty() {
            let partial = std::mem::take(&mut self.partial);
            self.push_line(partial);
        }
    }

    pub fn snapshot(&mut self) -> Vec<String> {
        self.flush_partial();
        self.lines.iter().cloned().collect()
    }

    pub fn restore(&mut self, lines: Vec<String>) {
        self.lines.clear();
        self.partial.clear();
        for line in lines {
            self.push_line(line);
        }
    }
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(std::collections::HashMap::new())),
            sink: None,
        }
    }

    pub fn set_sink(&mut self, sink: Arc<Sink>) {
        self.sink = Some(sink);
    }

    pub fn spawn(&self, spec: PtySpec, channel: Channel<PtyEvent>) -> Result<(), String> {
        let shell = match spec.shell.as_deref() {
            Some(s) => Shell::from_override(s).ok_or_else(|| format!("shell not found: {s}"))?,
            None => Shell::detect().ok_or_else(|| "no shell found on system".to_string())?,
        };

        let cwd = spec.cwd.clone().and_then(|c| {
            let p = PathBuf::from(&c);
            p.exists().then_some(p)
        });

        let mut cmd = CommandBuilder::new(&shell.path);
        for arg in shell_args_for(&shell, spec.command.as_deref()) {
            cmd.arg(arg);
        }
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("WINDOW_TITLE", "wmux");
        if let Some(cwd) = &cwd {
            cmd.cwd(cwd);
        }

        let system = native_pty_system();
        let pair = system
            .openpty(PtySize {
                rows: spec.rows,
                cols: spec.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("openpty failed: {e}"))?;

        let (master, slave) = (pair.master, pair.slave);
        let child: Box<dyn Child + Send + Sync> = slave
            .spawn_command(cmd)
            .map_err(|e| format!("spawn failed: {e}"))?;
        let mut reader = master
            .try_clone_reader()
            .map_err(|e| format!("try_clone_reader failed: {e}"))?;
        let writer = master
            .take_writer()
            .map_err(|e| format!("take_writer failed: {e}"))?;

        let stop = Arc::new(AtomicBool::new(false));
        let child_slot = Arc::new(Mutex::new(Some(child)));
        let entry = Arc::new(Entry {
            master: Mutex::new(master),
            writer: Mutex::new(writer),
            child: child_slot.clone(),
            channel: Mutex::new(Some(channel)),
            ring: Mutex::new(RingBuffer::new(RING_CAPACITY)),
            cwd: Mutex::new(spec.cwd.clone()),
            title: Mutex::new(None),
            stop: stop.clone(),
            alive: AtomicBool::new(true),
            label: spec.label.clone(),
        });

        self.entries
            .lock()
            .insert(spec.pty_id.clone(), entry.clone());

        let sink = self.sink.clone();
        let (tx, rx) = mpsc::channel::<FlushMsg>();
        let reader_tx = tx.clone();
        let reader_stop = stop.clone();

        std::thread::spawn(move || {
            let mut osc = OscParser::new();
            let mut buf = [0u8; 16384];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let events = osc.feed(&buf[..n]);
                        let data = String::from_utf8_lossy(&buf[..n]).into_owned();
                        if reader_tx.send(FlushMsg::Data(data, events)).is_err() {
                            break;
                        }
                    }
                }
                if reader_stop.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        let monitor_child = child_slot.clone();
        let monitor_tx = tx.clone();
        let monitor_id = spec.pty_id.clone();
        let entries = self.entries.clone();
        std::thread::spawn(move || {
            let t0 = std::time::Instant::now();
            let mut taken = monitor_child.lock().take();
            let code = match taken.as_mut() {
                Some(c) => {
                    loop {
                        if let Ok(Some(status)) = c.try_wait() {
                            break status.exit_code() as i32;
                        }
                        if t0.elapsed() > Duration::from_secs(3600) {
                            break -1;
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
                None => -1,
            };
            let _ = monitor_tx.send(FlushMsg::Exit(code));
            entries.lock().remove(&monitor_id);
        });

        let flusher_entry = entry.clone();
        let flusher_id = spec.pty_id.clone();
        std::thread::spawn(move || {
            let mut acc = String::new();
            loop {
                let msg = match rx.recv_timeout(BATCH_INTERVAL) {
                    Ok(m) => m,
                    Err(RecvTimeoutError::Timeout) => {
                        if !acc.is_empty() {
                            flusher_entry
                                .channel
                                .lock()
                                .as_ref()
                                .map(|c| c.send(PtyEvent::Output { data: std::mem::take(&mut acc) }));
                        }
                        if stop.load(Ordering::Relaxed) {
                            break;
                        }
                        continue;
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        if !acc.is_empty() {
                            flusher_entry
                                .channel
                                .lock()
                                .as_ref()
                                .map(|c| c.send(PtyEvent::Output { data: std::mem::take(&mut acc) }));
                        }
                        break;
                    }
                };
                match msg {
                    FlushMsg::Data(data, events) => {
                        flusher_entry.ring.lock().push(&data);
                        acc.push_str(&data);
                        for event in events {
                            flusher_entry.handle_osc(&flusher_id, event, sink.as_deref());
                        }
                        if acc.len() >= 65536 {
                            flusher_entry
                                .channel
                                .lock()
                                .as_ref()
                                .map(|c| c.send(PtyEvent::Output { data: std::mem::take(&mut acc) }));
                        }
                    }
                    FlushMsg::Exit(code) => {
                        flusher_entry.ring.lock().flush_partial();
                        flusher_entry.alive.store(false, Ordering::Relaxed);
                        flusher_entry
                            .channel
                            .lock()
                            .as_ref()
                            .map(|c| c.send(PtyEvent::Exit { code }));
                        if let Some(sink) = sink.as_deref() {
                            sink(&flusher_id, PtySinkEvent::Exited(code));
                        }
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn attach(
        &self,
        pty_id: &str,
        channel: Option<Channel<PtyEvent>>,
        replay: bool,
    ) -> Result<(), String> {
        let entries = self.entries.lock();
        let entry = entries
            .get(pty_id)
            .ok_or_else(|| format!("pty not found: {pty_id}"))?;
        if channel.is_some() {
            *entry.channel.lock() = channel;
        }
        if replay {
            let scrollback = entry.ring.lock().snapshot().join("\r\n");
            if !scrollback.is_empty() {
                entry
                    .channel
                    .lock()
                    .as_ref()
                    .map(|c| c.send(PtyEvent::Scrollback { data: scrollback }));
            }
        }
        Ok(())
    }

    pub fn detach(&self, pty_id: &str) {
        if let Some(entry) = self.entries.lock().get(pty_id) {
            *entry.channel.lock() = None;
        }
    }

    pub fn write(&self, pty_id: &str, data: &str) -> Result<(), String> {
        let entries = self.entries.lock();
        let entry = entries
            .get(pty_id)
            .ok_or_else(|| format!("pty not found: {pty_id}"))?;
        if !entry.alive.load(Ordering::Relaxed) {
            return Err("pty is not running".into());
        }
        let mut writer = entry.writer.lock();
        writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("write failed: {e}"))?;
        writer.flush().map_err(|e| format!("flush failed: {e}"))
    }

    pub fn resize(&self, pty_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let entries = self.entries.lock();
        let entry = entries
            .get(pty_id)
            .ok_or_else(|| format!("pty not found: {pty_id}"))?;
        let result = entry.master.lock().resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
        result.map_err(|e| format!("resize failed: {e}"))
    }

    pub fn kill(&self, pty_id: &str) -> Result<(), String> {
        let entries = self.entries.lock();
        if let Some(entry) = entries.get(pty_id) {
            entry.stop.store(true, Ordering::Relaxed);
            if let Some(child) = entry.child.lock().as_mut() {
                child.kill().ok();
            }
            entry.alive.store(false, Ordering::Relaxed);
            Ok(())
        } else {
            Err(format!("pty not found: {pty_id}"))
        }
    }

    pub fn kill_all(&self) {
        let entries = self.entries.lock();
        for entry in entries.values() {
            entry.stop.store(true, Ordering::Relaxed);
            if let Some(child) = entry.child.lock().as_mut() {
                child.kill().ok();
            }
            entry.alive.store(false, Ordering::Relaxed);
        }
    }

    pub fn is_alive(&self, pty_id: &str) -> bool {
        self.entries
            .lock()
            .get(pty_id)
            .map(|e| e.alive.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    pub fn scrollback(&self, pty_id: &str) -> Option<Vec<String>> {
        self.entries
            .lock()
            .get(pty_id)
            .map(|e| e.ring.lock().snapshot())
    }

    pub fn restore_scrollback(&self, pty_id: &str, lines: Vec<String>) {
        if let Some(entry) = self.entries.lock().get(pty_id) {
            entry.ring.lock().restore(lines);
        }
    }

    pub fn set_cwd(&self, pty_id: &str, cwd: String) {
        if let Some(entry) = self.entries.lock().get(pty_id) {
            *entry.cwd.lock() = Some(cwd);
        }
    }

    pub fn cwd(&self, pty_id: &str) -> Option<String> {
        self.entries.lock().get(pty_id).and_then(|e| e.cwd.lock().clone())
    }

    pub fn title(&self, pty_id: &str) -> Option<String> {
        self.entries.lock().get(pty_id).and_then(|e| e.title.lock().clone())
    }

    pub fn list(&self) -> Vec<PtyInfo> {
        let entries = self.entries.lock();
        let mut out: Vec<PtyInfo> = entries
            .iter()
            .map(|(id, e)| PtyInfo {
                id: id.clone(),
                label: e.label.clone(),
                alive: e.alive.load(Ordering::Relaxed),
                cwd: e.cwd.lock().clone(),
                title: e.title.lock().clone(),
            })
            .collect();
        out.sort_by(|a, b| a.label.cmp(&b.label));
        out
    }

    pub fn alive_pty_ids(&self) -> Vec<String> {
        self.entries
            .lock()
            .iter()
            .filter(|(_, e)| e.alive.load(Ordering::Relaxed))
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Entry {
    fn handle_osc(&self, pty_id: &str, event: OscEvent, sink: Option<&Sink>) {
        match event {
            OscEvent::Desktop { message } => {
                let n = AppNotification::desktop(message);
                self.emit_notification(pty_id, n.clone(), sink);
            }
            OscEvent::Progress { message } => {
                let n = AppNotification::progress(message);
                self.emit_notification(pty_id, n.clone(), sink);
            }
            OscEvent::Rich { title, message } => {
                let n = AppNotification::rich(title, message);
                self.emit_notification(pty_id, n.clone(), sink);
            }
            OscEvent::Title { title } => {
                *self.title.lock() = Some(title.clone());
                if let Some(sink) = sink {
                    sink(pty_id, PtySinkEvent::Title(title));
                }
            }
            OscEvent::Cwd { cwd } => {
                *self.cwd.lock() = Some(cwd.clone());
                if let Some(sink) = sink {
                    sink(pty_id, PtySinkEvent::Cwd(cwd));
                }
            }
        }
    }

    fn emit_notification(&self, pty_id: &str, n: AppNotification, sink: Option<&Sink>) {
        self.channel
            .lock()
            .as_ref()
            .map(|c| c.send(PtyEvent::Notification { notification: n.clone() }));
        if let Some(sink) = sink {
            sink(pty_id, PtySinkEvent::Notification(n));
        }
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_spec(id: &str, command: &str) -> PtySpec {
        PtySpec {
            pty_id: id.to_string(),
            label: id.to_string(),
            shell: None,
            command: Some(command.to_string()),
            cwd: None,
            cols: 120,
            rows: 40,
        }
    }

    #[test]
    fn spawn_echo_and_receive_output() {
        let mgr = PtyManager::new();
        let channel = Channel::new(|_| Ok(()));
        mgr.spawn(test_spec("echo-1", "echo hello-from-pty"), channel).unwrap();

        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            let lines = mgr.scrollback("echo-1").unwrap_or_default();
            if lines.iter().any(|l| l.contains("hello-from-pty")) {
                break;
            }
            if std::time::Instant::now() > deadline {
                panic!("did not receive echo output in time");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        mgr.kill("echo-1").ok();
    }

    #[test]
    fn exits_cleanly() {
        let mgr = PtyManager::new();
        let channel = Channel::new(|_| Ok(()));
        let mut spec = test_spec("exit-1", "exit 3");
        spec.shell = Some("cmd.exe".to_string());
        mgr.spawn(spec, channel).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while std::time::Instant::now() < deadline && mgr.is_alive("exit-1") {
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(!mgr.is_alive("exit-1"), "pty did not exit in time");
    }

    #[test]
    fn resize_and_write_do_not_error() {
        let mgr = PtyManager::new();
        let channel = Channel::new(|_| Ok(()));
        mgr.spawn(test_spec("rw-1", "powershell -NoProfile -Command Start-Sleep -Seconds 30"), channel).unwrap();
        mgr.resize("rw-1", 80, 24).unwrap();
        mgr.write("rw-1", "echo hi\r").unwrap();
        mgr.kill("rw-1").unwrap();
    }

    #[test]
    fn ring_buffer_caps_lines() {
        let mut ring = RingBuffer::new(3);
        ring.push("a\nb\nc\nd\n");
        ring.flush_partial();
        assert_eq!(ring.snapshot(), vec!["b", "c", "d"]);
    }

    #[test]
    fn ring_buffer_restore() {
        let mut ring = RingBuffer::new(10);
        ring.restore(vec!["x".into(), "y".into()]);
        assert_eq!(ring.snapshot(), vec!["x", "y"]);
    }
}