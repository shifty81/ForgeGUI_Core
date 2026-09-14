//! Shared runtime/editor scene model for ForgeGUI_Core.
//!
//! The scene representation is deliberately GUI-independent so editor,
//! headless runtime, PIE, tests, and shipped applications can share it.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const FORGE_SCENE_SCHEMA: &str = "forge.scene.v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub String);

impl EntityId {
    pub fn new(value: impl Into<String>) -> Result<Self, SceneError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SceneError::InvalidEntityId(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for EntityId {
    fn from(value: &str) -> Self {
        Self::new(value).expect("entity IDs must not be empty")
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transform2D {
    pub translation: [f32; 2],
    pub rotation_radians: f32,
    pub scale: [f32; 2],
    pub z_order: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0],
            rotation_radians: 0.0,
            scale: [1.0, 1.0],
            z_order: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transform3D {
    pub translation: [f32; 3],
    /// Quaternion in `[x, y, z, w]` order.
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Transform {
    TwoD(Transform2D),
    ThreeD(Transform3D),
}

impl Default for Transform {
    fn default() -> Self {
        Self::ThreeD(Transform3D::default())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SceneValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Color([f32; 4]),
    Reference(String),
    List(Vec<SceneValue>),
    Object(BTreeMap<String, SceneValue>),
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ComponentRecord {
    pub type_id: String,
    pub properties: BTreeMap<String, SceneValue>,
}

impl ComponentRecord {
    pub fn new(type_id: impl Into<String>) -> Self {
        Self {
            type_id: type_id.into(),
            properties: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorMetadata {
    pub hidden: bool,
    pub locked: bool,
    pub expanded_in_outliner: bool,
    pub generated: bool,
    pub provenance: Option<String>,
    pub notes: String,
}

impl Default for EditorMetadata {
    fn default() -> Self {
        Self {
            hidden: false,
            locked: false,
            expanded_in_outliner: true,
            generated: false,
            provenance: None,
            notes: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SceneEntity {
    pub id: EntityId,
    pub name: String,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
    pub transform: Transform,
    pub tags: BTreeSet<String>,
    pub layers: BTreeSet<String>,
    pub components: BTreeMap<String, ComponentRecord>,
    pub editor: EditorMetadata,
}

impl SceneEntity {
    pub fn new(id: impl Into<EntityId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            parent: None,
            children: Vec::new(),
            transform: Transform::default(),
            tags: BTreeSet::new(),
            layers: BTreeSet::new(),
            components: BTreeMap::new(),
            editor: EditorMetadata::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForgeScene {
    pub schema: String,
    pub id: String,
    pub name: String,
    pub entities: BTreeMap<EntityId, SceneEntity>,
    pub roots: Vec<EntityId>,
    pub metadata: BTreeMap<String, SceneValue>,
}

impl ForgeScene {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: FORGE_SCENE_SCHEMA.into(),
            id: id.into(),
            name: name.into(),
            entities: BTreeMap::new(),
            roots: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn insert_root(&mut self, entity: SceneEntity) -> Result<(), SceneError> {
        if self.entities.contains_key(&entity.id) {
            return Err(SceneError::DuplicateEntity(entity.id.0));
        }
        if entity.parent.is_some() {
            return Err(SceneError::InvalidHierarchy(
                "root entity may not already have a parent".into(),
            ));
        }
        let id = entity.id.clone();
        self.entities.insert(id.clone(), entity);
        self.roots.push(id);
        Ok(())
    }

    pub fn insert_child(
        &mut self,
        parent: &EntityId,
        mut entity: SceneEntity,
    ) -> Result<(), SceneError> {
        if self.entities.contains_key(&entity.id) {
            return Err(SceneError::DuplicateEntity(entity.id.0));
        }
        if !self.entities.contains_key(parent) {
            return Err(SceneError::MissingEntity(parent.0.clone()));
        }

        let child_id = entity.id.clone();
        entity.parent = Some(parent.clone());
        self.entities.insert(child_id.clone(), entity);
        self.entities
            .get_mut(parent)
            .expect("parent verified above")
            .children
            .push(child_id);
        Ok(())
    }

    pub fn get(&self, id: &EntityId) -> Option<&SceneEntity> {
        self.entities.get(id)
    }

    pub fn get_mut(&mut self, id: &EntityId) -> Option<&mut SceneEntity> {
        self.entities.get_mut(id)
    }

    pub fn validate(&self) -> Result<(), SceneError> {
        if self.schema != FORGE_SCENE_SCHEMA {
            return Err(SceneError::UnsupportedSchema(self.schema.clone()));
        }

        let mut roots_seen = BTreeSet::new();
        for root in &self.roots {
            if !roots_seen.insert(root.clone()) {
                return Err(SceneError::InvalidHierarchy(format!(
                    "duplicate root {}",
                    root.0
                )));
            }
            let entity = self
                .entities
                .get(root)
                .ok_or_else(|| SceneError::MissingEntity(root.0.clone()))?;
            if entity.parent.is_some() {
                return Err(SceneError::InvalidHierarchy(format!(
                    "root {} has a parent",
                    root.0
                )));
            }
        }

        for (id, entity) in &self.entities {
            if let Some(parent) = &entity.parent {
                let parent_entity = self
                    .entities
                    .get(parent)
                    .ok_or_else(|| SceneError::MissingEntity(parent.0.clone()))?;
                if !parent_entity.children.contains(id) {
                    return Err(SceneError::InvalidHierarchy(format!(
                        "parent {} does not reference child {}",
                        parent.0, id.0
                    )));
                }
            }

            for child in &entity.children {
                let child_entity = self
                    .entities
                    .get(child)
                    .ok_or_else(|| SceneError::MissingEntity(child.0.clone()))?;
                if child_entity.parent.as_ref() != Some(id) {
                    return Err(SceneError::InvalidHierarchy(format!(
                        "child {} does not reference parent {}",
                        child.0, id.0
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn runtime_clone(&self) -> Self {
        let mut scene = self.clone();
        for entity in scene.entities.values_mut() {
            entity.editor = EditorMetadata::default();
        }
        scene
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneError {
    InvalidEntityId(String),
    DuplicateEntity(String),
    MissingEntity(String),
    InvalidHierarchy(String),
    UnsupportedSchema(String),
}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntityId(value) => write!(f, "invalid entity id: {value}"),
            Self::DuplicateEntity(value) => write!(f, "duplicate entity: {value}"),
            Self::MissingEntity(value) => write!(f, "missing entity: {value}"),
            Self::InvalidHierarchy(value) => write!(f, "invalid hierarchy: {value}"),
            Self::UnsupportedSchema(value) => write!(f, "unsupported scene schema: {value}"),
        }
    }
}

impl Error for SceneError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_clone_strips_editor_state() {
        let mut scene = ForgeScene::new("demo.scene", "Demo");
        let mut entity = SceneEntity::new(EntityId::from("player"), "Player");
        entity.editor.hidden = true;
        scene.insert_root(entity).unwrap();

        let runtime = scene.runtime_clone();
        assert!(
            !runtime
                .get(&EntityId::from("player"))
                .unwrap()
                .editor
                .hidden
        );
    }

    #[test]
    fn hierarchy_round_trip_validates() {
        let mut scene = ForgeScene::new("demo.scene", "Demo");
        scene
            .insert_root(SceneEntity::new(EntityId::from("root"), "Root"))
            .unwrap();
        scene
            .insert_child(
                &EntityId::from("root"),
                SceneEntity::new(EntityId::from("child"), "Child"),
            )
            .unwrap();
        assert!(scene.validate().is_ok());
    }
}
