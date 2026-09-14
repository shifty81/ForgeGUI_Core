//! Renderer-neutral semantic icon identities for ForgeGUI.
//!
//! Projects depend on `IconId`, never on Unicode glyphs, font codepoints,
//! SVG paths, or a particular third-party icon library.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IconId {
    Add,
    Remove,
    Close,
    Save,
    Undo,
    Redo,
    Search,
    Settings,
    Folder,
    FolderOpen,
    File,
    Project,
    Asset,
    Inspector,
    Layers,
    History,
    Console,
    Terminal,
    Cortex,
    Problems,
    Notifications,
    Play,
    Pause,
    Stop,
    Step,
    Restart,
    Build,
    Test,
    Package,
    Patch,
    GitBranch,
    Success,
    Warning,
    Error,
    Info,
    World,
    Scene,
    Entity,
    Graph,
    Timeline,
    Ship,
    Module,
    Socket,
    Inventory,
    Crafting,
}

impl IconId {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Remove => "remove",
            Self::Close => "close",
            Self::Save => "save",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::Search => "search",
            Self::Settings => "settings",
            Self::Folder => "folder",
            Self::FolderOpen => "folder-open",
            Self::File => "file",
            Self::Project => "project",
            Self::Asset => "asset",
            Self::Inspector => "inspector",
            Self::Layers => "layers",
            Self::History => "history",
            Self::Console => "console",
            Self::Terminal => "terminal",
            Self::Cortex => "cortex",
            Self::Problems => "problems",
            Self::Notifications => "notifications",
            Self::Play => "play",
            Self::Pause => "pause",
            Self::Stop => "stop",
            Self::Step => "step",
            Self::Restart => "restart",
            Self::Build => "build",
            Self::Test => "test",
            Self::Package => "package",
            Self::Patch => "patch",
            Self::GitBranch => "git-branch",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Info => "info",
            Self::World => "world",
            Self::Scene => "scene",
            Self::Entity => "entity",
            Self::Graph => "graph",
            Self::Timeline => "timeline",
            Self::Ship => "ship",
            Self::Module => "module",
            Self::Socket => "socket",
            Self::Inventory => "inventory",
            Self::Crafting => "crafting",
        }
    }

    /// Text fallback for non-icon renderers, logs, accessibility descriptions,
    /// and hosts that intentionally do not install an icon font.
    pub const fn fallback(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Remove => "-",
            Self::Close => "x",
            Self::Save => "save",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::Search => "search",
            Self::Settings => "settings",
            Self::Folder | Self::FolderOpen | Self::Project => "folder",
            Self::File => "file",
            Self::Play => "play",
            Self::Pause => "pause",
            Self::Stop => "stop",
            Self::Step => "step",
            Self::Restart => "restart",
            Self::Build => "build",
            Self::Test => "test",
            Self::Package => "package",
            Self::Patch => "patch",
            Self::GitBranch => "branch",
            Self::Success => "ok",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Info => "info",
            Self::Console | Self::Terminal => "terminal",
            Self::Cortex => "cortex",
            Self::Problems => "problems",
            Self::Notifications => "notifications",
            Self::Asset => "asset",
            Self::Inspector => "inspector",
            Self::Layers => "layers",
            Self::History => "history",
            Self::World => "world",
            Self::Scene => "scene",
            Self::Entity => "entity",
            Self::Graph => "graph",
            Self::Timeline => "timeline",
            Self::Ship => "ship",
            Self::Module => "module",
            Self::Socket => "socket",
            Self::Inventory => "inventory",
            Self::Crafting => "crafting",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_ids_have_stable_nonempty_keys() {
        for id in [
            IconId::Project,
            IconId::Inspector,
            IconId::Cortex,
            IconId::Play,
            IconId::Build,
            IconId::Ship,
            IconId::Crafting,
        ] {
            assert!(!id.key().is_empty());
            assert!(!id.fallback().is_empty());
        }
    }
}
