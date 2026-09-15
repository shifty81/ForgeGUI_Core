//! Dialog and wizard state models.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DialogKind {
    Message,
    Confirm,
    Input,
    Progress,
    Custom,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DialogAction {
    pub id: String,
    pub label: String,
    pub primary: bool,
    pub destructive: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DialogSpec {
    pub id: String,
    pub title: String,
    pub message: String,
    pub kind: DialogKind,
    pub modal: bool,
    pub actions: Vec<DialogAction>,
}
#[derive(Default)]
pub struct DialogManager {
    queue: VecDeque<DialogSpec>,
    active: Option<DialogSpec>,
}
impl DialogManager {
    pub fn push(&mut self, d: DialogSpec) {
        if self.active.is_none() {
            self.active = Some(d)
        } else {
            self.queue.push_back(d)
        }
    }
    pub fn active(&self) -> Option<&DialogSpec> {
        self.active.as_ref()
    }
    pub fn resolve(&mut self) -> Option<DialogSpec> {
        let done = self.active.take();
        self.active = self.queue.pop_front();
        done
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WizardStep {
    pub id: String,
    pub title: String,
    pub complete: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WizardState {
    pub steps: Vec<WizardStep>,
    pub current: usize,
}
impl WizardState {
    pub fn can_next(&self) -> bool {
        self.steps.get(self.current).is_some_and(|s| s.complete)
            && self.current + 1 < self.steps.len()
    }
    pub fn advance(&mut self) -> bool {
        if self.can_next() {
            self.current += 1;
            true
        } else {
            false
        }
    }
}
