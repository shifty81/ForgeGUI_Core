//! Renderer-neutral ForgeGUI contracts.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const FORGE_GUI_API_VERSION: &str = "forge.gui.v0.4";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForgeGuiError {
    InvalidId(String),
    InvalidDefinition(String),
    DuplicatePanel(String),
    DuplicateCommand(String),
    DuplicatePreset(String),
    MissingPanel(String),
    InvalidLayout(String),
    Operation(String),
}
impl fmt::Display for ForgeGuiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, detail) = match self {
            Self::InvalidId(v) => ("invalid id", v),
            Self::InvalidDefinition(v) => ("invalid definition", v),
            Self::DuplicatePanel(v) => ("duplicate panel", v),
            Self::DuplicateCommand(v) => ("duplicate command", v),
            Self::DuplicatePreset(v) => ("duplicate preset", v),
            Self::MissingPanel(v) => ("missing panel", v),
            Self::InvalidLayout(v) => ("invalid layout", v),
            Self::Operation(v) => ("operation failed", v),
        };
        write!(f, "{kind}: {detail}")
    }
}
impl Error for ForgeGuiError {}
pub type GuiResult<T> = Result<T, ForgeGuiError>;

fn validate_namespaced_id(value: &str) -> GuiResult<()> {
    let value = value.trim();
    if value.is_empty() || !value.contains('.') {
        return Err(ForgeGuiError::InvalidId(value.to_owned()));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return Err(ForgeGuiError::InvalidId(value.to_owned()));
    }
    Ok(())
}

macro_rules! stable_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> GuiResult<Self> {
                let value = value.into();
                validate_namespaced_id(&value)?;
                Ok(Self(value))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value).expect("ForgeGUI IDs must be namespaced")
            }
        }
        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value).expect("ForgeGUI IDs must be namespaced")
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}
stable_id!(PanelId);
stable_id!(CommandId);
stable_id!(WorkspaceId);
stable_id!(ContributionId);
stable_id!(DocumentTypeId);
stable_id!(CapabilityId);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PanelInstanceId {
    pub panel: PanelId,
    pub instance: String,
}
impl PanelInstanceId {
    pub fn singleton(panel: impl Into<PanelId>) -> Self {
        Self {
            panel: panel.into(),
            instance: "singleton".into(),
        }
    }
    pub fn keyed(panel: impl Into<PanelId>, key: impl Into<String>) -> Self {
        Self {
            panel: panel.into(),
            instance: key.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PanelRole {
    Navigation,
    Explorer,
    Hierarchy,
    Inspector,
    Document,
    Canvas,
    Canvas2D,
    Viewport3D,
    Graph,
    Timeline,
    Data,
    Console,
    Diagnostics,
    Operations,
    Monitor,
    Utility,
    Settings,
    Transient,
    Notification,
    Assets,
    Layers,
    History,
    Tool,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PreferredDock {
    Left,
    Right,
    Center,
    Bottom,
    Floating,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PanelScope {
    Global,
    Project,
    Document,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostMode {
    Docked,
    Window,
    DockedOrWindow,
    Structural,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstancePolicy {
    Singleton,
    Multiple { max_instances: Option<u32> },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PanelDefinition {
    pub id: PanelId,
    pub title: String,
    pub category: String,
    pub summary: String,
    pub icon: Option<String>,
    pub scope: PanelScope,
    pub role: PanelRole,
    pub host_mode: HostMode,
    pub instance_policy: InstancePolicy,
    pub preferred_dock: PreferredDock,
    pub minimum_size: [f32; 2],
    pub preferred_size: [f32; 2],
    pub required_capabilities: Vec<CapabilityId>,
}
impl PanelDefinition {
    pub fn new(
        id: impl Into<PanelId>,
        title: impl Into<String>,
        role: PanelRole,
        dock: PreferredDock,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category: "General".into(),
            summary: String::new(),
            icon: None,
            scope: PanelScope::Project,
            role,
            host_mode: HostMode::DockedOrWindow,
            instance_policy: InstancePolicy::Singleton,
            preferred_dock: dock,
            minimum_size: [120.0, 90.0],
            preferred_size: [420.0, 300.0],
            required_capabilities: Vec::new(),
        }
    }
    pub fn validate(&self) -> GuiResult<()> {
        validate_namespaced_id(self.id.as_str())?;
        if self.title.trim().is_empty() {
            return Err(ForgeGuiError::InvalidDefinition(format!(
                "panel {} has empty title",
                self.id
            )));
        }
        if self
            .minimum_size
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err(ForgeGuiError::InvalidDefinition(format!(
                "panel {} has invalid minimum size",
                self.id
            )));
        }
        Ok(())
    }
    pub fn is_singleton(&self) -> bool {
        matches!(&self.instance_policy, InstancePolicy::Singleton)
    }
}

#[derive(Default)]
pub struct PanelCatalog {
    entries: BTreeMap<PanelId, PanelDefinition>,
}
impl PanelCatalog {
    pub fn register(&mut self, definition: PanelDefinition) -> GuiResult<()> {
        definition.validate()?;
        if self.entries.contains_key(&definition.id) {
            return Err(ForgeGuiError::DuplicatePanel(definition.id.to_string()));
        }
        self.entries.insert(definition.id.clone(), definition);
        Ok(())
    }
    pub fn get(&self, id: &PanelId) -> Option<&PanelDefinition> {
        self.entries.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &PanelDefinition> {
        self.entries.values()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ThemeTokens {
    pub background: Rgba,
    pub panel: Rgba,
    pub panel_raised: Rgba,
    pub panel_recessed: Rgba,
    pub border: Rgba,
    pub border_focus: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub success: Rgba,
    pub warning: Rgba,
    pub danger: Rgba,
    pub status_height: f32,
    pub toolbar_height: f32,
    pub corner_radius: f32,
    pub scrollbar_width: f32,
}
impl Default for ThemeTokens {
    fn default() -> Self {
        Self {
            background: Rgba(16, 18, 22, 255),
            panel: Rgba(24, 27, 32, 255),
            panel_raised: Rgba(34, 38, 45, 255),
            panel_recessed: Rgba(12, 15, 19, 255),
            border: Rgba(53, 60, 70, 255),
            border_focus: Rgba(83, 146, 255, 255),
            text: Rgba(232, 236, 242, 255),
            text_muted: Rgba(145, 153, 166, 255),
            accent: Rgba(83, 146, 255, 255),
            success: Rgba(67, 180, 108, 255),
            warning: Rgba(230, 167, 61, 255),
            danger: Rgba(220, 78, 78, 255),
            status_height: 24.0,
            toolbar_height: 38.0,
            corner_radius: 5.0,
            scrollbar_width: 10.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandDefinition {
    pub id: CommandId,
    pub title: String,
    pub category: String,
    pub default_shortcut: Option<String>,
}
#[derive(Default)]
pub struct CommandCatalog {
    entries: BTreeMap<CommandId, CommandDefinition>,
}
impl CommandCatalog {
    pub fn register(&mut self, command: CommandDefinition) -> GuiResult<()> {
        if self.entries.contains_key(&command.id) {
            return Err(ForgeGuiError::DuplicateCommand(command.id.to_string()));
        }
        self.entries.insert(command.id.clone(), command);
        Ok(())
    }
    pub fn get(&self, id: &CommandId) -> Option<&CommandDefinition> {
        self.entries.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &CommandDefinition> {
        self.entries.values()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LayoutNode {
    Stack {
        panels: Vec<PanelId>,
        active: usize,
    },
    Split {
        axis: SplitAxis,
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}
impl LayoutNode {
    pub fn stack(panels: impl IntoIterator<Item = PanelId>) -> Self {
        Self::Stack {
            panels: panels.into_iter().collect(),
            active: 0,
        }
    }
    pub fn split(axis: SplitAxis, ratio: f32, first: LayoutNode, second: LayoutNode) -> Self {
        Self::Split {
            axis,
            ratio,
            first: Box::new(first),
            second: Box::new(second),
        }
    }
    fn validate(&self, panels: &PanelCatalog) -> GuiResult<()> {
        match self {
            Self::Stack {
                panels: ids,
                active,
            } => {
                if ids.is_empty() {
                    return Err(ForgeGuiError::InvalidLayout("empty panel stack".into()));
                }
                if *active >= ids.len() {
                    return Err(ForgeGuiError::InvalidLayout(
                        "active tab index out of range".into(),
                    ));
                }
                for id in ids {
                    if panels.get(id).is_none() {
                        return Err(ForgeGuiError::MissingPanel(id.to_string()));
                    }
                }
            }
            Self::Split {
                ratio,
                first,
                second,
                ..
            } => {
                if !ratio.is_finite() || !(0.05..=0.95).contains(ratio) {
                    return Err(ForgeGuiError::InvalidLayout(format!(
                        "invalid split ratio {ratio}"
                    )));
                }
                first.validate(panels)?;
                second.validate(panels)?;
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InterfacePreset {
    pub id: WorkspaceId,
    pub label: String,
    pub locked_by_default: bool,
    pub root: LayoutNode,
}
impl InterfacePreset {
    pub fn validate(&self, panels: &PanelCatalog) -> GuiResult<()> {
        self.root.validate(panels)
    }
}
#[derive(Default)]
pub struct InterfacePresetCatalog {
    presets: BTreeMap<WorkspaceId, InterfacePreset>,
}
impl InterfacePresetCatalog {
    pub fn register(&mut self, preset: InterfacePreset, panels: &PanelCatalog) -> GuiResult<()> {
        preset.validate(panels)?;
        if self.presets.contains_key(&preset.id) {
            return Err(ForgeGuiError::DuplicatePreset(preset.id.to_string()));
        }
        self.presets.insert(preset.id.clone(), preset);
        Ok(())
    }
    pub fn get(&self, id: &WorkspaceId) -> Option<&InterfacePreset> {
        self.presets.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &InterfacePreset> {
        self.presets.values()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DockAnchor {
    Left,
    Right,
    Center,
    Bottom,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DockRegionKind {
    Structural,
    Workspace,
    Context,
    Activity,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DockLockMode {
    Flexible,
    SoftLocked { unlock_chord: String },
    Permanent,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PanelDockPolicy {
    pub panel: PanelId,
    pub anchor: DockAnchor,
    pub region: DockRegionKind,
    pub lock_mode: DockLockMode,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectionState {
    pub object_ids: Vec<String>,
    pub primary: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PropertyValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Reference(String),
    None,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PropertyField {
    pub id: String,
    pub label: String,
    pub value: PropertyValue,
    pub read_only: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PropertyObject {
    pub object_id: String,
    pub title: String,
    pub fields: Vec<PropertyField>,
}
pub trait InspectorProvider {
    fn inspect(&self, selection: &SelectionState) -> GuiResult<Option<PropertyObject>>;
    fn set_property(
        &self,
        object_id: &str,
        property_id: &str,
        value: PropertyValue,
    ) -> GuiResult<()>;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilitySet {
    values: BTreeSet<CapabilityId>,
}
impl CapabilitySet {
    pub fn insert(&mut self, id: impl Into<CapabilityId>) {
        self.values.insert(id.into());
    }
    pub fn contains(&self, id: &CapabilityId) -> bool {
        self.values.contains(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &CapabilityId> {
        self.values.iter()
    }
    pub fn from_ids(ids: impl IntoIterator<Item = CapabilityId>) -> Self {
        Self {
            values: ids.into_iter().collect(),
        }
    }
}
pub mod standard_capabilities {
    pub const DOCUMENTS: &str = "forge.gui.documents";
    pub const FILE_BROWSER: &str = "forge.gui.file-browser";
    pub const ASSETS: &str = "forge.gui.assets";
    pub const INSPECTOR: &str = "forge.gui.inspector";
    pub const INFINITE_CANVAS: &str = "forge.gui.infinite-canvas";
    pub const PIE: &str = "forge.gui.pie";
    pub const GRAPH: &str = "forge.gui.graph";
    pub const TIMELINE: &str = "forge.gui.timeline";
    pub const CURVE_EDITOR: &str = "forge.gui.curve-editor";
    pub const PROJECT_OPERATIONS: &str = "forge.gui.project-operations";
    pub const CONSOLE: &str = "forge.gui.console";
    pub const NOTIFICATIONS: &str = "forge.gui.notifications";
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentTypeDefinition {
    pub id: DocumentTypeId,
    pub title: String,
    pub extensions: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GuiContributionManifest {
    pub api_version: String,
    pub id: ContributionId,
    pub display_name: String,
    pub panels: Vec<PanelDefinition>,
    pub commands: Vec<CommandDefinition>,
    pub documents: Vec<DocumentTypeDefinition>,
    pub presets: Vec<InterfacePreset>,
}
impl GuiContributionManifest {
    pub fn validate(&self) -> GuiResult<()> {
        if self.api_version != FORGE_GUI_API_VERSION {
            return Err(ForgeGuiError::InvalidDefinition(format!(
                "unsupported api {}",
                self.api_version
            )));
        }
        let mut panels = PanelCatalog::default();
        for panel in &self.panels {
            panels.register(panel.clone())?;
        }
        let mut commands = CommandCatalog::default();
        for command in &self.commands {
            commands.register(command.clone())?;
        }
        for preset in &self.presets {
            preset.validate(&panels)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    pub title: String,
    pub state: TaskState,
    pub progress: Option<f32>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn panel(id: &str) -> PanelDefinition {
        PanelDefinition::new(id, "Panel", PanelRole::Tool, PreferredDock::Center)
    }
    #[test]
    fn ids_are_namespaced() {
        assert!(PanelId::new("demo.panel").is_ok());
        assert!(PanelId::new("panel").is_err());
    }
    #[test]
    fn duplicate_panel_is_fail_closed() {
        let mut c = PanelCatalog::default();
        c.register(panel("demo.one")).unwrap();
        assert!(c.register(panel("demo.one")).is_err());
        assert_eq!(c.len(), 1);
    }
    #[test]
    fn duplicate_command_is_fail_closed() {
        let mut c = CommandCatalog::default();
        let cmd = CommandDefinition {
            id: CommandId::from("demo.command"),
            title: "A".into(),
            category: "Test".into(),
            default_shortcut: None,
        };
        c.register(cmd.clone()).unwrap();
        assert!(c.register(cmd).is_err());
        assert_eq!(c.iter().count(), 1);
    }
    #[test]
    fn preset_validates() {
        let mut c = PanelCatalog::default();
        c.register(panel("demo.one")).unwrap();
        c.register(panel("demo.two")).unwrap();
        let p = InterfacePreset {
            id: WorkspaceId::from("demo.layout"),
            label: "Demo".into(),
            locked_by_default: false,
            root: LayoutNode::split(
                SplitAxis::Horizontal,
                0.7,
                LayoutNode::stack([PanelId::from("demo.one")]),
                LayoutNode::stack([PanelId::from("demo.two")]),
            ),
        };
        assert!(p.validate(&c).is_ok());
    }
}
