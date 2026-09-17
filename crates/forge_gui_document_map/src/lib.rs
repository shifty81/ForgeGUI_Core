//! IDE-style document-map / marker-lane surface for ForgeGUI_Core.
#![forbid(unsafe_code)]

use egui::{Color32, Rect, Sense, Stroke, Ui};
use forge_gui_core::Rgba;
use forge_gui_theme::ForgeTheme;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarkerKind {
    Search,
    Bookmark,
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapMarker {
    pub position: f32,
    pub kind: MarkerKind,
}

impl MapMarker {
    pub fn clamped(self) -> Self {
        Self {
            position: self.position.clamp(0.0, 1.0),
            ..self
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DocumentMap {
    pub viewport_start: f32,
    pub viewport_end: f32,
    pub markers: Vec<MapMarker>,
}

impl Default for DocumentMap {
    fn default() -> Self {
        Self {
            viewport_start: 0.0,
            viewport_end: 0.2,
            markers: Vec::new(),
        }
    }
}

impl DocumentMap {
    pub fn set_viewport(&mut self, start: f32, end: f32) {
        self.viewport_start = start.clamp(0.0, 1.0);
        self.viewport_end = end.clamp(self.viewport_start, 1.0);
    }
}

pub fn show_document_map(
    ui: &mut Ui,
    map: &mut DocumentMap,
    height: f32,
    theme: &ForgeTheme,
) -> Option<f32> {
    let width = 18.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, height), Sense::click_and_drag());

    ui.painter()
        .rect_filled(rect, 4.0, color(theme.base.panel_recessed));

    for marker in map.markers.iter().copied().map(MapMarker::clamped) {
        let y = egui::lerp(rect.top()..=rect.bottom(), marker.position);
        let marker_rect = Rect::from_min_max(
            egui::pos2(rect.left() + 2.0, y - 1.0),
            egui::pos2(rect.right() - 2.0, y + 1.0),
        );
        ui.painter()
            .rect_filled(marker_rect, 1.0, marker_color(marker.kind, theme));
    }

    let viewport_top = egui::lerp(rect.top()..=rect.bottom(), map.viewport_start);
    let viewport_bottom = egui::lerp(rect.top()..=rect.bottom(), map.viewport_end);
    let viewport_rect = Rect::from_min_max(
        egui::pos2(rect.left() + 1.0, viewport_top),
        egui::pos2(rect.right() - 1.0, viewport_bottom.max(viewport_top + 4.0)),
    );
    ui.painter().rect_stroke(
        viewport_rect,
        3.0,
        Stroke::new(1.0, color(theme.base.accent)),
        egui::StrokeKind::Inside,
    );

    if response.clicked() || response.dragged() {
        if let Some(pointer) = response.interact_pointer_pos() {
            let normalized = ((pointer.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
            return Some(normalized);
        }
    }

    None
}

fn marker_color(kind: MarkerKind, theme: &ForgeTheme) -> Color32 {
    color(match kind {
        MarkerKind::Search => theme.base.accent,
        MarkerKind::Bookmark => Rgba(160, 140, 255, 255),
        MarkerKind::Info => Rgba(100, 170, 255, 255),
        MarkerKind::Success => theme.base.success,
        MarkerKind::Warning => theme.base.warning,
        MarkerKind::Error => theme.base.danger,
    })
}

fn color(value: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_clamps_to_document() {
        let marker = MapMarker {
            position: 2.5,
            kind: MarkerKind::Error,
        }
        .clamped();

        assert_eq!(marker.position, 1.0);
    }

    #[test]
    fn viewport_stays_ordered() {
        let mut map = DocumentMap::default();
        map.set_viewport(0.8, 0.2);
        assert_eq!(map.viewport_start, 0.8);
        assert_eq!(map.viewport_end, 0.8);
    }
}
