use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum OscEvent {
    Desktop { message: String },
    Progress { message: String },
    Rich { title: String, message: String },
    Title { title: String },
    Cwd { cwd: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Scanning,
    Esc,
    Payload,
    EscInPayload,
}

const MAX_PAYLOAD: usize = 4096;

pub struct OscParser {
    state: State,
    payload: Vec<u8>,
    overflow: bool,
}

impl Default for OscParser {
    fn default() -> Self {
        Self::new()
    }
}

impl OscParser {
    pub fn new() -> Self {
        Self {
            state: State::Scanning,
            payload: Vec::new(),
            overflow: false,
        }
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Vec<OscEvent> {
        let mut events = Vec::new();
        for &b in chunk {
            self.step(b, &mut events);
        }
        events
    }

    fn step(&mut self, b: u8, events: &mut Vec<OscEvent>) {
        match self.state {
            State::Scanning => {
                if b == 0x1b {
                    self.state = State::Esc;
                }
            }
            State::Esc => {
                if b == 0x5d {
                    self.payload.clear();
                    self.overflow = false;
                    self.state = State::Payload;
                } else {
                    self.state = State::Scanning;
                }
            }
            State::Payload => {
                if b == 0x1b {
                    self.state = State::EscInPayload;
                } else if b == 0x07 {
                    self.emit(events);
                    self.state = State::Scanning;
                } else if self.payload.len() < MAX_PAYLOAD {
                    self.payload.push(b);
                } else {
                    self.overflow = true;
                }
            }
            State::EscInPayload => {
                if b == 0x5c {
                    self.emit(events);
                    self.state = State::Scanning;
                } else {
                    if !self.overflow && self.payload.len() < MAX_PAYLOAD {
                        self.payload.push(0x1b);
                    }
                    if b == 0x5d {
                        self.payload.clear();
                        self.overflow = false;
                        self.state = State::Payload;
                    } else {
                        if !self.overflow && self.payload.len() < MAX_PAYLOAD {
                            self.payload.push(b);
                        } else {
                            self.overflow = true;
                        }
                        self.state = State::Payload;
                    }
                }
            }
        }
    }

    fn emit(&mut self, events: &mut Vec<OscEvent>) {
        if self.overflow || self.payload.is_empty() {
            return;
        }
        let raw = String::from_utf8_lossy(&self.payload).into_owned();
        if let Some(event) = parse_osc(&raw) {
            events.push(event);
        }
    }
}

fn parse_osc(raw: &str) -> Option<OscEvent> {
    let (code, rest) = match raw.split_once(';') {
        Some((code, rest)) => (code, rest),
        None => return None,
    };
    match code {
        "9" => Some(OscEvent::Desktop {
            message: rest.to_string(),
        }),
        "99" => Some(OscEvent::Progress {
            message: rest.to_string(),
        }),
        "777" => {
            let (title, message) = match rest.split_once(';') {
                Some((t, m)) => (t.to_string(), m.to_string()),
                None => ("wmux".to_string(), rest.to_string()),
            };
            Some(OscEvent::Rich { title, message })
        }
        "0" | "2" => Some(OscEvent::Title {
            title: rest.to_string(),
        }),
        "7" => Some(OscEvent::Cwd {
            cwd: percent_decode(rest),
        }),
        _ => None,
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = hex_val(bytes[i + 1]);
            let lo = hex_val(bytes[i + 2]);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push(hi * 16 + lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_desktop_notification() {
        let mut p = OscParser::new();
        let events = p.feed(b"\x1b]9;build finished\x1b\\");
        assert_eq!(
            events,
            vec![OscEvent::Desktop {
                message: "build finished".into()
            }]
        );
    }

    #[test]
    fn parses_bel_terminated() {
        let mut p = OscParser::new();
        let events = p.feed(b"\x1b]9;done\x07");
        assert_eq!(events, vec![OscEvent::Desktop { message: "done".into() }]);
    }

    #[test]
    fn parses_rich_notification() {
        let mut p = OscParser::new();
        let events = p.feed(b"\x1b]777;agent;permission required\x1b\\");
        assert_eq!(
            events,
            vec![OscEvent::Rich {
                title: "agent".into(),
                message: "permission required".into()
            }]
        );
    }

    #[test]
    fn handles_chunked_sequence() {
        let mut p = OscParser::new();
        assert!(p.feed(b"\x1b]9;hel").is_empty());
        let events = p.feed(b"lo world\x1b\\tail");
        assert_eq!(
            events,
            vec![OscEvent::Desktop {
                message: "hello world".into()
            }]
        );
    }

    #[test]
    fn parses_cwd_percent_encoded() {
        let mut p = OscParser::new();
        let events = p.feed(b"\x1b]7;C%3A%5Cprojects%5Cmyproject\x1b\\");
        assert_eq!(
            events,
            vec![OscEvent::Cwd {
                cwd: "C:\\projects\\myproject".into()
            }]
        );
    }

    #[test]
    fn ignores_regular_text() {
        let mut p = OscParser::new();
        let events = p.feed(b"echo hello\n\x1b[32mgreen\x1b[0m");
        assert!(events.is_empty());
    }

    #[test]
    fn parses_progress() {
        let mut p = OscParser::new();
        let events = p.feed(b"\x1b]99;building 42%\x1b\\");
        assert_eq!(
            events,
            vec![OscEvent::Progress {
                message: "building 42%".into()
            }]
        );
    }
}