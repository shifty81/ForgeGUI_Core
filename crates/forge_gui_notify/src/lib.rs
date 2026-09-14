//! Durable notification center and toast-domain model.
#![forbid(unsafe_code)]
use forge_gui_core::GuiResult;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationSeverity {
    Info,
    Success,
    Warning,
    Error,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub command_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NotificationProgress {
    pub current: f32,
    pub total: f32,
    pub label: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NotificationRecord {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub read: bool,
    pub archived: bool,
    pub pinned: bool,
    pub progress: Option<NotificationProgress>,
    pub actions: Vec<NotificationAction>,
}
impl NotificationRecord {
    pub fn simple(
        id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        severity: NotificationSeverity,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            message: message.into(),
            severity,
            read: false,
            archived: false,
            pinned: false,
            progress: None,
            actions: Vec::new(),
        }
    }
}
#[derive(Default)]
pub struct NotificationCenter {
    records: Vec<NotificationRecord>,
}
impl NotificationCenter {
    pub fn push(&mut self, r: NotificationRecord) {
        self.records.push(r);
    }
    pub fn records(&self) -> impl Iterator<Item = &NotificationRecord> {
        self.records.iter().filter(|r| !r.archived)
    }
    pub fn records_mut(&mut self) -> impl Iterator<Item = &mut NotificationRecord> {
        self.records.iter_mut().filter(|r| !r.archived)
    }
    pub fn unread_count(&self) -> usize {
        self.records
            .iter()
            .filter(|r| !r.archived && !r.read)
            .count()
    }
    pub fn mark_all_read(&mut self) {
        for r in &mut self.records {
            r.read = true;
        }
    }
    pub fn archive(&mut self, id: &str) {
        if let Some(r) = self.records.iter_mut().find(|r| r.id == id) {
            r.archived = true;
        }
    }
}
pub trait DesktopNotificationBridge {
    fn publish(&mut self, record: &NotificationRecord) -> GuiResult<()>;
}
