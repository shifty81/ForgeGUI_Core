//! Creator workspace widgets backed by ForgeRender surfaces.
#![forbid(unsafe_code)]

use egui::{Align, Frame, Layout, Margin, Rect, Response, Sense, Stroke, Ui};
use forge_gui_icons::IconId;
use forge_gui_theme::ForgeTheme;
use forge_gui_widgets::{compact_tool_button, WidgetTone};
use forge_render_core::SurfaceExtent;
use forge_render_surface::{RenderFamily, RenderSurfaceHost};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkspaceTool {
    #[default]
    Select,
    Move,
    Rotate,
    Scale,
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

#[derive(Debug)]
pub struct WorkspaceResponse {
    pub surface_response: Response,
    pub family_changed: Option<RenderFamily>,
    pub requested_focus: bool,
}

pub fn show_renderer_workspace(
    ui: &mut Ui,
    surface: &mut RenderSurfaceHost,
    view: &mut WorkspaceViewState,
    theme: &ForgeTheme,
    presenter: impl FnOnce(&mut Ui, Rect, &RenderSurfaceHost),
) -> WorkspaceResponse {
    let mut family_changed = None;
    show_workspace_toolbar(ui, surface, view, theme, &mut family_changed);
    ui.add_space(4.0);

    let available = ui.available_size();
    let desired = egui::vec2(available.x.max(160.0), available.y.max(120.0));
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());

    let width = rect.width().round().max(1.0) as u32;
    let height = rect.height().round().max(1.0) as u32;
    let _ = surface.resize(SurfaceExtent { width, height });

    ui.painter()
        .rect_filled(rect, theme.base.corner_radius, color(theme.chrome.canvas));
    ui.painter().rect_stroke(
        rect,
        theme.base.corner_radius,
        Stroke::new(1.0, color(theme.base.border)),
        egui::StrokeKind::Inside,
    );

    presenter(ui, rect, surface);
    show_surface_overlay(ui, rect, surface, view, theme);

    WorkspaceResponse {
        requested_focus: response.clicked(),
        surface_response: response,
        family_changed,
    }
}

fn show_workspace_toolbar(
    ui: &mut Ui,
    surface: &mut RenderSurfaceHost,
    view: &mut WorkspaceViewState,
    theme: &ForgeTheme,
    family_changed: &mut Option<RenderFamily>,
) {
    Frame::new()
        .fill(color(theme.chrome.action_bar))
        .corner_radius(6)
        .inner_margin(Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (tool, icon, label) in [
                    (WorkspaceTool::Select, IconId::Asset, "Select"),
                    (WorkspaceTool::Move, IconId::Entity, "Move"),
                    (WorkspaceTool::Rotate, IconId::Restart, "Rotate"),
                    (WorkspaceTool::Scale, IconId::Module, "Scale"),
                ] {
                    if compact_tool_button(ui, icon, label, view.active_tool == tool, theme)
                        .clicked()
                    {
                        view.active_tool = tool;
                    }
                }

                ui.separator();
                ui.checkbox(&mut view.show_grid, "Grid");
                ui.checkbox(&mut view.snap_enabled, "Snap");
                ui.checkbox(&mut view.show_guides, "Guides");
                ui.separator();

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
                        *family_changed = Some(family);
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let _ = forge_gui_widgets::badge(
                        ui,
                        &format!("{} fps", surface.target_fps()),
                        WidgetTone::Accent,
                        theme,
                    );
                });
            });
        });
}

fn show_surface_overlay(
    ui: &mut Ui,
    rect: Rect,
    surface: &RenderSurfaceHost,
    view: &WorkspaceViewState,
    theme: &ForgeTheme,
) {
    let painter = ui.painter();
    let top_left = rect.left_top() + egui::vec2(12.0, 10.0);
    painter.text(
        top_left,
        egui::Align2::LEFT_TOP,
        format!(
            "{} · {} · {}",
            surface.descriptor().label,
            surface.descriptor().family.label(),
            surface.backend_id()
        ),
        egui::FontId::proportional(13.0),
        color(theme.base.text_muted),
    );

    let bottom_left = rect.left_bottom() + egui::vec2(12.0, -10.0);
    painter.text(
        bottom_left,
        egui::Align2::LEFT_BOTTOM,
        format!("{} · {}", view.camera_label, view.selection_label),
        egui::FontId::proportional(12.0),
        color(theme.base.text_muted),
    );

    if let Some(error) = surface.last_error() {
        painter.text(
            rect.right_top() + egui::vec2(-12.0, 10.0),
            egui::Align2::RIGHT_TOP,
            error,
            egui::FontId::proportional(12.0),
            color(theme.base.danger),
        );
    }
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
