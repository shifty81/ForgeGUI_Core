//! Structured log/console model suitable for virtualized viewers.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Success,
    Warning,
    Error,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogRecord {
    pub sequence: u64,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
    pub timestamp: String,
    pub fields: Vec<(String, String)>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogFilter {
    pub text: String,
    pub levels: Vec<LogLevel>,
    pub source: Option<String>,
}
pub struct LogBuffer {
    records: VecDeque<LogRecord>,
    capacity: usize,
    next_sequence: u64,
}
impl LogBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            records: VecDeque::new(),
            capacity: capacity.max(1),
            next_sequence: 1,
        }
    }
    pub fn push(&mut self, mut record: LogRecord) {
        record.sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.records.push_back(record);
        while self.records.len() > self.capacity {
            self.records.pop_front();
        }
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    pub fn filtered<'a>(&'a self, f: &'a LogFilter) -> impl Iterator<Item = &'a LogRecord> {
        let q = f.text.to_lowercase();
        self.records.iter().filter(move |r| {
            (f.levels.is_empty() || f.levels.contains(&r.level))
                && (f.source.as_deref().is_none_or(|s| s == r.source))
                && (q.is_empty()
                    || r.message.to_lowercase().contains(&q)
                    || r.source.to_lowercase().contains(&q))
        })
    }
}
