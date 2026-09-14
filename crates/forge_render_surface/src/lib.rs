//! Renderer-backed workspace surface lifecycle and scheduling.
#![forbid(unsafe_code)]

use forge_render_core::{
    RenderBackend, RenderRequest, RenderTargetHandle, SurfaceDimension, SurfaceExtent,
};
use forge_scene_core::ForgeScene;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum RenderFamily {
    TwoD,
    TwoPointFiveD,
    ThreeD,
    Voxel,
    #[default]
    Hybrid,
}

impl RenderFamily {
    pub const fn label(self) -> &'static str {
        match self {
            Self::TwoD => "2D",
            Self::TwoPointFiveD => "2.5D",
            Self::ThreeD => "3D",
            Self::Voxel => "Voxel",
            Self::Hybrid => "Hybrid",
        }
    }

    pub const fn base_dimension(self) -> SurfaceDimension {
        match self {
            Self::TwoD => SurfaceDimension::TwoD,
            Self::TwoPointFiveD => SurfaceDimension::Hybrid,
            Self::ThreeD | Self::Voxel => SurfaceDimension::ThreeD,
            Self::Hybrid => SurfaceDimension::Hybrid,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SurfaceVisibility {
    #[default]
    Focused,
    Visible,
    Obscured,
    Hidden,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FrameSchedule {
    pub focused_fps: u32,
    pub visible_fps: u32,
    pub obscured_fps: u32,
}

impl Default for FrameSchedule {
    fn default() -> Self {
        Self {
            focused_fps: 60,
            visible_fps: 30,
            obscured_fps: 5,
        }
    }
}

impl FrameSchedule {
    pub const fn target_fps(&self, visibility: SurfaceVisibility) -> u32 {
        match visibility {
            SurfaceVisibility::Focused => self.focused_fps,
            SurfaceVisibility::Visible => self.visible_fps,
            SurfaceVisibility::Obscured => self.obscured_fps,
            SurfaceVisibility::Hidden => 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RenderSurfaceDescriptor {
    pub id: String,
    pub label: String,
    pub family: RenderFamily,
    pub schedule: FrameSchedule,
    pub allow_editor_overlays: bool,
}

impl RenderSurfaceDescriptor {
    pub fn new(id: impl Into<String>, label: impl Into<String>, family: RenderFamily) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            family,
            schedule: FrameSchedule::default(),
            allow_editor_overlays: true,
        }
    }
}

pub struct RenderSurfaceHost {
    descriptor: RenderSurfaceDescriptor,
    backend: Box<dyn RenderBackend>,
    extent: SurfaceExtent,
    visibility: SurfaceVisibility,
    frame_index: u64,
    last_target: Option<RenderTargetHandle>,
    last_error: Option<String>,
}

impl RenderSurfaceHost {
    pub fn new(descriptor: RenderSurfaceDescriptor, backend: impl RenderBackend + 'static) -> Self {
        Self {
            descriptor,
            backend: Box::new(backend),
            extent: SurfaceExtent {
                width: 1,
                height: 1,
            },
            visibility: SurfaceVisibility::Focused,
            frame_index: 0,
            last_target: None,
            last_error: None,
        }
    }

    pub fn descriptor(&self) -> &RenderSurfaceDescriptor {
        &self.descriptor
    }

    pub fn descriptor_mut(&mut self) -> &mut RenderSurfaceDescriptor {
        &mut self.descriptor
    }

    pub fn backend_id(&self) -> &'static str {
        self.backend.backend_id()
    }

    pub const fn extent(&self) -> SurfaceExtent {
        self.extent
    }

    pub const fn visibility(&self) -> SurfaceVisibility {
        self.visibility
    }

    pub fn set_visibility(&mut self, visibility: SurfaceVisibility) {
        self.visibility = visibility;
    }

    pub const fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub fn last_target(&self) -> Option<&RenderTargetHandle> {
        self.last_target.as_ref()
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn target_fps(&self) -> u32 {
        self.descriptor.schedule.target_fps(self.visibility)
    }

    pub fn resize(&mut self, extent: SurfaceExtent) -> Result<(), String> {
        if !extent.is_valid() {
            return Err("render surface extent must be non-zero".into());
        }
        if extent == self.extent {
            return Ok(());
        }
        self.backend.resize(extent)?;
        self.extent = extent;
        Ok(())
    }

    pub fn render(
        &mut self,
        scene: &ForgeScene,
        request: &RenderRequest,
    ) -> Result<&RenderTargetHandle, String> {
        if !self.backend.supports(request.dimension) {
            let message = format!(
                "backend {} does not support {:?}",
                self.backend.backend_id(),
                request.dimension
            );
            self.last_error = Some(message.clone());
            return Err(message);
        }
        self.resize(request.extent)?;
        match self.backend.render(scene, request) {
            Ok(target) => {
                self.frame_index = self.frame_index.saturating_add(1);
                self.last_error = None;
                let stored = self.last_target.insert(target);
                Ok(&*stored)
            }
            Err(error) => {
                self.last_error = Some(error.clone());
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_render_core::NullRenderBackend;

    #[test]
    fn hidden_surfaces_suspend_their_frame_target() {
        let schedule = FrameSchedule::default();
        assert_eq!(schedule.target_fps(SurfaceVisibility::Hidden), 0);
    }

    #[test]
    fn null_backend_can_back_a_universal_surface_host() {
        let host = RenderSurfaceHost::new(
            RenderSurfaceDescriptor::new("surface.test", "Test", RenderFamily::Hybrid),
            NullRenderBackend::default(),
        );
        assert_eq!(host.backend_id(), "forge.render.null");
        assert_eq!(host.target_fps(), 60);
    }
}
