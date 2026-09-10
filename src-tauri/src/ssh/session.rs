use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

use serde::Serialize;

use super::auth::{SshAuth, SshTarget};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshTestResult {
    pub ok: bool,
    pub banner: Option<String>,
    pub error: Option<String>,
}

pub fn ssh_command(target: &SshTarget) -> Vec<String> {
    let mut args = Vec::new();
    if target.port != 22 {
        args.push("-p".to_string());
        args.push(target.port.to_string());
    }
    args.push(format!("{}@{}", target.user, target.host));
    args
}

pub fn ssh_test(target: &SshTarget, auth: &SshAuth) -> SshTestResult {
    let tcp = match TcpStream::connect((target.host.as_str(), target.port)) {
        Ok(s) => s,
        Err(e) => return SshTestResult { ok: false, banner: None, error: Some(format!("connect failed: {e}")) },
    };
    tcp.set_read_timeout(Some(Duration::from_secs(10))).ok();
    tcp.set_write_timeout(Some(Duration::from_secs(10))).ok();

    let mut session = match ssh2::Session::new() {
        Ok(s) => s,
        Err(e) => return SshTestResult { ok: false, banner: None, error: Some(format!("session failed: {e}")) },
    };
    session.set_tcp_stream(tcp);
    if let Err(e) = session.handshake() {
        return SshTestResult { ok: false, banner: None, error: Some(format!("handshake failed: {e}")) };
    }
    let banner = session.banner().map(|b| b.to_string());

    let auth_result = match auth {
        SshAuth::Password { password } => session.userauth_password(&target.user, password),
        SshAuth::Key { key_path, passphrase } => {
            session.userauth_pubkey_file(&target.user, None, key_path, passphrase.as_deref())
        }
    };
    if let Err(e) = auth_result {
        let _ = session.disconnect(Some(ssh2::DisconnectCode::ByApplication), "auth failed", None);
        return SshTestResult {
            ok: false,
            banner,
            error: Some(format!("authentication failed: {e}")),
        };
    }

    let error = if let Some(mut channel) = session.channel_session().ok() {
        let mut out = Vec::new();
        let _ = channel.read_to_end(&mut out);
        None
    } else {
        Some("channel session failed".to_string())
    };
    let _ = session.disconnect(Some(ssh2::DisconnectCode::ByApplication), "bye", None);
    SshTestResult {
        ok: error.is_none(),
        banner,
        error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ssh_args() {
        let target = SshTarget::new("example.com", "root", 22);
        assert_eq!(ssh_command(&target), vec!["root@example.com"]);
        let target = SshTarget::new("example.com", "root", 2222);
        assert_eq!(ssh_command(&target), vec!["-p", "2222", "root@example.com"]);
    }
}