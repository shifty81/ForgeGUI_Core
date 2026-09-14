//! Renderer-neutral finite/infinite canvas camera, grid and snap contracts.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExtentPolicy {
    Finite,
    Infinite,
    UserToggle,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanvasExtent {
    pub min: [f32; 2],
    pub max: [f32; 2],
}
impl CanvasExtent {
    pub fn from_size(width: f32, height: f32) -> Self {
        Self {
            min: [0.0, 0.0],
            max: [width.max(1.0), height.max(1.0)],
        }
    }
    pub fn clamp(&self, p: [f32; 2]) -> [f32; 2] {
        [
            p[0].clamp(self.min[0], self.max[0]),
            p[1].clamp(self.min[1], self.max[1]),
        ]
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanvasCamera {
    pub pan: [f32; 2],
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}
impl Default for CanvasCamera {
    fn default() -> Self {
        Self {
            pan: [0.0, 0.0],
            zoom: 1.0,
            min_zoom: 0.08,
            max_zoom: 16.0,
        }
    }
}
impl CanvasCamera {
    pub fn with_limits(mut self, min: f32, max: f32) -> Self {
        assert!(min > 0.0 && max >= min);
        self.min_zoom = min;
        self.max_zoom = max;
        self.zoom = self.zoom.clamp(min, max);
        self
    }
    pub fn pan_by(&mut self, d: [f32; 2]) {
        if d[0].is_finite() && d[1].is_finite() {
            self.pan[0] += d[0];
            self.pan[1] += d[1];
        }
    }
    pub fn zoom_by_factor(&mut self, f: f32) {
        if f.is_finite() && f > 0.0 {
            self.zoom = (self.zoom * f).clamp(self.min_zoom, self.max_zoom);
        }
    }
    pub fn zoom_at_screen(&mut self, screen: [f32; 2], origin: [f32; 2], factor: f32) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let before = self.screen_to_world(screen, origin);
        self.zoom_by_factor(factor);
        let after = self.world_to_screen(before, origin);
        self.pan[0] += screen[0] - after[0];
        self.pan[1] += screen[1] - after[1];
    }
    pub fn apply_scroll_zoom(&mut self, delta: f32, sensitivity: f32) {
        if !delta.is_finite() || !sensitivity.is_finite() {
            return;
        }
        let f = (1.0 + delta * sensitivity).clamp(0.85, 1.15);
        self.zoom_by_factor(f);
    }
    pub fn world_to_screen(&self, w: [f32; 2], o: [f32; 2]) -> [f32; 2] {
        [
            o[0] + self.pan[0] + w[0] * self.zoom,
            o[1] + self.pan[1] + w[1] * self.zoom,
        ]
    }
    pub fn screen_to_world(&self, s: [f32; 2], o: [f32; 2]) -> [f32; 2] {
        [
            (s[0] - o[0] - self.pan[0]) / self.zoom,
            (s[1] - o[1] - self.pan[1]) / self.zoom,
        ]
    }
    pub fn adaptive_grid_step(&self, base: f32, min_screen: f32) -> f32 {
        if base <= 0.0 || !base.is_finite() {
            return 1.0;
        }
        if self.zoom <= 0.0 || !self.zoom.is_finite() || min_screen <= 0.0 {
            return base;
        }
        let mut step = base;
        while step * self.zoom < min_screen {
            step *= 2.0;
        }
        while step * self.zoom > min_screen * 4.0 && step > base {
            step *= 0.5;
        }
        step
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanvasSettings {
    pub extent_policy: ExtentPolicy,
    pub infinite_enabled: bool,
    pub grid: bool,
    pub snap: bool,
    pub guides: bool,
    pub minimap: bool,
    pub snap_step: f32,
}
impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            extent_policy: ExtentPolicy::UserToggle,
            infinite_enabled: true,
            grid: true,
            snap: true,
            guides: false,
            minimap: true,
            snap_step: 16.0,
        }
    }
}
impl CanvasSettings {
    pub fn is_infinite(&self) -> bool {
        match self.extent_policy {
            ExtentPolicy::Finite => false,
            ExtentPolicy::Infinite => true,
            ExtentPolicy::UserToggle => self.infinite_enabled,
        }
    }
    pub fn snap_point(&self, p: [f32; 2]) -> [f32; 2] {
        if !self.snap || self.snap_step <= 0.0 {
            return p;
        }
        [
            (p[0] / self.snap_step).round() * self.snap_step,
            (p[1] / self.snap_step).round() * self.snap_step,
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_roundtrip() {
        let c = CanvasCamera::default();
        let p = [10.0, 20.0];
        assert_eq!(
            c.screen_to_world(c.world_to_screen(p, [2.0, 3.0]), [2.0, 3.0]),
            p
        );
    }
    #[test]
    fn grid_adapts() {
        assert!(CanvasCamera::default().adaptive_grid_step(16.0, 24.0) >= 16.0);
    }
    #[test]
    fn snap_works() {
        let s = CanvasSettings::default();
        assert_eq!(s.snap_point([17.0, 31.0]), [16.0, 32.0]);
    }
}
