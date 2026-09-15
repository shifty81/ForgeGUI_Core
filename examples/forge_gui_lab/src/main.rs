#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use forge_gui_browser::{show_browser, BrowserItem, BrowserItemKind, BrowserModel};
use forge_gui_chrome::{
    apply_creator_visuals, paint_edge_reveal_indicator, pointer_near_edge, show_chrome_bar,
    show_project_title_bar, show_status_items, show_tool_tray_tabs, show_viewport_resize_handles,
    show_workspace_tabs, ChromeBarKind, ChromeEdge, StatusItem, ToolTrayTab, WorkspaceTab,
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
use forge_gui_theme::{ForgeTheme, ForgeThemePreset};
use forge_gui_widgets::{
    gauge, icon_text, progress_bar, scrollbar, ScrollbarAxis, ScrollbarState, WidgetTone,
};
use forge_gui_workspace::{
    canvas_family_menu, canvas_options_menu, show_renderer_workspace, CanvasChromeState,
    WorkspaceTool, WorkspaceViewState,
};
use forge_render_core::{AuthoringCamera, NullRenderBackend, RenderRequest};
use forge_render_surface::{RenderFamily, RenderSurfaceDescriptor, RenderSurfaceHost};
use forge_scene_core::{ForgeScene, SceneEntity};

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ForgeGUI Core — GUI Lab")
            .with_decorations(false)
            .with_transparent(true)
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
    layout: ForgeLayoutState,
    workspace_tabs: Vec<WorkspaceTab>,
    show_workspace_tabs: bool,
    tool_tabs: Vec<ToolTrayTab>,
    active_tool_tab: String,
    browser: BrowserModel,
    inspector_object: PropertyObject,
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
        apply_creator_visuals(&cc.egui_ctx, &theme);

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

        Self {
            theme,
            theme_preset,
            layout,
            workspace_tabs: workspace_tabs(),
            show_workspace_tabs: false,
            tool_tabs: tool_tabs(),
            active_tool_tab: "tool.console".into(),
            browser: browser_model(),
            inspector_object: inspector_object(),
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
            last_action: "Canvas-first ForgeGUI Lab ready".into(),
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
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let metrics = self.theme.effective_metrics();

        if root.input(|i| i.modifiers.command && i.key_pressed(egui::Key::P)) {
            self.palette.open();
        }

        egui::Panel::top("forge.creator.project_title")
            .frame(egui::Frame::NONE)
            .exact_size(metrics.product_bar_height)
            .show(root, |ui| {
                let _ = show_project_title_bar(
                    ui,
                    "ForgeGUI",
                    "Core GUI Lab",
                    Some("main"),
                    &self.theme,
                );
            });

        let menu_theme = self.theme.clone();
        egui::Panel::top("forge.creator.menu")
            .frame(egui::Frame::NONE)
            .exact_size(metrics.menu_bar_height)
            .show(root, |ui| {
                show_chrome_bar(ui, ChromeBarKind::Menu, &menu_theme, |ui| {
                    ui.horizontal(|ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("New").clicked() {
                                self.last_action = "File > New".into();
                                ui.close();
                            }
                            if ui.button("Open…").clicked() {
                                self.last_action = "File > Open".into();
                                ui.close();
                            }
                            if ui.button("Save").clicked() {
                                self.last_action = "File > Save".into();
                                ui.close();
                            }
                            if ui.button("Save As…").clicked() {
                                self.last_action = "File > Save As".into();
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("Import…").clicked() {
                                self.last_action = "File > Import".into();
                                ui.close();
                            }
                            if ui.button("Export…").clicked() {
                                self.last_action = "File > Export".into();
                                ui.close();
                            }
                        });

                        ui.menu_button("Edit", |ui| {
                            if ui.button("Undo").clicked() {
                                self.last_action = "Edit > Undo".into();
                                ui.close();
                            }
                            if ui.button("Redo").clicked() {
                                self.last_action = "Edit > Redo".into();
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("Command Palette…   Ctrl+P").clicked() {
                                self.palette.open();
                                ui.close();
                            }
                            if ui.button("Preferences…").clicked() {
                                self.right_tab = "widgets".into();
                                self.right_panel_pinned = true;
                                ui.close();
                            }
                        });

                        ui.menu_button("View", |ui| {
                            ui.checkbox(&mut self.left_rail_pinned, "Pin tool rail");
                            ui.checkbox(&mut self.right_panel_pinned, "Pin context panel");
                            ui.checkbox(&mut self.show_workspace_tabs, "Document tabs");
                            ui.checkbox(
                                &mut self.layout.structural.bottom_visible,
                                "Bottom tool tray",
                            );
                            ui.checkbox(&mut self.layout.structural.status_visible, "Status bar");
                            ui.separator();
                            canvas_options_menu(
                                ui,
                                &mut self.workspace_view,
                                &mut self.canvas_chrome,
                            );
                            if let Some(family) = canvas_family_menu(ui, &mut self.render_surface) {
                                self.set_renderer_family(family);
                            }
                            ui.separator();
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
                        });

                        ui.menu_button("Help", |ui| {
                            if ui.button("GUI Widget Catalog").clicked() {
                                self.right_tab = "widgets".into();
                                self.right_panel_pinned = true;
                                ui.close();
                            }
                            if ui.button("Keyboard Shortcuts").clicked() {
                                self.last_action = "Help > Keyboard Shortcuts".into();
                                ui.close();
                            }
                            ui.separator();
                            ui.label("ForgeGUI Core · project-owned chrome");
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
                    let items = vec![
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
                            tone: WidgetTone::Success,
                        },
                    ];
                    show_status_items(ui, &items, &self.theme);
                });
        }

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
            self.last_action = format!("Command: {}", invocation.id);
        }

        show_viewport_resize_handles(root, 5.0);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
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
        let grid_color = egui::Color32::from_gray(31);
        let major_color = egui::Color32::from_gray(44);
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
            egui::Stroke::new(1.0, egui::Color32::from_gray(57)),
            egui::StrokeKind::Inside,
        );
    }

    if visibility.entities {
        let center = rect.center();
        let player = egui::Rect::from_center_size(center, egui::vec2(120.0, 86.0));
        painter.rect_filled(player, 5.0, egui::Color32::from_rgb(31, 68, 59));
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
            egui::Color32::WHITE,
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
            egui::Color32::from_rgba_unmultiplied(236, 183, 74, 20),
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
            ui.monospace("[PASS] Canvas-first workspace chrome attached");
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

fn workspace_tabs() -> Vec<WorkspaceTab> {
    vec![
        WorkspaceTab {
            id: "workspace.main".into(),
            label: "Main World".into(),
            icon: Some(IconId::World),
            dirty: true,
            closable: false,
        },
        WorkspaceTab {
            id: "workspace.ui".into(),
            label: "Interface".into(),
            icon: Some(IconId::Asset),
            dirty: false,
            closable: true,
        },
        WorkspaceTab {
            id: "workspace.logic".into(),
            label: "Logic".into(),
            icon: Some(IconId::Graph),
            dirty: false,
            closable: true,
        },
        WorkspaceTab {
            id: "workspace.animation".into(),
            label: "Animation".into(),
            icon: Some(IconId::Timeline),
            dirty: false,
            closable: true,
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
            BrowserItem::new("asset.scenes", "Scenes", BrowserItemKind::Folder),
            BrowserItem::new("asset.scene.main", "Main World", BrowserItemKind::Scene),
            BrowserItem::new("asset.objects", "Objects", BrowserItemKind::Folder),
            BrowserItem::new("asset.object.player", "Player", BrowserItemKind::Object),
            BrowserItem::new(
                "asset.object.ship",
                "Ship Prototype",
                BrowserItemKind::Object,
            ),
            BrowserItem::new("asset.sprites", "Sprites", BrowserItemKind::Folder),
            BrowserItem::new(
                "asset.sprite.player",
                "Player_Base",
                BrowserItemKind::Sprite,
            ),
            BrowserItem::new("asset.tilesets", "Tilesets", BrowserItemKind::Folder),
            BrowserItem::new("asset.audio", "Audio", BrowserItemKind::Folder),
            BrowserItem::new("asset.logic", "Logic", BrowserItemKind::Folder),
            BrowserItem::new(
                "asset.script.player",
                "Player Logic",
                BrowserItemKind::Script,
            ),
        ],
        ..Default::default()
    }
}

fn inspector_object() -> PropertyObject {
    PropertyObject {
        object_id: "object.player".into(),
        title: "Player".into(),
        fields: vec![
            PropertyField {
                id: "enabled".into(),
                label: "Enabled".into(),
                value: PropertyValue::Bool(true),
                read_only: false,
            },
            PropertyField {
                id: "position_x".into(),
                label: "Position X".into(),
                value: PropertyValue::Float(128.0),
                read_only: false,
            },
            PropertyField {
                id: "position_y".into(),
                label: "Position Y".into(),
                value: PropertyValue::Float(96.0),
                read_only: false,
            },
            PropertyField {
                id: "sprite".into(),
                label: "Sprite".into(),
                value: PropertyValue::Reference("asset.sprite.player".into()),
                read_only: false,
            },
            PropertyField {
                id: "runtime_id".into(),
                label: "Runtime ID".into(),
                value: PropertyValue::Text("player.main".into()),
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
