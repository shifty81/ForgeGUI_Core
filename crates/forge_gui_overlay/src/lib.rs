//! Generic overlay placement and collision policy.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Placement {
    Below,
    Above,
    Right,
    Left,
    Center,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OverlaySpec {
    pub id: String,
    pub anchor: Rect,
    pub size: [f32; 2],
    pub preferred: Placement,
    pub modal: bool,
}
pub fn place(spec: &OverlaySpec, bounds: Rect) -> Rect {
    let (mut x, mut y) = match spec.preferred {
        Placement::Below => (spec.anchor.x, spec.anchor.y + spec.anchor.h),
        Placement::Above => (spec.anchor.x, spec.anchor.y - spec.size[1]),
        Placement::Right => (spec.anchor.x + spec.anchor.w, spec.anchor.y),
        Placement::Left => (spec.anchor.x - spec.size[0], spec.anchor.y),
        Placement::Center => (
            bounds.x + (bounds.w - spec.size[0]) / 2.0,
            bounds.y + (bounds.h - spec.size[1]) / 2.0,
        ),
    };
    x = x.clamp(bounds.x, (bounds.x + bounds.w - spec.size[0]).max(bounds.x));
    y = y.clamp(bounds.y, (bounds.y + bounds.h - spec.size[1]).max(bounds.y));
    Rect {
        x,
        y,
        w: spec.size[0],
        h: spec.size[1],
    }
}
