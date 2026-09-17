//! Optional authoring-workspace widgets backed by ForgeRender surfaces.
//!
//! ForgeGUI workspaces keep the authored content dominant. Persistent application
//! chrome lives outside the workspace, while canvas-specific chrome (frame, rulers,
//! layers, lightweight HUD) is rendered inside the authoring surface.
#![forbid(unsafe_code)]

use egui::{Align2, FontId, Id, Rect, Response, Sense, Stroke, Ui};
use forge_gui_theme::ForgeTheme;
use forge_render_core::SurfaceExtent;
use forge_render_surface::{RenderFamily, RenderSurfaceHost};
use serde::{Deserialize, Serialize};

const RULER_THICKNESS: f32 = 16.0;
const LAYER_STACK_WIDTH: f32 = 34.0;
const LAYER_ROW_HEIGHT: f32 = 24.0;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkspaceTool {
    #[default]
    Select,
    Move,
    Rotate,
    Scale,
    Pan,
    Zoom,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceViewState {
    pub active_tool: WorkspaceTool,
    pub show_grid: bool,
    pub snap_enabled: bool,
    pub show_guides: bool,
    pub show_overlays: bool,
    pub camera_label: String,
    pub selection_label: String,
}

impl Default for WorkspaceViewState {
    fn default() -> Self {
        Self {
            active_tool: WorkspaceTool::Select,
            show_grid: true,
            snap_enabled: true,
            show_guides: true,
            show_overlays: true,
            camera_label: "Perspective".into(),
            selection_label: "No selection".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceLayer {
    pub id: String,
    pub label: String,
    pub visible: bool,
    pub locked: bool,
}

impl WorkspaceLayer {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            visible: true,
            locked: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CanvasChromeState {
    pub show_frame: bool,
    pub show_rulers: bool,
    pub show_layer_bar: bool,
    pub show_canvas_hud: bool,
    pub ruler_spacing: f32,
    pub layers: Vec<WorkspaceLayer>,
    pub active_layer: Option<String>,
}

impl Default for CanvasChromeState {
    fn default() -> Self {
        Self {
            show_frame: true,
            show_rulers: true,
            show_layer_bar: true,
            show_canvas_hud: true,
            ruler_spacing: 32.0,
            layers: vec![
                WorkspaceLayer::new("layer.world", "World"),
                WorkspaceLayer::new("layer.entities", "Entities"),
                WorkspaceLayer::new("layer.lighting", "Lighting"),
                WorkspaceLayer::new("layer.guides", "Guides"),
            ],
            active_layer: Some("layer.world".into()),
        }
    }
}

#[derive(Debug)]
pub struct WorkspaceResponse {
    pub surface_response: Response,
    pub family_changed: Option<RenderFamily>,
    pub requested_focus: bool,
    pub layer_toggled: Option<String>,
    pub active_layer_changed: Option<String>,
}

pub fn show_renderer_workspace(
    ui: &mut Ui,
    surface: &mut RenderSurfaceHost,
    view: &mut WorkspaceViewState,
    chrome: &mut CanvasChromeState,
    theme: &ForgeTheme,
    presenter: impl FnOnce(&mut Ui, Rect, &RenderSurfaceHost),
) -> WorkspaceResponse {
    let available = ui.available_size();
    let desired = egui::vec2(available.x.max(160.0), available.y.max(120.0));
    let (outer_rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());

    ui.painter().rect_filled(
        outer_rect,
        theme.interaction.surface_radius,
        color(theme.chrome.canvas),
    );

    let framed_rect = if chrome.show_frame {
        let frame_rect = outer_rect.shrink(2.0);
        ui.painter().rect_stroke(
            frame_rect,
            theme.interaction.surface_radius,
            Stroke::new(1.0, color(theme.chrome.separator)),
            egui::StrokeKind::Inside,
        );
        frame_rect.shrink(1.0)
    } else {
        outer_rect
    };

    let ruler_capable = matches!(
        surface.descriptor().family,
        RenderFamily::TwoD | RenderFamily::TwoPointFiveD | RenderFamily::Hybrid
    );
    let rulers_visible = chrome.show_rulers && ruler_capable;
    let (content_rect, ruler_origin) = if rulers_visible {
        let top_height = RULER_THICKNESS;
        let left_width = RULER_THICKNESS;
        draw_rulers(
            ui,
            framed_rect,
            top_height,
            left_width,
            chrome.ruler_spacing,
            theme,
        );
        (
            Rect::from_min_max(
                egui::pos2(
                    framed_rect.min.x + left_width,
                    framed_rect.min.y + top_height,
                ),
                framed_rect.max,
            ),
            egui::pos2(
                framed_rect.min.x + left_width,
                framed_rect.min.y + top_height,
            ),
        )
    } else {
        (framed_rect, framed_rect.min)
    };

    let width = content_rect.width().round().max(1.0) as u32;
    let height = content_rect.height().round().max(1.0) as u32;
    let _ = surface.resize(SurfaceExtent { width, height });

    ui.painter()
        .rect_filled(content_rect, 0.0, color(theme.chrome.canvas));

    presenter(ui, content_rect, surface);

    let (layer_toggled, active_layer_changed) = if chrome.show_layer_bar {
        show_layer_stack(ui, content_rect, chrome, theme)
    } else {
        (None, None)
    };

    if chrome.show_canvas_hud {
        show_canvas_hud(ui, content_rect, surface, view, ruler_origin, theme);
    }

    WorkspaceResponse {
        requested_focus: response.clicked(),
        surface_response: response,
        family_changed: None,
        layer_toggled,
        active_layer_changed,
    }
}

fn draw_rulers(
    ui: &Ui,
    rect: Rect,
    top_height: f32,
    left_width: f32,
    spacing: f32,
    theme: &ForgeTheme,
) {
    let painter = ui.painter();
    let fill = color(theme.base.panel_recessed);
    let border = color(theme.chrome.separator);
    let text = color(theme.base.text_muted);

    let top = Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.min.y + top_height));
    let left = Rect::from_min_max(rect.min, egui::pos2(rect.min.x + left_width, rect.max.y));
    let corner = Rect::from_min_size(rect.min, egui::vec2(left_width, top_height));

    painter.rect_filled(top, 0.0, fill);
    painter.rect_filled(left, 0.0, fill);
    painter.rect_filled(corner, 0.0, color(theme.chrome.shell));
    painter.line_segment(
        [top.left_bottom(), top.right_bottom()],
        Stroke::new(1.0, border),
    );
    painter.line_segment(
        [left.right_top(), left.right_bottom()],
        Stroke::new(1.0, border),
    );

    let spacing = spacing.max(12.0);
    let origin_x = rect.min.x + left_width;
    let mut x = origin_x;
    let mut index = 0usize;
    while x <= rect.max.x {
        let major = index.is_multiple_of(4);
        let tick = if major { 5.0 } else { 3.0 };
        painter.line_segment(
            [
                egui::pos2(x, top.bottom()),
                egui::pos2(x, top.bottom() - tick),
            ],
            Stroke::new(1.0, border),
        );
        if major {
            painter.text(
                egui::pos2(x + 2.0, top.top() + 2.0),
                Align2::LEFT_TOP,
                (index as i32 * spacing as i32).to_string(),
                FontId::monospace(8.0),
                text,
            );
        }
        x += spacing;
        index += 1;
    }

    let origin_y = rect.min.y + top_height;
    let mut y = origin_y;
    let mut row = 0usize;
    while y <= rect.max.y {
        let major = row.is_multiple_of(4);
        let tick = if major { 5.0 } else { 3.0 };
        painter.line_segment(
            [
                egui::pos2(left.right(), y),
                egui::pos2(left.right() - tick, y),
            ],
            Stroke::new(1.0, border),
        );
        if major && row > 0 {
            painter.text(
                egui::pos2(left.left() + 1.0, y + 1.0),
                Align2::LEFT_TOP,
                (row as i32 * spacing as i32).to_string(),
                FontId::monospace(7.5),
                text,
            );
        }
        y += spacing;
        row += 1;
    }
}

fn show_layer_stack(
    ui: &mut Ui,
    rect: Rect,
    chrome: &mut CanvasChromeState,
    theme: &ForgeTheme,
) -> (Option<String>, Option<String>) {
    let mut toggled = None;
    let mut active_changed = None;
    let stack_height = 8.0 + chrome.layers.len() as f32 * LAYER_ROW_HEIGHT;
    let stack = Rect::from_min_size(
        rect.left_top() + egui::vec2(6.0, 6.0),
        egui::vec2(LAYER_STACK_WIDTH, stack_height),
    );

    ui.painter().rect_filled(
        stack,
        theme.interaction.control_radius,
        color(theme.base.panel),
    );
    ui.painter().rect_stroke(
        stack,
        theme.interaction.control_radius,
        Stroke::new(1.0, color(theme.chrome.separator)),
        egui::StrokeKind::Inside,
    );

    for (index, layer) in chrome.layers.iter_mut().enumerate() {
        let active = chrome.active_layer.as_deref() == Some(layer.id.as_str());
        let row = Rect::from_min_size(
            egui::pos2(
                stack.left() + 4.0,
                stack.top() + 4.0 + index as f32 * LAYER_ROW_HEIGHT,
            ),
            egui::vec2(LAYER_STACK_WIDTH - 8.0, LAYER_ROW_HEIGHT - 3.0),
        );
        let response = ui.interact(
            row,
            Id::new(("forge.canvas.layer.stack", &layer.id)),
            Sense::click(),
        );

        if response.hovered() {
            ui.painter()
                .rect_filled(row, 3.0, color(theme.base.panel_raised));
        }
        if active {
            ui.painter().rect_stroke(
                row,
                3.0,
                Stroke::new(1.0, color(theme.base.accent)),
                egui::StrokeKind::Inside,
            );
        }

        let glyph = layer
            .label
            .chars()
            .next()
            .unwrap_or('L')
            .to_ascii_uppercase()
            .to_string();
        let glyph_color = if !layer.visible {
            color(theme.base.text_muted)
        } else if active {
            color(theme.base.accent)
        } else {
            color(theme.base.text)
        };
        ui.painter().text(
            row.center(),
            Align2::CENTER_CENTER,
            glyph,
            FontId::proportional(10.5),
            glyph_color,
        );

        let eye_center = row.right_bottom() + egui::vec2(-4.0, -4.0);
        if layer.visible {
            ui.painter()
                .circle_filled(eye_center, 1.7, color(theme.base.accent));
        } else {
            ui.painter().circle_stroke(
                eye_center,
                1.7,
                Stroke::new(1.0, color(theme.base.text_muted)),
            );
        }

        if response.clicked() {
            layer.visible = !layer.visible;
            chrome.active_layer = Some(layer.id.clone());
            toggled = Some(layer.id.clone());
            active_changed = Some(layer.id.clone());
        }
        response.on_hover_text(format!(
            "{} · click to {}",
            layer.label,
            if layer.visible { "hide" } else { "show" }
        ));
    }

    (toggled, active_changed)
}

fn show_canvas_hud(
    ui: &Ui,
    rect: Rect,
    surface: &RenderSurfaceHost,
    view: &WorkspaceViewState,
    _origin: egui::Pos2,
    theme: &ForgeTheme,
) {
    let painter = ui.painter();
    let muted = color(theme.base.text_muted);

    painter.text(
        rect.right_top() + egui::vec2(-10.0, 9.0),
        Align2::RIGHT_TOP,
        format!(
            "Infinite · {} · {}",
            view.camera_label,
            surface.descriptor().family.label()
        ),
        FontId::proportional(10.5),
        muted,
    );

    painter.text(
        rect.left_bottom() + egui::vec2(10.0, -9.0),
        Align2::LEFT_BOTTOM,
        format!("{} · {} fps", view.selection_label, surface.target_fps()),
        FontId::proportional(10.5),
        muted,
    );

    if let Some(error) = surface.last_error() {
        painter.text(
            rect.right_bottom() + egui::vec2(-10.0, -9.0),
            Align2::RIGHT_BOTTOM,
            error,
            FontId::proportional(10.5),
            color(theme.base.danger),
        );
    }
}

pub fn canvas_family_menu(ui: &mut Ui, surface: &mut RenderSurfaceHost) -> Option<RenderFamily> {
    let mut changed = None;
    ui.menu_button(surface.descriptor().family.label(), |ui| {
        for family in [
            RenderFamily::TwoD,
            RenderFamily::TwoPointFiveD,
            RenderFamily::ThreeD,
            RenderFamily::Voxel,
            RenderFamily::Hybrid,
        ] {
            if ui
                .selectable_label(surface.descriptor().family == family, family.label())
                .clicked()
            {
                surface.descriptor_mut().family = family;
                changed = Some(family);
                ui.close();
            }
        }
    });
    changed
}

pub fn canvas_options_menu(
    ui: &mut Ui,
    view: &mut WorkspaceViewState,
    chrome: &mut CanvasChromeState,
) {
    ui.menu_button("Canvas", |ui| {
        ui.checkbox(&mut chrome.show_frame, "Frame");
        ui.checkbox(&mut chrome.show_rulers, "Rulers");
        ui.checkbox(&mut chrome.show_layer_bar, "Vertical layer stack");
        ui.checkbox(&mut chrome.show_canvas_hud, "Canvas HUD");
        ui.separator();
        ui.checkbox(&mut view.show_grid, "Grid");
        ui.checkbox(&mut view.snap_enabled, "Snap");
        ui.checkbox(&mut view.show_guides, "Guides");
        ui.checkbox(&mut view.show_overlays, "Overlays");
    });
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_chrome_defaults_to_canvas_first_features() {
        let chrome = CanvasChromeState::default();
        assert!(chrome.show_frame);
        assert!(chrome.show_rulers);
        assert!(chrome.show_layer_bar);
        assert!(!chrome.layers.is_empty());
    }
}
