//! Durable creator-layout state independent of egui docking implementation details.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const FORGE_LAYOUT_SCHEMA: &str = "forge.workspace_layout.v1";

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayoutPreset {
    #[default]
    Creator,
    CreatorResourceLeft,
    WorldAuthoring,
    Animation,
    Scripting,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StructuralLayout {
    pub left_visible: bool,
    pub right_visible: bool,
    pub bottom_visible: bool,
    pub status_visible: bool,
    pub left_width: f32,
    pub right_width: f32,
    pub bottom_height: f32,
}

impl Default for StructuralLayout {
    fn default() -> Self {
        Self {
            left_visible: true,
            right_visible: true,
            bottom_visible: true,
            status_visible: true,
            left_width: 300.0,
            right_width: 320.0,
            bottom_height: 170.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForgeLayoutState {
    pub schema: String,
    pub preset: LayoutPreset,
    pub active_workspace: String,
    pub workspaces: Vec<String>,
    pub structural: StructuralLayout,
}

impl Default for ForgeLayoutState {
    fn default() -> Self {
        Self::from_preset(LayoutPreset::Creator)
    }
}

impl ForgeLayoutState {
    pub fn from_preset(preset: LayoutPreset) -> Self {
        let structural = match preset {
            LayoutPreset::Creator => StructuralLayout::default(),
            LayoutPreset::CreatorResourceLeft => StructuralLayout {
                left_width: 320.0,
                right_width: 300.0,
                ..Default::default()
            },
            LayoutPreset::WorldAuthoring => StructuralLayout {
                left_width: 280.0,
                right_width: 340.0,
                bottom_height: 190.0,
                ..Default::default()
            },
            LayoutPreset::Animation => StructuralLayout {
                left_width: 260.0,
                right_width: 300.0,
                bottom_height: 240.0,
                ..Default::default()
            },
            LayoutPreset::Scripting => StructuralLayout {
                left_width: 250.0,
                right_width: 290.0,
                bottom_height: 190.0,
                ..Default::default()
            },
        };

        Self {
            schema: FORGE_LAYOUT_SCHEMA.into(),
            preset,
            active_workspace: "workspace.main".into(),
            workspaces: vec![
                "workspace.main".into(),
                "workspace.ui".into(),
                "workspace.logic".into(),
                "workspace.animation".into(),
            ],
            structural,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != FORGE_LAYOUT_SCHEMA {
            return Err(format!("unsupported layout schema: {}", self.schema));
        }
        if self.workspaces.is_empty() {
            return Err("layout must contain at least one workspace".into());
        }
        if !self.workspaces.contains(&self.active_workspace) {
            return Err("active workspace is not present in workspace list".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creator_layout_is_valid() {
        ForgeLayoutState::default().validate().unwrap();
    }
}
