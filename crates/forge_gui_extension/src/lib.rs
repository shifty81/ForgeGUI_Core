//! Versioned extension contribution contracts.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const EXTENSION_API_VERSION: &str = "forge.gui.extension.v1";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContributionKind {
    Command,
    Menu,
    View,
    Navigation,
    Settings,
    Dialog,
    Status,
    Widget,
    Service,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contribution {
    pub id: String,
    pub kind: ContributionKind,
    pub target: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub api_version: String,
    pub required_capabilities: BTreeSet<String>,
    pub contributions: Vec<Contribution>,
}
impl ExtensionManifest {
    pub fn compatible(&self, available: &BTreeSet<String>) -> bool {
        self.api_version == EXTENSION_API_VERSION && self.required_capabilities.is_subset(available)
    }
}
