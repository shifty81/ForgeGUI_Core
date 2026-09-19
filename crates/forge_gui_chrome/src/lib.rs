//! Product chrome, creator bars, workspace tabs, tool trays, and status chrome.
#![forbid(unsafe_code)]

use egui::viewport::ResizeDirection;
use egui::{
    Align, Button, CursorIcon, Frame, Id, Layout, Margin, PointerButton, Rect, RichText, Sense,
    Stroke, Ui, Vec2, ViewportCommand,
};
use forge_gui_icons::IconId;
use forge_gui_theme::ForgeTheme;
use forge_gui_widgets::{add_default_icon_font, chrome_tab, panel_tab, tool_button, WidgetTone};
use serde::{Deserialize, Serialize};

/// Renderer-independent, versioned nested tab/split docking model.
pub mod docking;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChromeBarKind {
    Product,
    Menu,
    Action,
    WorkspaceTabs,
    BottomTray,
    Status,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChromeEdge {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EdgeRevealState {
    pub hover_open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowChromeAction {
    Minimize,
    MaximizeRestore,
    Close,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowChromeResponse {
    pub invoked: Option<WindowChromeAction>,
    pub drag_started: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellProfile {
    Minimal,
    ContentFirst,
    #[default]
    Standard,
    Workspace,
    Kiosk,
}

impl ShellProfile {
    pub const ALL: [Self; 5] = [
        Self::Minimal,
        Self::ContentFirst,
        Self::Standard,
        Self::Workspace,
        Self::Kiosk,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Minimal => "Minimal",
            Self::ContentFirst => "Content first",
            Self::Standard => "Standard",
            Self::Workspace => "Workspace",
            Self::Kiosk => "Kiosk",
        }
    }

    pub const fn policy(self) -> ShellPolicy {
        match self {
            Self::Minimal => ShellPolicy {
                menu_bar: true,
                workspace_tabs: false,
                status_bar: false,
                toolbar: false,
                modular_surfaces: false,
            },
            Self::ContentFirst => ShellPolicy {
                menu_bar: true,
                workspace_tabs: true,
                status_bar: false,
                toolbar: false,
                modular_surfaces: false,
            },
            Self::Standard => ShellPolicy {
                menu_bar: true,
                workspace_tabs: true,
                status_bar: true,
                toolbar: true,
                modular_surfaces: true,
            },
            Self::Workspace => ShellPolicy {
                menu_bar: true,
                workspace_tabs: true,
                status_bar: true,
                toolbar: true,
                modular_surfaces: true,
            },
            Self::Kiosk => ShellPolicy {
                menu_bar: false,
                workspace_tabs: false,
                status_bar: false,
                toolbar: false,
                modular_surfaces: false,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellPolicy {
    pub menu_bar: bool,
    pub workspace_tabs: bool,
    pub status_bar: bool,
    pub toolbar: bool,
    pub modular_surfaces: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceDock {
    Left,
    Center,
    #[default]
    Right,
    Bottom,
    Floating,
}

impl SurfaceDock {
    pub const ALL: [Self; 5] = [
        Self::Left,
        Self::Center,
        Self::Right,
        Self::Bottom,
        Self::Floating,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Center => "Center",
            Self::Right => "Right",
            Self::Bottom => "Bottom",
            Self::Floating => "Floating",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModularSurfaceState {
    pub id: String,
    pub title: String,
    pub dock: SurfaceDock,
    /// Remember the destination to restore after closing a detached OS window.
    #[serde(default = "default_surface_last_dock")]
    pub last_dock: SurfaceDock,
    pub visible: bool,
    pub locked: bool,
    pub preferred_size: [f32; 2],
    /// Stable identity of the native floating tab group. Older saved layouts
    /// omit this field and automatically use the panel ID as their host.
    #[serde(default)]
    pub floating_host: Option<String>,
}

impl ModularSurfaceState {
    pub fn new(id: impl Into<String>, title: impl Into<String>, dock: SurfaceDock) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            dock,
            last_dock: if dock == SurfaceDock::Floating {
                SurfaceDock::Center
            } else {
                dock
            },
            visible: true,
            locked: false,
            preferred_size: [320.0, 320.0],
            floating_host: None,
        }
    }

    /// Central transition for dock, detach, and redock operations. A locked surface
    /// may not be moved, but it may still be hidden via the panel manager.
    pub fn move_to(&mut self, destination: SurfaceDock) -> bool {
        if self.locked {
            return false;
        }
        if destination == SurfaceDock::Floating {
            if self.dock != SurfaceDock::Floating {
                self.last_dock = self.dock;
            }
            if self.floating_host.is_none() {
                self.floating_host = Some(self.id.clone());
            }
        } else {
            self.last_dock = destination;
            self.floating_host = None;
        }
        self.dock = destination;
        self.visible = true;
        true
    }

    /// Join an existing floating host without losing panel identity or its
    /// most recent in-shell dock. The host is a window identity, not a panel.
    pub fn join_floating_host(&mut self, host: &str) -> bool {
        if host.is_empty() || !self.move_to(SurfaceDock::Floating) {
            return false;
        }
        self.floating_host = Some(host.to_owned());
        true
    }

    pub fn floating_host_id(&self) -> &str {
        self.floating_host.as_deref().unwrap_or(&self.id)
    }

    /// Closing a native child window returns the same panel instance home.
    pub fn redock(&mut self) -> bool {
        let home = if self.last_dock == SurfaceDock::Floating {
            SurfaceDock::Center
        } else {
            self.last_dock
        };
        self.move_to(home)
    }
}

const fn default_surface_last_dock() -> SurfaceDock {
    SurfaceDock::Center
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolbarDock {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
    Floating,
    Hidden,
}

impl ToolbarDock {
    pub const ALL: [Self; 6] = [
        Self::Top,
        Self::Bottom,
        Self::Left,
        Self::Right,
        Self::Floating,
        Self::Hidden,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Floating => "Floating",
            Self::Hidden => "Hidden",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularToolbarState {
    pub dock: ToolbarDock,
    pub visible: bool,
    pub locked: bool,
}

impl Default for ModularToolbarState {
    fn default() -> Self {
        Self {
            dock: ToolbarDock::Top,
            visible: true,
            locked: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowSnapAction {
    #[default]
    None,
    Maximize,
    LeftHalf,
    RightHalf,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowSnapState {
    pub enabled: bool,
    pub drag_active: bool,
    pub edge_threshold: f32,
    pub last_action: WindowSnapAction,
}

impl Default for WindowSnapState {
    fn default() -> Self {
        Self {
            enabled: true,
            drag_active: false,
            edge_threshold: 10.0,
            last_action: WindowSnapAction::None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceTab {
    pub id: String,
    pub label: String,
    pub icon: Option<IconId>,
    pub dirty: bool,
    pub closable: bool,
}

impl WorkspaceTab {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            dirty: false,
            closable: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolTrayTab {
    pub id: String,
    pub label: String,
    pub badge: Option<String>,
}

impl ToolTrayTab {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            badge: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusItem {
    pub label: String,
    pub value: String,
    pub tone: WidgetTone,
}

/// Surface backgrounds are always opaque even when a theme token has alpha.
/// Alpha-bearing accent/overlay colors remain available through the regular color adapter.
pub fn opaque_background(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgb(value.0, value.1, value.2)
}

/// Restore only registered surfaces with matching default identities.
/// Unrecognized extension surfaces and their state are deliberately preserved.
pub fn restore_surface_layout(
    surfaces: &mut [ModularSurfaceState],
    defaults: &[ModularSurfaceState],
) -> usize {
    let mut restored = 0;
    for surface in surfaces {
        if let Some(default) = defaults.iter().find(|default| default.id == surface.id) {
            surface.dock = default.dock;
            surface.last_dock = default.last_dock;
            surface.visible = default.visible;
            surface.locked = default.locked;
            surface.preferred_size = default.preferred_size;
            surface.floating_host = default.floating_host.clone();
            restored += 1;
        }
    }
    restored
}

/// Clear an ordinary desktop application viewport with an opaque theme-backed color.
/// Hidden, floated or closed panels must never reveal the desktop behind the host.
pub fn opaque_shell_clear_color(theme: &ForgeTheme) -> [f32; 4] {
    let shell = theme.chrome.shell;
    [
        f32::from(shell.0) / 255.0,
        f32::from(shell.1) / 255.0,
        f32::from(shell.2) / 255.0,
        1.0,
    ]
}

pub fn apply_creator_visuals(ctx: &egui::Context, theme: &ForgeTheme) {
    let mut fonts = egui::FontDefinitions::default();
    add_default_icon_font(&mut fonts);
    ctx.set_fonts(fonts);

    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = opaque_background(theme.chrome.shell);
    style.visuals.window_fill = opaque_background(theme.base.panel_raised);
    style.visuals.extreme_bg_color = opaque_background(theme.chrome.canvas);
    style.visuals.selection.bg_fill = color(theme.selected_fill());
    style.visuals.selection.stroke = Stroke::new(
        theme.interaction.focus_width.max(1.0),
        color(theme.interaction_stroke(theme.interaction.selected)),
    );
    style.visuals.widgets.noninteractive.bg_fill = color(theme.base.panel);
    style.visuals.widgets.inactive.bg_fill = color(theme.base.panel);
    style.visuals.widgets.hovered.bg_fill = color(theme.hover_fill());
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(
        theme.interaction.border_width.max(1.0),
        color(theme.interaction_stroke(theme.interaction.hover)),
    );
    style.visuals.widgets.active.bg_fill = color(theme.pressed_fill());
    style.visuals.widgets.active.bg_stroke = Stroke::new(
        theme.interaction.focus_width.max(1.0),
        color(theme.interaction_stroke(theme.interaction.pressed)),
    );
    let control_radius =
        egui::CornerRadius::same(theme.interaction.control_radius.clamp(0.0, 16.0) as u8);
    style.visuals.widgets.noninteractive.corner_radius = control_radius;
    style.visuals.widgets.inactive.corner_radius = control_radius;
    style.visuals.widgets.hovered.corner_radius = control_radius;
    style.visuals.widgets.active.corner_radius = control_radius;
    style.visuals.widgets.open.corner_radius = control_radius;
    style.visuals.window_corner_radius =
        egui::CornerRadius::same(theme.interaction.popup_radius.clamp(0.0, 20.0) as u8);
    style.visuals.menu_corner_radius =
        egui::CornerRadius::same(theme.interaction.popup_radius.clamp(0.0, 20.0) as u8);
    style.visuals.disabled_alpha = theme.interaction.disabled_alpha.clamp(0.1, 1.0);
    style.spacing.item_spacing = egui::vec2(5.0, 4.0);
    style.spacing.button_padding = egui::vec2(7.0, 4.0);
    let mut scroll = egui::style::ScrollStyle::thin();
    scroll.bar_width = theme.base.scrollbar_width.max(7.0);
    scroll.floating_width = (scroll.bar_width * 0.35).max(2.0);
    scroll.handle_min_length = 24.0;
    scroll.dormant_handle_opacity = 0.45;
    scroll.active_handle_opacity = 0.72;
    scroll.interact_handle_opacity = 0.95;
    style.spacing.scroll = scroll;
    ctx.set_style_of(egui::Theme::Dark, style);
}

pub fn show_chrome_bar<R>(
    ui: &mut Ui,
    kind: ChromeBarKind,
    theme: &ForgeTheme,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let fill = match kind {
        ChromeBarKind::Product => theme.chrome.product_bar,
        ChromeBarKind::Menu => theme.chrome.menu_bar,
        ChromeBarKind::Action => theme.chrome.action_bar,
        ChromeBarKind::WorkspaceTabs => theme.chrome.workspace_tabs,
        ChromeBarKind::BottomTray => theme.chrome.bottom_tray,
        ChromeBarKind::Status => theme.chrome.status_bar,
    };

    let radius = theme.interaction.surface_radius.clamp(0.0, u8::MAX as f32) as u8;
    let corner_radius = match kind {
        ChromeBarKind::Product => egui::CornerRadius {
            nw: radius,
            ne: radius,
            sw: radius / 2,
            se: radius / 2,
        },
        ChromeBarKind::Status => egui::CornerRadius {
            nw: radius / 2,
            ne: radius / 2,
            sw: radius,
            se: radius,
        },
        _ => egui::CornerRadius::same((radius / 2).max(2)),
    };

    let vertical_padding = if matches!(kind, ChromeBarKind::Menu) {
        3
    } else {
        2
    };
    let response = Frame::new()
        .fill(opaque_background(fill))
        .corner_radius(corner_radius)
        .inner_margin(Margin::symmetric(8, vertical_padding))
        .show(ui, add_contents);

    let rect = response.response.rect;
    let separator = Stroke::new(1.0, color(theme.chrome.separator));
    match kind {
        ChromeBarKind::Menu => {
            // The application menu is global chrome, so give it a stronger lower edge
            // than canvas-local bars. This prevents File/Edit/View/Help from visually
            // merging into the workspace beneath it.
            ui.painter().line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                Stroke::new(1.5, color(theme.base.border)),
            );
            ui.painter().line_segment(
                [
                    rect.left_bottom() + egui::vec2(0.0, 2.0),
                    rect.right_bottom() + egui::vec2(0.0, 2.0),
                ],
                Stroke::new(1.0, color(theme.chrome.shell)),
            );
        }
        ChromeBarKind::Product | ChromeBarKind::Action | ChromeBarKind::WorkspaceTabs => {
            ui.painter()
                .line_segment([rect.left_bottom(), rect.right_bottom()], separator);
        }
        ChromeBarKind::BottomTray | ChromeBarKind::Status => {
            ui.painter()
                .line_segment([rect.left_top(), rect.right_top()], separator);
        }
    }

    response.inner
}

pub fn pointer_near_edge(
    ctx: &egui::Context,
    rect: Rect,
    edge: ChromeEdge,
    reveal_distance: f32,
) -> bool {
    let Some(pointer) = ctx.pointer_hover_pos() else {
        return false;
    };
    let distance = reveal_distance.max(2.0);
    let strip = match edge {
        ChromeEdge::Left => Rect::from_min_max(
            rect.min,
            egui::pos2((rect.left() + distance).min(rect.right()), rect.bottom()),
        ),
        ChromeEdge::Right => Rect::from_min_max(
            egui::pos2((rect.right() - distance).max(rect.left()), rect.top()),
            rect.max,
        ),
    };
    strip.contains(pointer)
}

pub fn paint_edge_reveal_indicator(ui: &Ui, rect: Rect, edge: ChromeEdge, theme: &ForgeTheme) {
    let x = match edge {
        ChromeEdge::Left => rect.left() + 1.0,
        ChromeEdge::Right => rect.right() - 1.0,
    };
    let center = rect.center().y;
    ui.painter().line_segment(
        [egui::pos2(x, center - 24.0), egui::pos2(x, center + 24.0)],
        Stroke::new(2.0, color(theme.base.accent)),
    );
}

#[allow(clippy::too_many_arguments)]
pub fn show_edge_reveal_surface<R>(
    ctx: &egui::Context,
    available_rect: Rect,
    edge: ChromeEdge,
    id: Id,
    width: f32,
    state: &mut EdgeRevealState,
    theme: &ForgeTheme,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let edge_hot = pointer_near_edge(ctx, available_rect, edge, 8.0);
    if !edge_hot && !state.hover_open {
        state.hover_open = false;
        return None;
    }

    let width = width.clamp(42.0, available_rect.width().max(42.0));
    let pos = match edge {
        ChromeEdge::Left => available_rect.left_top(),
        ChromeEdge::Right => egui::pos2(available_rect.right() - width, available_rect.top()),
    };
    let area = egui::Area::new(id)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            Frame::new()
                .fill(color(theme.chrome.shell))
                .stroke(Stroke::new(1.0, color(theme.chrome.separator)))
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(width, available_rect.height()));
                    ui.set_max_width(width);
                    add_contents(ui)
                })
                .inner
        });

    state.hover_open = edge_hot
        || ctx
            .pointer_hover_pos()
            .map(|pointer| area.response.rect.contains(pointer))
            .unwrap_or(false);
    Some(area.inner)
}

pub fn show_project_title_bar(
    ui: &mut Ui,
    product: &str,
    project: &str,
    branch: Option<&str>,
    theme: &ForgeTheme,
) -> WindowChromeResponse {
    let mut response = WindowChromeResponse::default();
    let maximized = ui
        .ctx()
        .input(|input| input.viewport().maximized.unwrap_or(false));
    let control_width = 40.0;

    show_chrome_bar(ui, ChromeBarKind::Product, theme, |ui| {
        let title_rect = ui.max_rect();

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(forge_gui_widgets::icon_text(IconId::Project))
                    .size(12.0)
                    .color(color(theme.base.accent)),
            );
            ui.label(
                RichText::new(product)
                    .strong()
                    .size(13.0)
                    .color(color(theme.base.text)),
            );
            ui.label(RichText::new("—").color(color(theme.base.text_muted)));
            ui.label(
                RichText::new(project)
                    .size(12.0)
                    .color(color(theme.base.text_muted)),
            );
            if let Some(branch) = branch {
                ui.separator();
                ui.label(
                    RichText::new(branch)
                        .small()
                        .color(color(theme.base.accent)),
                );
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                let close = ui
                    .add(
                        Button::new(
                            RichText::new(forge_gui_widgets::icon_text(IconId::Close))
                                .size(15.0)
                                .color(color(theme.base.text_muted)),
                        )
                        .frame_when_inactive(false)
                        .min_size(Vec2::new(control_width, 26.0)),
                    )
                    .on_hover_text("Close");
                if close.clicked() {
                    response.invoked = Some(WindowChromeAction::Close);
                    ui.send_viewport_cmd(ViewportCommand::Close);
                }

                let maximize_glyph = if maximized {
                    forge_gui_widgets::icon_text(IconId::WindowRestore)
                } else {
                    forge_gui_widgets::icon_text(IconId::WindowMaximize)
                };
                let maximize = ui
                    .add(
                        Button::new(
                            RichText::new(maximize_glyph)
                                .size(12.0)
                                .color(color(theme.base.text_muted)),
                        )
                        .frame_when_inactive(false)
                        .min_size(Vec2::new(control_width, 26.0)),
                    )
                    .on_hover_text(if maximized { "Restore" } else { "Maximize" });
                if maximize.clicked() {
                    response.invoked = Some(WindowChromeAction::MaximizeRestore);
                    ui.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                }

                let minimize = ui
                    .add(
                        Button::new(
                            RichText::new(forge_gui_widgets::icon_text(IconId::WindowMinimize))
                                .size(13.0)
                                .color(color(theme.base.text_muted)),
                        )
                        .frame_when_inactive(false)
                        .min_size(Vec2::new(control_width, 26.0)),
                    )
                    .on_hover_text("Minimize");
                if minimize.clicked() {
                    response.invoked = Some(WindowChromeAction::Minimize);
                    ui.send_viewport_cmd(ViewportCommand::Minimized(true));
                }
            });
        });

        // Keep the title surface draggable while reserving the three native-style
        // control hit targets. Double-click follows normal desktop maximize/restore.
        let drag_rect = Rect::from_min_max(
            title_rect.min,
            egui::pos2(
                (title_rect.max.x - control_width * 3.0).max(title_rect.min.x),
                title_rect.max.y,
            ),
        );
        let drag = ui.interact(
            drag_rect,
            Id::new("forge.project.title.drag"),
            Sense::click_and_drag(),
        );
        if drag.drag_started_by(PointerButton::Primary) {
            response.drag_started = true;
            ui.send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if drag.double_clicked() {
            response.invoked = Some(WindowChromeAction::MaximizeRestore);
            ui.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
        }
    });

    response
}

pub fn update_window_snap(
    ctx: &egui::Context,
    state: &mut WindowSnapState,
) -> Option<WindowSnapAction> {
    if !state.enabled || !state.drag_active {
        return None;
    }

    if ctx.input(|input| input.pointer.primary_down()) {
        return None;
    }

    state.drag_active = false;
    let (outer_rect, monitor_size) =
        ctx.input(|input| (input.viewport().outer_rect, input.viewport().monitor_size));
    let (Some(outer_rect), Some(monitor_size)) = (outer_rect, monitor_size) else {
        return None;
    };

    let threshold = state.edge_threshold.max(4.0);
    let action = if outer_rect.top() <= threshold {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
        WindowSnapAction::Maximize
    } else if outer_rect.left() <= threshold {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(false));
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(egui::pos2(0.0, 0.0)));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(
            (monitor_size.x * 0.5).max(480.0),
            monitor_size.y.max(360.0),
        )));
        WindowSnapAction::LeftHalf
    } else if outer_rect.right() >= monitor_size.x - threshold {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(false));
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(egui::pos2(
            monitor_size.x * 0.5,
            0.0,
        )));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(
            (monitor_size.x * 0.5).max(480.0),
            monitor_size.y.max(360.0),
        )));
        WindowSnapAction::RightHalf
    } else {
        WindowSnapAction::None
    };

    state.last_action = action;
    if matches!(action, WindowSnapAction::None) {
        None
    } else {
        Some(action)
    }
}

/// A detachable panel is a genuine OS window, never an egui::Window confined
/// to the application viewport. The content callback renders the existing panel
/// instance and therefore must not reconstruct a new document or tool state.
/// Close requests redock the same instance; geometry is reported to the layout
/// owner so later detaches restore the user's preferred size.
/// An opening size is emitted only once per continuous native-window lifetime.
#[derive(Clone, Debug, Default)]
pub struct NativeSurfaceLifecycle {
    open_ids: std::collections::HashSet<String>,
}

impl NativeSurfaceLifecycle {
    /// Give the OS the preferred size on creation, then relinquish sizing to it.
    /// Reapplying a measured inner size on every frame can cause resize jitter,
    /// especially when DPI conversion introduces fractional logical points.
    pub fn opening_size(&mut self, id: &str, preferred: [f32; 2]) -> Option<[f32; 2]> {
        self.open_ids.insert(id.to_owned()).then_some([
            preferred[0].clamp(260.0, 4096.0),
            preferred[1].clamp(180.0, 4096.0),
        ])
    }

    /// Forget windows that are no longer drawn. A reopened panel receives its
    /// last recorded preferred size instead of inheriting stale opening state.
    pub fn retain_visible(&mut self, floating: &[ModularSurfaceState]) {
        self.open_ids.retain(|id| {
            floating.iter().any(|surface| {
                surface.visible
                    && surface.dock == SurfaceDock::Floating
                    && surface.id == id.as_str()
            })
        });
    }

    pub fn clear(&mut self) {
        self.open_ids.clear();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeSurfaceResponse {
    pub close_requested: bool,
    pub inner_size: Option<[f32; 2]>,
    pub maximized: bool,
}

pub fn show_native_surface<R>(
    ctx: &egui::Context,
    id: &str,
    title: &str,
    opening_size: Option<[f32; 2]>,
    theme: &ForgeTheme,
    mut contents: impl FnMut(&mut Ui) -> R,
) -> NativeSurfaceResponse {
    let mut response = NativeSurfaceResponse::default();
    let mut builder = egui::ViewportBuilder::default()
        .with_title(title)
        // All panel chrome is painted by ForgeGUI, never duplicated by the OS.
        // Native resizing is restored with explicit border hit targets below.
        .with_decorations(false)
        .with_transparent(false)
        .with_resizable(true)
        .with_min_inner_size([260.0, 180.0]);
    // ViewportBuilder is submitted every frame. InnerSize is ONLY an opening
    // hint: resubmitting measured geometry during a live resize fights the OS.
    if let Some(size) = opening_size {
        if size.iter().all(|dimension| dimension.is_finite()) {
            builder = builder.with_inner_size([size[0].max(260.0), size[1].max(180.0)]);
        }
    }
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of(("forge.native.surface", id)),
        builder,
        |viewport_ui, class| {
            viewport_ui.input(|input| {
                response.close_requested = input.viewport().close_requested();
                response.maximized = input.viewport().maximized.unwrap_or(false);
                response.inner_size = input
                    .viewport()
                    .inner_rect
                    .map(|rect| [rect.width(), rect.height()]);
            });
            // When an integration has no native viewport support, egui embeds
            // this window. The UI stays operable and the fallback is explicit.
            if class == egui::ViewportClass::EmbeddedWindow {
                viewport_ui.label("Embedded panel: native windows unavailable in this host");
            }
            // Leave opaque shell pixels around the rounded interior: painting
            // a square panel over the viewport otherwise erases the radius.
            egui::CentralPanel::default()
                .frame(
                    Frame::new()
                        .fill(opaque_background(theme.chrome.shell))
                        .inner_margin(Margin::same(5)),
                )
                .show(viewport_ui, |ui| {
                    let available = ui.available_size();
                    modular_surface_frame(theme).show(ui, |ui| {
                        let inset =
                            2.0 * (theme.effective_metrics().panel_padding.max(4) as f32 + 4.0);
                        ui.set_min_size(egui::vec2(
                            (available.x - inset).max(0.0),
                            (available.y - inset).max(0.0),
                        ));
                        let _ = contents(ui);
                    });
                });
            // Frameless native viewports need explicit edge/corner resize hit
            // targets; avoid issuing InnerSize during an interactive resize.
            show_viewport_resize_handles(viewport_ui, 6.0);
        },
    );
    response
}

pub fn modular_surface_frame(theme: &ForgeTheme) -> Frame {
    Frame::new()
        .fill(opaque_background(theme.base.panel))
        .stroke(Stroke::new(
            theme.interaction.border_width.max(1.0),
            color(theme.chrome.separator),
        ))
        .corner_radius(theme.interaction.surface_radius.clamp(0.0, 20.0) as u8)
        .outer_margin(Margin::same(4))
        .inner_margin(Margin::same(theme.effective_metrics().panel_padding.max(4)))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SurfaceTabsResponse {
    pub selected: Option<String>,
    pub drag_started: Option<String>,
}

/// Every registered modular surface has a movable tab, not merely a title grip.
pub fn show_draggable_surface_tabs(
    ui: &mut Ui,
    surfaces: &[ModularSurfaceState],
    active: &str,
    theme: &ForgeTheme,
) -> SurfaceTabsResponse {
    let mut result = SurfaceTabsResponse::default();
    ui.horizontal_wrapped(|ui| {
        for surface in surfaces {
            let response = ui.add(
                Button::new(RichText::new(&surface.title).size(12.0))
                    .selected(surface.id == active)
                    .sense(Sense::click_and_drag())
                    .corner_radius(theme.interaction.surface_radius.clamp(0.0, 12.0) as u8),
            );
            if response.clicked() {
                result.selected = Some(surface.id.clone());
            }
            if !surface.locked && response.drag_started_by(PointerButton::Primary) {
                result.drag_started = Some(surface.id.clone());
            }
        }
    });
    result
}

pub fn show_modular_surface_tabs(
    ui: &mut Ui,
    surfaces: &[ModularSurfaceState],
    active: &str,
    theme: &ForgeTheme,
) -> Option<String> {
    let mut selected = None;
    ui.horizontal_wrapped(|ui| {
        for surface in surfaces {
            if panel_tab(ui, &surface.title, surface.id == active, theme).clicked() {
                selected = Some(surface.id.clone());
            }
        }
    });
    selected
}

pub fn show_viewport_resize_handles(ui: &mut Ui, thickness: f32) {
    let rect = ui.max_rect();
    let thickness = thickness.clamp(3.0, 12.0);
    let corner = (thickness * 2.0).max(10.0);

    let north = Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.min.y + thickness));
    let south = Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - thickness), rect.max);
    let west = Rect::from_min_max(rect.min, egui::pos2(rect.min.x + thickness, rect.max.y));
    let east = Rect::from_min_max(egui::pos2(rect.max.x - thickness, rect.min.y), rect.max);

    let northwest = Rect::from_min_size(rect.min, Vec2::splat(corner));
    let northeast = Rect::from_min_max(
        egui::pos2(rect.max.x - corner, rect.min.y),
        egui::pos2(rect.max.x, rect.min.y + corner),
    );
    let southwest = Rect::from_min_max(
        egui::pos2(rect.min.x, rect.max.y - corner),
        egui::pos2(rect.min.x + corner, rect.max.y),
    );
    let southeast = Rect::from_min_max(
        egui::pos2(rect.max.x - corner, rect.max.y - corner),
        rect.max,
    );

    resize_handle(
        ui,
        north,
        "north",
        ResizeDirection::North,
        CursorIcon::ResizeVertical,
    );
    resize_handle(
        ui,
        south,
        "south",
        ResizeDirection::South,
        CursorIcon::ResizeVertical,
    );
    resize_handle(
        ui,
        west,
        "west",
        ResizeDirection::West,
        CursorIcon::ResizeHorizontal,
    );
    resize_handle(
        ui,
        east,
        "east",
        ResizeDirection::East,
        CursorIcon::ResizeHorizontal,
    );

    // Corner targets are registered last so diagonal resizing wins in overlap zones.
    resize_handle(
        ui,
        northwest,
        "northwest",
        ResizeDirection::NorthWest,
        CursorIcon::ResizeNwSe,
    );
    resize_handle(
        ui,
        northeast,
        "northeast",
        ResizeDirection::NorthEast,
        CursorIcon::ResizeNeSw,
    );
    resize_handle(
        ui,
        southwest,
        "southwest",
        ResizeDirection::SouthWest,
        CursorIcon::ResizeNeSw,
    );
    resize_handle(
        ui,
        southeast,
        "southeast",
        ResizeDirection::SouthEast,
        CursorIcon::ResizeNwSe,
    );
}

fn resize_handle(
    ui: &mut Ui,
    rect: Rect,
    id: &str,
    direction: ResizeDirection,
    cursor: CursorIcon,
) {
    let response = ui
        .interact(rect, Id::new(("forge.project.resize", id)), Sense::drag())
        .on_hover_cursor(cursor);
    if response.drag_started_by(PointerButton::Primary) {
        ui.send_viewport_cmd(ViewportCommand::BeginResize(direction));
    }
}

pub fn show_product_identity(
    ui: &mut Ui,
    product: &str,
    project: &str,
    branch: Option<&str>,
    theme: &ForgeTheme,
) {
    show_chrome_bar(ui, ChromeBarKind::Product, theme, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(product)
                    .strong()
                    .size(13.0)
                    .color(color(theme.base.text)),
            );
            ui.label(RichText::new("/").color(color(theme.base.text_muted)));
            ui.label(
                RichText::new(project)
                    .size(12.0)
                    .color(color(theme.base.text_muted)),
            );
            if let Some(branch) = branch {
                ui.separator();
                ui.label(
                    RichText::new(branch)
                        .small()
                        .color(color(theme.base.accent)),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new("CREATOR")
                        .small()
                        .strong()
                        .color(color(theme.base.accent)),
                );
            });
        });
    });
}

pub fn show_action_group(
    ui: &mut Ui,
    actions: &[(String, IconId, String, bool, WidgetTone)],
    theme: &ForgeTheme,
) -> Option<String> {
    let mut invoked = None;
    for (id, icon, label, active, tone) in actions {
        if tool_button(ui, *icon, label, *active, *tone, theme).clicked() {
            invoked = Some(id.clone());
        }
    }
    invoked
}

pub fn show_workspace_tabs(
    ui: &mut Ui,
    tabs: &[WorkspaceTab],
    active: &str,
    theme: &ForgeTheme,
) -> Option<String> {
    let mut selected = None;
    show_chrome_bar(ui, ChromeBarKind::WorkspaceTabs, theme, |ui| {
        ui.horizontal_wrapped(|ui| {
            for tab in tabs {
                let mut label = tab.label.clone();
                if let Some(icon) = tab.icon {
                    label = format!("{}  {label}", forge_gui_widgets::icon_text(icon));
                }
                if chrome_tab(ui, &label, tab.id == active, tab.dirty, theme).clicked() {
                    selected = Some(tab.id.clone());
                }
            }
        });
    });
    selected
}

pub fn show_tool_tray_tabs(
    ui: &mut Ui,
    tabs: &[ToolTrayTab],
    active: &str,
    theme: &ForgeTheme,
) -> Option<String> {
    let mut selected = None;
    show_chrome_bar(ui, ChromeBarKind::BottomTray, theme, |ui| {
        ui.horizontal(|ui| {
            for tab in tabs {
                let mut label = tab.label.clone();
                if let Some(badge) = tab.badge.as_deref() {
                    label.push_str(&format!("  {badge}"));
                }
                if panel_tab(ui, &label, tab.id == active, theme).clicked() {
                    selected = Some(tab.id.clone());
                }
            }
        });
    });
    selected
}

pub fn show_status_items(ui: &mut Ui, items: &[StatusItem], theme: &ForgeTheme) {
    show_chrome_bar(ui, ChromeBarKind::Status, theme, |ui| {
        ui.horizontal(|ui| {
            for (index, item) in items.iter().enumerate() {
                let tone = match item.tone {
                    WidgetTone::Neutral => theme.base.text_muted,
                    WidgetTone::Accent => theme.base.accent,
                    WidgetTone::Success => theme.base.success,
                    WidgetTone::Warning => theme.base.warning,
                    WidgetTone::Danger => theme.base.danger,
                };
                ui.label(
                    RichText::new(format!("{}  {}", item.label, item.value))
                        .size(11.5)
                        .color(color(tone)),
                );
                if index + 1 != items.len() {
                    ui.label(RichText::new("│").color(color(theme.chrome.separator)));
                }
            }
        });
    });
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}

#[cfg(test)]
mod opaque_shell_tests {
    use super::*;

    #[test]
    fn viewport_clear_ignores_theme_alpha() {
        let mut theme = ForgeTheme::default();
        theme.chrome.shell = forge_gui_core::Rgba(20, 40, 60, 0);
        assert_eq!(
            opaque_shell_clear_color(&theme),
            [20.0 / 255.0, 40.0 / 255.0, 60.0 / 255.0, 1.0]
        );
        assert_eq!(opaque_background(theme.chrome.shell).a(), 255);
    }

    #[test]
    fn all_preset_shell_clears_are_opaque() {
        for preset in forge_gui_theme::ForgeThemePreset::ALL {
            let theme = ForgeTheme::from_preset(preset);
            assert_eq!(opaque_shell_clear_color(&theme)[3], 1.0);
            assert_eq!(opaque_background(theme.base.panel).a(), 255);
        }
    }

    #[test]
    fn native_surface_initial_size_is_one_shot_and_resets_on_close() {
        let mut lifetime = NativeSurfaceLifecycle::default();
        assert_eq!(
            lifetime.opening_size("assets", [410.0, 580.0]),
            Some([410.0, 580.0])
        );
        assert_eq!(lifetime.opening_size("assets", [560.0, 620.0]), None);
        assert_eq!(
            lifetime.opening_size("inspector", [320.0, 420.0]),
            Some([320.0, 420.0])
        );
        let visible = vec![ModularSurfaceState::new(
            "inspector",
            "Inspector",
            SurfaceDock::Floating,
        )];
        lifetime.retain_visible(&visible);
        assert_eq!(
            lifetime.opening_size("assets", [560.0, 620.0]),
            Some([560.0, 620.0])
        );
        assert_eq!(lifetime.opening_size("inspector", [600.0, 800.0]), None);
        lifetime.clear();
        assert_eq!(
            lifetime.opening_size("inspector", [600.0, 800.0]),
            Some([600.0, 800.0])
        );
    }

    #[test]
    fn native_surface_opening_size_is_bounded() {
        let mut lifetime = NativeSurfaceLifecycle::default();
        assert_eq!(
            lifetime.opening_size("small", [50.0, 40.0]),
            Some([260.0, 180.0])
        );
        assert_eq!(
            lifetime.opening_size("large", [9000.0, 9000.0]),
            Some([4096.0, 4096.0])
        );
    }

    #[test]
    fn floating_group_preserves_each_panels_original_dock() {
        let mut activity = ModularSurfaceState::new("activity", "Activity", SurfaceDock::Bottom);
        let mut assets = ModularSurfaceState::new("assets", "Assets", SurfaceDock::Right);
        assert!(activity.move_to(SurfaceDock::Floating));
        assert!(assets.join_floating_host(activity.floating_host_id()));
        assert_eq!(assets.floating_host_id(), "activity");
        assert_eq!(activity.last_dock, SurfaceDock::Bottom);
        assert_eq!(assets.last_dock, SurfaceDock::Right);
        assert!(assets.redock());
        assert_eq!(assets.dock, SurfaceDock::Right);
        assert_eq!(assets.floating_host, None);
        assert_eq!(activity.floating_host_id(), "activity");
    }

    #[test]
    fn locked_panel_cannot_be_joined_into_a_floating_host() {
        let mut state = ModularSurfaceState::new("locked", "Locked", SurfaceDock::Left);
        state.locked = true;
        assert!(!state.join_floating_host("another"));
        assert_eq!(state.dock, SurfaceDock::Left);
    }

    #[test]
    fn legacy_floating_layout_has_a_stable_default_window_identity() {
        let mut state = ModularSurfaceState::new("legacy", "Legacy", SurfaceDock::Floating);
        assert_eq!(state.floating_host_id(), "legacy");
        assert!(state.join_floating_host("shared"));
        assert_eq!(state.floating_host_id(), "shared");
    }

    #[test]
    fn native_detachment_preserves_identity_and_redocks_home() {
        let mut surface = ModularSurfaceState::new("asset", "Assets", SurfaceDock::Right);
        surface.preferred_size = [410.0, 580.0];
        assert!(surface.move_to(SurfaceDock::Floating));
        assert_eq!(surface.last_dock, SurfaceDock::Right);
        assert_eq!(surface.id, "asset");
        assert!(surface.redock());
        assert_eq!(surface.dock, SurfaceDock::Right);
        assert_eq!(surface.preferred_size, [410.0, 580.0]);
        assert!(surface.move_to(SurfaceDock::Left));
        assert!(surface.move_to(SurfaceDock::Floating));
        assert!(surface.redock());
        assert_eq!(surface.dock, SurfaceDock::Left);
        surface.locked = true;
        assert!(!surface.move_to(SurfaceDock::Floating));
        assert_eq!(surface.dock, SurfaceDock::Left);
    }

    #[test]
    fn restore_surface_layout_keeps_extension_surfaces() {
        let baseline = vec![
            ModularSurfaceState::new("content", "Content", SurfaceDock::Left),
            ModularSurfaceState::new("details", "Details", SurfaceDock::Right),
        ];
        let mut current = baseline.clone();
        current[0].dock = SurfaceDock::Floating;
        current[0].locked = true;
        current[0].visible = false;
        current[1].dock = SurfaceDock::Bottom;
        let mut extension = ModularSurfaceState::new("extension", "Extension", SurfaceDock::Center);
        extension.locked = true;
        current.push(extension.clone());
        assert_eq!(restore_surface_layout(&mut current, &baseline), 2);
        assert_eq!(&current[..2], baseline.as_slice());
        assert_eq!(current[2], extension);
    }
}
