//! Shared embedded Output/Terminal/Cortex/Activity console model.
#![forbid(unsafe_code)]
use forge_gui_core::GuiResult;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConsoleLevel {
    Trace,
    Info,
    Success,
    Warning,
    Error,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsoleKind {
    Output,
    Terminal,
    Cortex,
    Activity,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsoleEntry {
    pub sequence: u64,
    pub level: ConsoleLevel,
    pub source: String,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsoleChannel {
    pub id: String,
    pub title: String,
    pub kind: ConsoleKind,
    pub max_entries: usize,
    entries: VecDeque<ConsoleEntry>,
    next_sequence: u64,
    pub unread: u64,
}
impl ConsoleChannel {
    pub fn new(id: impl Into<String>, title: impl Into<String>, kind: ConsoleKind) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            kind,
            max_entries: 5000,
            entries: VecDeque::new(),
            next_sequence: 1,
            unread: 0,
        }
    }
    pub fn push(
        &mut self,
        level: ConsoleLevel,
        source: impl Into<String>,
        text: impl Into<String>,
    ) {
        if self.entries.len() >= self.max_entries.max(1) {
            self.entries.pop_front();
        }
        self.entries.push_back(ConsoleEntry {
            sequence: self.next_sequence,
            level,
            source: source.into(),
            text: text.into(),
        });
        self.next_sequence += 1;
        self.unread = self.unread.saturating_add(1);
    }
    pub fn entries(&self) -> impl Iterator<Item = &ConsoleEntry> {
        self.entries.iter()
    }
    pub fn matching<'a>(&'a self, filter: &'a str) -> impl Iterator<Item = &'a ConsoleEntry> {
        self.entries.iter().filter(move |e| {
            filter.is_empty()
                || e.text.to_lowercase().contains(&filter.to_lowercase())
                || e.source.to_lowercase().contains(&filter.to_lowercase())
        })
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.unread = 0;
    }
    pub fn mark_read(&mut self) {
        self.unread = 0;
    }
}
#[derive(Default)]
pub struct ConsoleHub {
    channels: BTreeMap<String, ConsoleChannel>,
}
impl ConsoleHub {
    pub fn ensure_channel(&mut self, id: &str, title: &str) -> &mut ConsoleChannel {
        self.ensure_typed_channel(id, title, ConsoleKind::Output)
    }
    pub fn ensure_typed_channel(
        &mut self,
        id: &str,
        title: &str,
        kind: ConsoleKind,
    ) -> &mut ConsoleChannel {
        self.channels
            .entry(id.into())
            .or_insert_with(|| ConsoleChannel::new(id, title, kind))
    }
    pub fn push(&mut self, id: &str, level: ConsoleLevel, source: &str, text: impl Into<String>) {
        self.ensure_channel(id, id).push(level, source, text);
    }
    pub fn channel(&self, id: &str) -> Option<&ConsoleChannel> {
        self.channels.get(id)
    }
    pub fn channel_mut(&mut self, id: &str) -> Option<&mut ConsoleChannel> {
        self.channels.get_mut(id)
    }
    pub fn channels(&self) -> impl Iterator<Item = &ConsoleChannel> {
        self.channels.values()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsoleViewState {
    pub active_channel: String,
    pub filter: String,
    pub follow_tail: bool,
}
impl Default for ConsoleViewState {
    fn default() -> Self {
        Self {
            active_channel: "output".into(),
            filter: String::new(),
            follow_tail: true,
        }
    }
}
pub trait ConsoleBackend {
    fn send(&mut self, channel: &str, text: &str) -> GuiResult<()>;
}
