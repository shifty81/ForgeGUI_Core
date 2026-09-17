//! Platform service abstractions.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileDialogKind {
    OpenFile,
    OpenFiles,
    OpenFolder,
    SaveFile,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileDialogRequest {
    pub kind: FileDialogKind,
    pub title: String,
    pub filters: Vec<(String, Vec<String>)>,
    pub initial_path: Option<String>,
}
pub trait FileDialogService {
    fn request(&mut self, request: &FileDialogRequest) -> Result<Vec<String>, String>;
}
pub trait ClipboardBridge {
    fn read_text(&mut self) -> Result<Option<String>, String>;
    fn write_text(&mut self, text: &str) -> Result<(), String>;
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentInfo {
    pub os: String,
    pub locale: String,
    pub scale_factor: f32,
    pub high_contrast: bool,
    pub reduced_motion: bool,
}
