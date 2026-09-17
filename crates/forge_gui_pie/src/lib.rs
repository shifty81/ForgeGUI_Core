//! Play-in-editor runtime session contracts.
#![forbid(unsafe_code)]
use forge_gui_core::GuiResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PieState {
    Stopped,
    Playing,
    Paused,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlayMode {
    Normal,
    FromHere,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputOwner {
    Editor,
    Runtime,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayRequest {
    pub mode: PlayMode,
    pub document_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RuntimeStatus {
    pub state: PieState,
    pub frame: u64,
    pub elapsed_seconds: f64,
    pub input_owner: InputOwner,
}
impl Default for RuntimeStatus {
    fn default() -> Self {
        Self {
            state: PieState::Stopped,
            frame: 0,
            elapsed_seconds: 0.0,
            input_owner: InputOwner::Editor,
        }
    }
}
pub trait RuntimeProvider: Send {
    fn start(&mut self, request: &PlayRequest) -> GuiResult<()>;
    fn stop(&mut self) -> GuiResult<()>;
    fn pause(&mut self, paused: bool) -> GuiResult<()>;
    fn step(&mut self) -> GuiResult<()>;
}
#[derive(Default)]
pub struct PieController {
    pub status: RuntimeStatus,
    pub last_request: Option<PlayRequest>,
}
impl PieController {
    pub fn play(&mut self, request: PlayRequest) {
        self.last_request = Some(request);
        self.status.state = PieState::Playing;
        self.status.input_owner = InputOwner::Runtime;
    }
    pub fn stop(&mut self) {
        self.status = RuntimeStatus::default();
    }
    pub fn pause(&mut self) {
        if self.status.state == PieState::Playing {
            self.status.state = PieState::Paused;
        }
    }
    pub fn resume(&mut self) {
        if self.status.state == PieState::Paused {
            self.status.state = PieState::Playing;
        }
    }
    pub fn step(&mut self) {
        if self.status.state == PieState::Paused {
            self.status.frame = self.status.frame.saturating_add(1);
        }
    }
    pub fn tick(&mut self, dt: f64) {
        if self.status.state == PieState::Playing {
            self.status.frame = self.status.frame.saturating_add(1);
            self.status.elapsed_seconds += dt.max(0.0);
        }
    }
    pub fn is_running(&self) -> bool {
        matches!(self.status.state, PieState::Playing | PieState::Paused)
    }
}
