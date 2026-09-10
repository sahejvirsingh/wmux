use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub rings: bool,
    pub toasts: bool,
    pub badges: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            rings: true,
            toasts: true,
            badges: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme_id: String,
    pub font_size: f64,
    pub font_family: String,
    pub shell: Option<String>,
    pub auto_restore: bool,
    pub agent_auto_resume: bool,
    pub notifications: NotificationSettings,
    pub keybindings: HashMap<String, Vec<String>>,
    pub sidebar_visible: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_id: "midnight".into(),
            font_size: 13.0,
            font_family: "Cascadia Code, JetBrains Mono, Consolas, monospace".into(),
            shell: None,
            auto_restore: true,
            agent_auto_resume: true,
            notifications: NotificationSettings::default(),
            keybindings: HashMap::new(),
            sidebar_visible: true,
        }
    }
}

pub fn wmux_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".wmux")
}

pub fn ensure_wmux_dir() -> Result<PathBuf, String> {
    let dir = wmux_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {} failed: {e}", dir.display()))?;
    Ok(dir)
}

pub fn settings_path() -> PathBuf {
    wmux_dir().join("wmux.json")
}

pub fn session_path() -> PathBuf {
    wmux_dir().join("session.json")
}

pub fn db_path() -> PathBuf {
    wmux_dir().join("wmux.db")
}

pub fn themes_dir() -> PathBuf {
    wmux_dir().join("themes")
}

pub fn load_settings() -> Settings {
    let path = settings_path();
    let mut settings = Settings::default();
    if let Ok(json) = std::fs::read_to_string(&path) {
        if let Ok(loaded) = serde_json::from_str::<Settings>(&json) {
            settings = loaded;
        }
    }
    settings
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = settings_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir failed: {e}"))?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| format!("serialize failed: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write {} failed: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let s = Settings::default();
        assert_eq!(s.theme_id, "midnight");
        assert!(s.auto_restore);
        assert!(s.agent_auto_resume);
        assert!(s.notifications.rings);
    }

    #[test]
    fn serde_roundtrip() {
        let mut s = Settings::default();
        s.font_size = 16.0;
        s.theme_id = "dracula".into();
        s.keybindings.insert("splitRight".into(), vec!["ctrl+b".into(), "%".into()]);
        let json = serde_json::to_string(&s).unwrap();
        let loaded: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.theme_id, "dracula");
        assert_eq!(loaded.font_size, 16.0);
        assert_eq!(loaded.keybindings["splitRight"], vec!["ctrl+b", "%"]);
    }
}