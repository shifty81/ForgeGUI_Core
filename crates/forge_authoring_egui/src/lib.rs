//! egui host for the universal authoring surface.
//!
//! Real applications may provide their own GPU render backend. This crate owns
//! shared editor interaction/chrome and a lightweight fallback visualization.

#![forbid(unsafe_code)]

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use forge_authoring_core::{AuthoringMode, AuthoringSession, GizmoMode};
use forge_gui_theme::ForgeTheme;
use forge_scene_core::{EntityId, Transform};

#[derive(Clone, Debug, Default)]
pub struct AuthoringSurfaceResponse {
    pub selected: Option<EntityId>,
    pub camera_changed: bool,
    pub context_requested: bool,
}

pub fn show_authoring_surface(
    ui: &mut Ui,
    session: &mut AuthoringSession,
    theme: &ForgeTheme,
) -> AuthoringSurfaceResponse {
    toolbar(ui, session);

    let available = ui.available_size();
    let size = Vec2::new(available.x.max(120.0), available.y.max(120.0));
    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

    ui.painter()
        .rect_filled(rect, 8.0, color(theme.base.background));
    match session.surface.mode {
        AuthoringMode::TwoD => draw_2d(ui, rect, session, theme),
        AuthoringMode::ThreeD => draw_3d(ui, rect, session, theme),
        AuthoringMode::Hybrid => {
            draw_3d(ui, rect, session, theme);
            draw_hybrid_overlay(ui, rect, theme);
        }
    }

    let mut out = AuthoringSurfaceResponse::default();
    apply_navigation(ui, rect, &response, session, &mut out);

    if response.secondary_clicked() {
        out.context_requested = true;
    }

    if response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            if let Some(entity) = nearest_entity(session, rect, pointer) {
                session.surface.selection.select_only(entity.clone());
                out.selected = Some(entity);
            } else {
                session.surface.selection.clear();
            }
        }
    }

    out
}

fn toolbar(ui: &mut Ui, session: &mut AuthoringSession) {
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(&mut session.surface.mode, AuthoringMode::TwoD, "2D");
        ui.selectable_value(&mut session.surface.mode, AuthoringMode::ThreeD, "3D");
        ui.selectable_value(&mut session.surface.mode, AuthoringMode::Hybrid, "Hybrid");

        ui.separator();

        ui.selectable_value(&mut session.surface.gizmo, GizmoMode::Select, "Select");
        ui.selectable_value(&mut session.surface.gizmo, GizmoMode::Translate, "Move");
        ui.selectable_value(&mut session.surface.gizmo, GizmoMode::Rotate, "Rotate");
        ui.selectable_value(&mut session.surface.gizmo, GizmoMode::Scale, "Scale");

        ui.separator();
        ui.checkbox(&mut session.surface.grid.visible, "Grid");
        ui.checkbox(&mut session.surface.snap.enabled, "Snap");
        ui.checkbox(&mut session.surface.show_guides, "Guides");
    });
    ui.separator();
}

fn apply_navigation(
    ui: &Ui,
    rect: Rect,
    response: &Response,
    session: &mut AuthoringSession,
    out: &mut AuthoringSurfaceResponse,
) {
    let pointer_over = response.hovered();
    let zoom_delta = if pointer_over {
        ui.input(|input| input.zoom_delta())
    } else {
        1.0
    };

    if (zoom_delta - 1.0).abs() > f32::EPSILON {
        match &mut session.surface.camera {
            forge_render_core::AuthoringCamera::TwoD(camera) => {
                camera.zoom = (camera.zoom * zoom_delta).clamp(0.02, 128.0);
            }
            forge_render_core::AuthoringCamera::ThreeD(camera) => {
                let factor = 1.0 / zoom_delta.max(0.01);
                for component in &mut camera.position {
                    *component *= factor;
                }
            }
            forge_render_core::AuthoringCamera::Hybrid {
                camera_2d,
                camera_3d,
                ..
            } => {
                camera_2d.zoom = (camera_2d.zoom * zoom_delta).clamp(0.02, 128.0);
                let factor = 1.0 / zoom_delta.max(0.01);
                for component in &mut camera_3d.position {
                    *component *= factor;
                }
            }
        }
        out.camera_changed = true;
    }

    if response.dragged_by(egui::PointerButton::Middle) {
        let delta = response.drag_delta();
        match &mut session.surface.camera {
            forge_render_core::AuthoringCamera::TwoD(camera) => {
                camera.center[0] -= delta.x / camera.zoom.max(0.001);
                camera.center[1] -= delta.y / camera.zoom.max(0.001);
            }
            forge_render_core::AuthoringCamera::ThreeD(camera) => {
                camera.position[0] -= delta.x * 0.01;
                camera.position[1] += delta.y * 0.01;
                camera.target[0] -= delta.x * 0.01;
                camera.target[1] += delta.y * 0.01;
            }
            forge_render_core::AuthoringCamera::Hybrid {
                camera_2d,
                camera_3d,
                ..
            } => {
                camera_2d.center[0] -= delta.x / camera_2d.zoom.max(0.001);
                camera_2d.center[1] -= delta.y / camera_2d.zoom.max(0.001);
                camera_3d.position[0] -= delta.x * 0.01;
                camera_3d.position[1] += delta.y * 0.01;
                camera_3d.target[0] -= delta.x * 0.01;
                camera_3d.target[1] += delta.y * 0.01;
            }
        }
        let _ = rect;
        out.camera_changed = true;
    }
}

fn draw_2d(ui: &Ui, rect: Rect, session: &AuthoringSession, theme: &ForgeTheme) {
    if session.surface.grid.visible {
        let spacing = session.surface.grid.spacing_2d[0].max(4.0);
        let line = Stroke::new(1.0, Color32::from_gray(42));
        let major = Stroke::new(1.0, Color32::from_gray(55));
        let mut index = 0usize;
        let mut x = rect.left();
        while x <= rect.right() {
            ui.painter().line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                if index.is_multiple_of(session.surface.grid.major_every.max(1) as usize) {
                    major
                } else {
                    line
                },
            );
            x += spacing;
            index += 1;
        }

        index = 0;
        let mut y = rect.top();
        while y <= rect.bottom() {
            ui.painter().line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                if index.is_multiple_of(session.surface.grid.major_every.max(1) as usize) {
                    major
                } else {
                    line
                },
            );
            y += spacing;
            index += 1;
        }
    }

    draw_entities_2d(ui, rect, session, theme);
}

fn draw_3d(ui: &Ui, rect: Rect, session: &AuthoringSession, theme: &ForgeTheme) {
    if session.surface.grid.visible {
        let horizon = rect.center().y + rect.height() * 0.08;
        let vanishing = Pos2::new(rect.center().x, rect.top() + rect.height() * 0.22);
        let grid_color = Color32::from_gray(44);
        let major_color = Color32::from_gray(58);

        for i in -12..=12 {
            let t = i as f32 / 12.0;
            let bottom = Pos2::new(
                egui::lerp(rect.left()..=rect.right(), (t + 1.0) * 0.5),
                rect.bottom(),
            );
            ui.painter()
                .line_segment([vanishing, bottom], Stroke::new(1.0, grid_color));
        }

        for i in 0usize..=14 {
            let t = i as f32 / 14.0;
            let eased = t * t;
            let y = egui::lerp(horizon..=rect.bottom(), eased);
            ui.painter().line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(
                    1.0,
                    if i.is_multiple_of(4) {
                        major_color
                    } else {
                        grid_color
                    },
                ),
            );
        }

        ui.painter().line_segment(
            [
                Pos2::new(rect.left(), horizon),
                Pos2::new(rect.right(), horizon),
            ],
            Stroke::new(1.0, color(theme.base.border)),
        );
    }

    draw_entities_3d(ui, rect, session, theme);
}

fn draw_hybrid_overlay(ui: &Ui, rect: Rect, theme: &ForgeTheme) {
    let overlay = Rect::from_min_size(
        Pos2::new(rect.left() + 16.0, rect.top() + 16.0),
        Vec2::new(180.0, 96.0),
    );
    ui.painter().rect_filled(
        overlay,
        8.0,
        Color32::from_rgba_unmultiplied(20, 25, 31, 220),
    );
    ui.painter().rect_stroke(
        overlay,
        8.0,
        Stroke::new(1.0, color(theme.base.accent)),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        overlay.center(),
        egui::Align2::CENTER_CENTER,
        "HYBRID\n3D world + 2D authoring overlay",
        egui::FontId::proportional(13.0),
        color(theme.base.text),
    );
}

fn draw_entities_2d(ui: &Ui, rect: Rect, session: &AuthoringSession, theme: &ForgeTheme) {
    for entity in session.scene.entities.values() {
        if entity.editor.hidden {
            continue;
        }

        let position = match entity.transform {
            Transform::TwoD(transform) => Pos2::new(
                rect.center().x + transform.translation[0],
                rect.center().y + transform.translation[1],
            ),
            Transform::ThreeD(transform) => Pos2::new(
                rect.center().x + transform.translation[0],
                rect.center().y - transform.translation[1],
            ),
        };

        draw_entity_marker(ui, position, &entity.id, &entity.name, session, theme);
    }
}

fn draw_entities_3d(ui: &Ui, rect: Rect, session: &AuthoringSession, theme: &ForgeTheme) {
    for entity in session.scene.entities.values() {
        if entity.editor.hidden {
            continue;
        }

        let xyz = match entity.transform {
            Transform::TwoD(transform) => [
                transform.translation[0],
                transform.z_order,
                transform.translation[1],
            ],
            Transform::ThreeD(transform) => transform.translation,
        };

        let depth = (xyz[2] + 20.0).max(1.0);
        let scale = 18.0 / depth.sqrt();
        let position = Pos2::new(
            rect.center().x + xyz[0] * scale * 8.0,
            rect.center().y - xyz[1] * scale * 8.0 + xyz[2] * 2.0,
        );

        draw_entity_marker(ui, position, &entity.id, &entity.name, session, theme);
    }
}

fn draw_entity_marker(
    ui: &Ui,
    position: Pos2,
    id: &EntityId,
    name: &str,
    session: &AuthoringSession,
    theme: &ForgeTheme,
) {
    let selected = session.surface.selection.entities.contains(id);
    let radius = if selected { 8.0 } else { 6.0 };
    let fill = if selected {
        color(theme.base.accent)
    } else {
        color(theme.base.panel_raised)
    };
    ui.painter().circle_filled(position, radius, fill);
    ui.painter().circle_stroke(
        position,
        radius,
        Stroke::new(1.0, color(theme.base.border_focus)),
    );
    ui.painter().text(
        Pos2::new(position.x + 11.0, position.y),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(12.0),
        color(theme.base.text),
    );
}

fn nearest_entity(session: &AuthoringSession, rect: Rect, pointer: Pos2) -> Option<EntityId> {
    session
        .scene
        .entities
        .values()
        .filter(|entity| !entity.editor.hidden)
        .map(|entity| {
            let pos = match entity.transform {
                Transform::TwoD(transform) => Pos2::new(
                    rect.center().x + transform.translation[0],
                    rect.center().y + transform.translation[1],
                ),
                Transform::ThreeD(transform) => Pos2::new(
                    rect.center().x + transform.translation[0] * 10.0,
                    rect.center().y - transform.translation[1] * 10.0,
                ),
            };
            (entity.id.clone(), pos.distance(pointer))
        })
        .filter(|(_, distance)| *distance <= 24.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

fn color(value: forge_gui_core::Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
