//! Reusable ForgeGUI application shell.
//!
//! This crate is the public high-level host for ordinary ForgeGUI applications.
//! Consumer projects register surfaces and provide their content; ForgeGUI owns
//! docking, floating, tab stacks, toolbar placement, locking, shell chrome and
//! layout persistence. The ForgeGUI Lab is only a showcase consumer of the same
//! framework capabilities.
#![forbid(unsafe_code)]

use egui::{Align, Layout, PointerButton, Sense};
use forge_gui_chrome::{
    modular_surface_frame, show_chrome_bar, show_modular_surface_tabs, show_project_title_bar,
    ChromeBarKind, ModularSurfaceState, ModularToolbarState, ShellProfile, SurfaceDock,
    ToolbarDock,
};
use forge_gui_theme::ForgeTheme;
use forge_gui_widgets::panel_chrome_button;
use serde::{Deserialize, Serialize};

pub const FORGE_SHELL_LAYOUT_SCHEMA: &str = "forge.gui.shell.layout.v1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForgeShellState {
    pub schema: String,
    pub profile: ShellProfile,
    pub show_menu_bar: bool,
    pub show_status_bar: bool,
    pub show_modular_surfaces: bool,
    pub toolbar: ModularToolbarState,
    pub surfaces: Vec<ModularSurfaceState>,
    pub active_left: String,
    pub active_center: String,
    pub active_right: String,
    pub active_bottom: String,
    #[serde(skip)]
    surface_drag: Option<String>,
    #[serde(skip)]
    surface_drag_preview: Option<SurfaceDock>,
}

impl ForgeShellState {
    pub fn new(profile: ShellProfile, surfaces: Vec<ModularSurfaceState>) -> Self {
        let policy = profile.policy();
        let mut state = Self {
            schema: FORGE_SHELL_LAYOUT_SCHEMA.into(),
            profile,
            show_menu_bar: policy.menu_bar,
            show_status_bar: policy.status_bar,
            show_modular_surfaces: policy.modular_surfaces,
            toolbar: ModularToolbarState {
                visible: policy.toolbar,
                ..Default::default()
            },
            surfaces,
            active_left: String::new(),
            active_center: String::new(),
            active_right: String::new(),
            active_bottom: String::new(),
            surface_drag: None,
            surface_drag_preview: None,
        };
        state.repair_active_surfaces();
        state
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != FORGE_SHELL_LAYOUT_SCHEMA {
            return Err(format!(
                "unsupported ForgeGUI shell schema: {}",
                self.schema
            ));
        }
        let mut ids = std::collections::BTreeSet::new();
        for surface in &self.surfaces {
            if surface.id.trim().is_empty() {
                return Err("surface id must not be empty".into());
            }
            if !ids.insert(surface.id.as_str()) {
                return Err(format!("duplicate surface id: {}", surface.id));
            }
        }
        Ok(())
    }

    pub fn apply_profile(&mut self, profile: ShellProfile) {
        self.profile = profile;
        let policy = profile.policy();
        self.show_menu_bar = policy.menu_bar;
        self.show_status_bar = policy.status_bar;
        self.show_modular_surfaces = policy.modular_surfaces;
        self.toolbar.visible = policy.toolbar && !matches!(self.toolbar.dock, ToolbarDock::Hidden);
    }

    pub fn register_surface(&mut self, surface: ModularSurfaceState) -> Result<(), String> {
        if self
            .surfaces
            .iter()
            .any(|existing| existing.id == surface.id)
        {
            return Err(format!("surface already registered: {}", surface.id));
        }
        let dock = surface.dock;
        let id = surface.id.clone();
        self.surfaces.push(surface);
        if self.active_surface(dock).is_none() {
            self.set_active_surface(dock, id);
        }
        Ok(())
    }

    pub fn surface(&self, id: &str) -> Option<&ModularSurfaceState> {
        self.surfaces.iter().find(|surface| surface.id == id)
    }

    pub fn surface_mut(&mut self, id: &str) -> Option<&mut ModularSurfaceState> {
        self.surfaces.iter_mut().find(|surface| surface.id == id)
    }

    pub fn surface_visible(&self, id: &str) -> bool {
        self.surface(id).is_some_and(|surface| surface.visible)
    }

    pub fn open_surface(&mut self, id: &str) -> bool {
        let Some(surface) = self.surface_mut(id) else {
            return false;
        };
        surface.visible = true;
        let dock = surface.dock;
        let id = surface.id.clone();
        self.set_active_surface(dock, id);
        true
    }

    pub fn hide_surface(&mut self, id: &str) -> bool {
        let Some(surface) = self.surface_mut(id) else {
            return false;
        };
        surface.visible = false;
        self.repair_active_surfaces();
        true
    }

    pub fn move_surface(&mut self, id: &str, dock: SurfaceDock) -> bool {
        let Some(surface) = self.surface_mut(id) else {
            return false;
        };
        if surface.locked && surface.dock != dock {
            return false;
        }
        surface.dock = dock;
        surface.visible = true;
        let id = surface.id.clone();
        self.set_active_surface(dock, id);
        self.repair_active_surfaces();
        true
    }

    pub fn toggle_surface_lock(&mut self, id: &str) -> bool {
        let Some(surface) = self.surface_mut(id) else {
            return false;
        };
        surface.locked = !surface.locked;
        true
    }

    pub fn show_all(&mut self) {
        for surface in &mut self.surfaces {
            surface.visible = true;
        }
        self.show_modular_surfaces = true;
        self.repair_active_surfaces();
    }

    pub fn hide_all(&mut self) {
        for surface in &mut self.surfaces {
            surface.visible = false;
        }
    }

    pub fn restore_registered(&mut self, defaults: &[ModularSurfaceState]) -> usize {
        let mut restored = 0;
        for surface in &mut self.surfaces {
            if let Some(default) = defaults.iter().find(|default| default.id == surface.id) {
                surface.title = default.title.clone();
                surface.dock = default.dock;
                surface.visible = default.visible;
                surface.locked = default.locked;
                surface.preferred_size = default.preferred_size;
                restored += 1;
            }
        }
        self.apply_profile(ShellProfile::Standard);
        self.toolbar = ModularToolbarState::default();
        self.surface_drag = None;
        self.surface_drag_preview = None;
        self.repair_active_surfaces();
        restored
    }

    pub fn active_surface(&self, dock: SurfaceDock) -> Option<&str> {
        let id = match dock {
            SurfaceDock::Left => &self.active_left,
            SurfaceDock::Center => &self.active_center,
            SurfaceDock::Right => &self.active_right,
            SurfaceDock::Bottom => &self.active_bottom,
            SurfaceDock::Floating => return None,
        };
        if id.is_empty() {
            None
        } else {
            Some(id)
        }
    }

    pub fn set_active_surface(&mut self, dock: SurfaceDock, id: impl Into<String>) {
        let id = id.into();
        match dock {
            SurfaceDock::Left => self.active_left = id,
            SurfaceDock::Center => self.active_center = id,
            SurfaceDock::Right => self.active_right = id,
            SurfaceDock::Bottom => self.active_bottom = id,
            SurfaceDock::Floating => {}
        }
    }

    pub fn ensure_active_surface(&mut self, dock: SurfaceDock) -> String {
        if let Some(current) = self.active_surface(dock).map(ToOwned::to_owned) {
            if self
                .surfaces
                .iter()
                .any(|surface| surface.visible && surface.dock == dock && surface.id == current)
            {
                return current;
            }
        }
        let fallback = self
            .surfaces
            .iter()
            .find(|surface| surface.visible && surface.dock == dock)
            .map(|surface| surface.id.clone())
            .unwrap_or_default();
        self.set_active_surface(dock, fallback.clone());
        fallback
    }

    pub fn dock_group_locked(&self, dock: SurfaceDock) -> bool {
        let active = self.active_surface(dock).unwrap_or_default();
        self.surfaces
            .iter()
            .find(|surface| surface.visible && surface.dock == dock && surface.id == active)
            .or_else(|| {
                self.surfaces
                    .iter()
                    .find(|surface| surface.visible && surface.dock == dock)
            })
            .is_some_and(|surface| surface.locked)
    }

    pub fn save_layout_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(|error| error.to_string())
    }

    pub fn load_layout_json(text: &str) -> Result<Self, String> {
        let mut state: Self = serde_json::from_str(text).map_err(|error| error.to_string())?;
        state.surface_drag = None;
        state.surface_drag_preview = None;
        state.validate()?;
        state.repair_active_surfaces();
        Ok(state)
    }

    fn repair_active_surfaces(&mut self) {
        for dock in [
            SurfaceDock::Left,
            SurfaceDock::Center,
            SurfaceDock::Right,
            SurfaceDock::Bottom,
        ] {
            self.ensure_active_surface(dock);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForgeShellEvent {
    Activated(String),
    Hidden(String),
    Docked { surface: String, dock: SurfaceDock },
    LockChanged(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ForgeShellResponse {
    pub events: Vec<ForgeShellEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForgeApplicationSpec {
    pub product: String,
    pub project: String,
    pub branch: Option<String>,
}

impl ForgeApplicationSpec {
    pub fn new(product: impl Into<String>, project: impl Into<String>) -> Self {
        Self {
            product: product.into(),
            project: project.into(),
            branch: None,
        }
    }
}

/// Content supplied by a consumer application. ForgeGUI owns the shell;
/// the application owns what each registered surface actually does.
pub trait ForgeShellContent {
    fn menu(&mut self, _ui: &mut egui::Ui) {}
    fn toolbar(&mut self, _ui: &mut egui::Ui) {}
    fn status(&mut self, _ui: &mut egui::Ui) {}
    fn surface(&mut self, id: &str, ui: &mut egui::Ui);
}

/// Render a complete reusable ForgeGUI desktop shell.
///
/// This is the preferred drop-in entry point for Rust/egui consumers: register
/// surfaces in `ForgeShellState`, implement `ForgeShellContent`, and let the
/// framework own common window chrome, menus, toolbars, docking and floating.
pub fn show_application_shell<C: ForgeShellContent>(
    root: &mut egui::Ui,
    ctx: &egui::Context,
    spec: &ForgeApplicationSpec,
    state: &mut ForgeShellState,
    theme: &ForgeTheme,
    content: &mut C,
) -> ForgeShellResponse {
    let mut response = ForgeShellResponse::default();
    let shell_rect = root.available_rect_before_wrap();

    egui::Panel::top("forge.shell.title")
        .frame(egui::Frame::NONE)
        .show(root, |ui| {
            show_project_title_bar(
                ui,
                &spec.product,
                &spec.project,
                spec.branch.as_deref(),
                theme,
            );
        });

    if state.show_menu_bar {
        egui::Panel::top("forge.shell.menu")
            .frame(egui::Frame::NONE)
            .show(root, |ui| {
                show_chrome_bar(ui, ChromeBarKind::Menu, theme, |ui| content.menu(ui));
            });
    }

    show_toolbar(root, ctx, state, theme, content);

    if state.show_status_bar {
        egui::Panel::bottom("forge.shell.status")
            .frame(egui::Frame::NONE)
            .show(root, |ui| {
                show_chrome_bar(ui, ChromeBarKind::Status, theme, |ui| content.status(ui));
            });
    }

    if state.show_modular_surfaces {
        if has_dock(state, SurfaceDock::Left) {
            let size = active_preferred(state, SurfaceDock::Left, 280.0, 0);
            egui::Panel::left("forge.shell.left")
                .frame(egui::Frame::NONE)
                .resizable(!state.dock_group_locked(SurfaceDock::Left))
                .default_size(size)
                .size_range(180.0..=720.0)
                .show(root, |ui| {
                    show_dock_group(ui, state, theme, content, SurfaceDock::Left, &mut response)
                });
        }
        if has_dock(state, SurfaceDock::Right) {
            let size = active_preferred(state, SurfaceDock::Right, 320.0, 0);
            egui::Panel::right("forge.shell.right")
                .frame(egui::Frame::NONE)
                .resizable(!state.dock_group_locked(SurfaceDock::Right))
                .default_size(size)
                .size_range(180.0..=720.0)
                .show(root, |ui| {
                    show_dock_group(ui, state, theme, content, SurfaceDock::Right, &mut response)
                });
        }
        if has_dock(state, SurfaceDock::Bottom) {
            let size = active_preferred(state, SurfaceDock::Bottom, 170.0, 1);
            egui::Panel::bottom("forge.shell.bottom")
                .frame(egui::Frame::NONE)
                .resizable(!state.dock_group_locked(SurfaceDock::Bottom))
                .default_size(size)
                .size_range(100.0..=520.0)
                .show(root, |ui| {
                    show_dock_group(
                        ui,
                        state,
                        theme,
                        content,
                        SurfaceDock::Bottom,
                        &mut response,
                    )
                });
        }
    }

    let center_frame = egui::Frame::new()
        .fill(opaque(theme.chrome.shell))
        .corner_radius(theme.interaction.surface_radius.clamp(0.0, 20.0) as u8)
        .inner_margin(egui::Margin::same(4));
    egui::CentralPanel::default()
        .frame(center_frame)
        .show(root, |ui| {
            if state.show_modular_surfaces && has_dock(state, SurfaceDock::Center) {
                show_dock_group(
                    ui,
                    state,
                    theme,
                    content,
                    SurfaceDock::Center,
                    &mut response,
                );
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No center surface is registered. Add one through ForgeShellState.");
                });
            }
        });

    show_floating_surfaces(ctx, state, theme, content, &mut response);
    update_surface_drag_drop(root, ctx, shell_rect, state, theme, &mut response);
    response
}

fn show_toolbar<C: ForgeShellContent>(
    root: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut ForgeShellState,
    theme: &ForgeTheme,
    content: &mut C,
) {
    if !state.toolbar.visible || matches!(state.toolbar.dock, ToolbarDock::Hidden) {
        return;
    }
    match state.toolbar.dock {
        ToolbarDock::Top => {
            egui::Panel::top("forge.shell.toolbar.top")
                .frame(egui::Frame::NONE)
                .show(root, |ui| toolbar_frame(ui, state, theme, content, true));
        }
        ToolbarDock::Bottom => {
            egui::Panel::bottom("forge.shell.toolbar.bottom")
                .frame(egui::Frame::NONE)
                .show(root, |ui| toolbar_frame(ui, state, theme, content, true));
        }
        ToolbarDock::Left => {
            egui::Panel::left("forge.shell.toolbar.left")
                .frame(egui::Frame::NONE)
                .default_size(46.0)
                .show(root, |ui| toolbar_frame(ui, state, theme, content, false));
        }
        ToolbarDock::Right => {
            egui::Panel::right("forge.shell.toolbar.right")
                .frame(egui::Frame::NONE)
                .default_size(46.0)
                .show(root, |ui| toolbar_frame(ui, state, theme, content, false));
        }
        ToolbarDock::Floating => {
            egui::Window::new("Toolbar")
                .id(egui::Id::new("forge.shell.toolbar.floating"))
                .collapsible(false)
                .resizable(false)
                .movable(!state.toolbar.locked)
                .frame(modular_surface_frame(theme))
                .show(ctx, |ui| content.toolbar(ui));
        }
        ToolbarDock::Hidden => {}
    }
}

fn toolbar_frame<C: ForgeShellContent>(
    ui: &mut egui::Ui,
    state: &mut ForgeShellState,
    theme: &ForgeTheme,
    content: &mut C,
    horizontal: bool,
) {
    modular_surface_frame(theme).show(ui, |ui| {
        let render = |ui: &mut egui::Ui, state: &mut ForgeShellState, content: &mut C| {
            ui.menu_button("Toolbar ▾", |ui| {
                ui.checkbox(&mut state.toolbar.locked, "Lock in place");
                ui.separator();
                for dock in ToolbarDock::ALL {
                    if ui
                        .selectable_label(state.toolbar.dock == dock, dock.label())
                        .clicked()
                    {
                        state.toolbar.dock = dock;
                        state.toolbar.visible = !matches!(dock, ToolbarDock::Hidden);
                        ui.close();
                    }
                }
            });
            ui.separator();
            content.toolbar(ui);
        };
        if horizontal {
            ui.horizontal(|ui| render(ui, state, content));
        } else {
            ui.vertical(|ui| render(ui, state, content));
        }
    });
}

fn show_dock_group<C: ForgeShellContent>(
    ui: &mut egui::Ui,
    state: &mut ForgeShellState,
    theme: &ForgeTheme,
    content: &mut C,
    dock: SurfaceDock,
    response: &mut ForgeShellResponse,
) {
    let surfaces: Vec<ModularSurfaceState> = state
        .surfaces
        .iter()
        .filter(|surface| surface.visible && surface.dock == dock)
        .cloned()
        .collect();
    if surfaces.is_empty() {
        return;
    }

    let active = state.ensure_active_surface(dock);
    let active_surface = surfaces
        .iter()
        .find(|surface| surface.id == active)
        .cloned()
        .unwrap_or_else(|| surfaces[0].clone());
    let locked = active_surface.locked;
    let mut selected = None;
    let mut requested_dock = None;
    let mut hide = false;
    let mut toggle_lock = false;
    let mut drag_started = false;

    modular_surface_frame(theme).show(ui, |ui| {
        ui.horizontal(|ui| {
            let drag = ui
                .add(
                    egui::Label::new(
                        egui::RichText::new(format!("⠿  {}", active_surface.title)).strong(),
                    )
                    .sense(Sense::click_and_drag()),
                )
                .on_hover_text(if locked {
                    "Surface is locked in place"
                } else {
                    "Drag surface to another dock zone"
                });
            if !locked && drag.drag_started_by(PointerButton::Primary) {
                drag_started = true;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if panel_chrome_button(ui, "×", "Hide surface", false, true, theme).clicked() {
                    hide = true;
                }
                if panel_chrome_button(
                    ui,
                    if locked { "▣" } else { "◇" },
                    if locked {
                        "Unlock surface"
                    } else {
                        "Lock surface in place"
                    },
                    locked,
                    false,
                    theme,
                )
                .clicked()
                {
                    toggle_lock = true;
                }
                ui.menu_button("Dock ▾", |ui| {
                    for destination in SurfaceDock::ALL {
                        if ui
                            .selectable_label(destination == dock, destination.label())
                            .clicked()
                        {
                            requested_dock = Some(destination);
                            ui.close();
                        }
                    }
                });
            });
        });
        ui.add_space(3.0);
        selected = show_modular_surface_tabs(ui, &surfaces, &active, theme);
        ui.separator();
        content.surface(&active, ui);
    });

    if drag_started {
        state.surface_drag = Some(active.clone());
        state.surface_drag_preview = Some(dock);
    }
    if let Some(selected) = selected {
        state.set_active_surface(dock, selected.clone());
        response.events.push(ForgeShellEvent::Activated(selected));
    }
    if hide && state.hide_surface(&active) {
        response
            .events
            .push(ForgeShellEvent::Hidden(active.clone()));
    }
    if toggle_lock && state.toggle_surface_lock(&active) {
        response
            .events
            .push(ForgeShellEvent::LockChanged(active.clone()));
    }
    if let Some(destination) = requested_dock {
        if state.move_surface(&active, destination) {
            response.events.push(ForgeShellEvent::Docked {
                surface: active,
                dock: destination,
            });
        }
    }
}

fn show_floating_surfaces<C: ForgeShellContent>(
    ctx: &egui::Context,
    state: &mut ForgeShellState,
    theme: &ForgeTheme,
    content: &mut C,
    response: &mut ForgeShellResponse,
) {
    let floating: Vec<ModularSurfaceState> = state
        .surfaces
        .iter()
        .filter(|surface| surface.visible && surface.dock == SurfaceDock::Floating)
        .cloned()
        .collect();
    for surface in floating {
        let mut open = true;
        let mut requested_dock = None;
        egui::Window::new(&surface.title)
            .id(egui::Id::new(("forge.shell.floating", &surface.id)))
            .open(&mut open)
            .collapsible(false)
            .resizable(!surface.locked)
            .movable(!surface.locked)
            .default_size(egui::vec2(
                surface.preferred_size[0],
                surface.preferred_size[1],
            ))
            .frame(modular_surface_frame(theme))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong(&surface.title);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.menu_button("Dock ▾", |ui| {
                            for dock in [
                                SurfaceDock::Left,
                                SurfaceDock::Center,
                                SurfaceDock::Right,
                                SurfaceDock::Bottom,
                            ] {
                                if ui.button(dock.label()).clicked() {
                                    requested_dock = Some(dock);
                                    ui.close();
                                }
                            }
                        });
                    });
                });
                ui.separator();
                content.surface(&surface.id, ui);
            });
        if !open {
            state.hide_surface(&surface.id);
            response
                .events
                .push(ForgeShellEvent::Hidden(surface.id.clone()));
        }
        if let Some(dock) = requested_dock {
            if state.move_surface(&surface.id, dock) {
                response.events.push(ForgeShellEvent::Docked {
                    surface: surface.id,
                    dock,
                });
            }
        }
    }
}

fn update_surface_drag_drop(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    shell_rect: egui::Rect,
    state: &mut ForgeShellState,
    theme: &ForgeTheme,
    response: &mut ForgeShellResponse,
) {
    let Some(surface_id) = state.surface_drag.clone() else {
        return;
    };
    if state
        .surface(&surface_id)
        .is_some_and(|surface| surface.locked)
    {
        state.surface_drag = None;
        state.surface_drag_preview = None;
        return;
    }

    let pointer = ctx.pointer_hover_pos();
    if let Some(pointer) = pointer {
        let side_width = (shell_rect.width() * 0.20).clamp(96.0, 260.0);
        let bottom_height = (shell_rect.height() * 0.20).clamp(90.0, 200.0);
        let destination = if pointer.x <= shell_rect.left() + side_width {
            SurfaceDock::Left
        } else if pointer.x >= shell_rect.right() - side_width {
            SurfaceDock::Right
        } else if pointer.y >= shell_rect.bottom() - bottom_height {
            SurfaceDock::Bottom
        } else {
            SurfaceDock::Center
        };
        state.surface_drag_preview = Some(destination);
        let preview = match destination {
            SurfaceDock::Left => egui::Rect::from_min_max(
                shell_rect.min,
                egui::pos2(shell_rect.left() + side_width, shell_rect.bottom()),
            ),
            SurfaceDock::Right => egui::Rect::from_min_max(
                egui::pos2(shell_rect.right() - side_width, shell_rect.top()),
                shell_rect.max,
            ),
            SurfaceDock::Bottom => egui::Rect::from_min_max(
                egui::pos2(shell_rect.left(), shell_rect.bottom() - bottom_height),
                shell_rect.max,
            ),
            SurfaceDock::Center | SurfaceDock::Floating => shell_rect.shrink2(egui::vec2(
                side_width.min(shell_rect.width() * 0.18),
                bottom_height.min(shell_rect.height() * 0.14),
            )),
        };
        let accent = theme.base.accent;
        ui.painter().rect_filled(
            preview.shrink(5.0),
            theme.interaction.surface_radius.clamp(0.0, 20.0),
            egui::Color32::from_rgba_unmultiplied(accent.0, accent.1, accent.2, 28),
        );
        ui.painter().rect_stroke(
            preview.shrink(5.0),
            theme.interaction.surface_radius.clamp(0.0, 20.0),
            egui::Stroke::new(2.0, opaque(accent)),
            egui::StrokeKind::Inside,
        );
    }

    if !ctx.input(|input| input.pointer.primary_down()) {
        let destination = if pointer.is_none() {
            SurfaceDock::Floating
        } else {
            state.surface_drag_preview.unwrap_or(SurfaceDock::Center)
        };
        if state.move_surface(&surface_id, destination) {
            response.events.push(ForgeShellEvent::Docked {
                surface: surface_id.clone(),
                dock: destination,
            });
        }
        state.surface_drag = None;
        state.surface_drag_preview = None;
    }
}

fn has_dock(state: &ForgeShellState, dock: SurfaceDock) -> bool {
    state
        .surfaces
        .iter()
        .any(|surface| surface.visible && surface.dock == dock)
}

fn active_preferred(state: &ForgeShellState, dock: SurfaceDock, fallback: f32, axis: usize) -> f32 {
    let active = state.active_surface(dock).unwrap_or_default();
    state
        .surfaces
        .iter()
        .find(|surface| surface.visible && surface.dock == dock && surface.id == active)
        .or_else(|| {
            state
                .surfaces
                .iter()
                .find(|surface| surface.visible && surface.dock == dock)
        })
        .map(|surface| surface.preferred_size[axis])
        .unwrap_or(fallback)
}

fn opaque(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgb(value.0, value.1, value.2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surfaces() -> Vec<ModularSurfaceState> {
        vec![
            ModularSurfaceState::new("nav", "Navigation", SurfaceDock::Left),
            ModularSurfaceState::new("home", "Home", SurfaceDock::Center),
            ModularSurfaceState::new("details", "Details", SurfaceDock::Right),
        ]
    }

    #[test]
    fn shell_registers_and_moves_surfaces() {
        let mut shell = ForgeShellState::new(ShellProfile::Standard, surfaces());
        assert_eq!(shell.active_surface(SurfaceDock::Left), Some("nav"));
        assert!(shell.move_surface("details", SurfaceDock::Bottom));
        assert_eq!(shell.active_surface(SurfaceDock::Bottom), Some("details"));
    }

    #[test]
    fn locked_surface_cannot_move() {
        let mut shell = ForgeShellState::new(ShellProfile::Standard, surfaces());
        shell.toggle_surface_lock("nav");
        assert!(!shell.move_surface("nav", SurfaceDock::Right));
        assert_eq!(shell.surface("nav").unwrap().dock, SurfaceDock::Left);
    }

    #[test]
    fn layout_round_trips() {
        let shell = ForgeShellState::new(ShellProfile::Standard, surfaces());
        let json = shell.save_layout_json().unwrap();
        let restored = ForgeShellState::load_layout_json(&json).unwrap();
        assert_eq!(restored.surfaces, shell.surfaces);
        assert_eq!(restored.profile, shell.profile);
    }
}
