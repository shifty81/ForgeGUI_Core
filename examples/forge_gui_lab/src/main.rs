#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use forge_gui_browser::{show_browser, BrowserItem, BrowserItemKind, BrowserModel};
use forge_gui_chrome::{
    apply_creator_visuals, modular_surface_frame, paint_edge_reveal_indicator, pointer_near_edge,
    restore_surface_layout, show_chrome_bar, show_draggable_surface_tabs, show_native_surface,
    show_project_title_bar, show_status_items, show_tool_tray_tabs, show_viewport_resize_handles,
    show_workspace_tabs, update_window_snap, ChromeBarKind, ChromeEdge, ModularSurfaceState,
    ModularToolbarState, NativeSurfaceLifecycle, ShellProfile, StatusItem, SurfaceDock,
    ToolTrayTab, ToolbarDock, WindowSnapState, WorkspaceTab,
};
use forge_gui_command::{show_command_palette, CommandPalette, PaletteEntry};
use forge_gui_core::{PropertyField, PropertyObject, PropertyValue};
use forge_gui_icons::IconId;
use forge_gui_inspector::show_property_object;
use forge_gui_layout::ForgeLayoutState;
use forge_gui_panels::{show_panel, show_panel_tabs, PanelKind};
use forge_gui_rails::{
    show_rail, LabelPolicy, RailDensity, RailEdge, RailItem, RailModel, RailPlacement, RailPreset,
    RailSection, RailStyle,
};
use forge_gui_theme::{ForgeDensity, ForgeTheme, ForgeThemePreset, HighlightMode};
use forge_gui_universal_egui::{show_universal_suite, UniversalSuiteState, UniversalTab};
use forge_gui_widgets::{
    compact_tool_button, gauge, icon_text, panel_chrome_button, progress_bar, scrollbar,
    ScrollbarAxis, ScrollbarState, WidgetTone,
};
use forge_gui_workspace::{
    canvas_family_menu, canvas_options_menu, show_renderer_workspace, CanvasChromeState,
    WorkspaceTool, WorkspaceViewState,
};
use forge_render_core::{AuthoringCamera, NullRenderBackend, RenderRequest};
use forge_render_surface::{RenderFamily, RenderSurfaceDescriptor, RenderSurfaceHost};
use forge_scene_core::{ForgeScene, SceneEntity};

const SURFACE_STORAGE_KEY: &str = "forgegui.core.modular_surfaces.v1";

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ForgeGUI Core — GUI Lab")
            .with_decorations(false)
            .with_transparent(false)
            .with_close_button(false)
            .with_minimize_button(false)
            .with_maximize_button(false)
            .with_resizable(true)
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ForgeGUI Core",
        native_options,
        Box::new(|cc| Ok(Box::new(CreatorStudioLab::new(cc)))),
    )
}

struct CreatorStudioLab {
    theme: ForgeTheme,
    theme_preset: ForgeThemePreset,
    shell_profile: ShellProfile,
    show_menu_bar: bool,
    show_modular_surfaces: bool,
    modular_surfaces: Vec<ModularSurfaceState>,
    native_surface_lifecycle: NativeSurfaceLifecycle,
    toolbar_state: ModularToolbarState,
    window_snap: WindowSnapState,
    surface_drag: Option<String>,
    surface_drag_preview: Option<SurfaceDock>,
    toolbar_dragging: bool,
    toolbar_drag_preview: Option<ToolbarDock>,
    active_left_surface: String,
    active_center_surface: String,
    active_right_surface: String,
    active_bottom_surface: String,
    layout: ForgeLayoutState,
    workspace_tabs: Vec<WorkspaceTab>,
    show_workspace_tabs: bool,
    tool_tabs: Vec<ToolTrayTab>,
    active_tool_tab: String,
    browser: BrowserModel,
    inspector_object: PropertyObject,
    universal_suite: UniversalSuiteState,
    right_tab: String,
    palette: CommandPalette,
    workspace_view: WorkspaceViewState,
    canvas_chrome: CanvasChromeState,
    universal_rail: RailModel,
    render_surface: RenderSurfaceHost,
    scene: ForgeScene,
    last_action: String,
    runtime_playing: bool,
    left_rail_pinned: bool,
    right_panel_pinned: bool,
    left_overlay_open: bool,
    right_overlay_open: bool,
    demo_progress: f32,
    demo_gauge: f32,
    h_scroll: ScrollbarState,
    v_scroll: ScrollbarState,
}

impl CreatorStudioLab {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme_preset = ForgeThemePreset::MidnightMint;
        let theme = ForgeTheme::from_preset(theme_preset);
        apply_creator_visuals(&cc.egui_ctx, &theme); // Includes the Phosphor font.
        let surfaces = restored_surfaces(
            modular_surfaces(),
            cc.storage
                .and_then(|storage| {
                    eframe::get_value::<Vec<ModularSurfaceState>>(storage, SURFACE_STORAGE_KEY)
                })
                .unwrap_or_default(),
        );

        let mut scene = ForgeScene::new("creator.lab.scene", "Creator Studio Demo");
        scene
            .insert_root(SceneEntity::new("entity.world", "World Root"))
            .expect("demo world root is valid");
        scene
            .insert_root(SceneEntity::new("entity.player", "Player"))
            .expect("demo player is valid");

        let mut layout = ForgeLayoutState::default();
        layout.structural.left_visible = false;
        layout.structural.bottom_visible = false;
        layout.structural.right_visible = true;
        layout.structural.right_width = 330.0;
        layout.active_workspace = "workspace.application".into();
        layout.workspaces = vec![
            "workspace.application".into(),
            "workspace.dashboard".into(),
            "workspace.canvas".into(),
        ];

        Self {
            theme,
            theme_preset,
            shell_profile: ShellProfile::Standard,
            show_menu_bar: true,
            show_modular_surfaces: true,
            modular_surfaces: surfaces,
            native_surface_lifecycle: NativeSurfaceLifecycle::default(),
            toolbar_state: ModularToolbarState::default(),
            window_snap: WindowSnapState::default(),
            surface_drag: None,
            surface_drag_preview: None,
            toolbar_dragging: false,
            toolbar_drag_preview: None,
            active_left_surface: "surface.content".into(),
            active_center_surface: "surface.home".into(),
            active_right_surface: "surface.universal".into(),
            active_bottom_surface: "surface.activity".into(),
            layout,
            workspace_tabs: workspace_tabs(),
            show_workspace_tabs: true,
            tool_tabs: tool_tabs(),
            active_tool_tab: "tool.console".into(),
            browser: browser_model(),
            inspector_object: inspector_object(),
            universal_suite: UniversalSuiteState::default(),
            right_tab: "inspector".into(),
            palette: command_palette(),
            workspace_view: WorkspaceViewState {
                selection_label: "Player".into(),
                camera_label: "Hybrid Camera".into(),
                ..Default::default()
            },
            canvas_chrome: CanvasChromeState::default(),
            universal_rail: universal_tool_rail(),
            render_surface: RenderSurfaceHost::new(
                RenderSurfaceDescriptor::new(
                    "surface.creator.main",
                    "Main Game View",
                    RenderFamily::Hybrid,
                ),
                NullRenderBackend::default(),
            ),
            scene,
            last_action: "Universal modular ForgeGUI shell ready".into(),
            runtime_playing: false,
            left_rail_pinned: true,
            right_panel_pinned: true,
            left_overlay_open: false,
            right_overlay_open: false,
            demo_progress: 0.68,
            demo_gauge: 72.0,
            h_scroll: ScrollbarState {
                position: 0.35,
                viewport_fraction: 0.28,
            },
            v_scroll: ScrollbarState {
                position: 0.22,
                viewport_fraction: 0.32,
            },
        }
    }

    fn dispatch_command(&mut self, id: &str) {
        match id {
            "forge.command.view.widgets" => {
                self.open_surface("surface.widgets");
                self.last_action = "Opened Widget Gallery".into();
            }
            "forge.command.view.universal" => {
                self.universal_suite.tab = UniversalTab::Overview;
                self.open_surface("surface.universal");
                self.last_action = "Opened Universal Application Suite".into();
            }
            "forge.command.run.play" => {
                self.runtime_playing = true;
                self.last_action = "Run > Play".into();
            }
            "forge.command.run.stop" => {
                self.runtime_playing = false;
                self.last_action = "Run > Stop".into();
            }
            "forge.command.file.new" => {
                self.last_action = "New unavailable: attach a host document service".into();
            }
            "forge.command.document.save" => {
                self.last_action = "Save unavailable: attach a host document service".into();
            }
            _ => self.last_action = format!("Command: {id}"),
        }
    }

    fn run_renderer(&mut self) {
        let request = RenderRequest {
            dimension: self.render_surface.descriptor().family.base_dimension(),
            extent: self.render_surface.extent(),
            camera: AuthoringCamera::default(),
            selected: Vec::new(),
            hovered: None,
            show_grid: self.workspace_view.show_grid,
            show_gizmos: true,
            show_editor_overlays: self.workspace_view.show_overlays,
        };

        if let Err(error) = self.render_surface.render(&self.scene, &request) {
            self.last_action = format!("Renderer: {error}");
        }
    }

    fn apply_theme(&mut self, ctx: &egui::Context, preset: ForgeThemePreset) {
        self.theme_preset = preset;
        self.theme = ForgeTheme::from_preset(preset);
        apply_creator_visuals(ctx, &self.theme);
        self.last_action = format!("Theme: {}", preset.label());
    }

    fn set_renderer_family(&mut self, family: RenderFamily) {
        self.workspace_view.camera_label = match family {
            RenderFamily::TwoD => "Orthographic 2D".into(),
            RenderFamily::TwoPointFiveD => "Layered Orthographic".into(),
            RenderFamily::ThreeD => "Perspective 3D".into(),
            RenderFamily::Voxel => "Voxel Perspective".into(),
            RenderFamily::Hybrid => "Hybrid Camera".into(),
        };
        self.last_action = format!("Renderer family: {}", family.label());
    }

    fn handle_rail_action(&mut self, action: &str) {
        match action {
            "tool.select" => self.workspace_view.active_tool = WorkspaceTool::Select,
            "tool.move" => self.workspace_view.active_tool = WorkspaceTool::Move,
            "tool.rotate" => self.workspace_view.active_tool = WorkspaceTool::Rotate,
            "tool.scale" => self.workspace_view.active_tool = WorkspaceTool::Scale,
            "tool.pan" => self.workspace_view.active_tool = WorkspaceTool::Pan,
            "tool.zoom" => self.workspace_view.active_tool = WorkspaceTool::Zoom,
            "panel.inspector" => {
                self.right_tab = "inspector".into();
                self.right_panel_pinned = true;
            }
            "panel.assets" => {
                self.right_tab = "assets".into();
                self.right_panel_pinned = true;
            }
            "panel.widgets" => {
                self.right_tab = "widgets".into();
                self.right_panel_pinned = true;
            }
            "panel.universal" => {
                self.right_tab = "universal".into();
                self.right_panel_pinned = true;
            }
            "run.play" => self.runtime_playing = true,
            "run.stop" => self.runtime_playing = false,
            _ => {}
        }
        self.universal_rail.set_active(action.to_owned());
        self.last_action = action.to_owned();
    }

    fn show_universal_rail(&mut self, ui: &mut egui::Ui) {
        let response = show_rail(ui, &mut self.universal_rail, &self.theme);
        if let Some(action) = response.activated {
            self.handle_rail_action(&action);
        }
    }

    fn apply_shell_profile(&mut self, profile: ShellProfile) {
        self.shell_profile = profile;
        let policy = profile.policy();
        self.show_menu_bar = policy.menu_bar;
        self.show_workspace_tabs = policy.workspace_tabs;
        self.layout.structural.status_visible = policy.status_bar;
        self.toolbar_state.visible =
            policy.toolbar && !matches!(self.toolbar_state.dock, ToolbarDock::Hidden);
        self.show_modular_surfaces = policy.modular_surfaces;
        self.last_action = format!("Shell profile: {}", profile.label());
    }

    /// Recover a predictable, complete shell without discarding extension registrations
    /// or the consumer's data models. Also clears transient drag/drop state.
    fn restore_standard_layout(&mut self) {
        self.apply_shell_profile(ShellProfile::Standard);
        let restored = restore_surface_layout(&mut self.modular_surfaces, &modular_surfaces());
        self.toolbar_state = ModularToolbarState::default();
        self.surface_drag = None;
        self.surface_drag_preview = None;
        self.toolbar_dragging = false;
        self.toolbar_drag_preview = None;
        self.active_left_surface = "surface.content".into();
        self.active_center_surface = "surface.home".into();
        self.active_right_surface = "surface.universal".into();
        self.active_bottom_surface = "surface.activity".into();
        self.layout.active_workspace = "workspace.application".into();
        self.show_menu_bar = true;
        self.show_workspace_tabs = true;
        self.show_modular_surfaces = true;
        self.layout.structural.status_visible = true;
        self.last_action = format!("Restored standard layout ({restored} core surfaces)");
    }

    fn surface_visible(&self, id: &str) -> bool {
        self.modular_surfaces
            .iter()
            .find(|surface| surface.id == id)
            .is_some_and(|surface| surface.visible)
    }

    fn open_surface(&mut self, id: &str) {
        let Some(surface) = self
            .modular_surfaces
            .iter_mut()
            .find(|surface| surface.id == id)
        else {
            return;
        };
        surface.visible = true;
        match surface.dock {
            SurfaceDock::Left => self.active_left_surface = surface.id.clone(),
            SurfaceDock::Center => self.active_center_surface = surface.id.clone(),
            SurfaceDock::Right => self.active_right_surface = surface.id.clone(),
            SurfaceDock::Bottom => self.active_bottom_surface = surface.id.clone(),
            SurfaceDock::Floating => {}
        }
        self.last_action = format!("Opened surface: {}", surface.title);
    }

    fn ensure_active_surface(
        &mut self,
        dock: SurfaceDock,
        surfaces: &[ModularSurfaceState],
    ) -> String {
        let current = match dock {
            SurfaceDock::Left => self.active_left_surface.clone(),
            SurfaceDock::Center => self.active_center_surface.clone(),
            SurfaceDock::Right => self.active_right_surface.clone(),
            SurfaceDock::Bottom => self.active_bottom_surface.clone(),
            SurfaceDock::Floating => String::new(),
        };
        if surfaces.iter().any(|surface| surface.id == current) {
            return current;
        }
        let fallback = surfaces
            .first()
            .map(|surface| surface.id.clone())
            .unwrap_or_default();
        match dock {
            SurfaceDock::Left => self.active_left_surface = fallback.clone(),
            SurfaceDock::Center => self.active_center_surface = fallback.clone(),
            SurfaceDock::Right => self.active_right_surface = fallback.clone(),
            SurfaceDock::Bottom => self.active_bottom_surface = fallback.clone(),
            SurfaceDock::Floating => {}
        }
        fallback
    }

    fn set_active_surface(&mut self, dock: SurfaceDock, id: String) {
        match dock {
            SurfaceDock::Left => self.active_left_surface = id,
            SurfaceDock::Center => self.active_center_surface = id,
            SurfaceDock::Right => self.active_right_surface = id,
            SurfaceDock::Bottom => self.active_bottom_surface = id,
            SurfaceDock::Floating => {}
        }
    }

    fn dock_group_locked(&self, dock: SurfaceDock) -> bool {
        let active = match dock {
            SurfaceDock::Left => self.active_left_surface.as_str(),
            SurfaceDock::Center => self.active_center_surface.as_str(),
            SurfaceDock::Right => self.active_right_surface.as_str(),
            SurfaceDock::Bottom => self.active_bottom_surface.as_str(),
            SurfaceDock::Floating => return false,
        };
        self.modular_surfaces
            .iter()
            .find(|surface| surface.visible && surface.dock == dock && surface.id == active)
            .or_else(|| {
                self.modular_surfaces
                    .iter()
                    .find(|surface| surface.visible && surface.dock == dock)
            })
            .is_some_and(|surface| surface.locked)
    }

    fn show_surface_content(&mut self, ui: &mut egui::Ui, id: &str) {
        let theme = self.theme.clone();
        match id {
            "surface.home" => self.show_application_workspace(ui),
            "surface.content" => {
                let response = show_browser(ui, &mut self.browser, &theme);
                if let Some(item) = response.activated {
                    self.last_action = format!("Open item: {item}");
                } else if response.selection_changed {
                    self.last_action = "Content selection changed".into();
                }
            }
            "surface.properties" => {
                let inspector = show_property_object(ui, &mut self.inspector_object, &theme);
                if let Some(change) = inspector.changes.last() {
                    self.last_action =
                        format!("Changed {}.{}", change.object_id, change.property_id);
                }
            }
            "surface.widgets" => self.show_widget_gallery(ui),
            "surface.universal" => {
                let response = show_universal_suite(ui, &mut self.universal_suite, &theme);
                if let Some(action) = response.last_action {
                    self.last_action = action;
                }
            }
            "surface.activity" => {
                show_bottom_tool(ui, &self.active_tool_tab, &self.last_action, &theme);
            }
            _ => {
                ui.label("Consumer-provided modular surface slot.");
            }
        }
    }

    fn show_dock_group(&mut self, ui: &mut egui::Ui, dock: SurfaceDock) {
        let surfaces: Vec<ModularSurfaceState> = self
            .modular_surfaces
            .iter()
            .filter(|surface| surface.visible && surface.dock == dock)
            .cloned()
            .collect();
        if surfaces.is_empty() {
            return;
        }

        let active = self.ensure_active_surface(dock, &surfaces);
        let active_surface = surfaces
            .iter()
            .find(|surface| surface.id == active)
            .cloned();
        let title = active_surface
            .as_ref()
            .map(|surface| surface.title.as_str())
            .unwrap_or("Surface");
        let locked = active_surface
            .as_ref()
            .is_some_and(|surface| surface.locked);
        let theme = self.theme.clone();
        let mut selected = None;
        let mut requested_dock = None;
        let mut toggle_lock = false;
        let mut hide_active = false;
        let mut drag_started = false;
        let mut tab_drag_started: Option<String> = None;

        modular_surface_frame(&theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                let drag = ui
                    .add(
                        egui::Label::new(egui::RichText::new(format!("⠿  {title}")).strong())
                            .sense(egui::Sense::click_and_drag()),
                    )
                    .on_hover_text(if locked {
                        "Surface is locked in place"
                    } else {
                        "Drag surface to dock left, center, right, or bottom"
                    });
                if !locked && drag.drag_started_by(egui::PointerButton::Primary) {
                    drag_started = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    if panel_chrome_button(
                        ui,
                        icon_text(IconId::Close),
                        "Hide surface",
                        false,
                        true,
                        &theme,
                    )
                    .clicked()
                    {
                        hide_active = true;
                    }
                    if panel_chrome_button(
                        ui,
                        if locked {
                            icon_text(IconId::PanelDock)
                        } else {
                            icon_text(IconId::PanelDetach)
                        },
                        if locked {
                            "Unlock surface"
                        } else {
                            "Lock surface in place"
                        },
                        locked,
                        false,
                        &theme,
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
            let tabs = show_draggable_surface_tabs(ui, &surfaces, &active, &theme);
            selected = tabs.selected;
            tab_drag_started = tabs.drag_started;
            ui.separator();
            self.show_surface_content(ui, &active);
        });

        if drag_started || tab_drag_started.is_some() {
            let dragged = tab_drag_started.unwrap_or_else(|| active.clone());
            self.set_active_surface(dock, dragged.clone());
            self.surface_drag = Some(dragged.clone());
            self.surface_drag_preview = Some(dock);
            self.last_action = format!("Dragging surface: {dragged}");
        }

        if let Some(selected) = selected {
            self.set_active_surface(dock, selected.clone());
            self.last_action = format!("Active surface: {selected}");
        }
        if hide_active {
            if let Some(surface) = self
                .modular_surfaces
                .iter_mut()
                .find(|surface| surface.id == active)
            {
                surface.visible = false;
            }
        }
        if toggle_lock {
            if let Some(surface) = self
                .modular_surfaces
                .iter_mut()
                .find(|surface| surface.id == active)
            {
                surface.locked = !surface.locked;
            }
        }
        if let Some(destination) = requested_dock {
            if !locked || destination == dock {
                if let Some(surface) = self
                    .modular_surfaces
                    .iter_mut()
                    .find(|surface| surface.id == active)
                {
                    let _ = surface.move_to(destination);
                }
                self.set_active_surface(destination, active.clone());
                self.last_action = format!("Docked {active} → {}", destination.label());
            }
        }
    }

    fn show_floating_surfaces(&mut self, ctx: &egui::Context) {
        let floating: Vec<ModularSurfaceState> = self
            .modular_surfaces
            .iter()
            .filter(|surface| surface.visible && surface.dock == SurfaceDock::Floating)
            .cloned()
            .collect();
        self.native_surface_lifecycle.retain_visible(&floating);

        for surface in floating {
            let opening_size = self
                .native_surface_lifecycle
                .opening_size(&surface.id, surface.preferred_size);
            let mut requested_dock = None;
            let mut hide_requested = false;
            let mut locked = surface.locked;
            let theme = self.theme.clone();
            // Native OS window: independent minimize/maximize, taskbar presence,
            // monitor movement and resizing are handled by the window backend.
            let native_response = show_native_surface(
                ctx,
                &surface.id,
                &surface.title,
                opening_size,
                &theme,
                |ui| {
                    ui.horizontal(|ui| {
                        if panel_chrome_button(
                            ui,
                            if locked {
                                icon_text(IconId::PanelDock)
                            } else {
                                icon_text(IconId::PanelDetach)
                            },
                            if locked {
                                "Unlock surface"
                            } else {
                                "Lock surface"
                            },
                            locked,
                            false,
                            &theme,
                        )
                        .clicked()
                        {
                            locked = !locked;
                        }
                        ui.add_enabled_ui(!locked, |ui| {
                            ui.menu_button(
                                format!("{} Dock", icon_text(IconId::PanelDock)),
                                |ui| {
                                    for dock in SurfaceDock::ALL {
                                        if dock != SurfaceDock::Floating
                                            && ui.button(dock.label()).clicked()
                                        {
                                            requested_dock = Some(dock);
                                            ui.close();
                                        }
                                    }
                                },
                            );
                        });
                        if ui
                            .button(format!("{} Hide", icon_text(IconId::Close)))
                            .clicked()
                        {
                            hide_requested = true;
                        }
                    });
                    ui.separator();
                    self.show_surface_content(ui, &surface.id);
                },
            );
            if let Some(current) = self
                .modular_surfaces
                .iter_mut()
                .find(|item| item.id == surface.id)
            {
                current.locked = locked;
                // Size is observed only; it must never be fed back to an open
                // viewport's builder. Keep the last restored size when maximized.
                if !native_response.maximized {
                    if let Some(size) = native_response.inner_size {
                        if size[0].is_finite()
                            && size[1].is_finite()
                            && size[0] >= 260.0
                            && size[1] >= 180.0
                        {
                            current.preferred_size =
                                [size[0].clamp(260.0, 4096.0), size[1].clamp(180.0, 4096.0)];
                        }
                    }
                }
                if hide_requested {
                    current.visible = false;
                } else if let Some(dock) = requested_dock {
                    let _ = current.move_to(dock);
                } else if native_response.close_requested {
                    // The window close control is not a document-close request.
                    // Return the unchanged panel to its original dock.
                    current.locked = false;
                    let _ = current.redock();
                    current.locked = locked;
                    requested_dock = Some(current.dock);
                }
            }
            if let Some(dock) = requested_dock {
                self.set_active_surface(dock, surface.id.clone());
                self.last_action = format!("Redocked {} → {}", surface.title, dock.label());
            }
        }
    }

    fn show_modular_toolbar_buttons(&mut self, ui: &mut egui::Ui) {
        if compact_tool_button(
            ui,
            IconId::FolderOpen,
            "Content surface",
            self.surface_visible("surface.content"),
            &self.theme,
        )
        .clicked()
        {
            self.open_surface("surface.content");
        }
        if compact_tool_button(
            ui,
            IconId::Inspector,
            "Properties surface",
            self.surface_visible("surface.properties"),
            &self.theme,
        )
        .clicked()
        {
            self.open_surface("surface.properties");
        }
        if compact_tool_button(
            ui,
            IconId::Graph,
            "Universal application suite",
            self.surface_visible("surface.universal"),
            &self.theme,
        )
        .clicked()
        {
            self.open_surface("surface.universal");
        }
        if compact_tool_button(
            ui,
            IconId::Settings,
            "Widget gallery",
            self.surface_visible("surface.widgets"),
            &self.theme,
        )
        .clicked()
        {
            self.open_surface("surface.widgets");
        }
        if compact_tool_button(ui, IconId::Search, "Command palette", false, &self.theme).clicked()
        {
            self.palette.open();
        }
        if compact_tool_button(
            ui,
            if self.runtime_playing {
                IconId::Stop
            } else {
                IconId::Play
            },
            if self.runtime_playing { "Stop" } else { "Run" },
            self.runtime_playing,
            &self.theme,
        )
        .clicked()
        {
            self.runtime_playing = !self.runtime_playing;
        }
    }

    fn show_modular_toolbar(&mut self, ui: &mut egui::Ui, horizontal: bool) {
        let theme = self.theme.clone();
        let locked = self.toolbar_state.locked;
        let mut drag_started = false;
        modular_surface_frame(&theme).show(ui, |ui| {
            let mut show_grip = |ui: &mut egui::Ui| {
                let grip = ui
                    .add(egui::Label::new("⠿").sense(egui::Sense::click_and_drag()))
                    .on_hover_text(if locked {
                        "Toolbar is locked in place"
                    } else {
                        "Drag toolbar to dock or float"
                    });
                if !locked && grip.drag_started_by(egui::PointerButton::Primary) {
                    drag_started = true;
                }
            };

            if horizontal {
                ui.horizontal(|ui| {
                    show_grip(ui);
                    ui.separator();
                    self.show_modular_toolbar_buttons(ui);
                });
            } else {
                ui.vertical(|ui| {
                    show_grip(ui);
                    ui.separator();
                    self.show_modular_toolbar_buttons(ui);
                });
            }
        });

        if drag_started {
            self.toolbar_dragging = true;
            self.toolbar_drag_preview = Some(self.toolbar_state.dock);
            self.last_action = "Dragging modular toolbar".into();
        }
    }

    fn update_surface_drag_drop(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        shell_rect: egui::Rect,
    ) {
        let Some(surface_id) = self.surface_drag.clone() else {
            return;
        };

        let locked = self
            .modular_surfaces
            .iter()
            .find(|surface| surface.id == surface_id)
            .is_some_and(|surface| surface.locked);
        if locked {
            self.surface_drag = None;
            self.surface_drag_preview = None;
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
            self.surface_drag_preview = Some(destination);

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

            let accent = self.theme.base.accent;
            ui.painter().rect_filled(
                preview.shrink(5.0),
                self.theme.interaction.surface_radius.clamp(0.0, 20.0),
                egui::Color32::from_rgba_unmultiplied(accent.0, accent.1, accent.2, 28),
            );
            ui.painter().rect_stroke(
                preview.shrink(5.0),
                self.theme.interaction.surface_radius.clamp(0.0, 20.0),
                egui::Stroke::new(2.0, color(accent)),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                preview.center(),
                egui::Align2::CENTER_CENTER,
                format!("Dock {}", destination.label()),
                egui::FontId::proportional(14.0),
                color(self.theme.base.text),
            );
        }

        if !ctx.input(|input| input.pointer.primary_down()) {
            let destination = if pointer.is_none() {
                SurfaceDock::Floating
            } else {
                self.surface_drag_preview.unwrap_or(SurfaceDock::Center)
            };
            if let Some(surface) = self
                .modular_surfaces
                .iter_mut()
                .find(|surface| surface.id == surface_id)
            {
                surface.visible = true;
                let _ = surface.move_to(destination);
            }
            self.set_active_surface(destination, surface_id.clone());
            self.last_action = format!("Docked {surface_id} → {}", destination.label());
            self.surface_drag = None;
            self.surface_drag_preview = None;
        }
    }

    fn update_toolbar_drag_drop(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        shell_rect: egui::Rect,
    ) {
        if !self.toolbar_dragging {
            return;
        }
        if self.toolbar_state.locked {
            self.toolbar_dragging = false;
            self.toolbar_drag_preview = None;
            return;
        }

        let pointer = ctx.pointer_hover_pos();
        if let Some(pointer) = pointer {
            let side_width = (shell_rect.width() * 0.12).clamp(64.0, 150.0);
            let edge_height = (shell_rect.height() * 0.12).clamp(52.0, 110.0);
            let destination = if pointer.y <= shell_rect.top() + edge_height {
                ToolbarDock::Top
            } else if pointer.y >= shell_rect.bottom() - edge_height {
                ToolbarDock::Bottom
            } else if pointer.x <= shell_rect.left() + side_width {
                ToolbarDock::Left
            } else if pointer.x >= shell_rect.right() - side_width {
                ToolbarDock::Right
            } else {
                ToolbarDock::Floating
            };
            self.toolbar_drag_preview = Some(destination);

            let preview = match destination {
                ToolbarDock::Top => egui::Rect::from_min_max(
                    shell_rect.min,
                    egui::pos2(shell_rect.right(), shell_rect.top() + edge_height),
                ),
                ToolbarDock::Bottom => egui::Rect::from_min_max(
                    egui::pos2(shell_rect.left(), shell_rect.bottom() - edge_height),
                    shell_rect.max,
                ),
                ToolbarDock::Left => egui::Rect::from_min_max(
                    shell_rect.min,
                    egui::pos2(shell_rect.left() + side_width, shell_rect.bottom()),
                ),
                ToolbarDock::Right => egui::Rect::from_min_max(
                    egui::pos2(shell_rect.right() - side_width, shell_rect.top()),
                    shell_rect.max,
                ),
                ToolbarDock::Floating | ToolbarDock::Hidden => {
                    egui::Rect::from_center_size(shell_rect.center(), egui::vec2(360.0, 72.0))
                }
            };
            let accent = self.theme.base.accent;
            ui.painter().rect_stroke(
                preview.shrink(5.0),
                self.theme.interaction.surface_radius.clamp(0.0, 20.0),
                egui::Stroke::new(2.0, color(accent)),
                egui::StrokeKind::Inside,
            );
        }

        if !ctx.input(|input| input.pointer.primary_down()) {
            self.toolbar_state.dock = if pointer.is_none() {
                ToolbarDock::Floating
            } else {
                self.toolbar_drag_preview.unwrap_or(ToolbarDock::Floating)
            };
            self.toolbar_state.visible = true;
            self.last_action = format!("Toolbar → {}", self.toolbar_state.dock.label());
            self.toolbar_dragging = false;
            self.toolbar_drag_preview = None;
        }
    }

    fn show_empty_workspace(&mut self, ui: &mut egui::Ui) {
        ui.add_space((ui.available_height() * 0.20).min(160.0));
        ui.vertical_centered(|ui| {
            ui.heading("Empty workspace");
            ui.label("All surfaces are hidden. The remaining area is an opaque application host.");
            ui.add_space(12.0);
            if ui.button("Restore standard layout  ·  F10").clicked() {
                self.restore_standard_layout();
            }
        });
    }

    fn show_application_workspace(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme.clone();
        modular_surface_frame(&theme).show(ui, |ui| {
            ui.heading("Application Surface");
            ui.label(
                egui::RichText::new(
                    "Generic primary content host. Canvas/editor tooling is optional and lives in its own workspace tab.",
                )
                .color(color(theme.base.text_muted)),
            );
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Open Content").clicked() {
                    self.open_surface("surface.content");
                }
                if ui.button("Open Properties").clicked() {
                    self.open_surface("surface.properties");
                }
                if ui.button("Open Settings").clicked() {
                    self.universal_suite.tab = UniversalTab::Settings;
                    self.open_surface("surface.universal");
                }
                if ui.button("Float Widgets").clicked() {
                    if let Some(surface) = self
                        .modular_surfaces
                        .iter_mut()
                        .find(|surface| surface.id == "surface.widgets")
                    {
                        surface.visible = true;
                        let _ = surface.move_to(SurfaceDock::Floating);
                    }
                }
            });
            ui.add_space(14.0);
            ui.separator();
            ui.add_space(8.0);
            ui.strong("Modular GUI certification");
            ui.label("Drag the ⠿ grip in any active surface title to re-dock it left, center, right, or bottom. Drag outside the application to detach it into a native OS window.");
            ui.label("Surfaces sharing the same dock become a tab stack. Every surface can be hidden, detached, re-docked, resized, or locked. Closing a detached window restores its dock.");
            ui.label("The toolbar has its own ⠿ grip and may dock on any edge, float, lock, or disappear. Shell profiles can strip chrome for content-first applications.");
            ui.add_space(8.0);
            let _ = progress_bar(
                ui,
                self.demo_progress,
                Some("Universal component coverage"),
                WidgetTone::Accent,
                &theme,
            );
        });
    }

    fn show_modular_shell(&mut self, root: &mut egui::Ui, ctx: &egui::Context) {
        let shell_rect = root.available_rect_before_wrap();

        if self.toolbar_state.visible && !matches!(self.toolbar_state.dock, ToolbarDock::Hidden) {
            match self.toolbar_state.dock {
                ToolbarDock::Top => {
                    egui::Panel::top("forge.modular.toolbar.top")
                        .frame(egui::Frame::NONE)
                        .exact_size(self.theme.effective_metrics().action_bar_height)
                        .show(root, |ui| self.show_modular_toolbar(ui, true));
                }
                ToolbarDock::Bottom => {
                    egui::Panel::bottom("forge.modular.toolbar.bottom")
                        .frame(egui::Frame::NONE)
                        .exact_size(self.theme.effective_metrics().action_bar_height)
                        .show(root, |ui| self.show_modular_toolbar(ui, true));
                }
                ToolbarDock::Left => {
                    egui::Panel::left("forge.modular.toolbar.left")
                        .frame(egui::Frame::NONE)
                        .exact_size(46.0)
                        .show(root, |ui| self.show_modular_toolbar(ui, false));
                }
                ToolbarDock::Right => {
                    egui::Panel::right("forge.modular.toolbar.right")
                        .frame(egui::Frame::NONE)
                        .exact_size(46.0)
                        .show(root, |ui| self.show_modular_toolbar(ui, false));
                }
                ToolbarDock::Floating => {
                    let frame = modular_surface_frame(&self.theme);
                    egui::Window::new("Toolbar")
                        .id(egui::Id::new("forge.modular.toolbar.floating"))
                        .collapsible(false)
                        .resizable(false)
                        .movable(!self.toolbar_state.locked)
                        .frame(frame)
                        .show(ctx, |ui| self.show_modular_toolbar_buttons(ui));
                }
                ToolbarDock::Hidden => {}
            }
        }

        if self.show_modular_surfaces {
            let left_exists = self
                .modular_surfaces
                .iter()
                .any(|surface| surface.visible && surface.dock == SurfaceDock::Left);
            if left_exists {
                egui::Panel::left("forge.modular.surfaces.left")
                    .frame(egui::Frame::NONE)
                    .resizable(!self.dock_group_locked(SurfaceDock::Left))
                    .default_size(280.0)
                    .size_range(220.0..=520.0)
                    .show(root, |ui| self.show_dock_group(ui, SurfaceDock::Left));
            }

            let right_exists = self
                .modular_surfaces
                .iter()
                .any(|surface| surface.visible && surface.dock == SurfaceDock::Right);
            if right_exists {
                egui::Panel::right("forge.modular.surfaces.right")
                    .frame(egui::Frame::NONE)
                    .resizable(!self.dock_group_locked(SurfaceDock::Right))
                    .default_size(330.0)
                    .size_range(240.0..=620.0)
                    .show(root, |ui| self.show_dock_group(ui, SurfaceDock::Right));
            }

            let bottom_exists = self
                .modular_surfaces
                .iter()
                .any(|surface| surface.visible && surface.dock == SurfaceDock::Bottom);
            if bottom_exists {
                egui::Panel::bottom("forge.modular.surfaces.bottom")
                    .frame(egui::Frame::NONE)
                    .resizable(!self.dock_group_locked(SurfaceDock::Bottom))
                    .default_size(160.0)
                    .size_range(110.0..=420.0)
                    .show(root, |ui| self.show_dock_group(ui, SurfaceDock::Bottom));
            }
        }

        let center_frame = egui::Frame::new()
            .fill(forge_gui_chrome::opaque_background(self.theme.chrome.shell))
            .corner_radius(self.theme.interaction.surface_radius.clamp(0.0, 20.0) as u8)
            .inner_margin(egui::Margin::same(4));
        egui::CentralPanel::default()
            .frame(center_frame)
            .show(root, |ui| {
                if self.layout.active_workspace == "workspace.dashboard" {
                    self.show_widget_gallery(ui);
                } else {
                    let center_exists = self.show_modular_surfaces
                        && self
                            .modular_surfaces
                            .iter()
                            .any(|surface| surface.visible && surface.dock == SurfaceDock::Center);
                    if center_exists {
                        self.show_dock_group(ui, SurfaceDock::Center);
                    } else if self.show_modular_surfaces
                        && self.modular_surfaces.iter().all(|surface| !surface.visible)
                    {
                        self.show_empty_workspace(ui);
                    } else {
                        self.show_application_workspace(ui);
                    }
                }
            });

        if self.show_modular_surfaces {
            self.show_floating_surfaces(ctx);
        } else {
            self.native_surface_lifecycle.clear();
        }

        self.update_surface_drag_drop(root, ctx, shell_rect);
        self.update_toolbar_drag_drop(root, ctx, shell_rect);
    }

    fn show_surface_configuration_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button("Show all surfaces").clicked() {
            for surface in &mut self.modular_surfaces {
                surface.visible = true;
            }
            self.show_modular_surfaces = true;
            self.last_action = "All surfaces visible".into();
            ui.close();
        }
        if ui.button("Hide all surfaces").clicked() {
            for surface in &mut self.modular_surfaces {
                surface.visible = false;
            }
            self.last_action = "All surfaces hidden; use F10 to restore".into();
            ui.close();
        }
        if ui.button("Reset surface arrangement").clicked() {
            self.restore_standard_layout();
            ui.close();
        }
        ui.separator();
        for index in 0..self.modular_surfaces.len() {
            let label = self.modular_surfaces[index].title.clone();
            ui.menu_button(label, |ui| {
                let surface = &mut self.modular_surfaces[index];
                ui.checkbox(&mut surface.visible, "Visible");
                ui.checkbox(&mut surface.locked, "Lock in place");
                ui.separator();
                ui.label("Dock position");
                ui.add_enabled_ui(!surface.locked, |ui| {
                    for dock in SurfaceDock::ALL {
                        if ui
                            .selectable_label(surface.dock == dock, dock.label())
                            .clicked()
                        {
                            let _ = surface.move_to(dock);
                        }
                    }
                });
            });
        }
    }

    fn show_toolbar_configuration_menu(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.toolbar_state.visible, "Visible");
        ui.checkbox(&mut self.toolbar_state.locked, "Lock in place");
        ui.separator();
        ui.label("Dock position");
        ui.add_enabled_ui(!self.toolbar_state.locked, |ui| {
            for dock in ToolbarDock::ALL {
                if ui
                    .selectable_label(self.toolbar_state.dock == dock, dock.label())
                    .clicked()
                {
                    self.toolbar_state.dock = dock;
                    self.toolbar_state.visible = !matches!(dock, ToolbarDock::Hidden);
                }
            }
        });
    }

    fn show_interaction_settings_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut changed = false;
        ui.menu_button("Hover highlight", |ui| {
            for mode in HighlightMode::ALL {
                if ui
                    .selectable_label(self.theme.interaction.hover == mode, mode.label())
                    .clicked()
                {
                    self.theme.interaction.hover = mode;
                    changed = true;
                }
            }
        });
        ui.menu_button("Pressed highlight", |ui| {
            for mode in HighlightMode::ALL {
                if ui
                    .selectable_label(self.theme.interaction.pressed == mode, mode.label())
                    .clicked()
                {
                    self.theme.interaction.pressed = mode;
                    changed = true;
                }
            }
        });
        ui.menu_button("Selected highlight", |ui| {
            for mode in HighlightMode::ALL {
                if ui
                    .selectable_label(self.theme.interaction.selected == mode, mode.label())
                    .clicked()
                {
                    self.theme.interaction.selected = mode;
                    changed = true;
                }
            }
        });
        ui.separator();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.theme.interaction.surface_radius, 0.0..=14.0)
                    .text("Surface rounding"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.theme.interaction.control_radius, 0.0..=12.0)
                    .text("Control rounding"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.theme.interaction.focus_width, 1.0..=3.0)
                    .text("Focus width"),
            )
            .changed();

        if changed {
            apply_creator_visuals(ctx, &self.theme);
            self.last_action = "Interaction styling updated".into();
        }
    }

    fn show_right_panel(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme.clone();
        let (_, header) = show_panel(
            ui,
            "Workspace",
            Some("Context"),
            PanelKind::Utility,
            &[],
            &theme,
            |ui| {
                if let Some(selected) = show_panel_tabs(
                    ui,
                    &[
                        ("inspector", "Inspector"),
                        ("assets", "Assets"),
                        ("widgets", "Widgets"),
                        ("universal", "Universal"),
                    ],
                    &self.right_tab,
                    &theme,
                ) {
                    self.right_tab = selected;
                }
                ui.add_space(4.0);

                match self.right_tab.as_str() {
                    "assets" => {
                        let response = show_browser(ui, &mut self.browser, &theme);
                        if let Some(asset) = response.activated {
                            self.last_action = format!("Open asset: {asset}");
                        } else if response.selection_changed {
                            self.last_action = "Asset selection changed".into();
                        }
                    }
                    "widgets" => self.show_widget_gallery(ui),
                    "universal" => {
                        let response = show_universal_suite(ui, &mut self.universal_suite, &theme);
                        if let Some(action) = response.last_action {
                            self.last_action = action;
                        }
                    }
                    _ => {
                        let inspector =
                            show_property_object(ui, &mut self.inspector_object, &theme);
                        if let Some(change) = inspector.changes.last() {
                            self.last_action =
                                format!("Changed {}.{}", change.object_id, change.property_id);
                        }
                    }
                }
            },
        );
        if let Some(action) = header.invoked {
            self.last_action = action;
        }
    }

    fn show_widget_gallery(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.strong("Data display");
            ui.add_space(4.0);
            let progress_label = format!(
                "Build pipeline · {:.0}%",
                self.demo_progress.clamp(0.0, 1.0) * 100.0
            );
            let _ = progress_bar(
                ui,
                self.demo_progress,
                Some(&progress_label),
                WidgetTone::Accent,
                &self.theme,
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let _ = gauge(
                    ui,
                    self.demo_gauge,
                    0.0,
                    100.0,
                    "Health",
                    WidgetTone::Success,
                    &self.theme,
                );
                let _ = gauge(
                    ui,
                    41.0,
                    0.0,
                    100.0,
                    "Load",
                    WidgetTone::Warning,
                    &self.theme,
                );
            });

            ui.separator();
            ui.strong("Scroll chrome");
            ui.add_space(4.0);
            let _ = scrollbar(
                ui,
                ScrollbarAxis::Horizontal,
                &mut self.h_scroll,
                &self.theme,
            );
            ui.horizontal(|ui| {
                ui.label(format!("Position {:.0}%", self.h_scroll.position * 100.0));
                ui.add_space(12.0);
                ui.allocate_ui(egui::vec2(20.0, 120.0), |ui| {
                    let _ = scrollbar(ui, ScrollbarAxis::Vertical, &mut self.v_scroll, &self.theme);
                });
            });

            ui.separator();
            ui.strong("Controls");
            ui.checkbox(&mut self.workspace_view.snap_enabled, "Snap to grid");
            ui.add(egui::Slider::new(&mut self.demo_progress, 0.0..=1.0).text("Progress"));
            ui.add(egui::Slider::new(&mut self.demo_gauge, 0.0..=100.0).text("Gauge value"));
            ui.horizontal_wrapped(|ui| {
                let _ = ui.button("Button");
                let _ = ui.small_button("Compact");
                let _ = ui.selectable_label(true, "Selected");
            });

            ui.separator();
            ui.strong("Current theme");
            ui.label(&self.theme.label);
            ui.label(
                egui::RichText::new(
                    "ForgeGUI owns widget styling; consumer projects inherit tokens.",
                )
                .small()
                .color(color(self.theme.base.text_muted)),
            );
        });
    }

    fn show_bottom_tray(&mut self, ui: &mut egui::Ui) {
        if let Some(selected) =
            show_tool_tray_tabs(ui, &self.tool_tabs, &self.active_tool_tab, &self.theme)
        {
            self.active_tool_tab = selected;
        }
        ui.separator();
        show_bottom_tool(ui, &self.active_tool_tab, &self.last_action, &self.theme);
    }
}

impl eframe::App for CreatorStudioLab {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, SURFACE_STORAGE_KEY, &self.modular_surfaces);
    }

    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let metrics = self.theme.effective_metrics();

        // Opaque full-host fallback is independent of optional panel visibility.
        let shell = self.theme.chrome.shell;
        root.painter().rect_filled(
            root.max_rect(),
            0.0,
            egui::Color32::from_rgb(shell.0, shell.1, shell.2),
        );

        if root.input(|i| i.modifiers.command && i.key_pressed(egui::Key::P)) {
            self.palette.open();
        }
        if root.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N)) {
            self.dispatch_command("forge.command.file.new");
        }
        if root.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.dispatch_command("forge.command.document.save");
        }
        if root.input(|i| i.key_pressed(egui::Key::F6)) {
            self.dispatch_command("forge.command.run.play");
        }
        if root.input(|i| i.key_pressed(egui::Key::F10)) {
            self.restore_standard_layout();
        }

        let title_response = egui::Panel::top("forge.creator.project_title")
            .frame(egui::Frame::NONE)
            .exact_size(metrics.product_bar_height)
            .show(root, |ui| {
                show_project_title_bar(
                    ui,
                    "ForgeGUI",
                    "Universal Modular GUI Lab",
                    Some("main"),
                    &self.theme,
                )
            })
            .inner;
        if title_response.drag_started {
            self.window_snap.drag_active = true;
        }
        if let Some(action) = update_window_snap(&ctx, &mut self.window_snap) {
            self.last_action = format!("Window snap: {action:?}");
        }

        if self.show_menu_bar {
            let menu_theme = self.theme.clone();
            egui::Panel::top("forge.creator.menu")
            .frame(egui::Frame::NONE)
            .exact_size(metrics.menu_bar_height)
            .show(root, |ui| {
                show_chrome_bar(ui, ChromeBarKind::Menu, &menu_theme, |ui| {
                    ui.horizontal(|ui| {
                        ui.menu_button("File", |ui| {
                            ui.add_enabled(false, egui::Button::new("New   Ctrl+N"))
                                .on_hover_text("Host document service is not attached in the certification Lab.");
                            ui.add_enabled(false, egui::Button::new("Open…"))
                                .on_hover_text("Host file/document service is not attached in the certification Lab.");
                            ui.add_enabled(false, egui::Button::new("Save   Ctrl+S"))
                                .on_hover_text("Host document service is not attached in the certification Lab.");
                            ui.add_enabled(false, egui::Button::new("Save As…"))
                                .on_hover_text("Host document service is not attached in the certification Lab.");
                            ui.separator();
                            ui.add_enabled(false, egui::Button::new("Import…"))
                                .on_hover_text("A consumer must register an import provider.");
                            ui.add_enabled(false, egui::Button::new("Export…"))
                                .on_hover_text("A consumer must register an export provider.");
                        });

                        ui.menu_button("Edit", |ui| {
                            ui.add_enabled(false, egui::Button::new("Undo"))
                                .on_hover_text("No host HistoryService transaction is active.");
                            ui.add_enabled(false, egui::Button::new("Redo"))
                                .on_hover_text("No host HistoryService transaction is active.");
                            ui.separator();
                            if ui.button("Command Palette…   Ctrl+P").clicked() {
                                self.palette.open();
                                ui.close();
                            }
                            if ui.button("Preferences…").clicked() {
                                self.universal_suite.tab = UniversalTab::Settings;
                                self.open_surface("surface.universal");
                                ui.close();
                            }
                        });

                        ui.menu_button("View", |ui| {
                            ui.menu_button("Shell profile", |ui| {
                                for profile in ShellProfile::ALL {
                                    if ui
                                        .selectable_label(self.shell_profile == profile, profile.label())
                                        .clicked()
                                    {
                                        self.apply_shell_profile(profile);
                                        ui.close();
                                    }
                                }
                            });
                            ui.checkbox(&mut self.show_workspace_tabs, "Workspace tabs");
                            ui.checkbox(&mut self.layout.structural.status_visible, "Status bar");
                            ui.checkbox(&mut self.show_modular_surfaces, "Modular surfaces");
                            ui.separator();
                            ui.menu_button("Surfaces", |ui| self.show_surface_configuration_menu(ui));
                            ui.menu_button("Toolbar", |ui| self.show_toolbar_configuration_menu(ui));
                            ui.separator();
                            ui.menu_button("Canvas / Viewport", |ui| {
                                ui.label("Optional authoring workspace");
                                ui.separator();
                                ui.checkbox(&mut self.left_rail_pinned, "Pin canvas tool rail");
                                ui.checkbox(&mut self.right_panel_pinned, "Pin canvas context panel");
                                ui.checkbox(
                                    &mut self.layout.structural.bottom_visible,
                                    "Canvas bottom tool tray",
                                );
                                ui.separator();
                                canvas_options_menu(
                                    ui,
                                    &mut self.workspace_view,
                                    &mut self.canvas_chrome,
                                );
                                if let Some(family) = canvas_family_menu(ui, &mut self.render_surface) {
                                    self.set_renderer_family(family);
                                }
                            });
                            ui.separator();
                            if ui.button("Restore complete layout   F10").clicked() {
                                self.restore_standard_layout();
                                ui.close();
                            }
                        });

                        ui.menu_button("Settings", |ui| {
                            ui.menu_button("Theme", |ui| {
                                for preset in ForgeThemePreset::ALL {
                                    if ui
                                        .selectable_label(
                                            self.theme_preset == preset,
                                            preset.label(),
                                        )
                                        .clicked()
                                    {
                                        self.apply_theme(&ctx, preset);
                                        ui.close();
                                    }
                                }
                            });
                            ui.menu_button("Density", |ui| {
                                for density in ForgeDensity::ALL {
                                    if ui
                                        .selectable_label(self.theme.density == density, density.label())
                                        .clicked()
                                    {
                                        self.theme.density = density;
                                        apply_creator_visuals(&ctx, &self.theme);
                                        self.last_action = format!("Density: {}", density.label());
                                        ui.close();
                                    }
                                }
                            });
                            ui.menu_button("Interaction & rounding", |ui| {
                                self.show_interaction_settings_menu(ui, &ctx)
                            });
                            ui.separator();
                            ui.checkbox(&mut self.window_snap.enabled, "Window edge snap");
                            if ui.button("Preferences…").clicked() {
                                self.universal_suite.tab = UniversalTab::Settings;
                                self.open_surface("surface.universal");
                                ui.close();
                            }
                        });

                        ui.menu_button("Help", |ui| {
                            if ui.button("Universal Application Suite").clicked() {
                                self.universal_suite.tab = UniversalTab::Overview;
                                self.open_surface("surface.universal");
                                ui.close();
                            }
                            if ui.button("GUI Widget Catalog").clicked() {
                                self.open_surface("surface.widgets");
                                ui.close();
                            }
                            if ui.button("Keyboard Shortcuts").clicked() {
                                self.universal_suite.tab = UniversalTab::Commands;
                                self.open_surface("surface.universal");
                                ui.close();
                            }
                            ui.separator();
                            ui.label("ForgeGUI Core · universal modular shell");
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(&self.theme.label)
                                    .small()
                                    .color(color(self.theme.base.text_muted)),
                            );
                            ui.separator();
                            let play = if self.runtime_playing {
                                "■ Stop"
                            } else {
                                "▶ Play"
                            };
                            if ui.small_button(play).clicked() {
                                self.runtime_playing = !self.runtime_playing;
                                self.last_action = if self.runtime_playing {
                                    "Run > Play".into()
                                } else {
                                    "Run > Stop".into()
                                };
                            }
                        });
                    });
                });
            });

            egui::Panel::top("forge.creator.menu_separation")
                .frame(egui::Frame::NONE)
                .exact_size(4.0)
                .show(root, |ui| {
                    ui.painter()
                        .rect_filled(ui.max_rect(), 0.0, color(self.theme.chrome.shell));
                });
        }

        if self.show_workspace_tabs {
            egui::Panel::top("forge.creator.workspace_tabs")
                .exact_size(metrics.workspace_tab_height)
                .show(root, |ui| {
                    if let Some(selected) = show_workspace_tabs(
                        ui,
                        &self.workspace_tabs,
                        &self.layout.active_workspace,
                        &self.theme,
                    ) {
                        self.layout.active_workspace = selected.clone();
                        self.last_action = format!("Workspace: {selected}");
                    }
                });
        }

        if self.layout.structural.status_visible {
            egui::Panel::bottom("forge.creator.status")
                .frame(egui::Frame::NONE)
                .exact_size(metrics.status_bar_height)
                .show(root, |ui| {
                    let items = if self.layout.active_workspace == "workspace.canvas" {
                        vec![
                            StatusItem {
                                label: "Tool".into(),
                                value: format!("{:?}", self.workspace_view.active_tool),
                                tone: WidgetTone::Accent,
                            },
                            StatusItem {
                                label: "Layer".into(),
                                value: self
                                    .canvas_chrome
                                    .active_layer
                                    .as_deref()
                                    .and_then(|id| {
                                        self.canvas_chrome
                                            .layers
                                            .iter()
                                            .find(|layer| layer.id == id)
                                    })
                                    .map(|layer| layer.label.clone())
                                    .unwrap_or_else(|| "None".into()),
                                tone: WidgetTone::Neutral,
                            },
                            StatusItem {
                                label: "Renderer".into(),
                                value: self.render_surface.descriptor().family.label().into(),
                                tone: WidgetTone::Neutral,
                            },
                            StatusItem {
                                label: "Frame".into(),
                                value: self.render_surface.frame_index().to_string(),
                                tone: WidgetTone::Neutral,
                            },
                            StatusItem {
                                label: "State".into(),
                                value: self.last_action.clone(),
                                tone: WidgetTone::Neutral,
                            },
                        ]
                    } else {
                        let visible_surfaces = self
                            .modular_surfaces
                            .iter()
                            .filter(|surface| surface.visible)
                            .count();
                        vec![
                            StatusItem {
                                label: "Profile".into(),
                                value: self.shell_profile.label().into(),
                                tone: WidgetTone::Accent,
                            },
                            StatusItem {
                                label: "Workspace".into(),
                                value: self.layout.active_workspace.replace("workspace.", ""),
                                tone: WidgetTone::Neutral,
                            },
                            StatusItem {
                                label: "Surfaces".into(),
                                value: visible_surfaces.to_string(),
                                tone: WidgetTone::Neutral,
                            },
                            StatusItem {
                                label: "Toolbar".into(),
                                value: self.toolbar_state.dock.label().into(),
                                tone: WidgetTone::Neutral,
                            },
                            StatusItem {
                                label: "State".into(),
                                value: self.last_action.clone(),
                                tone: WidgetTone::Neutral,
                            },
                        ]
                    };
                    show_status_items(ui, &items, &self.theme);
                });
        }

        if self.layout.active_workspace != "workspace.canvas" {
            self.show_modular_shell(root, &ctx);

            if let Some(invocation) = show_command_palette(&ctx, &mut self.palette, &self.theme) {
                self.dispatch_command(&invocation.id);
            }

            show_viewport_resize_handles(root, 5.0);
            return;
        }
        self.native_surface_lifecycle.clear();

        if self.layout.structural.bottom_visible {
            egui::Panel::bottom("forge.creator.bottom_tray")
                .resizable(true)
                .default_size(self.layout.structural.bottom_height)
                .size_range(96.0..=420.0)
                .show(root, |ui| self.show_bottom_tray(ui));
        }

        if self.left_rail_pinned {
            egui::Panel::left("forge.creator.universal_rail")
                .exact_size(54.0)
                .show(root, |ui| self.show_universal_rail(ui));
        }

        if self.right_panel_pinned {
            egui::Panel::right("forge.creator.context")
                .resizable(true)
                .default_size(self.layout.structural.right_width)
                .size_range(270.0..=520.0)
                .show(root, |ui| self.show_right_panel(ui));
        }

        let center_rect = root.available_rect_before_wrap();
        let left_edge_hot =
            !self.left_rail_pinned && pointer_near_edge(&ctx, center_rect, ChromeEdge::Left, 8.0);
        let right_edge_hot = !self.right_panel_pinned
            && pointer_near_edge(&ctx, center_rect, ChromeEdge::Right, 8.0);

        if !self.left_rail_pinned && (left_edge_hot || self.left_overlay_open) {
            let area = egui::Area::new(egui::Id::new("forge.creator.rail.overlay"))
                .order(egui::Order::Foreground)
                .fixed_pos(center_rect.left_top())
                .show(&ctx, |ui| {
                    egui::Frame::new()
                        .fill(color(self.theme.chrome.shell))
                        .stroke(egui::Stroke::new(1.0, color(self.theme.chrome.separator)))
                        .show(ui, |ui| {
                            ui.set_min_size(egui::vec2(54.0, center_rect.height()));
                            ui.set_max_width(54.0);
                            self.show_universal_rail(ui);
                        });
                });
            let pointer = ctx.pointer_hover_pos();
            self.left_overlay_open = left_edge_hot
                || pointer
                    .map(|position| area.response.rect.contains(position))
                    .unwrap_or(false);
        } else if !self.left_rail_pinned {
            self.left_overlay_open = false;
        }

        if !self.right_panel_pinned && (right_edge_hot || self.right_overlay_open) {
            let width = self.layout.structural.right_width.clamp(270.0, 520.0);
            let area = egui::Area::new(egui::Id::new("forge.creator.context.overlay"))
                .order(egui::Order::Foreground)
                .fixed_pos(egui::pos2(center_rect.right() - width, center_rect.top()))
                .show(&ctx, |ui| {
                    egui::Frame::new()
                        .fill(color(self.theme.chrome.shell))
                        .stroke(egui::Stroke::new(1.0, color(self.theme.chrome.separator)))
                        .show(ui, |ui| {
                            ui.set_min_size(egui::vec2(width, center_rect.height()));
                            ui.set_max_width(width);
                            self.show_right_panel(ui);
                        });
                });
            let pointer = ctx.pointer_hover_pos();
            self.right_overlay_open = right_edge_hot
                || pointer
                    .map(|position| area.response.rect.contains(position))
                    .unwrap_or(false);
        } else if !self.right_panel_pinned {
            self.right_overlay_open = false;
        }

        egui::CentralPanel::default().show(root, |ui| {
            let preview_visibility = {
                let layer_visible = |id: &str| {
                    self.canvas_chrome
                        .layers
                        .iter()
                        .find(|layer| layer.id == id)
                        .map(|layer| layer.visible)
                        .unwrap_or(true)
                };
                PreviewVisibility {
                    grid: self.workspace_view.show_grid,
                    world: layer_visible("layer.world"),
                    entities: layer_visible("layer.entities"),
                    lighting: layer_visible("layer.lighting"),
                    guides: self.workspace_view.show_guides && layer_visible("layer.guides"),
                }
            };
            let theme = self.theme.clone();
            let workspace_response = show_renderer_workspace(
                ui,
                &mut self.render_surface,
                &mut self.workspace_view,
                &mut self.canvas_chrome,
                &self.theme,
                |ui, rect, surface| {
                    draw_renderer_preview(ui, rect, surface, preview_visibility, &theme)
                },
            );

            if let Some(layer) = workspace_response.layer_toggled {
                self.last_action = format!("Layer toggled: {layer}");
            }
            if workspace_response.requested_focus {
                self.last_action = "Canvas focused".into();
            }

            if !self.left_rail_pinned && !self.left_overlay_open {
                paint_edge_reveal_indicator(ui, ui.max_rect(), ChromeEdge::Left, &self.theme);
            }
            if !self.right_panel_pinned && !self.right_overlay_open {
                paint_edge_reveal_indicator(ui, ui.max_rect(), ChromeEdge::Right, &self.theme);
            }
        });

        self.run_renderer();

        if let Some(invocation) = show_command_palette(&ctx, &mut self.palette, &self.theme) {
            self.dispatch_command(&invocation.id);
        }

        show_viewport_resize_handles(root, 5.0);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        forge_gui_chrome::opaque_shell_clear_color(&self.theme)
    }
}

#[derive(Clone, Copy)]
struct PreviewVisibility {
    grid: bool,
    world: bool,
    entities: bool,
    lighting: bool,
    guides: bool,
}

fn draw_renderer_preview(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    surface: &RenderSurfaceHost,
    visibility: PreviewVisibility,
    theme: &ForgeTheme,
) {
    let painter = ui.painter();

    if visibility.grid && visibility.world {
        let grid_color = color(theme.base.border);
        let major_color = color(theme.chrome.separator);
        let step = 32.0;

        let mut x = rect.left();
        let mut column = 0usize;
        while x <= rect.right() {
            let stroke = if column.is_multiple_of(4) {
                egui::Stroke::new(1.0, major_color)
            } else {
                egui::Stroke::new(1.0, grid_color)
            };
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                stroke,
            );
            x += step;
            column += 1;
        }

        let mut y = rect.top();
        let mut row = 0usize;
        while y <= rect.bottom() {
            let stroke = if row.is_multiple_of(4) {
                egui::Stroke::new(1.0, major_color)
            } else {
                egui::Stroke::new(1.0, grid_color)
            };
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                stroke,
            );
            y += step;
            row += 1;
        }
    }

    if visibility.world {
        let room = rect.shrink2(egui::vec2(58.0, 48.0));
        painter.rect_stroke(
            room,
            4.0,
            egui::Stroke::new(1.0, color(theme.base.border)),
            egui::StrokeKind::Inside,
        );
    }

    if visibility.entities {
        let center = rect.center();
        let player = egui::Rect::from_center_size(center, egui::vec2(120.0, 86.0));
        painter.rect_filled(player, 5.0, color(theme.base.panel_raised));
        painter.rect_stroke(
            player,
            5.0,
            egui::Stroke::new(2.0, color(theme.base.accent)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            player.center(),
            egui::Align2::CENTER_CENTER,
            "Player",
            egui::FontId::proportional(16.0),
            color(theme.base.text),
        );
        for handle in [
            player.left_top(),
            player.right_top(),
            player.left_bottom(),
            player.right_bottom(),
        ] {
            painter.circle_filled(handle, 4.0, color(theme.base.accent));
        }
    }

    if visibility.guides {
        let camera = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 90.0, rect.top() + 120.0),
            egui::vec2(190.0, 120.0),
        );
        painter.rect_stroke(
            camera,
            2.0,
            egui::Stroke::new(1.0, color(theme.base.warning)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            camera.left_top() + egui::vec2(6.0, 6.0),
            egui::Align2::LEFT_TOP,
            "Camera Preview",
            egui::FontId::proportional(12.0),
            color(theme.base.warning),
        );
    }

    if visibility.lighting {
        let light_center = rect.right_top() + egui::vec2(-88.0, 72.0);
        painter.circle_filled(
            light_center,
            34.0,
            egui::Color32::from_rgba_unmultiplied(
                theme.base.warning.0,
                theme.base.warning.1,
                theme.base.warning.2,
                20,
            ),
        );
        painter.circle_stroke(
            light_center,
            34.0,
            egui::Stroke::new(1.0, color(theme.base.warning)),
        );
    }

    if let Some(target) = surface.last_target() {
        painter.text(
            rect.right_bottom() + egui::vec2(-12.0, -10.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("target: {}", target.id),
            egui::FontId::monospace(11.0),
            color(theme.base.text_muted),
        );
    }
}

fn show_bottom_tool(ui: &mut egui::Ui, active: &str, last_action: &str, theme: &ForgeTheme) {
    match active {
        "tool.console" => {
            ui.monospace("[PASS] Project-owned application shell initialized");
            ui.monospace("[PASS] Optional authoring workspace chrome attached");
            ui.monospace("[INFO] Universal rail and edge-reveal panels active");
            ui.monospace(format!("[ACTION] {last_action}"));
        }
        "tool.problems" => {
            ui.label("Problems");
            ui.label(
                egui::RichText::new("No runtime problems in the Lab model")
                    .color(color(theme.base.success)),
            );
        }
        "tool.build" => {
            ui.label("Build / Certification");
            ui.monospace("PCC Full Gate remains the build authority.");
        }
        "tool.runtime" => {
            ui.label("Runtime / PIE");
            ui.monospace("Edit → Simulate → PIE → Detached → Standalone");
        }
        "tool.cortex" => {
            ui.label("Cortex");
            ui.monospace("Consumer-provided AI/tooling panel slot.");
        }
        _ => {
            ui.label("Tool tray");
        }
    }
}

fn universal_tool_rail() -> RailModel {
    let style = RailStyle {
        preset: RailPreset::SlimIcon,
        edge: RailEdge::Left,
        placement: RailPlacement::InsideEdge,
        density: RailDensity::Compact,
        labels: LabelPolicy::TooltipOnly,
        collapsible: false,
        auto_hide: true,
        ..Default::default()
    };
    let mut rail = RailModel::new("forge.rail.universal", style);
    rail.sections = vec![
        RailSection {
            label: Some("Transform".into()),
            items: vec![
                RailItem::new("tool.select", icon_text(IconId::Asset), "Select").tooltip("Select"),
                RailItem::new("tool.move", icon_text(IconId::Entity), "Move").tooltip("Move"),
                RailItem::new("tool.rotate", icon_text(IconId::Restart), "Rotate")
                    .tooltip("Rotate"),
                RailItem::new("tool.scale", icon_text(IconId::Module), "Scale").tooltip("Scale"),
                RailItem::new("tool.pan", icon_text(IconId::World), "Pan").tooltip("Pan canvas"),
                RailItem::new("tool.zoom", icon_text(IconId::Search), "Zoom")
                    .tooltip("Zoom canvas"),
            ],
        },
        RailSection {
            label: Some("Workspace".into()),
            items: vec![
                RailItem::new("panel.inspector", icon_text(IconId::Inspector), "Inspector")
                    .tooltip("Inspector"),
                RailItem::new("panel.assets", icon_text(IconId::FolderOpen), "Assets")
                    .tooltip("Asset browser"),
                RailItem::new("panel.widgets", icon_text(IconId::Settings), "Widgets")
                    .tooltip("GUI widget gallery"),
                RailItem::new("panel.universal", icon_text(IconId::Graph), "Universal")
                    .tooltip("Universal application certification suite"),
            ],
        },
        RailSection {
            label: Some("Runtime".into()),
            items: vec![
                RailItem::new("run.play", icon_text(IconId::Play), "Play").tooltip("Play"),
                RailItem::new("run.stop", icon_text(IconId::Stop), "Stop").tooltip("Stop"),
            ],
        },
    ];
    rail.set_active("tool.select");
    rail
}

/// Restore user placement without clobbering new registered surfaces or duplicating old IDs.
/// A persisted window may have moved off-screen; native viewport positioning stays
/// the host's responsibility. This restores logical dock, size and visibility only.
fn restored_surfaces(
    mut defaults: Vec<ModularSurfaceState>,
    saved: Vec<ModularSurfaceState>,
) -> Vec<ModularSurfaceState> {
    for mut old in saved {
        if old.id.is_empty() {
            continue;
        }
        if !old.preferred_size.iter().all(|value| value.is_finite()) {
            old.preferred_size = [320.0, 320.0];
        }
        old.preferred_size[0] = old.preferred_size[0].clamp(260.0, 4096.0);
        old.preferred_size[1] = old.preferred_size[1].clamp(180.0, 4096.0);
        if let Some(existing) = defaults.iter_mut().find(|surface| surface.id == old.id) {
            // Retain up-to-date titles and registrations from the active project.
            existing.dock = old.dock;
            existing.last_dock = old.last_dock;
            existing.visible = old.visible;
            existing.locked = old.locked;
            existing.preferred_size = old.preferred_size;
        }
    }
    defaults
}

fn modular_surfaces() -> Vec<ModularSurfaceState> {
    let mut home = ModularSurfaceState::new("surface.home", "Application", SurfaceDock::Center);
    home.preferred_size = [760.0, 560.0];

    let mut content = ModularSurfaceState::new("surface.content", "Content", SurfaceDock::Left);
    content.preferred_size = [290.0, 520.0];

    let mut properties =
        ModularSurfaceState::new("surface.properties", "Properties", SurfaceDock::Right);
    properties.preferred_size = [330.0, 520.0];

    let mut universal =
        ModularSurfaceState::new("surface.universal", "Universal", SurfaceDock::Right);
    universal.preferred_size = [390.0, 620.0];

    let mut widgets =
        ModularSurfaceState::new("surface.widgets", "Widget Gallery", SurfaceDock::Floating);
    widgets.visible = false;
    widgets.preferred_size = [420.0, 580.0];

    let mut activity =
        ModularSurfaceState::new("surface.activity", "Activity", SurfaceDock::Bottom);
    activity.preferred_size = [720.0, 170.0];

    vec![home, content, properties, universal, widgets, activity]
}

fn workspace_tabs() -> Vec<WorkspaceTab> {
    vec![
        WorkspaceTab {
            id: "workspace.application".into(),
            label: "Application".into(),
            icon: Some(IconId::Project),
            dirty: false,
            closable: false,
        },
        WorkspaceTab {
            id: "workspace.dashboard".into(),
            label: "Dashboard".into(),
            icon: Some(IconId::Graph),
            dirty: false,
            closable: false,
        },
        WorkspaceTab {
            id: "workspace.canvas".into(),
            label: "Canvas / Viewport".into(),
            icon: Some(IconId::World),
            dirty: false,
            closable: false,
        },
    ]
}

fn tool_tabs() -> Vec<ToolTrayTab> {
    vec![
        ToolTrayTab::new("tool.console", "Console"),
        ToolTrayTab::new("tool.problems", "Problems"),
        ToolTrayTab::new("tool.build", "Build"),
        ToolTrayTab::new("tool.runtime", "Runtime / PIE"),
        ToolTrayTab::new("tool.cortex", "Cortex"),
    ]
}

fn browser_model() -> BrowserModel {
    BrowserModel {
        items: vec![
            BrowserItem::new("content.documents", "Documents", BrowserItemKind::Folder),
            BrowserItem::new(
                "content.report.q3",
                "Quarterly Report",
                BrowserItemKind::Data,
            ),
            BrowserItem::new("content.media", "Media", BrowserItemKind::Folder),
            BrowserItem::new(
                "content.media.brand",
                "Brand Preview",
                BrowserItemKind::Other,
            ),
            BrowserItem::new(
                "content.configuration",
                "Configuration",
                BrowserItemKind::Folder,
            ),
            BrowserItem::new(
                "content.settings.user",
                "User Settings",
                BrowserItemKind::Data,
            ),
            BrowserItem::new("content.logs", "Logs", BrowserItemKind::Folder),
            BrowserItem::new(
                "content.log.latest",
                "Latest Session",
                BrowserItemKind::Data,
            ),
        ],
        ..Default::default()
    }
}

fn inspector_object() -> PropertyObject {
    PropertyObject {
        object_id: "content.selected".into(),
        title: "Selected Item".into(),
        fields: vec![
            PropertyField {
                id: "enabled".into(),
                label: "Enabled".into(),
                value: PropertyValue::Bool(true),
                read_only: false,
            },
            PropertyField {
                id: "title".into(),
                label: "Title".into(),
                value: PropertyValue::Text("Quarterly Report".into()),
                read_only: false,
            },
            PropertyField {
                id: "opacity".into(),
                label: "Opacity".into(),
                value: PropertyValue::Float(1.0),
                read_only: false,
            },
            PropertyField {
                id: "source".into(),
                label: "Source".into(),
                value: PropertyValue::Reference("content.documents".into()),
                read_only: false,
            },
            PropertyField {
                id: "stable_id".into(),
                label: "Stable ID".into(),
                value: PropertyValue::Text("content.report.q3".into()),
                read_only: true,
            },
        ],
    }
}

fn command_palette() -> CommandPalette {
    CommandPalette {
        open: false,
        query: String::new(),
        selected: 0,
        entries: vec![
            PaletteEntry {
                id: "forge.command.file.new".into(),
                title: "New".into(),
                category: "File".into(),
                shortcut: Some("Ctrl+N".into()),
            },
            PaletteEntry {
                id: "forge.command.document.save".into(),
                title: "Save Active Document".into(),
                category: "File".into(),
                shortcut: Some("Ctrl+S".into()),
            },
            PaletteEntry {
                id: "forge.command.view.widgets".into(),
                title: "Open Widget Gallery".into(),
                category: "View".into(),
                shortcut: None,
            },
            PaletteEntry {
                id: "forge.command.view.universal".into(),
                title: "Open Universal Application Suite".into(),
                category: "View".into(),
                shortcut: None,
            },
            PaletteEntry {
                id: "forge.command.run.play".into(),
                title: "Play In Editor".into(),
                category: "Run".into(),
                shortcut: Some("F6".into()),
            },
        ],
    }
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}

#[cfg(test)]
mod surface_persistence_tests {
    use super::*;

    #[test]
    fn saved_floating_panel_survives_and_keeps_registered_title() {
        let defaults = modular_surfaces();
        let mut old = defaults
            .iter()
            .find(|s| s.id == "surface.content")
            .unwrap()
            .clone();
        old.title = "Stale title".into();
        old.dock = SurfaceDock::Floating;
        old.last_dock = SurfaceDock::Left;
        old.preferred_size = [505.0, 330.0];
        let restored = restored_surfaces(defaults, vec![old]);
        let content = restored.iter().find(|s| s.id == "surface.content").unwrap();
        assert_eq!(content.title, "Content");
        assert_eq!(content.dock, SurfaceDock::Floating);
        assert_eq!(content.last_dock, SurfaceDock::Left);
        assert_eq!(content.preferred_size, [505.0, 330.0]);
    }

    #[test]
    fn malformed_sizes_and_unknown_or_duplicate_panels_cannot_corrupt_catalog() {
        let defaults = modular_surfaces();
        let n = defaults.len();
        let mut malformed = defaults[0].clone();
        malformed.preferred_size = [f32::NAN, f32::INFINITY];
        let unknown = ModularSurfaceState::new("old.deleted.panel", "Removed", SurfaceDock::Left);
        let restored = restored_surfaces(defaults, vec![unknown, malformed]);
        assert_eq!(restored.len(), n);
        assert_eq!(restored[0].preferred_size, [320.0, 320.0]);
        assert!(restored
            .iter()
            .all(|surface| surface.preferred_size.iter().all(|v| v.is_finite())));
    }
}
