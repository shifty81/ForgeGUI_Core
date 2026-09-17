//! Generic UI services shared by all ForgeGUI hosts.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandState {
    pub enabled: bool,
    pub visible: bool,
    pub checked: bool,
}
impl CommandState {
    pub const fn enabled() -> Self {
        Self {
            enabled: true,
            visible: true,
            checked: false,
        }
    }
}
impl Default for CommandState {
    fn default() -> Self {
        Self::enabled()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandSpec {
    pub id: String,
    pub label: String,
    pub category: String,
    pub shortcut: Option<String>,
    pub aliases: Vec<String>,
    pub state: CommandState,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvocationSource {
    Menu,
    Shortcut,
    Palette,
    Context,
    Toolbar,
    Automation,
    Other,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandInvocation {
    pub id: String,
    pub source: InvocationSource,
}

#[derive(Default)]
pub struct CommandRegistry {
    commands: BTreeMap<String, CommandSpec>,
}
impl CommandRegistry {
    pub fn register(&mut self, spec: CommandSpec) -> Result<(), String> {
        if self.commands.contains_key(&spec.id) {
            return Err(format!("duplicate command {}", spec.id));
        }
        self.commands.insert(spec.id.clone(), spec);
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&CommandSpec> {
        self.commands.get(id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut CommandSpec> {
        self.commands.get_mut(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &CommandSpec> {
        self.commands.values()
    }
    pub fn visible(&self) -> impl Iterator<Item = &CommandSpec> {
        self.commands.values().filter(|c| c.state.visible)
    }
    pub fn search(&self, q: &str) -> Vec<&CommandSpec> {
        let q = q.trim().to_lowercase();
        let mut out: Vec<_> = self
            .visible()
            .filter(|c| {
                q.is_empty()
                    || c.label.to_lowercase().contains(&q)
                    || c.category.to_lowercase().contains(&q)
                    || c.id.to_lowercase().contains(&q)
                    || c.aliases.iter().any(|a| a.to_lowercase().contains(&q))
            })
            .collect();
        out.sort_by_key(|c| {
            (
                !c.label.to_lowercase().starts_with(&q),
                c.category.clone(),
                c.label.clone(),
            )
        });
        out
    }
    pub fn shortcut_conflicts(&self) -> Vec<(String, Vec<String>)> {
        let mut m: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for c in self.commands.values() {
            if let Some(s) = &c.shortcut {
                m.entry(s.to_lowercase()).or_default().push(c.id.clone());
            }
        }
        m.into_iter().filter(|(_, v)| v.len() > 1).collect()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectionSnapshot {
    pub primary: Option<String>,
    pub items: Vec<String>,
    pub source: Option<String>,
}
#[derive(Default)]
pub struct SelectionService {
    current: SelectionSnapshot,
}
impl SelectionService {
    pub fn set(&mut self, source: impl Into<String>, items: Vec<String>, primary: Option<String>) {
        self.current = SelectionSnapshot {
            source: Some(source.into()),
            items,
            primary,
        };
    }
    pub fn clear(&mut self) {
        self.current = SelectionSnapshot::default();
    }
    pub fn snapshot(&self) -> &SelectionSnapshot {
        &self.current
    }
    pub fn contains(&self, id: &str) -> bool {
        self.current.items.iter().any(|x| x == id)
    }
    pub fn len(&self) -> usize {
        self.current.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.current.items.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FocusEntry {
    pub scope: String,
    pub target: String,
}
#[derive(Default)]
pub struct FocusService {
    current: Option<FocusEntry>,
    history: Vec<FocusEntry>,
}
impl FocusService {
    pub fn focus(&mut self, entry: FocusEntry) {
        if let Some(old) = self.current.replace(entry) {
            self.history.push(old);
            self.history.truncate(64);
        }
    }
    pub fn current(&self) -> Option<&FocusEntry> {
        self.current.as_ref()
    }
    pub fn restore_previous(&mut self) -> Option<&FocusEntry> {
        let prev = self.history.pop()?;
        self.current = Some(prev);
        self.current.as_ref()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub label: String,
    pub merge_key: Option<String>,
    pub payload: String,
}
#[derive(Default)]
pub struct HistoryService {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
}
impl HistoryService {
    pub fn push(&mut self, e: HistoryEntry) {
        if let Some(key) = e.merge_key.as_ref() {
            if self.undo.last().and_then(|x| x.merge_key.as_ref()) == Some(key) {
                self.undo.pop();
            }
        }
        self.undo.push(e);
        self.redo.clear();
    }
    pub fn undo(&mut self) -> Option<HistoryEntry> {
        let e = self.undo.pop()?;
        self.redo.push(e.clone());
        Some(e)
    }
    pub fn redo(&mut self) -> Option<HistoryEntry> {
        let e = self.redo.pop()?;
        self.undo.push(e.clone());
        Some(e)
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardPayload {
    pub mime: String,
    pub data: String,
}
#[derive(Default)]
pub struct ClipboardService {
    payloads: BTreeMap<String, ClipboardPayload>,
}
impl ClipboardService {
    pub fn set(&mut self, p: ClipboardPayload) {
        self.payloads.insert(p.mime.clone(), p);
    }
    pub fn get(&self, mime: &str) -> Option<&ClipboardPayload> {
        self.payloads.get(mime)
    }
    pub fn clear(&mut self) {
        self.payloads.clear();
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DragOperation {
    Copy,
    Move,
    Link,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DragPayload {
    pub kind: String,
    pub ids: Vec<String>,
    pub allowed: BTreeSet<DragOperation>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DragSession {
    pub payload: DragPayload,
    pub target: Option<String>,
    pub accepted: Option<DragOperation>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskRecord {
    pub id: String,
    pub label: String,
    pub state: TaskState,
    pub progress: Option<f32>,
    pub detail: String,
    pub cancellable: bool,
}
#[derive(Default)]
pub struct TaskRegistry {
    tasks: BTreeMap<String, TaskRecord>,
}
impl TaskRegistry {
    pub fn upsert(&mut self, t: TaskRecord) {
        self.tasks.insert(t.id.clone(), t);
    }
    pub fn iter(&self) -> impl Iterator<Item = &TaskRecord> {
        self.tasks.values()
    }
    pub fn active_count(&self) -> usize {
        self.tasks
            .values()
            .filter(|t| matches!(t.state, TaskState::Queued | TaskState::Running))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_defaults_are_usable() {
        assert_eq!(
            CommandState::enabled(),
            CommandState {
                enabled: true,
                visible: true,
                checked: false
            }
        );
    }
    #[test]
    fn shortcut_conflicts_detected() {
        let mut r = CommandRegistry::default();
        for id in ["a", "b"] {
            r.register(CommandSpec {
                id: id.into(),
                label: id.into(),
                category: "x".into(),
                shortcut: Some("Ctrl+S".into()),
                aliases: vec![],
                state: CommandState::enabled(),
            })
            .unwrap();
        }
        assert_eq!(r.shortcut_conflicts().len(), 1);
    }
}
