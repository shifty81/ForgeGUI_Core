//! Renderer-neutral 2D/3D/hybrid render contracts.
#![forbid(unsafe_code)]

use forge_scene_core::{EntityId, ForgeScene};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SurfaceDimension {
    TwoD,
    ThreeD,
    Hybrid,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SurfaceExtent {
    pub width: u32,
    pub height: u32,
}

impl SurfaceExtent {
    pub fn is_valid(self) -> bool {
        self.width > 0 && self.height > 0
    }

    pub fn aspect(self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Camera2D {
    pub center: [f32; 2],
    pub zoom: f32,
    pub rotation_radians: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            zoom: 1.0,
            rotation_radians: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Projection3D {
    Perspective {
        vertical_fov_radians: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        vertical_size: f32,
        near: f32,
        far: f32,
    },
}

impl Default for Projection3D {
    fn default() -> Self {
        Self::Perspective {
            vertical_fov_radians: 60.0_f32.to_radians(),
            near: 0.01,
            far: 100_000.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Camera3D {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub projection: Projection3D,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: [8.0, 8.0, 8.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            projection: Projection3D::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum AuthoringCamera {
    TwoD(Camera2D),
    ThreeD(Camera3D),
    Hybrid {
        camera_2d: Camera2D,
        camera_3d: Camera3D,
        blend: f32,
    },
}

impl Default for AuthoringCamera {
    fn default() -> Self {
        Self::ThreeD(Camera3D::default())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct PickingRay {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PickHit {
    pub entity: EntityId,
    pub distance: f32,
    pub world_position: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RenderRequest {
    pub dimension: SurfaceDimension,
    pub extent: SurfaceExtent,
    pub camera: AuthoringCamera,
    pub selected: Vec<EntityId>,
    pub hovered: Option<EntityId>,
    pub show_grid: bool,
    pub show_gizmos: bool,
    pub show_editor_overlays: bool,
}

impl RenderRequest {
    pub fn validate(&self) -> Result<(), String> {
        if !self.extent.is_valid() {
            return Err("render extent must be non-zero".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderTargetHandle {
    pub id: String,
}

pub trait RenderBackend {
    fn backend_id(&self) -> &'static str;
    fn supports(&self, dimension: SurfaceDimension) -> bool;
    fn resize(&mut self, extent: SurfaceExtent) -> Result<(), String>;
    fn render(
        &mut self,
        scene: &ForgeScene,
        request: &RenderRequest,
    ) -> Result<RenderTargetHandle, String>;
    fn pick(
        &self,
        scene: &ForgeScene,
        request: &RenderRequest,
        ray: PickingRay,
    ) -> Result<Option<PickHit>, String>;
}

pub trait OverlayRenderer {
    fn draw_overlay(&mut self, request: &RenderRequest) -> Result<(), String>;
}

#[derive(Default)]
pub struct NullRenderBackend {
    extent: Option<SurfaceExtent>,
}

impl RenderBackend for NullRenderBackend {
    fn backend_id(&self) -> &'static str {
        "forge.render.null"
    }

    fn supports(&self, _dimension: SurfaceDimension) -> bool {
        true
    }

    fn resize(&mut self, extent: SurfaceExtent) -> Result<(), String> {
        if !extent.is_valid() {
            return Err("render extent must be non-zero".into());
        }
        self.extent = Some(extent);
        Ok(())
    }

    fn render(
        &mut self,
        _scene: &ForgeScene,
        request: &RenderRequest,
    ) -> Result<RenderTargetHandle, String> {
        request.validate()?;
        if self.extent != Some(request.extent) {
            self.resize(request.extent)?;
        }
        Ok(RenderTargetHandle {
            id: format!("null:{}x{}", request.extent.width, request.extent.height),
        })
    }

    fn pick(
        &self,
        _scene: &ForgeScene,
        _request: &RenderRequest,
        _ray: PickingRay,
    ) -> Result<Option<PickHit>, String> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_renderer_accepts_all_dimensions() {
        let renderer = NullRenderBackend::default();
        assert!(renderer.supports(SurfaceDimension::TwoD));
        assert!(renderer.supports(SurfaceDimension::ThreeD));
        assert!(renderer.supports(SurfaceDimension::Hybrid));
    }
}
