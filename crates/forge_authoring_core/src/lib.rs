//! Universal 2D/3D/hybrid authoring session state.
#![forbid(unsafe_code)]

use forge_render_core::{AuthoringCamera, Camera2D, Camera3D, SurfaceDimension};
use forge_scene_core::{EntityId, ForgeScene, Transform};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthoringMode {
    TwoD,
    ThreeD,
    Hybrid,
}

impl From<AuthoringMode> for SurfaceDimension {
    fn from(value: AuthoringMode) -> Self {
        match value {
            AuthoringMode::TwoD => SurfaceDimension::TwoD,
            AuthoringMode::ThreeD => SurfaceDimension::ThreeD,
            AuthoringMode::Hybrid => SurfaceDimension::Hybrid,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GizmoMode {
    Select,
    Translate,
    Rotate,
    Scale,
    Pivot,
    Measure,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CoordinateSpace {
    World,
    Local,
    Parent,
    Screen,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct SnapSettings {
    pub enabled: bool,
    pub translation_2d: [f32; 2],
    pub translation_3d: [f32; 3],
    pub rotation_degrees: f32,
    pub scale_step: f32,
    pub pixel_snap: bool,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            translation_2d: [1.0, 1.0],
            translation_3d: [0.25, 0.25, 0.25],
            rotation_degrees: 15.0,
            scale_step: 0.1,
            pixel_snap: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct GridSettings {
    pub visible: bool,
    pub major_every: u32,
    pub spacing_2d: [f32; 2],
    pub spacing_3d: f32,
    pub adaptive: bool,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            visible: true,
            major_every: 8,
            spacing_2d: [32.0, 32.0],
            spacing_3d: 1.0,
            adaptive: true,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthoringSelection {
    pub entities: BTreeSet<EntityId>,
    pub primary: Option<EntityId>,
    pub hovered: Option<EntityId>,
}

impl AuthoringSelection {
    pub fn clear(&mut self) {
        self.entities.clear();
        self.primary = None;
        self.hovered = None;
    }

    pub fn select_only(&mut self, id: EntityId) {
        self.entities.clear();
        self.entities.insert(id.clone());
        self.primary = Some(id);
    }

    pub fn toggle(&mut self, id: EntityId) {
        if !self.entities.insert(id.clone()) {
            self.entities.remove(&id);
            if self.primary.as_ref() == Some(&id) {
                self.primary = self.entities.iter().next().cloned();
            }
        } else {
            self.primary = Some(id);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthoringSurfaceState {
    pub mode: AuthoringMode,
    pub camera: AuthoringCamera,
    pub selection: AuthoringSelection,
    pub gizmo: GizmoMode,
    pub coordinate_space: CoordinateSpace,
    pub snap: SnapSettings,
    pub grid: GridSettings,
    pub show_guides: bool,
    pub show_bounds: bool,
    pub show_origins: bool,
    pub show_runtime_overlay: bool,
}

impl Default for AuthoringSurfaceState {
    fn default() -> Self {
        Self {
            mode: AuthoringMode::ThreeD,
            camera: AuthoringCamera::ThreeD(Camera3D::default()),
            selection: AuthoringSelection::default(),
            gizmo: GizmoMode::Select,
            coordinate_space: CoordinateSpace::World,
            snap: SnapSettings::default(),
            grid: GridSettings::default(),
            show_guides: true,
            show_bounds: true,
            show_origins: false,
            show_runtime_overlay: true,
        }
    }
}

impl AuthoringSurfaceState {
    pub fn set_mode(&mut self, mode: AuthoringMode) {
        self.mode = mode;
        self.camera = match mode {
            AuthoringMode::TwoD => AuthoringCamera::TwoD(Camera2D::default()),
            AuthoringMode::ThreeD => AuthoringCamera::ThreeD(Camera3D::default()),
            AuthoringMode::Hybrid => AuthoringCamera::Hybrid {
                camera_2d: Camera2D::default(),
                camera_3d: Camera3D::default(),
                blend: 0.5,
            },
        };
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AuthoringCommand {
    SetTransform {
        entity: EntityId,
        before: Transform,
        after: Transform,
    },
    Rename {
        entity: EntityId,
        before: String,
        after: String,
    },
}

impl AuthoringCommand {
    pub fn apply(&self, scene: &mut ForgeScene) -> Result<(), String> {
        match self {
            Self::SetTransform { entity, after, .. } => {
                let target = scene
                    .get_mut(entity)
                    .ok_or_else(|| format!("missing entity {}", entity.0))?;
                target.transform = *after;
            }
            Self::Rename { entity, after, .. } => {
                let target = scene
                    .get_mut(entity)
                    .ok_or_else(|| format!("missing entity {}", entity.0))?;
                target.name = after.clone();
            }
        }
        Ok(())
    }

    pub fn undo(&self, scene: &mut ForgeScene) -> Result<(), String> {
        match self {
            Self::SetTransform { entity, before, .. } => {
                let target = scene
                    .get_mut(entity)
                    .ok_or_else(|| format!("missing entity {}", entity.0))?;
                target.transform = *before;
            }
            Self::Rename { entity, before, .. } => {
                let target = scene
                    .get_mut(entity)
                    .ok_or_else(|| format!("missing entity {}", entity.0))?;
                target.name = before.clone();
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct CommandStack {
    pub undo: Vec<AuthoringCommand>,
    pub redo: Vec<AuthoringCommand>,
}

impl CommandStack {
    pub fn execute(
        &mut self,
        scene: &mut ForgeScene,
        command: AuthoringCommand,
    ) -> Result<(), String> {
        command.apply(scene)?;
        self.undo.push(command);
        self.redo.clear();
        Ok(())
    }

    pub fn undo(&mut self, scene: &mut ForgeScene) -> Result<bool, String> {
        let Some(command) = self.undo.pop() else {
            return Ok(false);
        };
        command.undo(scene)?;
        self.redo.push(command);
        Ok(true)
    }

    pub fn redo(&mut self, scene: &mut ForgeScene) -> Result<bool, String> {
        let Some(command) = self.redo.pop() else {
            return Ok(false);
        };
        command.apply(scene)?;
        self.undo.push(command);
        Ok(true)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthoringSession {
    pub document_id: String,
    pub scene: ForgeScene,
    pub surface: AuthoringSurfaceState,
    pub commands: CommandStack,
    pub dirty: bool,
}

impl AuthoringSession {
    pub fn new(document_id: impl Into<String>, scene: ForgeScene) -> Self {
        Self {
            document_id: document_id.into(),
            scene,
            surface: AuthoringSurfaceState::default(),
            commands: CommandStack::default(),
            dirty: false,
        }
    }

    pub fn execute(&mut self, command: AuthoringCommand) -> Result<(), String> {
        self.commands.execute(&mut self.scene, command)?;
        self.dirty = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_scene_core::{SceneEntity, Transform3D};

    #[test]
    fn mode_switch_resets_to_appropriate_camera() {
        let mut surface = AuthoringSurfaceState::default();
        surface.set_mode(AuthoringMode::TwoD);
        assert!(matches!(surface.camera, AuthoringCamera::TwoD(_)));
    }

    #[test]
    fn command_stack_round_trips_transform() {
        let mut scene = ForgeScene::new("demo.scene", "Demo");
        scene
            .insert_root(SceneEntity::new(EntityId::from("player"), "Player"))
            .unwrap();

        let before = Transform::ThreeD(Transform3D::default());
        let after = Transform::ThreeD(Transform3D {
            translation: [4.0, 5.0, 6.0],
            ..Default::default()
        });

        let mut stack = CommandStack::default();
        stack
            .execute(
                &mut scene,
                AuthoringCommand::SetTransform {
                    entity: EntityId::from("player"),
                    before,
                    after,
                },
            )
            .unwrap();
        assert!(stack.undo(&mut scene).unwrap());
        assert_eq!(
            scene.get(&EntityId::from("player")).unwrap().transform,
            before
        );
    }
}
