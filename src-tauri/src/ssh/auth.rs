use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SshAuth {
    Password { password: String },
    Key { key_path: PathBuf, passphrase: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshTarget {
    pub host: String,
    pub port: u16,
    pub user: String,
}

impl SshTarget {
    #[allow(dead_code)]
    pub fn new(host: impl Into<String>, user: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            user: user.into(),
        }
    }
}