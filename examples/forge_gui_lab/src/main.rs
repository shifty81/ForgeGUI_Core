#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use forge_gui_browser::{show_browser, BrowserItem, BrowserItemKind, BrowserModel};
use forge_gui_chrome::docking::{
    drop_preview_rect, drop_zone, DockDrop, DockNode, ModularDockTree, SplitAxis,
};
use forge_gui_chrome::{
    apply_creator_visuals, modular_surface_frame, paint_edge_reveal_indicator, pointer_near_edge,
    restore_surface_layout, show_chrome_bar, show_draggable_surface_tabs, show_native_surface,
    show_project_title_bar, show_status_items, show_tool_tray_tabs, show_viewport_resize_handles,
    show_workspace_tabs, ChromeBarKind, ChromeEdge, ModularSurfaceState, ModularToolbarState,
    NativeSurfaceLifecycle, ShellProfile, StatusItem, SurfaceDock, ToolTrayTab, ToolbarDock,
    WorkspaceTab,
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
// Kept separate from the legacy surface layout so old saves remain readable.
const FLOATING_ACTIVE_STORAGE_KEY: &str = "forgegui.core.floating_active.v1";
const FLOATING_TAB_ORDER_STORAGE_KEY: &str = "forgegui.core.floating_tab_order.v1";
const DOCK_TREE_STORAGE_KEY: &str = "forgegui.core.dock_tree.v1";
const FLOATING_DOCK_TREES_STORAGE_KEY: &str = "forgegui.core.floating_dock_trees.v1";

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
    active_floating_surfaces: std::collections::BTreeMap<String, String>,
    floating_tab_order: Vec<String>,
    // Geometry is measured by the actual panels, never guessed from shell edges.
    dock_tree: ModularDockTree,
    // Each native viewport owns its own validated subtree, but all use the
    // same DockNode model and rendering logic as the main application.
    floating_dock_trees: std::collections::BTreeMap<String, ModularDockTree>,
    floating_leaf_rects: Vec<(String, String, egui::Rect)>,
    // Actual leaf rectangles and stable target IDs, not legacy region guesses.
    dock_leaf_rects: Vec<(String, egui::Rect)>,
    toolbar_state: ModularToolbarState,
    surface_drag: Option<String>,
    floating_drag_origin: Option<String>,
    surface_drag_preview: Option<SurfaceDock>,
    toolbar_dragging: bool,
    toolbar_drag_preview: Option<ToolbarDock>,
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

        let dock_tree = ModularDockTree::restored(
            cc.storage.and_then(|storage| {
                eframe::get_value::<ModularDockTree>(storage, DOCK_TREE_STORAGE_KEY)
            }),
            &surfaces,
        );

        let saved_floating_dock_trees = cc
            .storage
            .and_then(|storage| {
                eframe::get_value::<std::collections::BTreeMap<String, ModularDockTree>>(
                    storage,
                    FLOATING_DOCK_TREES_STORAGE_KEY,
                )
            })
            .unwrap_or_default();
        let floating_dock_trees =
            reconcile_floating_dock_trees(saved_floating_dock_trees, &surfaces);

        let active_floating_surfaces = cc
            .storage
            .and_then(|storage| {
                eframe::get_value::<std::collections::BTreeMap<String, String>>(
                    storage,
                    FLOATING_ACTIVE_STORAGE_KEY,
                )
            })
            .unwrap_or_default();
        let floating_tab_order = normalize_floating_tab_order(
            &surfaces,
            cc.storage
                .and_then(|storage| {
                    eframe::get_value::<Vec<String>>(storage, FLOATING_TAB_ORDER_STORAGE_KEY)
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
            active_floating_surfaces,
            floating_tab_order,
            dock_tree,
            floating_dock_trees,
            floating_leaf_rects: Vec::new(),
            dock_leaf_rects: Vec::new(),
            toolbar_state: ModularToolbarState::default(),
            surface_drag: None,
            floating_drag_origin: None,
            surface_drag_preview: None,
            toolbar_dragging: false,
            toolbar_drag_preview: None,
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
        self.floating_drag_origin = None;
        self.surface_drag_preview = None;
        self.active_floating_surfaces.clear();
        self.floating_tab_order.clear();
        self.floating_dock_trees.clear();
        self.floating_leaf_rects.clear();
        self.dock_tree = ModularDockTree::restored(None, &self.modular_surfaces);
        self.dock_leaf_rects.clear();
        self.toolbar_dragging = false;
        self.toolbar_drag_preview = None;
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
        let title = surface.title.clone();
        self.dock_tree =
            ModularDockTree::restored(Some(self.dock_tree.clone()), &self.modular_surfaces);
        self.dock_tree.activate(id);
        self.last_action = format!("Opened surface: {title}");
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

    /// Render a snapshot of the validated tree. UI events are committed to the
    /// authoritative model, and the next frame renders the new structure.
    /// The model is never reconstructed from four fixed regional panels.
    fn show_dock_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &DockNode,
        rect: egui::Rect,
        path: &[bool],
        host: Option<&str>,
    ) {
        if rect.width() < 32.0 || rect.height() < 32.0 {
            return;
        }
        match node {
            DockNode::Tabs { tabs, active } => {
                let leaf_id = format!("forge.dock.leaf.{}.{path:?}", host.unwrap_or("shell"));
                ui.scope_builder(
                    egui::UiBuilder::new().id_salt(leaf_id).max_rect(rect),
                    |ui| self.show_dock_leaf(ui, tabs, active, rect, host),
                );
            }
            DockNode::Split {
                axis,
                ratio,
                first,
                second,
            } => {
                let gap = 6.0;
                let ratio = (*ratio).clamp(0.12, 0.88);
                let (first_rect, splitter, second_rect) = match axis {
                    SplitAxis::Horizontal => {
                        let x = rect.left() + rect.width() * ratio;
                        (
                            egui::Rect::from_min_max(
                                rect.min,
                                egui::pos2(x - gap * 0.5, rect.bottom()),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(x - gap * 0.5, rect.top()),
                                egui::pos2(x + gap * 0.5, rect.bottom()),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(x + gap * 0.5, rect.top()),
                                rect.max,
                            ),
                        )
                    }
                    SplitAxis::Vertical => {
                        let y = rect.top() + rect.height() * ratio;
                        (
                            egui::Rect::from_min_max(
                                rect.min,
                                egui::pos2(rect.right(), y - gap * 0.5),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(rect.left(), y - gap * 0.5),
                                egui::pos2(rect.right(), y + gap * 0.5),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(rect.left(), y + gap * 0.5),
                                rect.max,
                            ),
                        )
                    }
                };
                let mut first_path = path.to_vec();
                first_path.push(false);
                self.show_dock_node(ui, first, first_rect, &first_path, host);
                let mut second_path = path.to_vec();
                second_path.push(true);
                self.show_dock_node(ui, second, second_rect, &second_path, host);
                let handle = ui.interact(
                    splitter.expand(3.0),
                    ui.id().with(("forge.dock.split", path.to_vec())),
                    egui::Sense::click_and_drag(),
                );
                if handle.dragged_by(egui::PointerButton::Primary) {
                    if let Some(pointer) = ui.ctx().pointer_hover_pos() {
                        let next = match axis {
                            SplitAxis::Horizontal => (pointer.x - rect.left()) / rect.width(),
                            SplitAxis::Vertical => (pointer.y - rect.top()) / rect.height(),
                        };
                        if let Some(host) = host {
                            if let Some(tree) = self.floating_dock_trees.get_mut(host) {
                                tree.resize_split(path, next);
                            }
                        } else {
                            self.dock_tree.resize_split(path, next);
                        }
                    }
                }
                let cursor = match axis {
                    SplitAxis::Horizontal => egui::CursorIcon::ResizeHorizontal,
                    SplitAxis::Vertical => egui::CursorIcon::ResizeVertical,
                };
                handle.on_hover_cursor(cursor);
                ui.painter()
                    .rect_filled(splitter, 2.0, color(self.theme.chrome.separator));
            }
        }
    }

    fn show_dock_leaf(
        &mut self,
        ui: &mut egui::Ui,
        tabs: &[String],
        selected: &str,
        leaf_rect: egui::Rect,
        host: Option<&str>,
    ) {
        let surfaces: Vec<ModularSurfaceState> = tabs
            .iter()
            .filter_map(|id| {
                self.modular_surfaces
                    .iter()
                    .find(|surface| {
                        surface.id == *id
                            && surface.visible
                            && match host {
                                None => surface.dock != SurfaceDock::Floating,
                                Some(host) => {
                                    surface.dock == SurfaceDock::Floating
                                        && surface.floating_host_id() == host
                                }
                            }
                    })
                    .cloned()
            })
            .collect();
        if surfaces.is_empty() {
            return;
        }
        let active = if surfaces.iter().any(|surface| surface.id == selected) {
            selected.to_owned()
        } else {
            surfaces[0].id.clone()
        };
        let title = surfaces
            .iter()
            .find(|surface| surface.id == active)
            .map(|surface| surface.title.as_str())
            .unwrap_or("Surface");
        let locked = surfaces
            .iter()
            .find(|surface| surface.id == active)
            .is_some_and(|surface| surface.locked);
        let theme = self.theme.clone();
        let mut selected_tab = None;
        let mut hide_active = false;
        let mut dragged_tab = None;
        let available = ui.available_size();
        modular_surface_frame(&theme).show(ui, |ui| {
            ui.set_min_size(egui::vec2(
                (available.x - 24.0).max(0.0),
                (available.y - 24.0).max(0.0),
            ));
            // Dragging any unused header area should move the panel; never
            // overlap tab or Hide hit targets with the background drag target.
            let mut header_content_right = None;
            let mut header_close_left = None;
            let header = ui.horizontal(|ui| {
                let grip = ui
                    .add(
                        egui::Label::new(egui::RichText::new("⠿").strong())
                            .sense(egui::Sense::click_and_drag()),
                    )
                    .on_hover_text(if locked {
                        "Surface locked"
                    } else {
                        "Drag to tab, split, or float"
                    });
                if !locked && grip.drag_started_by(egui::PointerButton::Primary) {
                    dragged_tab = Some(active.clone());
                }
                if surfaces.len() == 1 {
                    let tab = ui.add(
                        egui::Button::new(egui::RichText::new(title).strong())
                            .selected(true)
                            .sense(egui::Sense::click_and_drag()),
                    );
                    if !locked && tab.drag_started_by(egui::PointerButton::Primary) {
                        dragged_tab = Some(active.clone());
                    }
                    header_content_right = Some(tab.rect.right());
                } else {
                    let label =
                        ui.label(egui::RichText::new(format!("{} panels", surfaces.len())).small());
                    header_content_right = Some(label.rect.right());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let hide = panel_chrome_button(
                        ui,
                        icon_text(IconId::Close),
                        "Hide surface",
                        false,
                        true,
                        &theme,
                    );
                    header_close_left = Some(hide.rect.left());
                    if hide.clicked() {
                        hide_active = true;
                    }
                });
            });
            if !locked {
                if let (Some(content_right), Some(close_left)) =
                    (header_content_right, header_close_left)
                {
                    let drag_rect = egui::Rect::from_min_max(
                        egui::pos2(content_right + 3.0, header.response.rect.top()),
                        egui::pos2(close_left - 3.0, header.response.rect.bottom()),
                    );
                    if drag_rect.width() > 8.0 && drag_rect.height() > 0.0 {
                        let drag = ui.interact(
                            drag_rect,
                            ui.id().with(("forge.dock.header", active.as_str())),
                            egui::Sense::click_and_drag(),
                        );
                        if drag.drag_started_by(egui::PointerButton::Primary) {
                            dragged_tab = Some(active.clone());
                        }
                        drag.on_hover_cursor(egui::CursorIcon::Grab);
                    }
                }
            }
            if surfaces.len() > 1 {
                let response = show_draggable_surface_tabs(ui, &surfaces, &active, &theme);
                selected_tab = response.selected;
                if dragged_tab.is_none() {
                    dragged_tab = response.drag_started;
                }
            }
            ui.separator();
            self.show_surface_content(ui, &active);
        });
        // Use the reserved leaf rectangle, not content-dependent frame overflow.
        if let Some(host) = host {
            self.floating_leaf_rects
                .push((host.to_owned(), active.clone(), leaf_rect));
        } else {
            self.dock_leaf_rects.push((active.clone(), leaf_rect));
        }
        if let Some(selected) = selected_tab {
            if let Some(host) = host {
                if let Some(tree) = self.floating_dock_trees.get_mut(host) {
                    tree.activate(&selected);
                }
                self.active_floating_surfaces
                    .insert(host.to_owned(), selected.clone());
            } else {
                self.dock_tree.activate(&selected);
            }
            self.last_action = format!("Active surface: {selected}");
        }
        if let Some(dragged) = dragged_tab {
            if let Some(host) = host {
                if let Some(tree) = self.floating_dock_trees.get_mut(host) {
                    tree.activate(&dragged);
                }
            } else {
                self.dock_tree.activate(&dragged);
            }
            self.surface_drag = Some(dragged.clone());
            self.floating_drag_origin = host.map(str::to_owned);
            self.last_action = format!("Dragging surface: {dragged}");
        }
        if hide_active {
            if let Some(surface) = self
                .modular_surfaces
                .iter_mut()
                .find(|surface| surface.id == active)
            {
                surface.visible = false;
            }
            if let Some(host) = host {
                let saved = self.floating_dock_trees.get(host).cloned();
                self.floating_dock_trees.insert(
                    host.to_owned(),
                    ModularDockTree::restored_for_host(saved, &self.modular_surfaces, host),
                );
            } else {
                self.dock_tree =
                    ModularDockTree::restored(Some(self.dock_tree.clone()), &self.modular_surfaces);
            }
        }
    }

    fn show_floating_surfaces(&mut self, ctx: &egui::Context) {
        // Same recursive tabs/splits renderer inside every native host. The
        // viewport owns size/coordinates; only the catalog owns panel identity.
        let groups = floating_groups(&self.modular_surfaces);
        let hosts: Vec<ModularSurfaceState> = groups
            .iter()
            .map(|(host, panels)| {
                let mut representative = panels[0].clone();
                representative.id = host.clone();
                representative
            })
            .collect();
        self.native_surface_lifecycle.retain_visible(&hosts);
        self.floating_dock_trees = reconcile_floating_dock_trees(
            std::mem::take(&mut self.floating_dock_trees),
            &self.modular_surfaces,
        );
        self.active_floating_surfaces
            .retain(|host, _| groups.iter().any(|(id, _)| id == host));
        self.floating_leaf_rects.clear();

        let mut source_released = false;
        for (host, tabs) in groups {
            let Some(tree) = self.floating_dock_trees.get(&host).cloned() else {
                continue;
            };
            let Some(node) = tree.root.clone() else {
                continue;
            };
            let active = self
                .active_floating_surfaces
                .get(&host)
                .filter(|id| tree.has(id))
                .cloned()
                .unwrap_or_else(|| tabs[0].id.clone());
            let panel = tabs
                .iter()
                .find(|panel| panel.id == active)
                .unwrap_or(&tabs[0]);
            let opening_size = self
                .native_surface_lifecycle
                .opening_size(&host, panel.preferred_size);
            let title = panel.title.clone();
            let theme = self.theme.clone();
            let mut drop_requested: Option<(String, DockDrop)> = None;
            let mut native_pointer_released = false;
            let native_response =
                show_native_surface(ctx, &host, &title, opening_size, &theme, |ui| {
                    ui.horizontal(|ui| {
                        let grip = ui
                            .add(egui::Label::new("⠿").sense(egui::Sense::click_and_drag()))
                            .on_hover_text("Drag this floating window");
                        if grip.drag_started_by(egui::PointerButton::Primary) {
                            ui.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                        ui.label(egui::RichText::new(&title).small());
                    });
                    ui.separator();
                    let rect = ui.available_rect_before_wrap();
                    self.show_dock_node(ui, &node, rect, &[], Some(&host));
                    let (pointer, released) = ui
                        .ctx()
                        .input(|input| (input.pointer.hover_pos(), input.pointer.any_released()));
                    native_pointer_released = released;
                    if let Some(source) = self.surface_drag.as_deref() {
                        let can_transfer = self.modular_surfaces.iter().any(|surface| {
                            surface.id == source
                                && ((surface.dock == SurfaceDock::Floating
                                    && surface.floating_host_id() == host.as_str()
                                    && !surface.locked)
                                    || forge_gui_chrome::docking::can_join_native_host(
                                        surface, &host,
                                    ))
                        });
                        if can_transfer {
                            let rects: Vec<_> = self
                                .floating_leaf_rects
                                .iter()
                                .filter(|(id, _, _)| id == &host)
                                .map(|(_, id, rect)| (id.clone(), *rect))
                                .collect();
                            let target = pointer.and_then(|point| {
                                resolve_actionable_drop_target(point, &rects, source, &tree)
                            });
                            if let Some((target, zone, rect)) = target {
                                let preview = drop_preview_rect(rect, zone);
                                ui.painter().rect_filled(
                                    preview,
                                    4.0,
                                    egui::Color32::from_rgba_unmultiplied(
                                        theme.base.accent.0,
                                        theme.base.accent.1,
                                        theme.base.accent.2,
                                        32,
                                    ),
                                );
                                ui.painter().rect_stroke(
                                    preview,
                                    4.0,
                                    egui::Stroke::new(2.0, color(theme.base.accent)),
                                    egui::StrokeKind::Inside,
                                );
                                if released {
                                    drop_requested = Some((target, zone));
                                }
                            }
                        }
                    }
                });
            if self.floating_drag_origin.as_deref() == Some(host.as_str())
                && native_pointer_released
            {
                source_released = true;
            }
            if let Some((target, zone)) = drop_requested {
                if let Some(source) = self.surface_drag.clone() {
                    let previous_host = self
                        .modular_surfaces
                        .iter()
                        .find(|item| item.id == source)
                        .filter(|item| item.dock == SurfaceDock::Floating)
                        .map(|item| item.floating_host_id().to_owned());
                    let mut candidate = self.modular_surfaces.clone();
                    let transitioned = candidate
                        .iter_mut()
                        .find(|item| item.id == source)
                        .is_some_and(|item| {
                            previous_host.as_deref() == Some(host.as_str())
                                || item.join_floating_host(&host)
                        });
                    if transitioned {
                        let mut proposal = self
                            .floating_dock_trees
                            .get(&host)
                            .cloned()
                            .unwrap_or_default();
                        let committed = if previous_host.as_deref() == Some(host.as_str()) {
                            proposal
                                .move_relative_in_host(&source, &target, zone, &candidate, &host)
                        } else {
                            proposal.attach_in_host(&source, &target, zone, &candidate, &host)
                        };
                        if committed {
                            self.floating_dock_trees.insert(host.clone(), proposal);
                            if previous_host.as_deref() != Some(host.as_str()) {
                                if let Some(old) = previous_host {
                                    if let Some(tree) = self.floating_dock_trees.get_mut(&old) {
                                        tree.detach(&source);
                                    }
                                } else {
                                    self.dock_tree.detach(&source);
                                }
                                if let Some(updated) =
                                    candidate.into_iter().find(|item| item.id == source)
                                {
                                    if let Some(item) = self
                                        .modular_surfaces
                                        .iter_mut()
                                        .find(|item| item.id == source)
                                    {
                                        *item = updated;
                                    }
                                }
                            }
                            self.active_floating_surfaces
                                .insert(host.clone(), source.clone());
                            self.last_action =
                                format!("Docked {source} in floating host {host}: {zone:?}");
                            self.surface_drag = None;
                            self.floating_drag_origin = None;
                            self.surface_drag_preview = None;
                        }
                    }
                }
            }
            for item in self.modular_surfaces.iter_mut().filter(|item| {
                item.visible
                    && item.dock == SurfaceDock::Floating
                    && item.floating_host_id() == host.as_str()
            }) {
                if !native_response.maximized {
                    if let Some([width, height]) = native_response.inner_size {
                        if width.is_finite()
                            && height.is_finite()
                            && width >= 260.0
                            && height >= 180.0
                        {
                            item.preferred_size =
                                [width.clamp(260.0, 4096.0), height.clamp(180.0, 4096.0)];
                        }
                    }
                }
                if native_response.close_requested {
                    let locked = item.locked;
                    item.locked = false;
                    let _ = item.redock();
                    item.locked = locked;
                }
            }
            if native_response.close_requested {
                self.last_action = format!("Redocked floating group {host}");
            }
        }
        // Preserve the drag through the shell's subsequent drop dispatcher if
        // the native source released over a measured main-window leaf.
        if should_cancel_floating_drag(
            source_released,
            ctx.pointer_hover_pos(),
            &self.dock_leaf_rects,
        ) {
            self.surface_drag = None;
            self.floating_drag_origin = None;
            self.surface_drag_preview = None;
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
        let Some(source) = self.surface_drag.clone() else {
            return;
        };
        if self
            .modular_surfaces
            .iter()
            .find(|panel| panel.id == source)
            .is_none_or(|panel| !panel.visible || panel.locked)
        {
            self.surface_drag = None;
            self.floating_drag_origin = None;
            return;
        }
        let pointer = ctx.pointer_hover_pos();
        let target = pointer
            .filter(|point| shell_rect.contains(*point))
            .and_then(|point| {
                resolve_actionable_drop_target(
                    point,
                    &self.dock_leaf_rects,
                    &source,
                    &self.dock_tree,
                )
            });
        self.surface_drag_preview = target.as_ref().and_then(|(id, _, _)| {
            self.modular_surfaces
                .iter()
                .find(|panel| panel.id == *id)
                .map(|panel| panel.dock)
        });
        if let Some((_, zone, rect)) = &target {
            let accent = self.theme.base.accent;
            let preview = drop_preview_rect(*rect, *zone);
            ui.painter().rect_filled(
                preview,
                self.theme.interaction.surface_radius.clamp(0.0, 20.0),
                egui::Color32::from_rgba_unmultiplied(accent.0, accent.1, accent.2, 28),
            );
            ui.painter().rect_stroke(
                preview,
                self.theme.interaction.surface_radius.clamp(0.0, 20.0),
                egui::Stroke::new(2.0, color(accent)),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                preview.center(),
                egui::Align2::CENTER_CENTER,
                match zone {
                    DockDrop::Tab => "Group as tabs",
                    DockDrop::Left => "Split left",
                    DockDrop::Right => "Split right",
                    DockDrop::Top => "Split above",
                    DockDrop::Bottom => "Split below",
                },
                egui::FontId::proportional(14.0),
                color(self.theme.base.text),
            );
        }
        if ctx.input(|input| input.pointer.primary_down()) {
            return;
        }
        let was_floating = self
            .modular_surfaces
            .iter()
            .any(|panel| panel.id == source && panel.dock == SurfaceDock::Floating);
        // A different native viewport is responsible for its own pointer release.
        if was_floating && pointer.is_none() {
            return;
        }
        if let Some((destination, zone, _)) = target {
            // No mutation when dropped onto itself: this is not a request to float.
            if destination != source {
                let destination_dock = self
                    .modular_surfaces
                    .iter()
                    .find(|panel| panel.id == destination)
                    .map(|panel| panel.dock);
                if let Some(dock) = destination_dock {
                    // Commit native-to-shell attach only when the tree accepts it;
                    // a failed proposal leaves both model and catalog intact.
                    let mut candidate = self.modular_surfaces.clone();
                    if let Some(panel) = candidate.iter_mut().find(|panel| panel.id == source) {
                        panel.move_to(dock);
                    }
                    let committed = if was_floating {
                        self.dock_tree
                            .attach(&source, &destination, zone, &candidate)
                    } else {
                        self.dock_tree
                            .move_relative(&source, &destination, zone, &candidate)
                    };
                    if committed {
                        if let Some(panel) = self
                            .modular_surfaces
                            .iter_mut()
                            .find(|panel| panel.id == source)
                        {
                            panel.move_to(dock);
                        }
                        self.dock_tree.activate(&source);
                        self.last_action =
                            format!("Docked {source} relative to {destination}: {zone:?}");
                    }
                }
            }
        } else if pointer.is_some_and(|point| !shell_rect.contains(point)) && !was_floating {
            // Dragging beyond the workspace detaches. Dropping on menus/title
            // inside the shell cancels instead of silently changing the dock.
            if let Some(panel) = self
                .modular_surfaces
                .iter_mut()
                .find(|panel| panel.id == source)
            {
                if panel.move_to(SurfaceDock::Floating) {
                    self.dock_tree.detach(&source);
                    self.last_action = format!("Detached {source}");
                }
            }
        }
        self.surface_drag = None;
        self.floating_drag_origin = None;
        self.surface_drag_preview = None;
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
            ui.label("Drag a panel tab onto a leaf center to group as tabs, or onto an edge to split that leaf. Drag outside the workspace to float. Drag dividers to resize nested splits.");
            ui.label("Floating surfaces can share native tabbed windows. Close the host to return its tabs to their previous docks. Native floating splits and cross-viewport gestures require Windows certification.");
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

        self.dock_leaf_rects.clear();
        if self.show_modular_surfaces {
            // Validate incoming changes from panel menus, native-host closes,
            // visibility and consumer registrations before rendering a frame.
            self.dock_tree =
                ModularDockTree::restored(Some(self.dock_tree.clone()), &self.modular_surfaces);
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
                } else if self.show_modular_surfaces {
                    if let Some(node) = self.dock_tree.root.clone() {
                        let rect = ui.available_rect_before_wrap();
                        self.show_dock_node(ui, &node, rect, &[], None);
                    } else {
                        self.show_empty_workspace(ui);
                    }
                } else {
                    self.show_application_workspace(ui);
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
        eframe::set_value(
            storage,
            FLOATING_ACTIVE_STORAGE_KEY,
            &self.active_floating_surfaces,
        );
        eframe::set_value(
            storage,
            FLOATING_TAB_ORDER_STORAGE_KEY,
            &self.floating_tab_order,
        );
        eframe::set_value(storage, DOCK_TREE_STORAGE_KEY, &self.dock_tree);
        eframe::set_value(
            storage,
            FLOATING_DOCK_TREES_STORAGE_KEY,
            &self.floating_dock_trees,
        );
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

        egui::Panel::top("forge.creator.project_title")
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
            });
        // Do not synthesize maximization based on the monitor's (0,0) edge.
        // A native host must own WM_NCHITTEST/Snap Layouts and DWM corners;
        // the old pseudo-snap path could hide the undecorated title region.

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
                            ui.add_enabled(
                                false,
                                egui::Label::new("Windows Snap Layouts (native host pending)"),
                            )
                            .on_hover_text("Synthetic edge snapping was disabled because it could hide the frameless title bar. Proper Windows hit testing is still required.");
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

/// Find a live tree leaf by measured bounds, with no shell-center fallback.
/// A target is a panel identity and an actionable zone, not a legacy region.
fn resolve_leaf_drop_target(
    pointer: egui::Pos2,
    leaves: &[(String, egui::Rect)],
) -> Option<(String, DockDrop, egui::Rect)> {
    leaves
        .iter()
        .filter_map(|(id, rect)| drop_zone(*rect, pointer).map(|zone| (id.clone(), zone, *rect)))
        .min_by(|(_, _, a), (_, _, b)| {
            (a.width() * a.height()).total_cmp(&(b.width() * b.height()))
        })
}

/// An active tab can be split out of its own tab group by dropping against
/// that leaf's edge. The underlying tree requires a *different* target ID;
/// choose a peer from the same leaf, never another unrelated dock branch.
fn peer_in_source_leaf(node: &DockNode, source: &str) -> Option<String> {
    match node {
        DockNode::Tabs { tabs, .. } => {
            if tabs.iter().any(|id| id == source) {
                tabs.iter().find(|id| id.as_str() != source).cloned()
            } else {
                None
            }
        }
        DockNode::Split { first, second, .. } => {
            if first.contains(source) {
                peer_in_source_leaf(first, source)
            } else if second.contains(source) {
                peer_in_source_leaf(second, source)
            } else {
                None
            }
        }
    }
}

/// Keep a source-only tab leaf stationary while allowing a source tab in a
/// multi-tab leaf to split relative to a peer. Center-on-self never detaches.
fn resolve_actionable_drop_target(
    pointer: egui::Pos2,
    leaves: &[(String, egui::Rect)],
    source: &str,
    tree: &ModularDockTree,
) -> Option<(String, DockDrop, egui::Rect)> {
    let (target, zone, rect) = resolve_leaf_drop_target(pointer, leaves)?;
    if target != source {
        return Some((target, zone, rect));
    }
    if zone == DockDrop::Tab {
        return None;
    }
    let peer = tree
        .root
        .as_ref()
        .and_then(|node| peer_in_source_leaf(node, source))?;
    Some((peer, zone, rect))
}

/// The native viewport draws before the shell commits drag/drop. Do not
/// discard a valid main-window destination merely because the source released.
fn should_cancel_floating_drag(
    released: bool,
    shell_pointer: Option<egui::Pos2>,
    shell_leaves: &[(String, egui::Rect)],
) -> bool {
    released
        && shell_pointer.is_none_or(|point| resolve_leaf_drop_target(point, shell_leaves).is_none())
}

/// Unique native-host buckets. Catalog ownership is not inferred from layout IDs.
fn floating_groups(surfaces: &[ModularSurfaceState]) -> Vec<(String, Vec<ModularSurfaceState>)> {
    let mut groups: Vec<(String, Vec<ModularSurfaceState>)> = Vec::new();
    for panel in surfaces
        .iter()
        .filter(|p| p.visible && p.dock == SurfaceDock::Floating)
    {
        let host = panel.floating_host_id().to_owned();
        if let Some((_, members)) = groups.iter_mut().find(|(id, _)| *id == host) {
            members.push(panel.clone());
        } else {
            groups.push((host, vec![panel.clone()]));
        }
    }
    groups
}

fn reconcile_floating_dock_trees(
    mut saved: std::collections::BTreeMap<String, ModularDockTree>,
    surfaces: &[ModularSurfaceState],
) -> std::collections::BTreeMap<String, ModularDockTree> {
    let mut trees = std::collections::BTreeMap::new();
    for (host, _) in floating_groups(surfaces) {
        let tree = ModularDockTree::restored_for_host(saved.remove(&host), surfaces, &host);
        if tree.root.is_some() {
            trees.insert(host, tree);
        }
    }
    trees
}

/// Drop stale/duplicate tab IDs from older saves without changing the order
/// of valid entries. Appends newly registered floating tabs deterministically.
fn normalize_floating_tab_order(
    surfaces: &[ModularSurfaceState],
    saved: Vec<String>,
) -> Vec<String> {
    let mut order = Vec::new();
    for id in saved
        .into_iter()
        .chain(surfaces.iter().map(|s| s.id.clone()))
    {
        if surfaces.iter().any(|surface| surface.id == id) && !order.contains(&id) {
            order.push(id);
        }
    }
    order
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
            existing.floating_host = if old.dock == SurfaceDock::Floating {
                old.floating_host.filter(|host| !host.is_empty())
            } else {
                None
            };
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
    fn floating_host_group_survives_layout_restore() {
        let defaults = modular_surfaces();
        let mut moved = defaults
            .iter()
            .find(|s| s.id == "surface.properties")
            .unwrap()
            .clone();
        assert!(moved.join_floating_host("surface.activity"));
        let restored = restored_surfaces(defaults, vec![moved]);
        let properties = restored
            .iter()
            .find(|s| s.id == "surface.properties")
            .unwrap();
        assert_eq!(properties.dock, SurfaceDock::Floating);
        assert_eq!(properties.floating_host_id(), "surface.activity");
        assert_eq!(properties.last_dock, SurfaceDock::Right);
    }

    #[test]
    fn restored_docked_surface_cannot_keep_stale_floating_host() {
        let defaults = modular_surfaces();
        let mut old = defaults
            .iter()
            .find(|s| s.id == "surface.content")
            .unwrap()
            .clone();
        old.floating_host = Some("stale".into());
        let restored = restored_surfaces(defaults, vec![old]);
        let content = restored.iter().find(|s| s.id == "surface.content").unwrap();
        assert_eq!(content.floating_host, None);
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

    #[test]
    fn dock_target_uses_actual_panel_bounds_instead_of_shell_quadrants() {
        let a = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(160.0, 220.0));
        let b = egui::Rect::from_min_size(egui::pos2(170.0, 10.0), egui::vec2(400.0, 220.0));
        let leaves = vec![("content".to_owned(), a), ("canvas".to_owned(), b)];
        assert_eq!(
            resolve_leaf_drop_target(egui::pos2(12.0, 110.0), &leaves)
                .unwrap()
                .0,
            "content"
        );
        let center = resolve_leaf_drop_target(egui::pos2(370.0, 120.0), &leaves).unwrap();
        assert_eq!(center.0, "canvas");
        assert_eq!(center.1, DockDrop::Tab);
        assert_eq!(
            resolve_leaf_drop_target(egui::pos2(800.0, 500.0), &leaves),
            None
        );
    }

    #[test]
    fn active_tab_can_split_from_its_own_group_but_not_onto_itself() {
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(300.0, 200.0));
        let leaves = vec![("surface.content".to_owned(), rect)];
        let tree = ModularDockTree {
            version: 1,
            root: Some(DockNode::Tabs {
                tabs: vec!["surface.content".into(), "surface.properties".into()],
                active: "surface.content".into(),
            }),
        };
        let edge = resolve_actionable_drop_target(
            egui::pos2(12.0, 110.0),
            &leaves,
            "surface.content",
            &tree,
        )
        .expect("a grouped tab can split from its peer");
        assert_eq!(edge.0, "surface.properties");
        assert_eq!(edge.1, DockDrop::Left);
        assert!(
            resolve_actionable_drop_target(rect.center(), &leaves, "surface.content", &tree,)
                .is_none()
        );
        let lone = ModularDockTree {
            version: 1,
            root: Some(DockNode::Tabs {
                tabs: vec!["surface.content".into()],
                active: "surface.content".into(),
            }),
        };
        assert!(resolve_actionable_drop_target(
            egui::pos2(12.0, 110.0),
            &leaves,
            "surface.content",
            &lone,
        )
        .is_none());
    }

    #[test]
    fn peer_lookup_does_not_cross_nested_split_boundaries() {
        let tree = DockNode::Split {
            axis: SplitAxis::Horizontal,
            ratio: 0.5,
            first: Box::new(DockNode::Tabs {
                tabs: vec!["source".into(), "peer".into()],
                active: "source".into(),
            }),
            second: Box::new(DockNode::Tabs {
                tabs: vec!["unrelated".into()],
                active: "unrelated".into(),
            }),
        };
        assert_eq!(
            peer_in_source_leaf(&tree, "source").as_deref(),
            Some("peer")
        );
        assert_eq!(peer_in_source_leaf(&tree, "unrelated"), None);
        assert_eq!(peer_in_source_leaf(&tree, "unknown"), None);
    }

    #[test]
    fn lab_bootstrap_and_restore_retain_single_authoritative_panel_instances() {
        let surfaces = modular_surfaces();
        let tree = ModularDockTree::restored(None, &surfaces);
        let expected = surfaces
            .iter()
            .filter(|panel| panel.visible && panel.dock != SurfaceDock::Floating)
            .count();
        assert_eq!(tree.ordered_tabs().len(), expected);
        let restored = ModularDockTree::restored(Some(tree.clone()), &surfaces);
        assert_eq!(tree, restored);
        let mut seen = std::collections::BTreeSet::new();
        assert!(restored
            .ordered_tabs()
            .into_iter()
            .all(|id| seen.insert(id)));
    }

    #[test]
    fn shell_drop_creates_split_and_saved_restore_preserves_ratio() {
        let surfaces = modular_surfaces();
        let mut tree = ModularDockTree::restored(None, &surfaces);
        assert!(tree.move_relative(
            "surface.activity",
            "surface.properties",
            DockDrop::Left,
            &surfaces
        ));
        assert!(tree.resize_split(&[], 0.41));
        let saved = tree.clone();
        assert_eq!(
            ModularDockTree::restored(Some(saved.clone()), &surfaces),
            saved
        );
        assert_eq!(
            saved.ordered_tabs().len(),
            surfaces
                .iter()
                .filter(|p| p.visible && p.dock != SurfaceDock::Floating)
                .count()
        );
    }

    #[test]
    fn shell_drop_never_promotes_hidden_or_floating_panels_into_saved_tree() {
        let mut surfaces = modular_surfaces();
        let mut tree = ModularDockTree::restored(None, &surfaces);
        let panel = surfaces
            .iter_mut()
            .find(|p| p.id == "surface.content")
            .unwrap();
        assert!(panel.move_to(SurfaceDock::Floating));
        tree = ModularDockTree::restored(Some(tree), &surfaces);
        assert!(!tree.has("surface.content"));
        let panel = surfaces
            .iter_mut()
            .find(|p| p.id == "surface.properties")
            .unwrap();
        panel.visible = false;
        tree = ModularDockTree::restored(Some(tree), &surfaces);
        assert!(!tree.has("surface.properties"));
    }

    #[test]
    fn floating_tab_order_discards_stale_ids_and_duplicates() {
        let surfaces = modular_surfaces();
        let order = normalize_floating_tab_order(
            &surfaces,
            vec![
                "unknown".into(),
                "surface.activity".into(),
                "surface.activity".into(),
                "surface.content".into(),
            ],
        );
        assert_eq!(
            order[..2].iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["surface.activity", "surface.content"],
        );
        assert_eq!(order.len(), surfaces.len());
    }

    #[test]
    fn floating_host_trees_reconcile_saved_splits_and_remove_cross_host_duplicates() {
        let mut surfaces = modular_surfaces();
        for id in ["surface.content", "surface.properties", "surface.activity"] {
            let panel = surfaces.iter_mut().find(|p| p.id == id).unwrap();
            assert!(panel.join_floating_host("host.one"));
        }
        let mut original = reconcile_floating_dock_trees(Default::default(), &surfaces);
        let tree = original.get_mut("host.one").unwrap();
        assert!(tree.move_relative_in_host(
            "surface.content",
            "surface.properties",
            DockDrop::Left,
            &surfaces,
            "host.one"
        ));
        let saved = tree.clone();
        let restored = reconcile_floating_dock_trees(original, &surfaces);
        assert_eq!(restored.get("host.one"), Some(&saved));
        let panel = surfaces
            .iter_mut()
            .find(|p| p.id == "surface.content")
            .unwrap();
        assert!(panel.join_floating_host("host.two"));
        let changed = reconcile_floating_dock_trees(restored, &surfaces);
        assert!(!changed.get("host.one").unwrap().has("surface.content"));
        assert!(changed.get("host.two").unwrap().has("surface.content"));
        let total = changed
            .values()
            .map(|t| t.ordered_tabs().len())
            .sum::<usize>();
        assert_eq!(total, 3);
    }

    #[test]
    fn floating_host_tree_removes_hidden_and_reopened_panel_without_ghost_split() {
        let mut surfaces = modular_surfaces();
        for id in ["surface.content", "surface.properties"] {
            let panel = surfaces.iter_mut().find(|p| p.id == id).unwrap();
            assert!(panel.join_floating_host("floating"));
        }
        let mut saved = reconcile_floating_dock_trees(Default::default(), &surfaces);
        assert!(saved.get_mut("floating").unwrap().move_relative_in_host(
            "surface.content",
            "surface.properties",
            DockDrop::Right,
            &surfaces,
            "floating"
        ));
        surfaces
            .iter_mut()
            .find(|p| p.id == "surface.content")
            .unwrap()
            .visible = false;
        let without = reconcile_floating_dock_trees(saved, &surfaces);
        assert_eq!(
            without.get("floating").unwrap().ordered_tabs(),
            vec!["surface.properties".to_owned()]
        );
        surfaces
            .iter_mut()
            .find(|p| p.id == "surface.content")
            .unwrap()
            .visible = true;
        let reopened = reconcile_floating_dock_trees(without, &surfaces);
        assert_eq!(reopened.get("floating").unwrap().ordered_tabs().len(), 2);
    }

    #[test]
    fn native_release_preserves_valid_shell_drop_then_cancels_otherwise() {
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(200.0, 150.0));
        let leaves = vec![("content".to_owned(), rect)];
        assert!(!should_cancel_floating_drag(
            true,
            Some(rect.center()),
            &leaves
        ));
        assert!(should_cancel_floating_drag(
            true,
            Some(egui::pos2(700.0, 700.0)),
            &leaves
        ));
        assert!(should_cancel_floating_drag(true, None, &leaves));
        assert!(!should_cancel_floating_drag(false, None, &leaves));
    }
}
