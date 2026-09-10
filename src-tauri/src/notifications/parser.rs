use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppNotification {
    pub title: String,
    pub message: String,
    pub kind: NotificationKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NotificationKind {
    Desktop,
    Progress,
    Rich,
}

impl AppNotification {
    pub fn desktop(message: impl Into<String>) -> Self {
        Self {
            title: "wmux".into(),
            message: message.into(),
            kind: NotificationKind::Desktop,
        }
    }

    pub fn progress(message: impl Into<String>) -> Self {
        Self {
            title: "wmux".into(),
            message: message.into(),
            kind: NotificationKind::Progress,
        }
    }

    pub fn rich(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            kind: NotificationKind::Rich,
        }
    }
}